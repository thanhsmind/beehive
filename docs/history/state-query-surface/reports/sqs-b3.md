# Cell sqs-b3 — report

**Status:** [DONE]

**Outcome:** Added a READ-ONLY `--show` flag to `state scribing-run`
(canonical `skills/bee-hive/templates/bee.mjs` + `lib/command-registry.mjs`,
propagated to `.bee/bin` + 4 plugin mirrors). `--show` (optionally with
`--feature <slug>`) returns the most-recent scribing stamp — overall, or for
one feature — with no ledger append and no phase advance. The read-only
branch returns at the very top of `handleStateScribingRun`, above
`rejectDryRun`/`requireFlags`, so `--show` never needs `--areas`/
`--next-action` (the trap named in validation-slice1.md's sqs-b3 WARNING).
Reuses `readScribingLedger` and a newly-extracted `bestScribingStampMs` (both
exported from `lib/cells.mjs`, the latter factored out of
`globalScribingDebt`'s per-feature max-stamp logic) instead of a second
reader implementation. Registered `--show` in `command-registry.mjs` and in
`FLAG_ALONE_BOOLEANS` (else `--show --feature X` would wrongly consume
`--feature` as its own value). Regenerated all skill-tree mirrors, the
onboarding hash ledger, and the release manifest.

**Files touched:** `skills/bee-hive/templates/bee.mjs`,
`skills/bee-hive/templates/lib/command-registry.mjs`,
`skills/bee-hive/templates/lib/cells.mjs` (deviation — export +
extraction, see cell trace),
`skills/bee-hive/templates/tests/test_cli_state.mjs`,
`.bee/bin/bee.mjs`, `.bee/bin/lib/command-registry.mjs`,
`.bee/bin/lib/cells.mjs`,
`.agents/skills/bee-hive/templates/bee.mjs`,
`.agents/skills/bee-hive/templates/lib/command-registry.mjs`,
`.agents/skills/bee-hive/templates/lib/cells.mjs`,
`.agents/skills/bee-hive/templates/tests/test_cli_state.mjs`,
`.claude-plugin/skills/bee-hive/templates/bee.mjs`,
`.claude-plugin/skills/bee-hive/templates/lib/command-registry.mjs`,
`.claude-plugin/skills/bee-hive/templates/lib/cells.mjs`,
`.claude-plugin/skills/bee-hive/templates/tests/test_cli_state.mjs`,
`.claude/skills/bee-hive/templates/bee.mjs`,
`.claude/skills/bee-hive/templates/lib/command-registry.mjs`,
`.claude/skills/bee-hive/templates/lib/cells.mjs`,
`.claude/skills/bee-hive/templates/tests/test_cli_state.mjs`,
`.codex-plugin/skills/bee-hive/templates/bee.mjs`,
`.codex-plugin/skills/bee-hive/templates/lib/command-registry.mjs`,
`.codex-plugin/skills/bee-hive/templates/lib/cells.mjs`,
`.codex-plugin/skills/bee-hive/templates/tests/test_cli_state.mjs`,
`docs/history/codex-harness-hardening/release-manifest.json`,
`.agents/skills/.bee-render.json`, `.claude-plugin/skills/.bee-render.json`,
`.claude/skills/.bee-render.json`, `.codex-plugin/skills/.bee-render.json`,
`.bee/onboarding.json`.

Full trace/evidence: `.bee/cells/sqs-b3.json`.
