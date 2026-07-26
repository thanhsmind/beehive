# i54-closeout-5 — [DONE]

Extended `doctorHookSourcesCodex` (canonical `skills/bee-hive/templates/bee.mjs`)
per D5/D9: when both `hooks/hooks.json` (plugin projection) and
`.codex/hooks.json` (repo fallback) are present, the row's evidence text now
names the dual-source risk explicitly — two hook sources exist,
hook-source-exclusivity B14 (exactly-one-active), the current premise
(capability matrix row B1, plugin hooks not-observed on the probed codex
version), and that the premise must be re-proved when the probed version
changes. Also distinguishes `hooks/claude-hooks.json` (plugin.json-declared
Claude manifest) from `hooks/hooks.json` in both the `configured` object
(new `claude_hooks_manifest_checked_in` field) and the surfaced text,
closing #54 item 8. Verdict semantics (ok/warn from `repoPresent` alone) and
`active: unknown` honesty are unchanged; no blocking row was added.

Added a both-present regression test to `test_bee_cli.mjs` (single-source
baseline + both-present fixture) — confirmed red against the pre-change
function via `git stash` before restoring the fix. Regenerated vendored
trees (`render_plugin_skill_trees`, `onboard --apply`, `release_manifest
--write`).

## Files touched

- `skills/bee-hive/templates/bee.mjs` (canonical)
- `skills/bee-hive/templates/tests/test_bee_cli.mjs` (canonical)
- `docs/history/codex-harness-hardening/release-manifest.json`
- Vendored/projected mirrors: `.bee/bin/bee.mjs`, `.agents/skills/...`,
  `.claude/skills/...`, `.claude-plugin/skills/...`, `.codex-plugin/skills/...`
  (synced via the regen chain, not hand-edited)

## Verification

- `node scripts/run_verify.mjs --impacted-from-git --level 1 && node scripts/release_manifest.mjs --check` — PASS (25/25 suites, release manifest matches)
- Full trace/evidence: `.bee/cells/i54-closeout-5.json`

## Reservations

Released (agent `mel`).

## Notes

Unrelated concurrent-swarm artifacts were observed in the working tree
(`.bee/decisions.jsonl`, `docs/decisions/taxonomy.json`, `docs/handbook/`,
other cells' untracked files) — left untouched, outside this cell's scope.
