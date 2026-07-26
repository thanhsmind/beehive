# Validation — packages-restructure, slice 1 (4 cells)

Date: 2026-07-25 · Lane: standard · Verdict: **READY WITH CONSTRAINTS** (after in-loop cell repairs, iteration 1 of 3)

## Reality gate

- **MODE FIT: PASS** — 3 flags (public contracts, covered-test behavior, multi-domain), ~60 product files; standard. No hard-gate flag.
- **REPO FIT: PASS** — `PLUGIN_ROOT` root-relative resolution already exists and ships hooks in both install modes (`onboard_bee.mjs:90-91`); the move reuses the proven mechanism.
- **ASSUMPTIONS: PASS (repaired)** — original cells under-scoped by ~25 files; the adversarial pass enumerated the true surface (below) and cells were repaired in place via `cells update`.
- **SMALLER PATH: PASS** — flat `packages/bee/` mirror of `templates/` (D2) is the smallest honest shape; deeper nesting rejected in plan.
- **PROOF SURFACE: PASS** — full `run_verify` + `release_manifest --check` per cell; distribution proof cell (3) runs installer e2e + plugin distribution suites explicitly.

## Feasibility matrix

| assumption | risk | proof | evidence | result |
|---|---|---|---|---|
| PLUGIN_ROOT reaches packages/bee in both install modes | MED | code read | `onboard_bee.mjs:82-91` — PLUGIN_ROOT = 3×dirname; PLUGIN_HOOKS_DIR precedent ships via manifest role `plugin_hook`, proven by `proveInstalledPackage` (`plugin_distribution.mjs:168-198`) | OK |
| Plugin package carries non-skills trees | MED | code read | manifest roles enumerate repo-root `hooks/` today (`release_manifest.mjs:133`); installed-package proof is role-based; `test_installers_e2e.mjs:200-216` materializes from manifest records | OK — **constraint: payload role must land in cell 1** (C5), applied |
| Write-guard survives path change | MED | code read | single executable literal `guards.mjs:1454` regex `(?:bin\/lib|templates\/lib)`; segment swap + keep `bin/lib` | OK (must_have pinned) |
| No spurious ledger drift | LOW | code read | `buildManagedVersions` keys are **basenames**, hashes content-addressed (`onboard_bee.mjs:3194-3199`); names+contents unchanged | OK |
| Plugin hook config survives | HIGH (found) | code read | `plugin.json:9` → `./hooks/claude-hooks.json`; 22 command strings `${CLAUDE_PLUGIN_ROOT}/hooks/...` (`claude-hooks.json`, `hooks.json`); generator template `catalog.mjs:86` | repaired into cell 2 (C4/B8) |
| bee self-identification probe | HIGH (found) | code read | `repoOwnsHookCatalog` (`onboard_bee.mjs:2205`) checks `hooks/catalog.mjs` — post-move must point at `packages/bee/hooks/catalog.mjs` or onboard clobbers bee's own `.codex/hooks.json` | repaired into cell 2 (C3) |
| Windows installer fetches payload | HIGH (found) | code read | `install.ps1:317` sparse-checkout lacks `packages`; guarded by `test_installers_e2e.mjs:347-355` | repaired: cell 1 adds `packages`, cell 2 drops `hooks` (C6/B7) |
| Schedule | — | `bee cells schedule` | 4 waves, serial, zero cycles | OK |

## Adversarial findings (plan-checker: 8 BLOCKER, 8 WARNING · cell reviewer: 8 CRITICAL, 6 MINOR)

All BLOCKER/CRITICAL findings repaired in-place via `cells update` (iteration 1):

- B1/C1 regen crash (`render_plugin_skill_trees.mjs:31` static import) → cell 1 files+action.
- B2/C2 ~20 unlisted suites with live path resolution → cell 1 acceptance switched to rg-clean, files widened.
- B3/C5 manifest payload role one cell too late → D5 moved into cell 1 (role `package_payload`, excl. hooks subtree; `plugin_hook` role name kept, repointed cell 2). Logged as D6.
- B4 windows.yml templates refs → cell 1. B5/B6 missed hooks refs → cell 2. B7/C6 install.ps1 sparse set → cells 1/2.
- B8/C4 plugin.json + 22 hook command strings + catalog template → cell 2.
- C3 self-identification probe → cell 2 (with legacy fallback).
- C7 cell-4 verify false-green (`!` inverts rg exit 2) + wrong targets → verify rewritten over real targets.
- C8 156 tracked render files → deletion owned by cell 1.
- W1 `--check` tautology → mitigated by payload-record must_have in cell 1 + explicit suite quotes in cell 3. W2/W3/M6 cell-4 scope/prohibition contradictions → rewritten, CREATION-LOGs dropped from scope. W4/M1/M2 undercounts → counts replaced by rg-clean acceptance. W5 fixture restructure named in cell 1. W6 role design settled (D6). W7 regex must_have pinned. M3/M4 conditionals stated flatly.

Remaining constraint (W8, accepted scope): `onboard_bee.mjs`/`plugin_distribution.mjs` stay in `skills/bee-hive/scripts/` this slice (plan D4) — `skills/` is instruction-only for *payload*, not yet for the installer engine. Follow-up candidate, user-visible in plan.

## Cell review

Post-repair: cells carry exact anchors, flat instructions, rg-based acceptance, runnable verifies (`run_verify` flags confirmed real: `run_verify.mjs:808,818,1117,1133`; cell-4 rg targets exist). JUDGE advisory (red_failure_evidence) noted: mechanical-move refactor rides suite-green as proof per scripts-tests-move precedent; workers record verify evidence at cap.

## Approval

Verdict: READY WITH CONSTRAINTS (constraints folded into cells). Gate 3: auto-approved under gate_bypass=total (standard lane, no hard-gate flag → advisor consult not required).
