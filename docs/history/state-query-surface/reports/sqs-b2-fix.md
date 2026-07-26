# sqs-b2-fix

**Status:** DONE

**Outcome:** Fixed the wave-close red in `test_bee_cli.mjs` — the coverage
check ("every registry entry had its example executed at least once") was
failing with `backlog.findings` never-exercised. Added a per-entry
example-execution assertion for `backlog.findings` (seeds a matching
friction row in the shared backlog fixture, runs both registry examples
through the real dispatcher), matching the existing sibling pattern
(capture.flush's seed-then-assertExampleOk). Coverage check flips FAIL -> PASS.

**Files touched:**
- `skills/bee-hive/templates/tests/test_bee_cli.mjs` (canonical)
- `.agents/skills/bee-hive/templates/tests/test_bee_cli.mjs` (mirror)
- `.claude/skills/bee-hive/templates/tests/test_bee_cli.mjs` (mirror)
- `.claude-plugin/skills/bee-hive/templates/tests/test_bee_cli.mjs` (mirror)
- `.codex-plugin/skills/bee-hive/templates/tests/test_bee_cli.mjs` (mirror)
- `.agents/skills/.bee-render.json`, `.claude/skills/.bee-render.json`, `.claude-plugin/skills/.bee-render.json`, `.codex-plugin/skills/.bee-render.json` (render hashes)
- `docs/history/codex-harness-hardening/release-manifest.json` (regen)
- `.bee/onboarding.json` (regen ledger timestamp)

Full trace/evidence: `.bee/cells/sqs-b2-fix.json`.
