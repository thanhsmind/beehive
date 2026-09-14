# LaneBrief: Semantic Role Routing

## Question

From the perspective named by your assigned role, is plan revision 1 ready for
Gate 2? Identify material risks, missing facts, and the smallest required
corrections.

## Constraints

- `bee-plan/v2` stores a classified workflow-stage role plan.
- Dispatch preparation enforces cell and non-cell stage roles.
- `bee cells reroute` records controlled cell role changes.
- Native model guards and `scripts/release.sh` block bypass paths.
- Pi can move directly from one granted linked worktree to another.
- The existing role-to-agent resolver remains the only resolver.
- Semantic selection uses model judgment and role descriptions.
- Keyword matching is prohibited.
- Legacy plans remain compatible.
- Independent review remains user-invoked.
- Pi transition source, session, grant, and Git-link checks remain strict.

## Read diet

- `docs/history/semantic-role-routing/CONTEXT.md`
- `docs/history/semantic-role-routing/plan.md`
- `packages/bee-rs/crates/bee/src/verbs/state_group/plan_packets.rs`
- `packages/bee-rs/crates/bee/src/verbs/drivers/prepare.rs`
- `packages/bee-rs/crates/bee/src/hooks/model_guard.rs`
- `packages/bee-rs/crates/bee/src/verbs/worktree/handlers.rs`
- `scripts/release.sh`

## Digest contract

Return the following items:

1. Material risks or missing facts from your assigned perspective.
2. The smallest correction required before Gate 2.
3. A verdict: `ready`, `ready-with-corrections`, or `not-ready`.
4. Exact file or plan-section anchors for each correction.
