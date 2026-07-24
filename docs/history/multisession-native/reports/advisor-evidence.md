# CONTEXT — multisession-native

**Feature:** Rebuild bee's coordination core from multi-session-aware to multi-session-native, per the architecture review in GitHub issue #56 (written against v1.11.2, re-verified against v1.16.1 on 2026-07-24).

**Mode:** high-risk (core state/lock/guard overhaul; every session and hook touches these paths).

**Verification baseline:** issue #56's nine findings were re-checked against v1.16.1 before locking these decisions — 7/9 still true verbatim, 2/9 half-fixed (heartbeat now locked but `bindSessionLane` is not; lock takeover is pid-aware but `worktree-admin` still spans a multi-minute verify). See the review digest in this feature's reports/.

## Locked decisions

### D1 — Workflow-first state (source of truth moves off state.json)
A **workflow** record (`.bee/runtime/workflows/<workflow-id>/state.json`) becomes the unit of state: `id`, `feature`, `phase`, `mode`, `plan_rev`, `gates`, `summary`, `next_action`, `status`. `workflow_id` is a generated id, **not** the feature slug (a feature can reopen or run competing attempts). The legacy `.bee/state.json` and `.bee/lanes/*.json` become read-only compatibility projections rebuilt from workflow records. `startFeature` creates a new workflow instead of seizing a global pipeline; its lock becomes `workflow:<id>`, ending cross-feature contention on the single `state` lock (issue 3.1/3.2).

### D2 — Control plane / data plane split (`resolveContext()`)
Replace `resolveRoots()` with `resolveContext(cwd) => { projectRoot, controlRoot, workspaceRoot, localRuntimeRoot, gitCommonDir, workspaceId, worktreeId }`. `controlRoot` (session records, workflow state, claims, leases, workspace registry, integration queue) is **shared across all worktrees**; `workspaceRoot` is the physical checkout; `localRuntimeRoot` holds caches/temp/derived state that never needs sharing. Principle: control plane shared, data plane isolated.

### D3 — Second write session defaults to isolation
Three policy modes: `observe` (read/analyze/review — unlimited concurrent sessions, same checkout), `shared-disjoint` (opt-in only: exact-path leases mandatory, no broad writes), `isolated` (default whenever a write-capable session already lives: bee auto-creates a worktree workspace instead of answering "another session is active, wait"). A workspace has exactly one `write_owner_session`; others attach read-only.

### D4 — Sharded leases replace `reservations.json`
Per-resource lease records: `controlRoot/leases/cells/<cell-id>.json`, `controlRoot/leases/paths/<prefix>/<path-hash>.json` with `{resource, mode, workflow_id, session_id, workspace_id, epoch, acquired_at, expires_at}`. Acquire = canonical-normalize, hash-sort (deadlock-free), exclusive-create each, roll back on partial failure. **Intent scope** (planning-declared, warn/schedule only) is separated from **write lease** (hard block). Conflict policy: same workspace → hard conflict; different workspace → advisory warning, hard block only for resources explicitly marked exclusive (migrations, schema snapshots, release files, lockfiles, generated clients). Stale takeover honors epoch so a zombified session cannot write with an old lease. Ownership key is `session/workflow/workspace/cell`; `agentName` is display-only. (Issues 3.3/3.4/3.5.)

### D5 — Handoff mailbox per workflow
`controlRoot/handoffs/<workflow-id>/<seq>.json` with `{id, workflow_id, from_session, target_role, previous_cell, next_cell, claim_epoch, kind, status}` replaces the single `.bee/HANDOFF.json`. One workflow pausing never blocks another; reviewer handoffs never clobber implementer handoffs. Existing adopt semantics (transfer claim first, clear handoff idempotently) carry over per-mailbox. (Issue 3.7.)

### D6 — Workers derived, never stored
Drop the hand-mutated `state.workers` array. "Active workers" = live-heartbeat sessions joined with their `workflow_id`, `current_cell`, and cell claims. `state worker add/update/remove/clear/prune` verbs retire into compatibility no-ops that read the derived view. (Issue 3.6/mục 8.)

### D7 — Gates scoped to plan revision
Gate approval records `approved_for_plan_rev`. Plan rev bump invalidates only that workflow's execution gate; no other workflow is touched. (Mục 9.)

