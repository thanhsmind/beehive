# packages-restructure-2 — report

**Status:** [DONE]

**Outcome:** `git mv hooks packages/bee/hooks` (history-preserving), then fix-forward every
path reference across `scripts/`, `skills/`, `.github/`, both plugin manifests, and
`packages/bee/**` so the full suite stays green: `onboard_bee.mjs`'s `PLUGIN_HOOKS_DIR`
and `repoOwnsHookCatalog` self-identification probe now resolve `packages/bee/hooks`
(legacy `hooks/` kept as an OR-fallback for older checkouts, per the cell's instruction);
`catalog.mjs`'s `commandFor` template plus the regenerated `claude-hooks.json`/`hooks.json`
projections; both plugin manifests (`.codex-plugin/plugin.json` now carries an explicit
`"hooks"` override matching `.claude-plugin/plugin.json`'s existing pattern, since the file
moved off Codex's undocumented plugin-root default lookup); `run_verify.mjs`'s discovery
root; `release_manifest.mjs`'s `plugin_hook` walk; `okf_instructions_fence.mjs`'s
`hook-output` surface (found and fixed a real bug: its own `--selftest` fixtures still
targeted the bare `hooks/` path after the surface spec moved — would have silently stopped
fencing hook output; reverified 23/23 green); `install.ps1`'s sparse-checkout set; CI
comments; and every test suite resolving the real repo's `hooks/` tree, including several
outside the cell's declared `files` list found via a broader sweep. Regenerated
impact-registry, plugin skill trees, and the release manifest via self-onboard `--apply`.

**Files touched:** 57 files — see `.bee/cells/packages-restructure-2.json` trace
`files_changed` for the full list (top-level: `packages/bee/hooks/**` new tree,
`skills/bee-hive/scripts/{onboard_bee,test_onboard_bee}.mjs`, `packages/bee/bee.mjs`,
`packages/bee/tests/{test_bee_cli,test_bee_write_guard_hook}.mjs`, `scripts/**`,
`.github/workflows/{windows,canary}.yml`, `.claude-plugin/`, `.codex-plugin/`, `.claude/`,
`.agents/` skill mirrors, `docs/history/codex-harness-hardening/release-manifest.json`,
`scripts/impact-registry.json`).

**Verification:** `BEE_VERIFY_CONCURRENCY=12 node scripts/run_verify.mjs && node
scripts/release_manifest.mjs --check` → PASS, 105 suites green, 404-file manifest match.
Full trace/evidence: `.bee/cells/packages-restructure-2.json`.

**Deviations (auto-fixed/auto-added, in scope of the move):**
1. 4 static relative imports inside the moved `packages/bee/hooks/*.mjs` test files
   (`../scripts/lib/run-module-worker.mjs`, and `test_write_guard.mjs`'s own
   `../.bee/bin/lib/lease-store.mjs`) were correct only at the old 1-level-deep location;
   fixed to the new 3-up depth, along with each file's own `REPO_ROOT` dirname-climb constant.
2. `okf_instructions_fence.mjs`'s `isSurface`/`walk` still scanned bare `hooks/` after the
   `SURFACES` spec moved — a real bug found and fixed in the same edit.
3. `packages/bee/bee.mjs` carries its own hand-ported copy of `onboard_bee.mjs`'s
   `repoOwnsHookCatalog`/`doctorHookHandlersResolvable`/`doctorHookSourcesCodex`/
   `doctorClaudeHandlersResolvable` (not in the cell's `files` list) — updated all 5 call
   sites, legacy OR-fallback kept to match the pattern the cell mandated for the
   `onboard_bee.mjs` original.
4. 7 more files outside the cell's declared list read the real repo's `hooks/` tree via
   `REPO_ROOT`-relative paths and would have gone red without updating: `test_onboard_bee.mjs`
   (4 spots), `test_conformance.mjs`, `test_hook_vendor_closure.mjs`, `test_heartbeat_touch.mjs`,
   `test_lib_mirror.mjs`, `test_bee_write_guard_hook.mjs`, `test_bee_cli.mjs`.
5. `.codex-plugin/plugin.json` had no explicit `"hooks"` field; added one pointing at the new
   canonical path, and updated the paired regression guard in `test_hook_contracts.mjs`
   (`codex-default-hooks-route`) to require that explicit override instead of the stale
   pre-move literal.
6. `test_hook_contracts.mjs`'s `codex-plugin-subagent-audit-topology` check pinned a literal
   expected command string at the old path — updated to match `catalog.mjs`'s regenerated
   projections.

**Deliberate exceptions (not fixed):**
- 3 `write-guard-hold` checks inside `test_hook_contracts.mjs`/`test_write_guard.mjs` fail
  when those suites are invoked standalone in an interactive shell, but pass through the
  actual verify path (`run_verify.mjs` isolates each suite's execution environment).
  Confirmed pre-existing and unrelated to this cell's diff via `git stash` back to commit
  `8bff2a8` (cell 1's own cap commit), reproduced identically there with a clean env. Not
  filed as a fix-first cell since it never surfaces through `commands.verify`/`commands.test`.

Full trace and evidence: `.bee/cells/packages-restructure-2.json`.
