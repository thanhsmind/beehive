# wc-2 — Arm both close-door predicates on the unrecorded marker

**Status:** [DONE]

**Outcome:** `featureVerifyDebt` and `testCellDebt` now arm on `trace.proof = "unrecorded"`
as well as `trace.feature_verify = "pending"` (D11/D12); the freshness clock runs over the
union of both, so a green feature-verify newer than the newest pending cap but older than a
newer unrecorded cap no longer opens the door; a capped test cell whose pass was asserted
with no recorded output no longer satisfies the test-cell door.

**Files touched**

- `packages/bee/lib/state.mjs` — both predicates widened; `newestPendingCapMs` → `newestOwedCapMs`
  computed over the union; the `feature-verify` kind's `owed` renderer, `FEATURE_VERIFY_FIX_TAIL`,
  and `testCellDebtFixTail('not-green')` widened to name the second road honestly.
- `packages/bee/tests/test_bee_cli.mjs` — `writeUnrecordedCappedCell` /
  `seedFreshGreenFeatureVerify` helpers, `DEBT_KIND_UNRECORDED_FIXTURES` (`{seed, refusal}` per
  kind), the coverage meta-check extended to demand one, a second generated door × kind matrix
  over the new marker, plus the union-staleness case, the union-clears case, bypass-`total` on
  both doors, the `not green` (never `missing`) branch for an unrecorded test cell, and a
  negative control proving only the literal marker arms.

**Commit:** `50bc9610`

**Full trace / evidence:** `.bee/cells/wc-2.json`

**Design notes**

- No new debt kind was added — the coverage-completeness meta-check did not demand one, so both
  existing kinds were widened and every registered door inherits the change through
  `guardFeatureDebt`. The structural check forbidding direct detector calls from `bee.mjs`
  stays green.
- `testCellDebt` keys on the marker rather than re-deriving "asserted pass with empty output"
  structurally: re-deriving would re-open D14 (a red-first cell holding real
  `red_failure_evidence` would be flagged despite holding the strongest proof in the system).

**Known gap (advisory, not fixed here):** cells capped before wc-1 ships carry no marker, so a
legacy asserted-pass test cell stays invisible to `testCellDebt`. Backfill is out of this cell's
reach from `state.mjs`.

**Friction:** `packages/bee/lib/{cells,state}.mjs` have diverged from their `.bee/bin/lib`
vendored copies, so `test_misc.mjs`'s byte-identity guard is red for the whole wave.
`cells.mjs` was already stale before this cell touched anything. Clearing it is the
orchestrator's wave-close regen (`onboard_bee.mjs`), which this cell's `wave-barrier` ack
forbids the worker to run.

## Consults

1 consult — **fable**, via Agent dispatch (`advisor-consult wc-2: fable`).

- **Ask:** five questions on the pre-cap diff — does the union clock actually close D12's
  staleness hole; is any consumer of the old return shape broken; can arming on any capped cell
  wall a feature in; is keying on the marker (vs re-deriving structurally) correct under D14;
  is the new coverage load-bearing.
- **Answer:** cap-safe. Advisor independently re-ran `test_bee_cli.mjs` (388/0) and
  `test_cli_state.mjs` (120/0). Union clock closes the hole; the only consumer is
  `FEATURE_DEBT_KINDS[0].detect` in the same file; the remedy is always executable and
  `commands.verify: "none"` repos are never marked; marker-keying is correct precisely because
  re-deriving would re-open D14. One soft spot raised and **fixed in this cell**: the generated
  matrix row asserted only `/unrecorded/`, `/FIX:/` and the feature name, so a regression in the
  `test-cell` fixture's green seeding would have let a feature-verify refusal satisfy the row —
  each fixture now carries a kind-discriminating `refusal` regex, and the meta-check demands it.
  Second note left open: the legacy-cap backfill gap above.
