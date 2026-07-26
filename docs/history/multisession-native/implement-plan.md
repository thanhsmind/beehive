# Implement Plan — multisession-native

> Rendered for Gate 2/3 review. Sources: CONTEXT.md (D1-D11), plan.md, reports/issue-56-verification.md. Mode: **high-risk**.

## What and why

GitHub issue #56 diagnosed bee as "multi-session-aware but not multi-session-native": coordination primitives (sessions, claims, lanes, reservations, worktree holds) accreted around a core that still assumes one repo = one pipeline = one phase = one gate set. Re-verification against v1.16.1 confirmed 7 of 9 findings unchanged and 2 half-fixed. The cost is real: unrelated features contend on one `state` lock, one global `reservations.json`, one `HANDOFF.json`; a second write session is told to wait instead of being isolated.

This feature rebuilds the substrate in five shippable stages plus two immediate bugfixes, keeping bee's no-daemon JSON-file philosophy and leaving the user-visible skills chain untouched.

## Target architecture (locked)

- **Hierarchy:** Project → Workflow → Session → Cell → Workspace. Workflow is the unit of state (own id, phase, gates-per-plan-rev); session is an actor; workspace is the unit of source isolation; cell/lease is the unit of ownership.
- **`resolveContext()`** replaces `resolveRoots()`: shared `controlRoot` (sessions, workflows, claims, leases, registry, integration queue) across all worktrees; per-checkout `workspaceRoot`; per-checkout `localRuntimeRoot` for caches. Control plane shared, data plane isolated.
- **Leases** shard `reservations.json` into per-resource epoch-fenced records; intent scope (advisory) split from write lease (hard); cross-workspace conflicts advisory except explicitly exclusive resources.
- **Handoffs** become per-workflow mailboxes. **Workers** become a derived view. **Gates** bind to plan revisions.
- **Policy modes:** observe (unlimited) / shared-disjoint (opt-in) / isolated (default for a second write-capable session — bee auto-creates a worktree).

## Stage map

| Stage | Ships | Cells | Status |
|-------|-------|-------|--------|
| 0 | Quick wins: `bindSessionLane` sessions-lock fix; `worktree-admin` lock released around merge-verify child | multisession-native-1, -2 | cells created, open |
| 1 | Contention telemetry (`contention.jsonl`) + status surfacing | multisession-native-3, -4 | cells created, open |
| 2 | Workflow-first state, derived workers, plan-rev gates, projections | msn-5..10 | PBI p-3416fb38 |
| 3 | Sharded epoch leases, intent/lease split, handoff mailboxes, reservations shim | msn-11..16 | PBI p-e20d82c9 |
| 4 | resolveContext, control/data plane split, workspace registry, isolation default | msn-17..21 | PBI p-ed2de0d0 |
| 5 | Integration queue, 15-invariant acceptance suite green, legacy retirement, release | msn-22..25 | PBI p-4f055a6f |

Each stage ends green on `node scripts/run_verify.mjs --impacted`, is independently releasable, and keeps compatibility projections until stage 5. Later slices re-enter validation with a plan-rev bump before their cells instantiate.

## Acceptance (D9)

The feature closes only when issue #56's 15 invariants are automated and green — including: two workflows plan concurrently without lock contention; heartbeat can never erase a binding; hooks never wait on store locks; no long operation holds a filesystem lock; stale sessions are epoch-fenced; projections rebuild from records; write operations require full actor context.

## Key risks

1. Blast radius: every hook/verb touches these paths → stage ordering + projections + invariant suite growing from stage 2.
2. Race testing: deterministic interleaving hooks, never sleeps.
3. `controlRoot` migration for existing checkouts: onboard must upgrade in place, idempotent.
4. Known flaky suite (v1.16.1 release disclosure): capture logs on any red; never build on red.

## Immediate next

Validate stage 0-1 (feasibility spike on lock re-entry semantics in `mergeFeatureWorktree`), then execute multisession-native-1..4.
