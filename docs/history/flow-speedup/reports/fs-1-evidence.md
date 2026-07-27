# fs-1 — Slice-tail test batching (spec #80/#85 P1/P2/P4/P5)

Cell fs-1 · lane standard · change_class behavior · worker exec-fs1

## Proof-tier matrix — before → after

| change_class | lane | before | after |
|---|---|---|---|
| `security` / `migration` | every | `red-first` | `red-first` (unchanged) |
| `refactor` / `formatting` | every | `suite-green` | `suite-green` (unchanged) |
| `bugfix` | tiny/small/standard | `targeted-green` | **`targeted-green` (unchanged — repro-first survives)** |
| `bugfix` | high-risk | `red-first` | `red-first` (unchanged) |
| `behavior` / `api` | tiny/small/standard | `targeted-green` | **`existing-targeted-green`** (P1) |
| `behavior` / `api` | high-risk | `red-first` | `red-first` (unchanged) |
| `test` | every | *(not a class)* | **`targeted-green`** (P2) |
| unclassified | any | `null` | `null` (unchanged) |

## Probes

| probe | expected | got | pass |
|---|---|---|---|
| `requiredProofTier('behavior','standard')` | `existing-targeted-green` | same | yes |
| `requiredProofTier('bugfix', tiny/small/standard)` | `targeted-green` | same | yes |
| `requiredProofTier('bugfix','high-risk')` | `red-first` | same | yes |
| `requiredProofTier('test', every lane)` | `targeted-green` | same | yes |
| capCell behavior×standard, no new test, no `red_failure_evidence` | caps | caps | yes |
| capCell behavior×standard with no verify record | refuses | refuses | yes |
| capCell test-cell adding a new suite without `new_suite_reason` | refuses | refuses | yes |
| leave `swarming` with uncapped `test` cell | throws, state byte-identical | same | yes |
| same, with `gate_bypass: "total"` | still throws | still throws | yes |
| `test` cell capped with `verify_passed:false` | throws | throws | yes |
| `test` cell capped green | departure allowed | allowed | yes |
| uncapped `behavior` cell (not `test`) | no block | no block | yes |
| `scribing-run` out of swarming, red test cell | throws, nothing stamped | same | yes |

**bugfix unchanged:** `requiredProofTier` now handles `bugfix` in its **own branch**, textually separate from `behavior`/`api`, so the P1 edit could not sweep it along. Pinned twice: the table-driven matrix row and a dedicated negative-control check asserting `bugfix` is `targeted-green` (and explicitly **not** `existing-targeted-green`) on tiny/small/standard, `red-first` on high-risk.

## P4 wiring

`guardTestCellDebt(root, record, targetPhase)` in `packages/bee/bee.mjs` (co-located with `closeGuardScribingDebt`, same choke-point rationale — `checkPhaseTransition` must stay pure because `cells.mjs` imports `state.mjs`). Called from **both** doors out of `swarming`: `handleStateSet` (after `checkPhaseTransition`, before any field write, inside the mutation lock) and `handleStateScribingRun` (before the `last_scribing_run` stamp and the ledger append). Reads neither `bypassLevel` nor any headless flag, and has no waiver flag. Scope is the **active feature's** `change_class:'test'` cells — no slice record was invented, because none exists in this codebase.

## Verify

`node scripts/ledger_parity.mjs --check && node scripts/release_manifest.mjs --check && node scripts/run_verify.mjs`
→ exit 0 · ledger parity OK · 396 manifest files match · **108 suites PASS, 0 failed**, wall 75.4s
