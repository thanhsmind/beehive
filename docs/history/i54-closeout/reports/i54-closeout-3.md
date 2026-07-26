# i54-closeout-3 — report

**[DONE]** — `knowledge context` gains an optional `--lane tiny|small|standard|high-risk`
shorthand that resolves to a budget preset (8k/12k/20k/30k) before the generic
flag validator runs, so explicit `--budget` always wins and a bare call with
neither flag refuses exactly as before this cell. The session preamble's
recommended command now picks its `--budget` from the active record's mode
through the same shared preset table, falling back to the unchanged 20000
default for an unmapped mode. `AGENTS.block.md`'s step-4 line was updated
(one line, D11-scoped) and the vendored/rendered trees were regenerated.

Files touched: `skills/bee-hive/templates/lib/knowledge.mjs`,
`skills/bee-hive/templates/bee.mjs`,
`skills/bee-hive/templates/lib/command-registry.mjs`,
`skills/bee-hive/templates/lib/inject.mjs`,
`skills/bee-hive/templates/AGENTS.block.md`,
`skills/bee-hive/templates/tests/test_misc.mjs`,
`skills/bee-hive/templates/tests/test_bee_cli.mjs`,
`scripts/test_conformance.mjs` (deviation, see below),
`docs/history/codex-harness-hardening/release-manifest.json` (regen),
`AGENTS.md` (regen), plus the `.bee/bin` and `.agents/.claude/.claude-plugin/.codex-plugin`
vendored projections synced via `onboard_bee.mjs --apply`.

Reservations: released (10 paths).

Verification: recorded passed, full trace and evidence in
`.bee/cells/i54-closeout-3.json`.

## Deviation

Cell's assigned verify (`run_verify.mjs --impacted-from-git --level 1`) surfaced
a failure in `scripts/test_conformance.mjs`'s `adapter-regression-spawn-guard`
scenario, unrelated to this cell's files. Root cause: cell `i54-closeout-1`
(already capped, separate commit) widened the codex spawn guard
(`evaluateCodexSpawn`) to judge every `spawn_agent` payload carrying a
`message` string regardless of `agent_type`/`task_name` — correct per its own
locked D1. The test's `unobservedFailOpen` fixture still used the pre-D1
`{agent_type, message}` shape, which the widened guard now correctly denies
(unmarked) instead of the stale "fails open" expectation. Fixed the fixture
in-cell to a genuinely unobserved shape (no `message` field, which hits
`evaluateCodexSpawn`'s own `noOpinion()` branch) — zero production code
touched, confirmed against the already-locked D1 text. No advisor consult was
used: root cause was traced with certainty by reading the exact guard code
path before editing.

## Consults

None.
