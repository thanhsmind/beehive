# herding-tier — Plan (scope B)

**Lane:** standard (public-contracts, covered-contract-change, multi-domain; ~8 product files)
**Source:** docs/history/herding-tier/CONTEXT.md D1–D6 (+ herding-executor D2/D4/D7 cited)
**Date:** 2026-08-20

## Shape — one slice, 4 cells

| Cell | What | Files | Decisions |
|---|---|---|---|
| ht-1 | Resolution layer: `normalize_tier_value` accepts `{kind:"herding"}`; new `Resolved::Herding`; `resolve_tier` returns it for cell purpose and the runtime default model for gather purpose (invert of cli's for_gather). Tests both purposes + malformed shapes. | `verbs/drivers/models.rs`, `verbs/drivers/tests.rs` | D1, D3 |
| ht-2 | Run verb stdin task: `--task-file -` reads the task from stdin (or a smaller-diff equivalent); registry example untouched and green. Tests: stdin task renders the brief; regular file path unchanged. | `herding/run.rs`, `generated/registry_payload.json` (only if the flags block changes) | D4 |
| ht-3 | Dispatch seam: `prepare_dispatch` Resolved::Herding arm — Bash payload, `channel:"herding-exec"`, command `bee herding run --task-file - --json` (+`--cwd` for granted worktrees), prompt in `stdin`; model guard denies Agent/Task on a herding tier with the herding-exec fix (mirror cli-tier-denied). Tests both. | `verbs/drivers/prepare.rs`, `hooks/model_guard.rs` | D4, D5, D6 |
| ht-4 | Surfaces + docs: status_full store/render + onboard/agents.rs display the herding kind; config-reference models section documents `{kind:"herding"}` (cell-only, global agent_command, D2); both samples gain a commented models example; operational-invariants names the config route. wave-barrier regen ack. | `verbs/status_full/store.rs`, `verbs/status_full/render.rs`, `onboard/agents.rs`, `docs/config-reference.md`, `.bee/config-sample.json`, `.bee/config-sample-cli-executors.json`, `skills/bee-herding/references/operational-invariants.md` | D2, D3 |

Deps: ht-1 → (ht-2 ∥ ht-3 ∥ ht-4); ht-2 and ht-3 disjoint files (run.rs vs prepare.rs/model_guard.rs); ht-3 tests may reference ht-2's stdin flag — contract fixed here (D4's exact command string), so parallel is safe.

Proof: per-cell scoped cargo test on touched modules; ht-2 adds `--test registry_dispatch`; ht-4 knowledge/docs checks. Full suite before merge.

## Smaller-path check

Fold ht-2 into ht-3? Different files, parallel wins; the stdin sentinel is one function edit. Shape stands: 4 cells.

## Rollback

Additive kind: configs without `{kind:"herding"}` hit no new code path. Revert = merge revert.