### D8 — Five-stage migration, each stage ships green alone
1. **Telemetry** — measure contention before changing behavior: `{lock_wait_ms, lock_name, holder_session, caller_session, workflow_id, workspace_id, resource, result}` appended fail-open; status answers "why is this session waiting".
2. **Workflow-first state** (D1, D6, D7) behind compat projections.
3. **Shard runtime stores** (D4, D5) — leases + mailboxes; `bee-state-sync` writes projections only, never source of truth.
4. **Workspace isolation default** (D2, D3).
5. **Integration queue** — serialized merge/integration as its own queue, no filesystem lock held across long operations.
Compatibility projections are maintained until stage 5 closes; deleting a projection must never lose source of truth.

### D9 — Issue #56's 15 invariants are the acceptance suite
Each invariant lands as an automated test; the feature is complete only when all 15 are green. Highlights: two workflows plan concurrently without sharing a lock; heartbeat can never erase a lane/workflow binding; hooks never wait on store locks; no long operation holds a filesystem lock; overview rebuilds fully from records; no write-capable operation runs without `session_id`+`workflow_id`+`workspace_id`.

### D10 — Stage 0 quick wins land first, independently
(a) Wrap `bindSessionLane`/`unbindSessionLane` in the same `sessions` store lock as `heartbeatSession` (`claims.mjs:297-319`) — closes the remaining lost-update race (issue 3.8). (b) Restructure `mergeFeatureWorktree` so the `worktree-admin` lock is released around the multi-minute verify child and re-acquired for the write step (`worktree-store.mjs:1300,1525`) — no lock across subprocesses (issue 3.9). Both ship before stage 1.

### D11 — Philosophy preserved
No daemon, no message broker, no external orchestration. JSON files, atomic writes, `O_EXCL` locks, CLI-driven. Bee stays a control plane (identity, ownership, state) — Claude Code/Codex remain the executors.

## Scope boundaries
- Skills-chain semantics (gates, phases, lanes-as-UX) unchanged for the user; only the storage/locking substrate moves.
- No behavior change lands in a stage before its telemetry/compat groundwork (stage order is a dependency chain).
- Issue 3.4's illustrative example is factually wrong (`src/api/users/*` vs `src/api/orders/x.ts` does not conflict today) — the general prefix/glob pessimism claim stands and is what D4 addresses.

## Outstanding questions
None blocking — all product decisions auto-resolved to the issue's recommended architecture under total autopilot; recorded in decisions log.
# Plan — multisession-native (frozen at Gate 2)

