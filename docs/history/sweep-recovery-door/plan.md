---
artifact_contract: bee-plan/v1
mode: high-risk
# approved_gate2: <unset until approval>
---

# Plan: Sweep Recovery Door

Mode: `high-risk` — 4 risk flags: public-contracts, covered-contract-change,
multi-domain, data-model. No hard-gate flag.

Why this is the least workflow that protects the work: the feature serves a
command the registry has advertised-and-refused for months, on a door that must
span the control plane and a store at once, and it writes a new field onto a
record two independent hook paths also write. Nothing smaller proves the
release set, the mark set, and the revival path are each the right one.

## Requirements (from CONTEXT.md)

- **D2** — a heartbeat-stale session record is marked dead in place
  (`status: "dead"`, `dead_at`), never deleted.
- **D3** — `bee recovery scan` is built as a releasing door: releases on
  invocation, no flag, no confirmation. `recovery window` stays unbuilt.
- **D7** — only `bee recovery scan` writes the dead mark; the sweep reached
  from `orient` and `claim-next` never touches a session record.
- **D8** — the dead mark is reversible, cleared with `revived_at` by **both**
  heartbeat writers (`state_sync.rs:494`, `prompt_context.rs:1597`).
- **D9** — the mark set is every heartbeat-stale session record, from a
  sessions-directory pass independent of the claims pass.

Inherited from the shipped sweep: caller exclusion (R97), the parked verdict
(R99), the store boundary (R100). **R98 is not inherited** — see Discovery 6.

## Discovery

