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
