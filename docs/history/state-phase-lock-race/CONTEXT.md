# CONTEXT — state-phase-lock-race (GH #70)

Mode: **high-risk** · Source: GH issue #70, reported by @vantt from a downstream
repo running bee 1.18.2 (STR92 swarm, 2026-07-26).

## Problem

Three concurrent sessions in ONE worktree each hit a spurious intake-gate denial
(`phase: compounding-complete` / `phase: idle`) on their own `git add`/`commit`/
`Edit`, while that same worktree's `bee.mjs status`, run at essentially the same
instant, correctly reported `phase: swarming`. Retrying the write, or shelling
git through a Node subprocess, worked around it every time.

The reporter ruled out cwd-pin, cross-worktree confusion, and the
`worktree-grants.json` lost-update (already closed by `hardening-4b`), and
suspected new `controlRoot` surface from PR #64.

## Root cause (evidence, not hypothesis)

Not `controlRoot`. With all three sessions in one worktree,
`controlRoot === storeRoot === root` for all of them — root resolution never
diverges, and `bee.mjs status` reads the identical file through the identical
function the hook uses (`readState`, `bee.mjs:732`). The symptom is timing, not
a code-path split.

The defect is a **lost update on `.bee/state.json` between writers that take
different locks**:

| Writer | Lock held | Anchor |
|---|---|---|
| `bee-state-sync.mjs` hook (fires on every session's PostToolUse/Stop) | `'state'` | `packages/bee/hooks/bee-state-sync.mjs:127` |
| CLI mutation verbs (`state set`/`gate`/`scribing-run`/worker) when a live workflow record exists | `workflow:<id>` **only** | `packages/bee/bee.mjs:2552` |
| `handleStateStartFeature`'s post-`startFeature` projection rebuild | **none** | `packages/bee/bee.mjs:3284` |
| `state rebuild-projections` (`rebuildAllProjections`) | **none** | `packages/bee/bee.mjs:3304` |

All four end in `rebuildStateProjection` / `writeState`, which is a bare
read-modify-write of `.bee/state.json` and takes no lock of its own
(`packages/bee/lib/state-projection.mjs:212` read → `:224/:251/:276` write) — it
trusts its caller to serialize it, and the callers do not agree on what to
serialize against.

`'state'` and `workflow:<id>` are different lock names and do not exclude each
other. `withMutationLock`'s own comment states the divergence outright
(`bee.mjs:2528-2539`): *"a workflow-record-routed target … now locks on
`workflow:<id>` ONLY."* That was deliberate for msn-10 (two sessions mutating
two different workflows should not contend) — but the same code path also writes
the one shared `state.json` projection, which every session reads.

Mechanism producing the exact observed phase: the hook reads `current` at T0
(still carrying the previous feature's `compounding-complete`, or an idle
bootstrap), the CLI writes `swarming` at T0.5 under `workflow:<id>`, and the
hook writes its full stale-derived object at T1 under `'state'` — reverting the
phase. The write-guard's unlocked read (`bee-write-guard.mjs:778`) then lands
inside that window and denies. Nothing is corrupt; the JSON is valid and stale.
Atomic tmp+rename (`fsutil.mjs:90`) rules out torn reads, and confirms this is a
logical lost update rather than a parse failure.

This is verbatim the critical pattern recorded 2026-07-24:
*"A lost-update race between two writers of one record closes only when BOTH
lock — locking one side moves the race, it does not close it."*

## Locked decisions

| ID | Decision | Why |
|---|---|---|
| D1 | Every write of `.bee/state.json` and of a lane projection must happen while the `'state'` store lock is held, no matter which workflow lock the caller already holds. Acquisition order is always `workflow:<id>` → `'state'`, never the reverse | The record has one shared name; only a lock every writer shares closes a lost update. The audit found no existing `'state'` → `workflow:<id>` edge anywhere, so this new edge introduces no cycle |
| D2 | The `'state'` hold is narrowed to the projection rebuild+write, NOT the whole mutation body; `workflow:<id>` keeps guarding the workflow-record portion as it does today | Wrapping the whole body in `'state'` would re-serialize two sessions on two different workflows and undo msn-10's concurrency win. Only the shared projection write needs the shared lock |
| D3 | The lock stays OUTSIDE `rebuildStateProjection`/`rebuildLaneProjection` — those functions remain caller-serialized and acquire nothing | `lock.mjs` is non-reentrant (`lock.mjs:474`; `workflow-store.mjs:335` — *"a second acquire of the SAME lock name from the SAME process … a self-deadlock-shaped bug"*). `bee-state-sync.mjs` already holds `'state'` when it calls the function, so an internal acquire would stall ~5 s and throw `LockBusyError` on every hook tick |
| D4 | The two currently unlocked writers (`bee.mjs:3284`, `bee.mjs:3304`) are in scope. The fix is not done while any writer of this record stays unlocked | Same pattern as D1: three of four writers locking still leaves the race open. Fixing only the pair named in the issue would move the race, not close it |
| D5 | Proof is a real multi-process red-first test in `scripts/tests/test_store_lock.mjs`'s style (OS child processes, shared active-flag detector + exact final-count assertion, with a no-lock negative control), asserting mutual exclusion between a `'state'`-locked writer and a `workflow:<id>`-locked writer against one `state.json`. No assertion may be satisfiable by the live environment | An in-process async "concurrency" test never exercises this; and `test_state_write_concurrency.mjs:24-33` already names this exact race as knowingly out of its scope, so no existing suite can bite |
| D6 | Rust needs no production change in this feature. `crates/queen-bee/src/hooks/state_sync.rs:131` already takes `'state'`, and no Rust CLI `state` group exists yet (`crates/queen-bee/src/groups.rs:34`) — so the second, `workflow:<id>`-routed writer has no Rust twin to fix. The constraint is recorded so the future `state`-group port lands with D1 already satisfied | Porting the defect forward is the real risk here, not the current Rust binary |
| D7 | Per rust-port D1 (mjs feature freeze, critical bugfixes only), this lands as a critical bugfix with the mandated mirror artifact: a `rust-port`-tagged note of the behavior delta plus any affected parity fixture updated in the same change | The freeze is checkable only if every mjs bugfix carries the artifact |
| D8 | The Windows CI red on `main` (`test_msn_invariants.mjs`, `test_worktree_store.mjs`, `test_herding_cli.mjs`, failing since 2026-07-24) is NOT caused by this defect and is NOT fixed here. It is surfaced and filed as its own fix-first work | Never build on red, and never quietly absorb someone else's red into an unrelated feature's scope |

## Out of scope

- `controlRoot` resolution itself — investigated and cleared; the reporter's
  suspicion was reasonable but the evidence points elsewhere.
- The unlocked *reads* on the guard's deny path (`bee-write-guard.mjs:778`,
  `state.mjs:1762`, `state.mjs:1785`). Hooks deliberately never wait on a store
  lock (`claims.mjs:319`); once every writer serializes, a read sees a valid
  committed value, which is the contract those reads assume.
- The Windows CI red (D8).

## Outstanding questions

None blocking.
