# stdi-21 — check of stdi-19 (regen) and stdi-20 (knowledge)

Round 1. I checked d2ac1ce50 (`cell: stdi-19`) and 11b28daa9 (`cell: stdi-20`) against CONTEXT.md D1 and D7 to D10. Line numbers are at 318e6bd48.

Verdict: pass. I found no defects. Three notes follow the truths. None of them fails a truth.

## Truths

- PASS — every rendered copy matches its source. The cell verify exits 0. `bee onboard --repo-root . --json` reports `"status": "up_to_date"` with an empty `plan`, and all three skill targets (`repo-claude`, `repo-agents`, `repo-opencode`) have zero items and no block. `bee dev release-manifest --check` prints `970 file(s) match stored manifest, 1 unhashed artifact(s) present` and exits 0.
- PASS — no Skillstation hub alias under `.claude/skills` changed in d2ac1ce50. Twenty-one files under `.claude/skills` carry `skillstation_alias` (for example `bee-knowledge-upkeep`, `bee-design`, `product-description`). `git diff --name-only d2ac1ce50~1 d2ac1ce50` over those files lists none. The commit touches only `.claude/skills/{bee-herding,bee-hive,bee-planning,bee-swarming}` and `.claude/skills/.bee-render.json`. None of those four `SKILL.md` files was an alias in the parent. The alias `.claude/skills/bee-knowledge-upkeep/SKILL.md` was last changed in e529bdc51, before this feature. Only its source hash in `.bee-render.json` moved, which is the render record and not the alias.
- PASS — the two concepts state D7 to D9 as the code behaves. They cite semantic-token-diet D-numbers and add no raw decision ids.
  - D7: `docs/knowledge/areas/hook-runtime/token-economy-of-injected-context.md:123-130` matches `TRAILER_LINE` at `packages/bee-rs/crates/bee/src/hooks/session_preamble/budget.rs:519`. No `Proof-per-change-type` text is left in `src/`; the only hit is the negative pin at `hooks/session_preamble/tests.rs:295`.
  - D8: `token-economy-of-injected-context.md:132-136` and `docs/knowledge/areas/bee-herding/the-run-verb-and-worker-outcomes.md:857` match the `spec.cell_id.is_some()` guard at `packages/bee-rs/crates/bee/src/herding/mailbox.rs:343-347`. The test at `mailbox.rs:1661` pins it.
  - D9: `token-economy-of-injected-context.md:138-145` matches `flags:` then `details:` in `packages/bee-rs/crates/bee/src/verbs/help.rs:634-646` (pinned at `help.rs:931`). The `herding.control-loop` description in `src/generated/registry_payload.json` names dispatch, merge, route, supervisor and broker.
  - Ids: the decisions lines in both concepts name `semantic-token-diet D7`, `D8` and `D9`. The added lines of 11b28daa9 carry no `f21a5bf3`, `bf647234`, `cf38764c` or `2a2604e3`. The two raw UUIDs on the token-economy decisions line (`691fcac4…`, `8f63adb4…`) were already there at 11b28daa9~1.
- PASS — `docs/history/semantic-token-diet/delivery.md:12-21` names a delivering cell for each of D1 to D10. Each commit it lists belongs to the cell named beside it. Most carry a `cell:` trailer. b519417c6 (stdi-1), 66ca1f554 (stdi-3) and 4a97259bc (stdi-15) have `cell: <id>` in the body above the Co-Authored-By line, so git's trailer parser misses them, but the cell is named.

## Notes (not defects)

- `.bee/bin/bee` in this worktree links to the main checkout's binary. That binary is older than 73d8810d5: `bee herding control-loop --help` still prints the old long description with decision ids and "dispatch or merge". The source and the concept are right. The help that a user sees changes only after the binary is reinstalled from this branch.
- The token-economy quote of `TRAILER_LINE` (lines 124-127) drops the backticks around the two commands. The wording is otherwise the same as the code.
- The D8 row in delivery.md lists 66ca1f554 only. 90112fcda (`cell: stdi-3`, a let-chain cleanup of the same guard) landed after stdi-20 and is not listed. Behavior is unchanged.

## Verify

`.bee/bin/bee onboard --repo-root . --json && .bee/bin/bee dev release-manifest --check && .bee/bin/bee knowledge check` exits 0. Tail of the output:

```
release_manifest --check: 970 file(s) match stored manifest, 1 unhashed artifact(s) present
knowledge check: 328 concept(s) in 349 file(s), 0 OKF error(s), 0 profile error(s), 601 profile warning(s) — OK
```