**Mode:** high-risk. **Source decisions:** CONTEXT.md D1-D11 (issue #56, re-verified against v1.16.1 — reports/issue-56-verification.md).

**Goal:** move bee from multi-session-aware to multi-session-native: workflow-first state, shared control plane / isolated data plane, sharded epoch leases, per-workflow handoff mailboxes, derived workers, plan-rev-scoped gates — in five independently-shippable stages plus a stage-0 bugfix pair, closed by the 15-invariant acceptance suite (D9).

**Delivery rhythm:** each slice ends green on `node scripts/run_verify.mjs --impacted`, ships as its own release-able increment, and keeps compatibility projections alive until slice 5 retires them. Slices are a strict dependency chain (D8).

---

## Slice 0 — quick wins (2 cells, land first, no redesign dependency)

| Cell | Work | Verify |
|------|------|--------|
| msn-1 | Wrap `bindSessionLane`/`unbindSessionLane` (`claims.mjs:297-319`) in the same `sessions` store lock `heartbeatSession` uses; add a race regression test (bind during heartbeat read-modify-write → binding survives) | `node scripts/run_verify.mjs --impacted` |
| msn-2 | Restructure `mergeFeatureWorktree` (`worktree-store.mjs:~1300-1525`) so `worktree-admin` lock is released before the verify `spawnSync` child and re-acquired for the post-verify write step, with a re-check of preconditions after re-acquire; test asserts no lock file exists while the child runs | `node scripts/run_verify.mjs --impacted` |

## Slice 1 — contention telemetry (2 cells; measure before changing behavior)

| Cell | Work | Verify |
|------|------|--------|
| msn-3 | Lock-contention telemetry in `lock.mjs`: on every acquire (success, retry, LOCK_BUSY) append `{ts, lock_name, lock_wait_ms, holder_session, caller_session, workflow_id, workspace_id, resource, result}` to `.bee/logs/contention.jsonl`, fail-open, same discipline as `timings.jsonl` | `node scripts/run_verify.mjs --impacted` |
| msn-4 | `bee status` surfaces contention: recent LOCK_BUSY events, top contended lock names, holder/waiter session pairs — answers "why is this session waiting" | `node scripts/run_verify.mjs --impacted` |

## Slice 2 — workflow-first state (D1, D6, D7) — 6 cells

| Cell | Work |
|------|------|
| msn-5 | Workflow store module: `.bee/runtime/workflows/<workflow-id>/state.json` schema `{id, feature, phase, mode, plan_rev, gates:{name→{approved, approved_for_plan_rev}}, summary, next_action, status}`; generated `workflow_id` distinct from feature slug; per-workflow lock `workflow:<id>` |
| msn-6 | `startFeature` creates a workflow record instead of seizing the global pipeline; preconditions re-scoped to the new workflow (feature-name collision against *live* workflows only); legacy lane path becomes an alias onto workflow creation |
| msn-7 | Compatibility projection: `.bee/state.json` and `.bee/lanes/*.json` rebuilt read-only from workflow records; `bee-state-sync` writes projections only, never source of truth; deleting a projection loses nothing (invariants 13/14) |
| msn-8 | Derived workers (D6): active workers computed from live-heartbeat sessions × `workflow_id` × `current_cell` × claims; `state worker *` verbs become compat readers of the derived view |
| msn-9 | Gates per plan revision (D7): approval writes `approved_for_plan_rev`; plan-rev bump invalidates only that workflow's execution gate |
| msn-10 | Migrate all `state set/gate/scribing-run/advisor-ref` target resolution and `bee-prompt-context` reads onto workflow records (session→workflow binding replacing session→lane) |

## Slice 3 — sharded runtime stores (D4, D5) — 6 cells

| Cell | Work |
|------|------|
| msn-11 | Lease store: `controlRoot/leases/cells/<cell-id>.json` + `controlRoot/leases/paths/<prefix>/<path-hash>.json`, record `{resource, mode, workflow_id, session_id, workspace_id, epoch, acquired_at, expires_at}`; acquire = canonicalize, hash-sort, `O_EXCL` create each, rollback on partial failure |
| msn-12 | Epoch fencing: claims and leases carry `epoch`; a stale session's write with an outdated epoch refuses (invariant 10) |
| msn-13 | Intent-scope vs write-lease split (D4): planning-declared paths become advisory intent records (warn/schedule); hard blocks only from exact-path write leases |
| msn-14 | Cross-workspace policy: same workspace → hard conflict; different workspace → advisory warning; hard block only for resources marked exclusive in config (migrations, lockfiles, release files, generated clients); wire into `guards.mjs` xwh path |
| msn-15 | Handoff mailboxes (D5): `controlRoot/handoffs/<workflow-id>/<seq>.json`; adopt = transfer claim then idempotent clear, per mailbox; single `HANDOFF.json` becomes a projection of the newest open handoff until slice 5 |
| msn-16 | Retire `reservations.json` behind a compat shim: reserve/release verbs write lease records; sweep/renew operate per-record — no whole-file rewrites, no global `reservations` lock |

## Slice 4 — workspace isolation by default (D2, D3) — 5 cells

| Cell | Work |
|------|------|
| msn-17 | `resolveContext(cwd)` replacing `resolveRoots()`: `{projectRoot, controlRoot, workspaceRoot, localRuntimeRoot, gitCommonDir, workspaceId, worktreeId}`; controlRoot = shared across worktrees (`<git-common-dir>/bee/` or `<main-root>/.bee/runtime/control/`) |
| msn-18 | Split stores by plane: session records, workflow state, claims, leases, workspace registry → controlRoot; injection cache, temp, transcript index, derived state → localRuntimeRoot |
| msn-19 | Workspace registry: `{id, type, root, branch, base_sha, write_owner_session}`; one write owner per workspace, others attach read-only |
| msn-20 | Policy modes `observe` / `shared-disjoint` (opt-in) / `isolated` (default): second write-capable session triggers automatic worktree workspace creation instead of "another session is active, wait" |
| msn-21 | Write guard rewired onto `resolveContext` + leases + workspace ownership; drop lane-cache special cases made redundant |

## Slice 5 — integration queue + closure — 4 cells

| Cell | Work |
|------|------|
| msn-22 | Integration queue: merge/integration requests serialized through `controlRoot/integration/queue/`, processed without holding any filesystem lock across long operations (invariant 12) |
| msn-23 | Acceptance suite: all 15 invariants from issue #56 as automated tests (concurrent planning, no cross-workflow gate bleed, heartbeat-vs-binding, hook try-once, epoch fencing, projection rebuild, actor-context required) — every one green |
| msn-24 | Retire legacy: delete compat shims scheduled for removal, final projection semantics documented, `docs/knowledge/` workflow-state area rewritten to the new model |
| msn-25 | Release: version bump, migration notes (existing `.bee/` auto-upgrades on onboard), close issue #56 with the invariant results |

---

## Risks
- **Blast radius:** every hook and CLI verb touches state/lock paths — mitigated by stage order, compat projections until slice 5, and the invariant suite growing from slice 2 onward.
- **Concurrent-session testing is inherently racy** — invariant tests must use deterministic interleaving hooks (inject wait points), not sleeps.
- **controlRoot in git-common-dir** changes what `.bee/` means to existing checkouts — onboarding must migrate in place, idempotently.
- **Flaky suite disclosed at v1.16.1 release** (one unexplained red) — capture logs on any red during this feature; do not build on red (CI gate rule).

## Out of scope
- Skills-chain UX changes (gates/phases stay as the user sees them).
- Daemons, brokers, external services (D11).
- Herding cockpit changes beyond what resolveContext forces.

## Cell/PBI policy
Slice 0-1 cells (msn-1..4) are created now and are the current work queue. Slices 2-5 are registered as four PBIs carrying their cell tables; each slice's cells are instantiated when the previous slice caps green (plan-rev bump + re-validation per slice, D7 discipline applies to this feature itself).
# Issue #56 claims re-verified against v1.16.1 (2026-07-24)

| # | Claim (v1.11.2) | v1.16.1 status | Evidence |
|---|-----------------|----------------|----------|
| 3.1 | Global `state.json` pipeline; `startFeature` demands repo-wide idle under `state` lock | STILL TRUE | `state.mjs:1768-1866` (preconditions), `:1782` (lock) |
| 3.2 | Lanes are a compat layer; `startLane()` contends on the same `state` lock | STILL TRUE | `state.mjs:1782-1791`; `lock.mjs:88-97` (lock file from name only) |
| 3.3 | `reservations.json` one global file+array, one lock, whole-file rewrites | STILL TRUE | `reservations.mjs:16,162,190,207` |
| 3.4 | `pathsOverlap` prefix/glob pessimism; no intent vs lease split | STILL TRUE mechanism; issue's example wrong (`src/api/users/*` vs `src/api/orders/x.ts` does NOT conflict — bases don't prefix) | `reservations.mjs:55-71`; reused by `worktree-holds.mjs:52-56,205` |
| 3.5 | Cross-worktree holds hard-block all paths, no advisory mode | STILL TRUE | `guards.mjs:659-701` ("a cross-worktree hold is a hard block") |
| 3.6 | `bee-state-sync` writes global aggregate into default `state.json` under `state` lock | STILL TRUE | `bee-state-sync.mjs:80-109` |
| 3.7 | Single global `HANDOFF.json` | STILL TRUE (lane-name filtering added: `state.mjs:1934-1939`; path fixed at `state.mjs:963`) | |
| 3.8 | Heartbeat vs `bindSessionLane` lost-update race | PARTIALLY FIXED — heartbeat now locked (`claims.mjs:241-260`, sessions lock `:401,432-438`); `bindSessionLane`/`unbindSessionLane` still lock-free (`claims.mjs:297-319`) | |
| 3.9 | ~5s lock retry; long op holds lock across multi-minute verify | PARTIALLY FIXED — retry unchanged (`lock.mjs:32-34`); pid-aware stale takeover closed false-steal (`lock.mjs:10-22,193-208`); `mergeFeatureWorktree` still holds `worktree-admin` across `spawnSync(verify)` (`worktree-store.mjs:1300,1525`) | |

Mitigations since v1.11.2 that reduce symptoms without changing primitives: lane overlay (B12/R56), lane-write session auto-resolve (i54-closeout D7), pid-liveness lock takeover (rel180-4). None change the single-`state`-lock, global-store architecture the issue targets.

Issue #56's 15 invariants (mục 15) adopted verbatim as the feature's acceptance suite — see CONTEXT.md D9.
