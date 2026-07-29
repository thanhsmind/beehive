# wc-2 — Arm both close-door predicates on the unrecorded marker

**Status:** [DONE] (revision pass — the cell was reopened on a NEEDS_REVISION verdict and re-capped)

**Outcome:** `featureVerifyDebt` and `testCellDebt` arm on `trace.proof = "unrecorded"` as well
as `trace.feature_verify = "pending"` (D11/D12); the freshness clock runs over the union of
both; a capped test cell whose pass was asserted with no recorded output no longer satisfies
the test-cell door. The revision adds the missing proof that the marker's **writer and reader
are actually connected**.

**Files touched**

- `packages/bee/lib/state.mjs` — both predicates widened; `newestPendingCapMs` → `newestOwedCapMs`
  over the union; the `owed` renderer and both FIX tails widened. *(original pass, commit 50bc9610)*
- `packages/bee/tests/test_bee_cli.mjs` — the hand-written fixture layer (`writeUnrecordedCappedCell`,
  `DEBT_KIND_UNRECORDED_FIXTURES`, the second generated door × kind matrix, union-staleness,
  union-clears, bypass-`total`, the `not green` branch, the negative control) *(original pass)*,
  plus the four end-to-end seam rows below *(revision)*.
- `docs/history/codex-harness-hardening/release-manifest.json`, `.bee/onboarding.json` — regen chain.

**Commits:** `50bc9610` (original) · `d52b86c8` (seam, first reader) · `6188af19` (seam, second reader)

**Full trace / evidence:** `.bee/cells/wc-2.json`

## Revision — what the judge caught and what closed it

The verdict was NEEDS_REVISION on exactly one of seven checks, `seam-end-to-end-unproven`:
`trace.proof = "unrecorded"` is written by `capCell` (`lib/cells.mjs:2239`) and read by both
door predicates (`lib/state.mjs:2527`, `:2592`), with **every** door row seeded by a
hand-written fixture — so a rename or shape drift on either side left both suites green and
the door silently dead.

Four rows now cross the seam in one run, and none of them writes the marker:

| Row | Reader crossed | What it drives |
|---|---|---|
| `wc-2 SEAM` | `featureVerifyDebt` | real `cells add/claim/verify --passed true` (no `--output`) `/cap` in a real git repo → asserts the on-disk cell carries what **capCell** stamped → real close door refuses, naming the cell and the marker |
| `wc-2 SEAM (mirror)` | same | identical flow with real verify output → no marker → the **same door opens** |
| `wc-2 SEAM (second reader)` | `testCellDebt` | same, on a `change_class: "test"` cap, with the feature-verify door cleared honestly first via a real `state feature-verify record`, so the refusal under test can only be the test-cell one |
| `wc-2 SEAM (second reader, mirror)` | same | real recorded output discharges the debt → door opens |

Lane `tiny` is the vehicle: decision 0004's "an assertion is not evidence" refusal
(`cells.mjs:2159`) is scoped to small+, so tiny is where a cap legally reaches the marker
today — which is precisely the D10 hole (`cells verify --passed true` with no `--output` is
legal). The marker computation is lane-independent, so the rows stay valid after this
feature's D1 loosens that refusal.

Every pre-existing hand-written fixture row was kept — pure insertion, zero deletions. Those
rows cover the door × kind matrix cheaply; this proof sits beside them.

**Verify:** `node packages/bee/tests/test_bee_cli.mjs` → 392 passed, 0 failed (388 before).
`node packages/bee/tests/test_misc.mjs` → 118 passed, 0 failed — the wave's byte-identity
friction is cleared, and the regen chain (skill trees → onboarding → release manifest) ran
before the cap.

**Cap channel:** `--override-judge`, audited. The recorded NEEDS_REVISION verdict blocks the
cap by design; a worker recording its own PASS via `judge-record` would be self-judging, so
the override carries the honest reason and re-judging remains the orchestrator's to run.

**Known gap (advisory, unchanged):** cells capped before wc-1 shipped carry no marker, so a
legacy asserted-pass test cell stays invisible to `testCellDebt`. Backfill is out of reach
from `state.mjs`.

**Friction:** the judge's failure signature named **both** readers, but my first revision
commit crossed only `featureVerifyDebt` — a `refactor` cell never enters `testCellDebt`'s
test-class branch at all. The advisor consult caught it; without it this cell would have
capped a second time believing a half-closed seam was closed. A one-reader proof reads as
complete because the row is green.

## Consults

2 consults total — **fable**, via Agent dispatch (`advisor-consult wc-2: fable`).

1. *(original pass)* **Ask:** five questions on the pre-cap diff — does the union clock close
   D12's staleness hole; any broken consumer of the old return shape; can arming on any capped
   cell wall a feature in; is marker-keying correct under D14; is the new coverage load-bearing.
   **Answer:** cap-safe; independently re-ran the suites. One soft spot fixed in-cell: each
   unrecorded fixture now carries a kind-discriminating `refusal` regex.
2. *(revision pass)* **Ask:** does commit `d52b86c8` actually close the seam — walk all four
   drift directions, check the mirror for vacuity, check the tiny-lane vehicle's durability
   past D1, check flakiness, confirm no existing row was weakened. **Answer:** cap-safe; all
   four drift directions caught, mirror genuine, tiny lane durable, no flakiness, pure
   insertion. **One residual raised and fixed in this cell:** the second reader
   (`state.mjs:2592`) had no end-to-end row — closed by commit `6188af19`.
