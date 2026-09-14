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

- **Preview legacy v1 plan.** Put a plan with `artifact_contract: bee-plan/v1` and no role block at `docs/history/<feature>/plan.md`. Run `control-bee cli -- gate --preview --json`. The payload reports `ok: true`, valid `plan_sha256`, parsed `cells[]`, and `role_plan: null`.
- **Enforce v2 role plan requirements.** Put a plan with `artifact_contract: bee-plan/v2` but omitting the role section. Run `control-bee cli -- gate --preview --json`. It refuses with `error: "role_plan_required"` and non-zero exit.
- **Preview and approve v2 role assignments.** Put a conforming `bee-plan/v2` plan with a valid `## Role assignments` block covering all configured roles for runtime `pi`. Run `control-bee cli -- gate --preview --json`. The payload reports `ok: true`, `role_plan` containing `runtime`, `roster_sha256`, and `stages[]`. Approve Gate 2 with `control-bee cli -- gate --name execution --approved true --actor user --reason "approved" --json`.
- **Refuse mismatched dispatch runtime.** With an approved Pi role plan, run `control-bee cli -- dispatch prepare --runtime claude --kind cell --cell demo-1 --worker w1 --json`. The payload reports `ok: false`, `reason: "role_plan_runtime_mismatch"`.
- **Refuse missing or not-applicable non-cell stage.** Run `control-bee cli -- dispatch prepare --runtime pi --kind gather --feature demo --json` (omitting `--stage`). The payload reports `ok: false`, `reason: "stage_required"`. Run with `--stage generic-advisor` (classified not-applicable). It reports `ok: false`, `reason: "stage_not_applicable"`.
- **Enforce cell role equality.** Run `control-bee cli -- dispatch prepare --runtime pi --kind cell --cell demo-1 --worker w1 --role review --json` on a cell whose planned role is `code`. The payload reports `ok: false`, `reason: "planned_role_mismatch"`.
- **Perform structured cell reroute.** Log a decision tagged `role-reroute`: `control-bee cli -- decisions log --decision "Switch cell to test" --rationale "TDD proof" --relation none --tag role-reroute --json`. Read its `id`. Run `control-bee cli -- cells reroute --id demo-1 --role test --decision <decision-id> --json`. The payload reports `ok: true`, updated `role: "test"`, and `role_reroutes[]` containing the previous role, new role, decision id, and plan hash.
- **Refuse invalid cell reroutes.** Attempt to reroute demo-1 again with `--role test` (unchanged); it reports `error: "role_reroute_unchanged_role"`. Attempt to reroute with an unconfigured role; it reports `error: "role_reroute_unconfigured_role"`. Claim the cell with `control-bee cli -- cells claim --id demo-1 --worker w1 --json` and attempt reroute; it reports `error: "role_reroute_claimed"`.
- **Refuse direct release script execution.** Run `control-bee sh -- bash scripts/release.sh 9.9.9`. It fails before git mutation or checks with refusal stating deploy authorization is missing.
- **Authorize deploy permit and reject replay.** Run `control-bee cli -- dispatch prepare --runtime pi --kind gather --feature demo --stage deployment --role deploy --release-version 9.9.9 --json`. Read `dispatch_id`. Run `control-bee cli -- dispatch authorize --id <dispatch-id> --release-version 9.9.9 --json`. The payload reports `authorized: true`. Run the exact same authorize command a second time; it refuses with `reason: "deploy_authorization_consumed"`.
- **Relocate between linked worktrees in Pi.** Create worktrees `wt-a` and `wt-b`. Symlink `.bee/bin/bee` in the sandbox. Launch an installed Pi 0.84.x runtime in `wt-a` with `.pi/extensions/bee-guard.ts` active. Trigger worktree enter for linked worktree `wt-b` through Pi via `/bee-worktree-enter --id repo--wt--wt-b`. Verify that the extension calls `ctx.switchSession` and changes the active Pi session to `wt-b` while retaining conversation history, and verify that same-worktree enter is refused. A CLI transition marker alone is not acceptance.

## Gotchas

- **V2 plans require complete roster coverage.** The `Role assignments` stages table must list every configured role in `team.<runtime>`, including not-applicable ones; missing a single role causes Gate 2 preview refusal.
- **Reroute citations require the role-reroute tag.** A decision without `--tag role-reroute` is rejected by `cells reroute` even if its text discusses routing.
- **Deploy authorizations are single-use.** Once consumed by `bee dispatch authorize`, a permit cannot be re-used; a restart of release preparation requires a fresh deploy dispatch.
