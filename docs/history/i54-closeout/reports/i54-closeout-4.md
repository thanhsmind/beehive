# i54-closeout-4 — worker report

Status: [DONE]

Outcome: herding's working-agent and control-pane spawn commands are now
config-driven (D4). `control-loop.sh` reads optional `.bee/config.json`
`herding.control_command` (a JSON array of argv-token strings); when present,
each token is substituted per-token with `{PROMPT}`/`{MODEL}`/`{MAX_TURNS}`/
`{ALLOWED_TOOLS}` and run verbatim (never joined into one string and re-split
or `eval`'d — the shell-injection-prone shape the cell prohibited). When the
key is absent, invalid, or empty, the built default is byte-equivalent to the
pre-change hardcoded `claude -p ... --model sonnet --max-turns ... --allowedTools ...`
invocation. `SKILL.md` documents the matching `herding.agent_command` seam for
the working-agent's spawn tail (dispatch role §8, and the quick-reference
table), plus a new "Herding runtime adapter" section with both key shapes and
a codex adapter example explicitly marked illustrative-only (full
codex-native herding stays out of scope per D4). Enable/disable/status verbs,
the dispatch interlock, and the merge owner-gesture are untouched. Ran the
cell's regen obligation (`render_plugin_skill_trees.mjs`, self-onboard
`--apply`, `release_manifest.mjs --write`) so every vendored copy and the
release manifest match the canonical edit.

Files touched:
- skills/bee-herding/scripts/control-loop.sh
- skills/bee-herding/SKILL.md
- docs/history/codex-harness-hardening/release-manifest.json
- vendored/rendered copies under `.claude/skills/bee-herding/`,
  `.agents/skills/bee-herding/`, `.claude-plugin/skills/bee-herding/`,
  `.codex-plugin/skills/bee-herding/`, and their `.bee-render.json` files
  (synced by the regen chain, not hand-edited)

Verify: `bash -n skills/bee-herding/scripts/control-loop.sh && node scripts/run_verify.mjs --impacted-from-git --level 1 && node scripts/release_manifest.mjs --check` — exit 0 (syntax OK; 0 suites mapped from 28 changed files, full verify delegated to CI per ci-owned-verify; 510 files match the stored release manifest). The existing `skills/bee-hive/templates/tests/test_herding.mjs` regression suite was re-run and still passes 10/10.

Full trace and verification evidence (including the isolated scratch-directory
tests proving the byte-equivalent default, the config-driven override, and the
shell-injection-safety check): `.bee/cells/i54-closeout-4.json`.
