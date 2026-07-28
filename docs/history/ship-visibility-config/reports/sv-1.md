# sv-1 — ship_visibility config key surfaced in status + preamble

[DONE] `ship_visibility` (`off` default/absent, `draft-pr` recognized, any other value normalizes to `off` with a one-line stderr warning) now surfaces in `status --json` and, only when `draft-pr`, as one line in the session preamble; zero cost when `off`.

**Files touched:** `packages/bee/bee.mjs`, `packages/bee/lib/state.mjs`, `packages/bee/lib/inject.mjs`, `packages/bee/tests/test_misc.mjs` (EXPECTED_STATE_EXPORTS allowlist fix, deviation — new exports broke the exact-set export census), `docs/history/codex-harness-hardening/release-manifest.json`, `.bee/bin/bee.mjs`, `.bee/bin/lib/state.mjs`, `.bee/bin/lib/inject.mjs` (regen obligation: `onboard_bee.mjs --apply`, `release_manifest.mjs --write`/`--check`, both green).

**Verify:** cell's literal `verify` (`node scripts/tests/test_ship_visibility.mjs && ...`) could not run — that suite is sv-2's not-yet-written deliverable. Per orchestrator instruction, capped on inline probes (`.bee/config.local.json` overlay, reverted) plus `release_manifest --check`, recorded via `cells verify`. Full trace/evidence: `.bee/cells/sv-1.json`.
