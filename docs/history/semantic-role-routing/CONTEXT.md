# Semantic Role Routing — Context

**Feature slug:** semantic-role-routing
**Date:** 2026-09-14
**Shaping session:** complete
**Scope:** high-risk

## Feature Boundary

Make role assignment an approved planning output and enforce it at execution doors.
The planner reads the complete configured role roster and each role description.
It classifies each workflow stage as required, conditional, or not applicable.
It then assigns each applicable stage and each cell to the semantically correct role.

The approved plan stores these assignments. Dispatch preparation and native model
guards reject a different role. A controlled re-route records why a cell role changed.
The deterministic role-to-agent resolver remains unchanged.

Known lifecycle work also follows the plan. In particular, an approved release uses
the configured `deploy` role. Independent review remains a separate, user-invoked
stage.

This feature also repairs the Pi navigation defect found while entering this
worktree. Pi must replace the active session when it moves directly between two
Bee-managed worktrees. The repair must preserve all transition authenticity checks.

## Locked Decisions

| ID | Decision | Rationale |
|----|----------|-----------|
| D1 | Role assignment is a planning output. The planner compares work meaning with the complete configured role roster and descriptions before execution. | Dispatch-time selection let transport or habit replace semantic selection. Decision `fc2bb09a`. |
| D2 | The runtime-specific role plan classifies every workflow stage as `required`, `conditional`, or `not-applicable`. Every row names the configured role it considered and explains the choice. The rows cover the complete roster. | Reviewers must see omissions as explicit decisions, not missing rows. |
| D3 | The approved gate packet stores the role plan. Cell dispatches use the cell role. Non-cell dispatches use the approved stage role. A mismatched role is a typed refusal. | Guidance alone cannot enforce D1. Decision `68bc3484`. |
| D4 | A cell role can change only through a structured re-route operation. The operation cites a decision tagged `role-reroute`, records the old and new roles, and refuses a claimed or capped cell. | Free-form prose is not safe input for mechanical enforcement. |
| D5 | Semantic selection does not use keyword matching. After selection, the existing deterministic role-to-agent resolver stays the single resolver. | Bee must not add a second mapping system or change transport selection. |
| D6 | Release execution requires a fresh, one-use dispatch authorization for the approved `deployment` stage and its `deploy` role. The authorization binds the version, plan, and main commit. `scripts/release.sh` remains the release implementation. | Release `2.37.2` proved that a direct script call can bypass a configured deploy role. Decision `c0a4d406`. |
| D7 | Existing plans remain valid. Plans without a role-plan block keep legacy behavior. New `bee-plan/v2` plans require the block and receive enforcement after Gate 2 approval. | The change must not invalidate historical plans or open lanes. |
| D8 | Pi worktree navigation supports a verified linked-worktree-to-linked-worktree transition. The emitted `sourceCwd` is the active worktree root. Source path, target path, session ID, grant, and Git-link checks remain strict. | Main-only `worktree enter` produced a marker whose source differed from Pi `ctx.cwd`. Decision `bf919268`. |
| D9 | Independent review stays conditional and user-invoked. Planning can assign `review`, but completion never starts review automatically. | Semantic routing must not turn review into an automatic workflow stage. |

## Existing Code Context

### Reusable Assets

- `packages/bee-rs/crates/bee/src/verbs/state_group/plan_packets.rs` parses,
  validates, hashes, previews, and stores approved cell packets.
- Cell packets already require a non-empty `role` and compare it during add.
- `packages/bee-rs/crates/bee/src/verbs/drivers/prepare.rs` is the dispatch door.
- `packages/bee-rs/crates/bee/src/hooks/model_guard.rs` protects native dispatch.
- `packages/bee-rs/crates/bee/src/verbs/cells/handlers_write.rs` owns cell updates.
- `scripts/release.sh` is the only release implementation.
- `packages/bee-rs/crates/bee/src/verbs/worktree/handlers.rs` emits transition intent.
- `.pi/extensions/bee-guard.ts` validates and performs Pi session replacement.

### Established Patterns

- Role names are an open set defined by `team.<runtime>` configuration.
- A cell role is the cell's sole model selector.
- Gate preview stores immutable plan bytes by SHA-256.
- Pi performs relocation with `SessionManager.forkFrom` and `ctx.switchSession`.
- Gate bypass changes approval ownership. It does not remove preview validation.

## Out of Scope

- Keyword or regex job classification.
- A second role-to-agent resolver.
- Automatic independent review.
- A new release implementation.
- Weaker Pi transition validation.

## Handoff Note

`CONTEXT.md` is authoritative. The earlier docs-only plan was superseded by decision
`68bc3484`. Existing documentation commits can remain as groundwork, but they do
not complete this feature.