1. **The crash-candidate list is the wrong release set.** `has_clean_end_trio`
   (`recovery.rs:57-109`) is applied as a bare `continue` at `recovery.rs:343`,
   before the work-signal block at `:365-396` where `session_has_active_claim`
   (`:378`) is consulted. A cleanly-ended session is skipped outright and never
   reaches the claim check. `recovery.rs:341` widens the hole further: a
   session with no resolvable transcript is skipped the same way. Closing a
   session mid-cell produces exactly these shapes. **Authorization for treating
   the report set and the release set as different sets is CONTEXT.md's own
   deferred question** ("Does `detect_crash_candidates`' clean-end-trio check
   belong in the release criteria, or only in the report? ... needs deciding
   against the code"), read together with D7's rationale that "the release
   criteria stay heartbeat-age based and unchanged". This plan answers that
   question; it does not reinterpret D3.
2. **The release criteria already live elsewhere and are correct.**
   `sweep_expired_claims` (`handlers_select.rs:75`) consults neither transcript
   nor trio; it releases on expired TTL plus stale owner heartbeat, caller
   excluded, re-judging both under the gate (`:103-118`). Signature:
   `(control: &Path, now: f64, caller_session: Option<&str>) -> MR<()>`.
   Callers: `handlers_select.rs:620`, `orient.rs:271`.
3. **The sweep reports nothing to its caller.** It returns `MR<()>`; the
   released, parked and unreachable facts leave only through `log_decision`
   (`:141`, `:158`) and one `eprintln!` (`:152`), and the `Unreachable` arm
   prints nothing. A command whose purpose is to report what it released
   cannot be built on that return type — see Approach.
4. **The full door is a named function.** `resolve_store_root_worktree`
   (`roots.rs:580-610`), matched as `RootsWt::Go(StoreRoots)`; the narrow
   `resolve_store_root` (`roots.rs:622-643`) demotes a granted worktree at
   `:635`. `cells finish` uses the wide door (`handlers_close.rs:607`) and
   splits it with `finish_topology` (`finish_support.rs:360-368`) — which is
   finish-shaped, not general, so this feature derives its own split.
   Note the naming trap: `roots.rs:35` calls `resolve_store_root_worktree` the
   **FULL** door, while `roots.rs:645-653` reserves the name **WIDE** for
   `resolve_store_root_any`, which is explicitly forbidden to any verb reading
   sessions or claims. This plan says FULL door and means `:580`.
5. **`detect_crash_candidates` is not reachable from a sibling module.**
   `detect_crash_candidates(ctx: &mut Ctx, projects_root: &str)`
   (`recovery.rs:264`) needs `status_full::Ctx`, whose four fields are all
   private and which exposes no constructor (`status_full/mod.rs:159-196`);
   every construction site is inside `status_full`. A `verbs::recovery`
   sibling cannot build one. The `cells` half has no such problem:
   `cells/mod.rs:395,397` re-export `sweep_expired_claims` and
   `finish_topology` as `pub(crate)`.
6. **R98 lives in the caller, not the sweep.** `sweep_expired_claims(control,
   now, None)` sweeps with no exclusion at all — `cells/tests.rs:2045, 2092,
   2226` call it that way deliberately. The decline-without-identity behavior
   is `orient.rs:262-270`, which resolves
   `resolve_session_flag_env(None).or_else(resolve_session_adopt)` and emits
   the blocker itself. A new door must implement its own decline; inheriting
   it is not possible.
7. **Two heartbeat writers, not one.** `hooks/state_sync.rs:494`
   (PostToolUse/SubagentStop/Stop) and `hooks/prompt_context.rs:1597`
   (UserPromptSubmit) each take the `sessions` lock via
   `acquire_store_lock_once` and each write `last_heartbeat`. Both are
   throttled and fail-open (`Busy => Ok(())`, `state_sync.rs:496`).
8. **The registry flip obliges a JSON surface.**
   `tests/registry_dispatch.rs:141` runs each served entry's `examples[0]`
   verbatim; `recovery.scan`'s is `bee recovery scan --json`. An argv the new
   `try_native` does not match falls through `router.rs:246` to the catalog and
   lands on `router.rs:441-451`'s "unsupported argument shape", which is
   `REFUSAL_MARKERS[4]` → `advertised_but_dead` → failure at
   `registry_dispatch.rs:160`. `--json` is matched by hand per group
   (`verbs/timings.rs:58`); nothing is automatic. The selective flip itself is
   supported: `catalog::resolve` is per-entry (`catalog.rs:113-122`), so
   `recovery window` keeps refusing while `scan` serves.
9. **The registry payload is not write-guarded.** `guards.rs:216` is a *read*
   deny for `SCOUT_DIRS` (`guards.rs:183-186`), which excludes
   `src/generated/`; `direct_edit_verb` (`guards.rs:42-63`) has no entry for
   it. Only the execution-gate phase check fires on that path. An earlier
   draft of CONTEXT.md claimed otherwise and is corrected there.

Evidence commands: `rg -n "has_clean_end_trio|session_has_active_claim" packages/bee-rs/crates/bee/src/verbs/status_full/recovery.rs`,
`rg -n "fn heartbeat_session" packages/bee-rs/crates/bee/src/hooks`,
`PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml`.

## Approach

**Recommended path.** `bee recovery scan` performs three passes and reports
them as three things.

- **Release** — call `sweep_expired_claims` with its own resolved caller. The
  release set is the release criteria (expired TTL, stale owner heartbeat,
  caller excluded), inheriting R97, R99 and R100 unchanged.
- **Mark** — a separate pass over `.bee/sessions/`, marking every
  heartbeat-stale record dead (D9), re-judging staleness under the `sessions`
  lock exactly as the sweep re-judges under its gate (`handlers_select.rs:103-118`),
  so a record read stale before the lock is not marked after its owner's
  heartbeat lands.
- **Report** — the crash candidates `detect_crash_candidates` finds: the
  trio-filtered, transcript-shaped set that tells a human which sessions are
  worth mining. Named as its own set, never merged with the released set.

**Two enabling changes this feature owns**, because the recommended path is
unbuildable without them and neither is a criteria change:

1. `sweep_expired_claims` gains a return value describing what it did — the
   released claims, the parked units, and the units it could not reach. Its
   criteria, ordering and locking are untouched; only the return type changes,
   across 2 production and 13 test call sites (`cells/tests.rs:2045…2549`).
   Without it the command has nothing to report and matrix row 10 cannot be
   written (Discovery 3).
2. The new group's implementation lives **under `status_full`** —
   `verbs/status_full/recovery_verb.rs` — registered from `verbs/mod.rs` like
   any other group. This is the smaller of the two ways past Discovery 5: the
   alternative is widening `Ctx`'s fields or adding a `pub(crate)` constructor,
   which loosens a type four other files depend on staying closed. The group
   name on the command line is unaffected.

**Rejected alternatives.**
- *Release exactly the reported crash candidates* — Discovery 1: never releases
  a cleanly-ended or transcript-less session's expired claim.
- *Derive the mark set from the release set* — rejected by D9.
- *Widen `Ctx` so a sibling group can call the detector* — rejected above.
- *Move the trio check after the claim check so the report widens* — out of
  scope: it changes what `bee status --full`'s recovery block reports, which no
  decision asks for. Deferred idea.
- *Write the dead mark from the sweep* — rejected by D7.
- *Clear the mark in `state_sync.rs` only* — Discovery 7: leaves the
  UserPromptSubmit path renewing a heartbeat under a stale mark.

**Risk map.**

| Component | Risk | Reason | Proof needed |
|---|---|---|---|
| Sweep return-type change | MEDIUM | 15 call sites; a mis-migrated site silently drops facts the command reports | Compiler across all 15, plus a test that the returned sets match the decision rows the sweep already logs |
| Mark-set pass and its race | MEDIUM | A record judged stale before the lock could be marked after its owner's heartbeat lands; the hooks' `Busy => Ok(())` drops a contended renewal rather than queueing it | Test: a record whose heartbeat lands between the read and the lock is NOT marked |
| Door width | MEDIUM | The verb writes session records (control plane) and parks units (store) | Test: `recovery scan` runs from an ordinary checkout, a granted worktree and an ungranted one; a unit not readable in the sweeping store is reported unreachable, never written elsewhere (R100) |
| Registry flip + `--json` | MEDIUM | The tripwire runs `bee recovery scan --json` verbatim against the served entry | `registry_dispatch.rs` green with `scan` served and `window` refusing |
| Three-set output | MEDIUM | Three lists in one report invite a reader to read them as one | Test: a cleanly-ended session holding an expired claim is RELEASED, is MARKED, and is NOT a reported crash candidate |
| D8 across two writers | MEDIUM | A test against one writer goes green while the other still leaves the mark | Test per writer: `state_sync.rs` path and `prompt_context.rs` path each clear the mark and stamp `revived_at` |
| Dead-mark field | LOW | Additive fields under a lock the writer already holds | Test: no record deleted; transcript pointer and lane preserved |

## Shape

**Feature outcome.** A crashed or abandoned session's held work frees itself
when anyone runs `bee recovery scan`: its expired claims are released and their
units parked, its record says it is dead whether or not it held anything, and
the command separately reports which sessions still carry unmined work. A
session that comes back stops being described as dead on its next heartbeat,
by whichever path that heartbeat arrives on.

| Epic | Capability / Risk Area | Why It Exists | Slices | Proof Needed |
|---|---|---|---|---|
| **E4** | The releasing door | D3 — the registry advertises `recovery scan` and refuses it | 1 | Releases on the sweep criteria, marks per D9, reports candidates as a distinct set, serves `--json`, runs from a granted worktree; `recovery window` still refuses |
| **E3** | The dead mark, both directions | D2/D7/D9 written by that door only; D8 cleared by both heartbeat writers | 1 | Every stale record marked with `dead_at`, live record untouched, no record deleted, mark cleared with `revived_at` on either heartbeat path |

**Slice queue.** One slice, three cells.

- **`srd-1`** — the sweep's return value (enabling change 1). Signature and
  call-site migration only; criteria, ordering and locking untouched. Depends
  on nothing.
- **`srd-2`** — the `recovery` group: the FULL door, the release pass, the D9
  mark pass with its under-lock re-judgement, the candidate report, the text
  and `--json` shapes, the `recovery.scan` marker flip and description
  correction, and the `docs/handbook/register.md:304,384` rows. Depends on
  `srd-1`.
- **`srd-3`** — D8's revival clearing in **both** heartbeat writers, and
  repointing `orient`'s D6 decline text plus the two stale comments that make
  the same now-false claim. Depends on `srd-2` (a FIX line may not name a
  command that does not exist).

srd-1 was split out of the old single cell precisely because it is a
mechanical 15-call-site migration whose red is unrelated to the new command's
logic; capping it first gives srd-2 a green base and a settled signature.

**Cell-facing specifics** (stated here so the plan is the single source):

- Module: `verbs/status_full/recovery_verb.rs`, registered from `verbs/mod.rs`
  with one `pub mod`-adjacent chain entry, following `verbs/timings.rs:46-58`
  for argv matching including `--json`.
- The FULL door is `resolve_store_root_worktree` (`roots.rs:580`), matched as
  `RootsWt::Go`. Do not use `resolve_store_root_any` — `roots.rs:645-653`
  forbids it to verbs reading sessions or claims.
- `control` for the sweep call derives the same way `orient` derives it:
  `sweep_cells::control_root(&root)` (`claims.rs:42-45`), not
  `StoreRoots::main_root()`.
- Caller resolution and decline (R98 is NOT inherited, Discovery 6): resolve
  `resolve_session_flag_env(None)` then `resolve_session_adopt(&control)`,
  exactly as `orient.rs:260-261`; when both fail, report the counts observed
  and write nothing — no release, no mark.
- Error bridge: `cells::MR<T> = Result<T, Fail>` crosses into the group's own
  type the way `orient.rs:205-207`'s `bridge_sweep_fail` does. Map, never
  unify.
- Lock helper for the mark pass: `cells::acquire_sessions_lock_bounded`
  (`handlers_select.rs:111`, bounded retry) — not the hooks'
  single-attempt fail-open `acquire_store_lock_once`, because a dropped mark
  is a silent miss where a dropped heartbeat touch is merely late.
- `revived_at` holds only the most recent revival (CONTEXT.md leaves the shape
  to discretion; this is the choice, and matrix row 5 assumes it).
- Only `recovery.scan`'s `unavailable` marker is removed; `recovery.window`
  keeps its marker and description untouched. `recovery.scan`'s description
  must lose "Cheap and side-effect-free — never triggers mining".
- The D6 decline text is the `blockers` string at `orient.rs:268`; two comments
  at `orient.rs:193` and `:238` repeat the same now-false "only sweep trigger"
  claim and are corrected with it.
- Knowledge sync owed at capture: `docs/knowledge/areas/workflow-state/recovery.md`,
  and R101 in `claims-and-ownership.md:184-189`, which enumerates the sweep's
  triggers and gains a third.

## Test matrix

High-risk — the 12 dimensions. Judge existing coverage first
(`verbs/status_full/tests.rs`, `verbs/cells/tests.rs`, `tests/registry_dispatch.rs`).

| # | Dimension | Cell | Probe |
|---|---|---|---|
| 1 | User types | srd-2 | Caller with a session id; caller with none (declines, writes nothing — Discovery 6); claim whose own session is null |
| 2 | Input extremes | srd-2 | Session record with no `last_heartbeat`; record already marked dead; record carrying an unknown `status` value |
| 3 | Timing | srd-2, srd-3 | Heartbeat exactly at the staleness boundary; a heartbeat landing between the mark pass's read and its lock (must NOT mark); dead → revived → stale again |
| 4 | Scale | srd-2 | Many session records — one unreadable record must not abort the pass |
| 5 | State transitions | srd-3 | live → dead → revived; `dead_at` never survives a revival; `revived_at` holds the most recent revival only |
| 6 | Environment | srd-2 | `recovery scan` from an ordinary checkout, a granted worktree and an ungranted one; a unit not readable in the sweeping store is reported unreachable and written nowhere (R100) |
| 7 | Error cascades | srd-2 | Corrupt session record, corrupt claim, missing transcript — each skips loudly; the pass completes |
| 8 | Authorization | srd-2 | `recovery scan` never releases its own caller's claim and never marks its own session dead |
| 9 | Data integrity | srd-1, srd-2 | No session record deleted; transcript pointer and lane preserved; the sweep's returned sets match the decision rows it already logs |
| 10 | Integration | srd-2 | A cleanly-ended session holding an expired claim is RELEASED, is MARKED, and is NOT a reported crash candidate; `registry_dispatch.rs` green with `scan` served and `window` refusing |
| 11 | Compliance | — | No regulatory surface |
| 12 | Business logic | srd-2, srd-3 | `orient` and `claim-next` still write no session record (D7); `orient`'s decline text names `bee recovery scan` and that command runs |

Declared suite at every cap:
`PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml`.

## Out of scope

- `bee recovery window` — stays unbuilt per D3, marker and description untouched.
- Widening the crash-candidate report so cleanly-ended or transcript-less
  claim-holders appear in it — deferred idea; it would change
  `bee status --full`'s existing block.
- Any change to the sweep's release **criteria**, ordering or locking. Its
  return type changes; what it decides does not.
- Widening `status_full::Ctx`'s visibility.
- Pid-liveness on session records; closing the >900s heartbeat gap.
