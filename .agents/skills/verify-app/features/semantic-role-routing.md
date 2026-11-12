# Semantic role routing

Semantic role routing makes role assignment an explicit, verified planning output
and enforces it at dispatch preparation, native agent guards, and release execution.
Plans under `bee-plan/v2` require a structured `Role assignments` section covering
the configured runtime roster; cell and non-cell dispatches must obey the approved
roles, cell role changes require a structured `bee cells reroute` record tied to a
decision tagged `role-reroute`, and release execution requires an authorized deploy
dispatch permit.

## Sub-features

- `v1-plan-compatibility` keeps legacy `bee-plan/v1` and plans without role blocks valid and unenforced.
- `v2-role-plan-visibility` requires a complete runtime role block for `bee-plan/v2` and exposes it in gate preview.
- `dispatch-runtime-refusal` refuses dispatches for a feature whose approved role plan names a different runtime.
- `dispatch-stage-refusal` refuses non-cell dispatches with missing, unknown, or not-applicable stages under an approved v2 plan.
- `dispatch-cell-role` derives the planned cell role for cell dispatch, refusing an explicit mismatched `--role`.
- `conditional-review-dormant` ensures conditional stages such as independent review remain dormant until explicitly invoked.
- `cell-reroute-structured` updates an open or blocked cell role when citing a decision tagged `role-reroute`.
- `cell-reroute-refusals` rejects reroute requests on claimed or capped cells, untagged decisions, or unconfigured roles.
- `deploy-permit-stale-replay` validates release permits, rejecting stale, replayed, or forged authorizations.
- `release-direct-refusal` stops direct execution of `scripts/release.sh` without a valid deploy authorization permit.
- `release-authorized-preflight` authorizes release preflight via `bee dispatch authorize` without publishing an unapproved version.
- `worktree-linked-relocation` supports verified Pi session replacement when transitioning directly between two Bee-managed worktrees.

## How to get to it (user POV)

- Author `plan.md` with frontmatter `artifact_contract: bee-plan/v2` and a `## Role assignments` block.
- Run `bee gate --preview --json` to preview cell and role plan assignments.
- Run `bee gate --name execution --approved true --actor user --reason "approved"` to approve Gate 2.
- Run `bee dispatch prepare --runtime <rt> --kind gather --feature <f> --stage <stage> --role <role> --json`.
- Run `bee dispatch prepare --runtime <rt> --kind cell --cell <id> --worker <name> --claim --json`.
- Run `bee cells reroute --id <id> --role <role> --decision <id> --json`.
- Run `bee dispatch prepare --runtime <rt> --kind gather --feature <f> --stage deployment --role deploy --release-version <semver> --json`.
- Run `bee dispatch authorize --id <dispatch-id> --release-version <semver> --json`.
- Run `bash scripts/release.sh <semver>` (invoked by deploy role or CI).
- Run `bee worktree enter --id <other-worktree> --json` from inside a linked worktree.

## Driving it with control-bee

Preconditions:

- A launched sandbox, `control-bee doctor` fully `ok`.
- An active feature and pipeline.

