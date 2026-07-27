# fs-3 — Closing the P4 half-guard, and being the slice's own test cell

Cell fs-3 · lane standard · change_class `test` · worker exec-fs3

Two jobs about one rule. The slice that invented "a slice can never close red" was
itself about to close without a test cell, through the exact hole its own guard left.

## Job 1 — the half-guard

`guardTestCellDebt` (`packages/bee/bee.mjs`) refused only when a `change_class: 'test'`
cell **existed** and was uncapped or red. A feature holding capped `behavior`/`api` cells
and **no test cell at all** left `swarming` clean — verified live on flow-speedup itself
(two capped behavior cells, zero test cells, `state set --phase scribing` succeeded with
no complaint). The guarantee rested on planning *remembering* to emit the cell: prose,
not machine, which is the precise gap P4 exists to close.

The guard now has two branches:

| branch | condition | verdict |
|---|---|---|
| (a) *original* | a `test` cell exists, uncapped or capped with `verify_passed: false` | refuse |
| (b) **fs-3** | **zero** `test`-class cells **and** ≥1 **capped** `behavior`/`api` cell | refuse |

Branch (a) is evaluated first and its message is unchanged, so a feature that *has* a
test cell always gets the "not green" refusal — (b) can never mask it. Pinned by a test.

### How (b) is scoped, and the three edges

**Bootstrap — `capped`, not merely present.** The trigger is `cell.status === 'capped'`.
A feature whose behavior cells are all still open has shipped no behavior yet, so it owes
no tests yet; triggering on presence would wall in every feature at the moment it starts.
Debt begins exactly when behavior *caps*, which under P1 is the moment a cell is allowed
to cap **without authoring tests**. Two tests hold this: the new bootstrap-edge check, and
fs-1's pre-existing "an uncapped behavior cell is not this guard's business".

**No-behavior features — `behavior`/`api` only.** Nothing else counts, deliberately.
`refactor`/`formatting` already cap on `suite-green` (the *existing* suite), and docs work
carries no `change_class` and is not `behavior_change`, so `deriveChangeClass` resolves it
to `null`. All three fall through the collector untouched, and (b) stays silent for a
feature that authored no behavior and would have nothing to consolidate.

**Both doors, unliftable.** One function, both call sites untouched: `handleStateSet`
(inside the mutation lock, before any field write) and `handleStateScribingRun` (before the
`last_scribing_run` stamp and the ledger append). Branch (b) reads neither `bypassLevel`
nor any headless flag and has no waiver flag — same shape as branch (a), for the same
reason: this is a mechanical precondition, not a human gate. Both refusals name what is
missing and how to satisfy it, in the existing voice (`WHAT IS MISSING:` + `FIX:` +
the standing "no gate_bypass level (including \"total\") lifts it" line).

## Job 2 — the slice test cell

**D5 read-first was the whole story here.** Reading the suites before writing found that
fs-1 and fs-2 had already landed consolidated coverage for nearly all of this slice's net
behavior. Duplicating it would have been ceremony, so this cell **added nothing where
coverage already existed** and extended the single suite with a real gap.

### Audit — slice net behavior vs. existing coverage

| Subject | Where | Status |
|---|---|---|
| Proof-tier matrix, table-driven (all classes × lanes) | `test_cells.mjs:1968` | pre-existing (fs-1) |
| **`bugfix` row unchanged** — first-class negative control | `test_cells.mjs:2024` | pre-existing (fs-1) |
| `existing-targeted-green` caps with no authored test | `test_cells.mjs:2063` | pre-existing (fs-1) |
| `test` in the 8-member `CHANGE_CLASSES` enum | `test_cells.mjs:1822`, `:1825` | pre-existing (fs-1) |
| a `test` cell caps on its own targeted green; `new_suite_reason` still governs it | `test_cells.mjs:2083` | pre-existing (fs-1) |
| branch (a) at both doors, capped-red, capped-green, bypass `total`, non-test cell, outside swarming | `test_cli_state.mjs:3212`–`3320` | pre-existing (fs-1) |
| staleness: source sha / deleted source / decision id / `plan.md` sha | `test_bee_cli.mjs:2274`, `:2291`, `:2302`, `:2317` | pre-existing (fs-2) |
| degradation: missing / unreadable / malformed / unknown version / partially-valid (rows **and** anchors) / no entry / hash-less row | `test_bee_cli.mjs:2331`–`2449` | pre-existing (fs-2) |
| command evidence + `--outputs-file` absent/unreadable/mismatched → re-prove, never fatal | `test_bee_cli.mjs:2420` | pre-existing (fs-2) |
| **no TTL** — backdated timestamps stay fresh | `test_bee_cli.mjs:2450` | pre-existing (fs-2) |
| **branch (b): the no-test-cell block** | `test_cli_state.mjs` (this cell) | **8 tests added** |

