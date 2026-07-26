# install-tty-fix-1 — [DONE]

`can_prompt()` in `scripts/install.sh` now attempts a real `/dev/tty` open before trusting `-r`/`-w`, and `confirm()`'s read is hardened against a mid-read failure — the non-tty path now always hits the designed fail-safe message instead of crashing on "unbound variable" under `set -u`.

Files touched: `scripts/install.sh`, `docs/history/codex-harness-hardening/release-manifest.json` (regenerated per the cell's regen obligation: render_plugin_skill_trees.mjs, onboard_bee.mjs --apply, release_manifest.mjs --write).

Full trace/evidence: `.bee/cells/install-tty-fix-1.json`.