- **Preview legacy v1 plan.** Put a plan with `artifact_contract: bee-plan/v1` and no role block at `docs/history/<feature>/plan.md`. Run `control-bee cli -- gate --preview --json`. The payload has no `ok` key; it reports a valid `plan_sha256`, parsed `cells[]`, and `role_plan: null`. `fixtures/demo-note-plan.md` is a plan without a role block.
- **Enforce v2 role plan requirements.** Put a plan with `artifact_contract: bee-plan/v2` but omitting the role section. Run `control-bee cli -- gate --preview --json`. It refuses with a non-zero exit and an error sentence that carries the code: `gate preview: role_plan_required: …`.
- **Preview and approve v2 role assignments.** Put `fixtures/role-plan-v2.md` — its `roster_sha256` is `REPLACE_FROM_ROSTER_STALE_ERROR` — and run `control-bee cli -- gate --preview --json` once. It refuses `role_plan_roster_stale`; copy the expected digest out of that error into the fixture's `roster_sha256`, put it again, and preview again. The payload carries `role_plan` containing `runtime`, `roster_sha256`, and `stages[]`. Approve Gate 2 with `control-bee cli -- gate --name execution --approved true --actor user --reason "approved" --json`.
- **Refuse mismatched dispatch runtime.** With an approved Pi role plan, run `control-bee cli -- dispatch prepare --runtime claude --kind cell --cell demo-1 --worker w1 --claim --json`. On an open cell, a call without `--claim` refuses `claim_ownership` first; after any refused probe run `control-bee cli -- cells unclaim --id demo-1 --json` (a backlog row records that the refusal still claims). The payload reports `ok: false`, `reason: "role_plan_runtime_mismatch"`.
- **Refuse missing or not-applicable non-cell stage.** Run `control-bee cli -- dispatch prepare --runtime pi --kind gather --feature demo --json` (omitting `--stage`). The payload reports `ok: false`, `reason: "stage_required"`. Run with `--stage generic-advisor` (classified not-applicable). It reports `ok: false`, `reason: "stage_not_applicable"`.
- **Enforce cell role equality.** Run `control-bee cli -- dispatch prepare --runtime pi --kind cell --cell demo-1 --worker w1 --role review --claim --json` on a cell whose planned role is `code`. The payload reports `ok: false`, `reason: "planned_role_mismatch"`.
- **Perform structured cell reroute.** Log a decision tagged `role-reroute`: `control-bee cli -- decisions log --decision "Switch cell to test" --rationale "TDD proof" --relation none --tags role-reroute --feature <f> --json`. Without `--feature <f>`, reroute refuses `role_reroute_decision_feature_mismatch`. Read its `id`. Run `control-bee cli -- cells reroute --id demo-1 --role test --decision <decision-id> --json`. The payload is the cell, with updated `role: "test"` and `role_reroutes[]` containing the previous role, new role, decision id, and plan hash.
- **Refuse invalid cell reroutes.** Attempt to reroute demo-1 again with `--role test` (unchanged); it reports `role_unchanged`. Attempt to reroute with an unconfigured role; it reports `role_reroute_unconfigured_role`. Claim the cell with `control-bee cli -- cells claim --id demo-1 --worker w1 --json` and attempt reroute; it reports `role_reroute_status`.
- **Refuse direct release script execution — unreachable in the sandbox.** The sandbox has no `scripts/release.sh`: `control-bee sh -- bash scripts/release.sh 9.9.9` exits 127. Report the step unreachable with that unmet precondition.
- **Authorize deploy permit and reject replay.** The permit needs `team.pi.deploy` configured (the default sandbox has none), branch `main`, a lane feature with its conflict check derived, and the issuer session registered through `hook session-init` with `session_id: verify-<run-id>`. Run `control-bee cli -- dispatch prepare --runtime pi --kind gather --feature demo --stage deployment --role deploy --release-version 9.9.9 --json`. Read `dispatch_id`. Run `control-bee cli -- dispatch authorize --id <dispatch-id> --release-version 9.9.9 --json`. The payload reports `authorized: true`. Run the exact same authorize command a second time; it refuses with `reason: "deploy_authorization_consumed"`.
- **Relocate between linked worktrees in Pi.** Create worktrees `wt-a` and `wt-b`. Symlink `.bee/bin/bee` in the sandbox. Launch an installed Pi 0.84.x runtime in `wt-a` with `.pi/extensions/bee-guard/` (entry `index.ts`, commands in `commands.ts`, transition in `transition.ts`) active. Trigger worktree enter for linked worktree `wt-b` through Pi via `/bee-worktree-enter --id repo--wt--wt-b`. Verify that the extension calls `ctx.switchSession` (in `transition.ts`) and changes the active Pi session to `wt-b` while retaining conversation history, and verify that same-worktree enter is refused. A CLI transition marker alone is not acceptance.

## Gotchas

- **V2 plans require complete roster coverage.** The `Role assignments` stages table must list every configured role in `team.<runtime>`, including not-applicable ones; every stage role must be configured for the runtime, not-applicable ones included (`role_plan_unconfigured_role`), and missing a single role causes Gate 2 preview refusal.
- **Reroute citations require the role-reroute tag.** A decision without `--tags role-reroute`, or one logged without `--feature <f>`, is rejected by `cells reroute` even if its text discusses routing.
- **Deploy authorizations are single-use.** Once consumed by `bee dispatch authorize`, a permit cannot be re-used; a restart of release preparation requires a fresh deploy dispatch.
