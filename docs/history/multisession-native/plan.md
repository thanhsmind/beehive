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
