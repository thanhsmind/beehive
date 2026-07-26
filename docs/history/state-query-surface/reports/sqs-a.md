# Cell sqs-a — report

**Status:** [DONE]

**Outcome:** Added `checkBinLibImportBashCommand` to `guards.mjs` (canonical
`skills/bee-hive/templates/lib`, propagated to `.bee/bin/lib` + 4 plugin
mirrors) and wired it into `hooks/bee-write-guard.mjs`'s Bash branch alongside
`checkGitBashCommand`. It denies an inline-eval `node -e`/`--eval`/`-p` Bash
command whose script text imports/requires a `bin/lib/` or `templates/lib/`
module, with a FIX line naming the paved read (`bee status --json`,
`bee <group> --help --json`). File-based `node <path>.mjs` runs are never
touched. Two-direction negative control added to `hooks/test_write_guard.mjs`
(row70 deny, row71 allow).

**Files touched:** `skills/bee-hive/templates/lib/guards.mjs`,
`.bee/bin/lib/guards.mjs`, `.agents/skills/bee-hive/templates/lib/guards.mjs`,
`.claude-plugin/skills/bee-hive/templates/lib/guards.mjs`,
`.claude/skills/bee-hive/templates/lib/guards.mjs`,
`.codex-plugin/skills/bee-hive/templates/lib/guards.mjs`,
`hooks/bee-write-guard.mjs`, `.bee/bin/hooks/bee-write-guard.mjs`,
`hooks/test_write_guard.mjs`,
`docs/history/codex-harness-hardening/release-manifest.json`,
`.agents/skills/.bee-render.json`, `.claude-plugin/skills/.bee-render.json`,
`.claude/skills/.bee-render.json`, `.codex-plugin/skills/.bee-render.json`,
`.bee/onboarding.json`.

Full trace/evidence: `.bee/cells/sqs-a.json`.
