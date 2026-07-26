# Cell sqs-b2 — report

**Status:** [DONE]

**Outcome:** Added a read verb `backlog findings --feature <slug> [--text
<terms>]` (canonical `skills/bee-hive/templates/bee.mjs` +
`lib/command-registry.mjs`, propagated to `.bee/bin` + 4 plugin mirrors)
listing friction/finding rows from `.bee/backlog.jsonl`. Both coexisting
schemas are read: legacy rows carry `kind: "friction"|"finding"`, current
rows carry `type: "friction"|"finding"` — a row is returned if either field
matches; `kind:'pbi'` rows are always skipped. `--feature` is a
word-boundary/exact match (reuses sqs-b1's `matchesWholeToken`), so `--feature
auth` excludes `authz`. Optional `--text` narrows further by substring over
title+detail. Registered in `command-registry.mjs` so `--help --json`
advertises `--feature`/`--text`.

**Files touched:** `skills/bee-hive/templates/bee.mjs`,
`skills/bee-hive/templates/lib/command-registry.mjs`,
`skills/bee-hive/templates/tests/test_backlog_capture.mjs`,
`.bee/bin/bee.mjs`, `.bee/bin/lib/command-registry.mjs`,
`.agents/skills/bee-hive/templates/bee.mjs`,
`.agents/skills/bee-hive/templates/lib/command-registry.mjs`,
`.agents/skills/bee-hive/templates/tests/test_backlog_capture.mjs`,
`.claude-plugin/skills/bee-hive/templates/bee.mjs`,
`.claude-plugin/skills/bee-hive/templates/lib/command-registry.mjs`,
`.claude-plugin/skills/bee-hive/templates/tests/test_backlog_capture.mjs`,
`.claude/skills/bee-hive/templates/bee.mjs`,
`.claude/skills/bee-hive/templates/lib/command-registry.mjs`,
`.claude/skills/bee-hive/templates/tests/test_backlog_capture.mjs`,
`.codex-plugin/skills/bee-hive/templates/bee.mjs`,
`.codex-plugin/skills/bee-hive/templates/lib/command-registry.mjs`,
`.codex-plugin/skills/bee-hive/templates/tests/test_backlog_capture.mjs`,
`docs/history/codex-harness-hardening/release-manifest.json`,
`.agents/skills/.bee-render.json`, `.claude-plugin/skills/.bee-render.json`,
`.claude/skills/.bee-render.json`, `.codex-plugin/skills/.bee-render.json`,
`.bee/onboarding.json`.

Full trace/evidence: `.bee/cells/sqs-b2.json`.
