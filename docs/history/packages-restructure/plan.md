---
artifact_contract: bee-plan/v1
mode: standard
feature: packages-restructure
approved_gate2: 2026-07-25 (auto — gate_bypass total)
---

# packages-restructure — plan

## Mode gate (mechanical record)

Risk flags counted: **3** — public contracts (installer surface: `install.sh` distribution modes, plugin package roles), changes behavior existing tests assert (path-parity guards `test_lib_mirror`, `test_misc:1847`, `test_verify_manifest` hardcoded suite list), multi-domain (onboarding engine + verify registry + release manifest + plugin render + CI). Product files: ~20 source + 35 test files (import-depth edit) + 2 CI workflows. → **standard**. Tiny/small insufficient: file count and cross-domain contract edits far exceed lane caps; high-risk not warranted: no hard-gate flag (no auth/data-loss/external-provider surface — installer is our own contract, fully covered by our own suites).

## Decisions (scoping synthesis — user-approved direction)

- **D1** — Bee's vendor payload lives in `packages/bee/` as the single standard code set; install scripts for any platform resolve from it. Skills become instruction-only (SKILL.md + references + scripts that ARE the skill's engine). User approved 2026-07-25.
- **D2** — Internal layout of `packages/bee/` mirrors today's `templates/` exactly (`bee.mjs`, `lib/`, `tests/`, `agents/`, `statusline/`, `AGENTS.block.md`) **plus** repo-root `hooks/` moving in as `packages/bee/hooks/`. Keeps 160 `../lib/` test imports untouched; only the 35×2 four-up imports to `scripts/lib/` change depth (4-up → 3-up).
- **D3** — Path resolution in `onboard_bee.mjs` switches from self-relative `../templates` to `PLUGIN_ROOT`-relative `packages/bee` — the exact mechanism `PLUGIN_HOOKS_DIR` already uses (`onboard_bee.mjs:91`), proven in both `plugin-first` and `repo-copy` modes.
- **D4** — `onboard_bee.mjs` and distribution/test engines stay in `skills/bee-hive/scripts/` this slice. "Instruction-only" targets the *vendor payload*; the onboarding engine is the skill's own machinery, and `install.sh` + SKILL.md + plugin render all point at its current path. Moving it is a separate follow-up if ever wanted.
- **D5** — Release manifest gains a `package_payload` role enumerating `packages/bee/` (replacing implicit coverage via the `skills/` walk and the dedicated `hooks/` walk); `plugin_distribution.mjs` `PACKAGE_ROLES` accepts it so `proveInstalledPackage` keeps proving the payload ships.

## Discovery (L1 — repo-internal, evidence gathered)

Sweep inventory (gather digests, 2026-07-25):

- `onboard_bee.mjs`: 3 ESM imports `../templates/lib/*` (:65-67), `TEMPLATES_DIR` consts (:82-91), tuple literal `skills/bee-hive/templates/lib/state.mjs` (:492), ~15 refs total; `test_onboard_bee.mjs` 14; `test_split_brain_regression.mjs` 3.
- `templates/tests/`: 35 files, 160 `../lib/` imports (unchanged by D2), ~35×2 `../../../../scripts/lib/{run-module-worker,test-fixture}.mjs` → 3-up.
- `templates/lib/guards.mjs`: write-guard classification keyed on literal `templates/lib/` path segment (:1416-1498) — behavior, covered by `test_guards`/write-guard suites.
- `scripts/run_verify.mjs:91-96` `DISCOVERY_ROOTS`: `skills/bee-hive/templates/tests` and `hooks` entries.
- `scripts/impact-registry.json`: 842 generated refs — regenerate, never hand-edit; builder `scripts/impact_registry.mjs` has zero hardcoded template paths (derives from import closure + discovery roots).
- `scripts/release_manifest.mjs`: `:131` skills walk (silently loses templates after move), `:133` hooks walk, stale `source_lib` doc comment `:5`.
- `scripts/render_plugin_skill_trees.mjs`: renders whole `skills/` tree — after move, regen drops templates from `.claude-plugin/skills/` + `.codex-plugin/skills/` automatically; stale committed subtrees must be removed.
- `scripts/lib/release-tuple.mjs`: 2 path refs. `okf_specs_fence.mjs`: 2 imports. Misc scripts/tests: `test_verify_manifest` (16), `test_gate_bypass_doctrine` (11), `test_lib_mirror` (6), `test_compact_capsule` (5).
- CI: `.github/workflows/windows.yml` 8 lines incl. `BEE_VERIFY_ROOT_FILTER`; `canary.yml` comment.
- Repo-root `hooks/` refs: `onboard_bee.mjs` `PLUGIN_HOOKS_DIR` + 6 use sites; `release_manifest.mjs:133`; `run_verify.mjs:95`; write-guard comments/fixtures (3).

Knowledge invariants that must hold (onboarding area concepts):
- Ledger stays content-addressed + directory-scanned (`listTemplateLibModules` etc. adapt path, keep scan behavior); removal stays ledger-diff derived (R27) — module *names* and *contents* don't change, so no spurious drift expected.
- Release tuple registry (`release-tuple.mjs`) must carry the new component path.
- Vendored import closure completeness (hook-vendoring concept) unaffected: lib set unchanged.

## Approach

One mechanical relocation executed as `git mv` (history-preserving), then fix-forward in dependency order, one commit per cell:

1. **Move templates → `packages/bee/`** + every source/test path reference that keeps the suite runnable (resolution consts, imports, guard literals, discovery root, scripts/tests literals).
2. **Move `hooks/` → `packages/bee/hooks/`** + its resolution/manifest/CI references.
3. **Distribution surface**: manifest role, `PACKAGE_ROLES`, plugin-tree regen (stale committed templates removed), impact-registry regen, self-onboard `--apply` refresh of `.bee/bin/` mirrors, parity suites green.
4. **Prose sweep**: skill docs + README references to old paths.

Rejected alternative: restructure into `packages/bee/bin/bee.mjs` (deeper nesting) — rejected this slice; churns the 160 `../lib/` imports and parity guards for zero functional gain (D2).

Risk map:
- Plugin-first installed package must carry `packages/bee/` — **MEDIUM** → proof in validating: confirm manifest-role mechanism covers non-skills trees today (hooks precedent) and `proveInstalledPackage` accepts the new role.
- Write-guard classification literals — **MEDIUM** → `test_guards` + write-guard hook suites must stay green with updated segments.
- Parity guards (`test_lib_mirror`, `test_misc:1847` byte-identity vs `.bee/bin/`) — **LOW** → paths updated in cell 1, mirrors refreshed by self-onboard in cell 3.
- Impact-registry churn — **LOW** → regenerated artifact.

## Test matrix (edge dimensions, scaled)

- Path resolution: both install modes (plugin-first proof via manifest roles; repo-copy via self-onboard on this repo).
- Drift honesty: self-onboard after move must report clean (no spurious drift — hashes unchanged).
- Removal path: no lib module may be orphan-deleted from `.bee/bin/lib/` (names unchanged — assert count parity).
- Guards: write-guard still classifies canonical-source writes under new paths.
- CI: `windows.yml` root filter runs the moved suites.

## Slice (current)

4 cells, serial (1→2→3→4), verify scoped per cell; transitive impacted run at close.
