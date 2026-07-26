# Cell sqs-b1 — report

**Status:** [DONE]

**Outcome:** Added `--cell` and `--feature` to both `decisions active` and
`decisions search` (canonical `skills/bee-hive/templates/bee.mjs` +
`lib/command-registry.mjs`, propagated to `.bee/bin` + 4 plugin mirrors).
Neither is a structural field on a decide event — both match as whole
tokens over `decision`/`rationale`/`alternatives` text via a lookaround
regex (`(?<![\w-])id(?![\w-])`) that excludes both a digit-suffix collision
(`si-1` vs `si-10`) and a hyphen-suffix collision (`billing-export` vs
`billing-export-v2`). Both params registered in `command-registry.mjs` so
`--help --json` advertises them on both commands. Deviation: exported
`escapeRegExp` from `lib/decisions.mjs` (was module-private) so the new
matcher reuses the same escaping helper as `sweepDecisionCitations`.

**Files touched:** `skills/bee-hive/templates/bee.mjs`,
`skills/bee-hive/templates/lib/command-registry.mjs`,
`skills/bee-hive/templates/lib/decisions.mjs`,
`skills/bee-hive/templates/tests/test_decisions_propagation.mjs`,
`.bee/bin/bee.mjs`, `.bee/bin/lib/command-registry.mjs`,
`.bee/bin/lib/decisions.mjs`,
`.agents/skills/bee-hive/templates/bee.mjs`,
`.agents/skills/bee-hive/templates/lib/command-registry.mjs`,
`.agents/skills/bee-hive/templates/lib/decisions.mjs`,
`.agents/skills/bee-hive/templates/tests/test_decisions_propagation.mjs`,
`.claude-plugin/skills/bee-hive/templates/bee.mjs`,
`.claude-plugin/skills/bee-hive/templates/lib/command-registry.mjs`,
`.claude-plugin/skills/bee-hive/templates/lib/decisions.mjs`,
`.claude-plugin/skills/bee-hive/templates/tests/test_decisions_propagation.mjs`,
`.claude/skills/bee-hive/templates/bee.mjs`,
`.claude/skills/bee-hive/templates/lib/command-registry.mjs`,
`.claude/skills/bee-hive/templates/lib/decisions.mjs`,
`.claude/skills/bee-hive/templates/tests/test_decisions_propagation.mjs`,
`.codex-plugin/skills/bee-hive/templates/bee.mjs`,
`.codex-plugin/skills/bee-hive/templates/lib/command-registry.mjs`,
`.codex-plugin/skills/bee-hive/templates/lib/decisions.mjs`,
`.codex-plugin/skills/bee-hive/templates/tests/test_decisions_propagation.mjs`,
`docs/history/codex-harness-hardening/release-manifest.json`,
`.agents/skills/.bee-render.json`, `.claude-plugin/skills/.bee-render.json`,
`.claude/skills/.bee-render.json`, `.codex-plugin/skills/.bee-render.json`,
`.bee/onboarding.json`.

Full trace/evidence: `.bee/cells/sqs-b1.json`.
