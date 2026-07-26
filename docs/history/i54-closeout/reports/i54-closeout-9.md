# i54-closeout-9 — report

[DONE]

Fixed the fresh-install write-guard crash canary P5 caught: `HOOK_FILENAMES`
in `skills/bee-hive/scripts/onboard_bee.mjs` was missing
`tokenize-command.mjs`, so every fresh `--repo-hooks` install shipped a
`bee-write-guard.mjs` that crashed `ERR_MODULE_NOT_FOUND` at import instead
of enforcing the pre-Gate-3 write block. Added the filename with a
provenance comment, and wrote `scripts/test_hook_vendor_closure.mjs` — a
static import-closure parser over `HOOK_FILENAMES` (self-tested to prove it
actually bites) plus a real fresh-install/subprocess proof that the vendored
guard no longer crashes at import. Synced canonical → vendored trees via
`render_plugin_skill_trees.mjs`, self-onboard `--apply`, and
`release_manifest.mjs --write` (D10).

Files touched: `skills/bee-hive/scripts/onboard_bee.mjs`,
`scripts/test_hook_vendor_closure.mjs`,
`docs/history/codex-harness-hardening/release-manifest.json`, and the synced
vendored/rendered projections (`.agents/skills/**`, `.claude-plugin/skills/**`,
`.claude/skills/**`, `.codex-plugin/skills/**`, `.bee/onboarding.json`).

Full trace/evidence: `.bee/cells/i54-closeout-9.json`.

Reservations released; one commit, cell id `i54-closeout-9` in the message.