### What was added — extended, not created

**No new suite file**, so no `new_suite_reason` is owed (D3): all 8 checks extend
`packages/bee/tests/test_cli_state.mjs`, appended to fs-1's P4 block and ahead of the
si-3 isolation guard that must run last. They reuse that block's own fixtures
(`makeSwarmingRepo`, `writeCellFile`, `addCell`/`makeCell`); one local helper, `cappedCell`,
composes a capped fixture for the same reason `writeCellFile` exists — `capCell` refuses to
produce a capped cell without a passing verify (critical rule 2, untouched).

| # | check | proves |
|---|---|---|
| 1 | capped behavior + no test cell → `state set` refuses | (b) fires; names the cell; names `change_class: "test"`; `state.json` byte-identical |
| 2 | capped **api** + no test cell → refuses | (b) covers both behavior-bearing classes |
| 3 | `gate_bypass: "total"` | does **not** lift (b) |
| 4 | `state scribing-run` | second door enforces (b); stamps nothing on refusal |
| 5 | **bootstrap edge**: behavior + api cells all **open** | not blocked; phase advances |
| 6 | **no-behavior edge**: capped refactor + formatting + docs | never asked for a test cell; phase advances |
| 7 | capped behavior + capped-green test cell | door opens — how a real slice closes |
| 8 | capped behavior + **uncapped** test cell | branch (a)'s "not green" message, never (b)'s |

### Falsifiability — mutation-tested, not trusted because green

A guard is only worth its refusals, so each was checked against a deliberate bug:

| mutation | expected catch | result |
|---|---|---|
| `if (false && …)` — branch (b) disabled | the 4 blocking checks fail, the 4 negative controls stay green | **exactly that**: 4 failed (#1–#4), #5–#8 passed, 116 passed |
| trigger on **presence** instead of `capped` (`status !== 'nope'`) | the bootstrap edge fails | #5 failed **plus** fs-1's own "uncapped behavior cell is not this guard's business" — 2 failed, double-pinned |
| widen classes to any non-null (`changeClass !== null`) | the no-behavior edge fails | #6 failed, 1 failed — that edge is held by exactly this check |

All three reverted; the suite returned green each time. No existing test was weakened,
loosened, or deleted to reach green — the two pre-existing failures above are fs-1's
tests correctly catching a mutation, and both pass against the shipped implementation.

## Test economy

- **New suite files: 0** — `new_suite_reason` not owed (D3).
- Ratio, computed against the slice **aggregate** as the cell requires: 155 test lines
  added against **580** source lines across fs-1 + fs-2 + fs-3 = **0.27**. The D3 ratio
  ceiling is in any case scoped to `tiny`/`small` lanes and is a non-blocking warning;
  fs-3 is `standard`, so it does not fire.

## Verify

`node scripts/ledger_parity.mjs --check && node scripts/release_manifest.mjs --check && node scripts/run_verify.mjs`

```
ledger_parity  --check: .bee/bin/** matches the .bee/onboarding.json managed-hash ledger
release_manifest --check: 396 file(s) match stored manifest
PASS run_verify: 108 suite(s), concurrency=5, wall=73701ms   EXIT=0
```

Vendored `.bee/bin/bee.mjs` regenerated through `onboard_bee.mjs --apply`; skill trees and
the release manifest re-rendered. Ledger parity passes, so the vendored copy carries branch (b).

## Files

- `packages/bee/bee.mjs` — branch (b) in `guardTestCellDebt` (+ vendored `.bee/bin/bee.mjs`)
- `packages/bee/tests/test_cli_state.mjs` — the 8 consolidated checks
- `docs/history/codex-harness-hardening/release-manifest.json` — regenerated

## Note for the feature

Capping this cell green is what lets `flow-speedup` leave `swarming` — and it now leaves
through **both** branches rather than past a hole in one. The feature is the first user
of its own rule, and the first case branch (b) would have caught.
