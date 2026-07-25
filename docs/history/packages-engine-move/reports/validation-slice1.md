# Validation — packages-engine-move, slice 1 (3 cells)

Date: 2026-07-26 · Lane: standard · Verdict: **READY WITH CONSTRAINTS** (after in-loop cell repairs, iteration 1 of 3)

## Reality gate

- **MODE FIT: PASS** — 3 flags (installer public contract, covered-test behavior, multi-domain); standard.
- **REPO FIT: PASS** — PLUGIN_ROOT arithmetic identical from the new location (`scripts→bee→packages→root`, checker+reviewer both verified); manifest role migration automatic (`package_payload` walk vs `plugin_skill` walk); vendoring safe (`listTemplateHelpers` filters `isFile()` — the engine dir never gets vendored into `.bee/bin`).
- **ASSUMPTIONS: PASS (repaired)** — original cells mis-prescribed `classifySource` geometry change and a tautological identity anchor; both corrected from checker/reviewer evidence.
- **SMALLER PATH: PASS** — engine rides to `packages/bee/scripts/` with zero change to `source-identity.mjs`; strict-flags scoped down to unknown-flag naming only.
- **PROOF SURFACE: PASS** — full suite + manifest check per cell; move acceptance is an exit-code-guarded rg clause inside the verify itself.

## Key repairs (plan-checker: 8 BLOCKER, 4 WARNING · cell reviewer: 10 CRITICAL, 4 MINOR)

- **classifySource contract untouched** (B1/C3): 5 callers in 2 geometries; only the engine's 2 call sites change, passing `PLUGIN_ROOT/skills/bee-hive`. `source-identity.mjs` is now a prohibited edit.
- **HIVE_DIR is three semantics** (C1): engine geometry / classifySource input / skills-root-for-sync — split into ENGINE_DIR + SKILLS_ROOT with the 8 use lines enumerated; `dirname(HIVE_DIR)` as skills root would have made sync walk `packages/`.
- **Identity anchor made falsifiable** (B5/C2): the proposed `realpath(PLUGIN_ROOT/packages/bee/scripts)===SCRIPTS_DIR` is true by construction; re-anchored on skills-tree existence + payload presence.
- **Engine test fixtures are re-authors, not path swaps** (B4/C4): `makeFakeSkillsRoot` launcher moves outside the fake skills tree (18 call sites); `test_split_brain_regression`'s scenario is dissolved by D3 and rewritten as "projection carries no launcher → blocked_no_source".
- **Missed live callers folded in** (B2/B3/C5): `canary_codex.mjs` (CI spawns the engine), `install.ps1` backslash forms + `:328`, `ledger_parity` FIX_HINT, `bump_version` runbook line, okf fence fixtures, `LLM.md`, `INSTALL.md`; acceptance pattern widened to `bee-hive[/\\]scripts` and moved INTO the verify with `[ $? -eq 1 ]` (rg exit-2 false-green fixed, again — C6).
- **Cell 2 verify made non-vacuous** (B6): positive assertion that SKILL.md names the new entrypoint + relative-form sweep incl. `routing-and-contracts.md:52`.
- **Cell 3 rescoped** (C7/C8/C9): premise corrected — `capture add --text` exits 1 today via `requireFlag`; the real gaps are (a) the error never names the unknown flag, (b) 7 handlers read undeclared flags. Central check goes stderr/exit-1 message-compatible; both bespoke loops stay; `required:[]` DB3 semantics untouched; registry gaps declared, validator never loosened.
- **Knowledge sweep owned by scribing** (C10): 12 onboarding-area concepts carry the old engine path; the close's scribing pass owns that sweep — recorded here so it cannot be forgotten.
- Regen chains completed in cells 2/3 (`impact_registry --write`, W3); regen-written files declared for reservation (W4).

## Feasibility matrix

| assumption | risk | proof | result |
|---|---|---|---|
| PLUGIN_ROOT arithmetic survives the move | LOW | `onboard_bee.mjs:83-85` — 3×dirname identical | OK |
| Projections keep refusing as source | MED | `classifySource` untouched + `blocked_no_source` branches `:490-534`; test-pinned in cell 1 | OK |
| Manifest role migration automatic | LOW | `release_manifest.mjs:138-141` walks | OK |
| Engine never vendored into hosts | LOW | `listTemplateHelpers` `isFile()` filter | OK |
| Strict flags break no legit caller | MED | 7 known gaps declared up front (C9 static scan); any further red = declare, never loosen | OK with constraint |
| Schedule | — | 3 waves serial, zero cycles; shared files force serial (checker confirmed) | OK |

## Approval

READY WITH CONSTRAINTS (constraints folded into cells). Gate 3 auto-approved under gate_bypass=total (standard lane, no hard-gate flag → advisor consult not required).
