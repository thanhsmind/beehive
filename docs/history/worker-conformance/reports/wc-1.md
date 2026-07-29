# wc-1 — Stamp `trace.proof = "unrecorded"` when a cap carries no real proof

**Status:** [DONE]

**Outcome:** `capCell` now stamps a new inert trace field `trace.proof = "unrecorded"` on a
cap that recorded neither real verify output nor `verification_evidence`, computed after the
entire refusal chain has run, so the absence of proof arms the feature close-door instead of
passing silently.

**Files touched**

- `packages/bee/lib/cells.mjs` — the D10/D12/D14 predicate + the additive trace stamp
  (`isNoTestCommand` newly imported from `state.mjs`).
- `packages/bee/tests/test_cells.mjs` — four table-driven rows written red-first.

**Shape of the change**

- The marker is a **new** field, never a reuse of `feature_verify_pending` (D12): that local
  flag short-circuits six refusal sites, so reusing it would have voided D2's red-first tier
  and D6's brakes. `trace.proof` is read by nothing inside `capCell`.
- Marked only when **neither** channel carried proof (D14) — a tiny-lane `security` cell whose
  `red_failure_evidence` already passed the red-first door is never marked, even with an empty
  `verify_output`.
- Exempt: the explicit `--feature-verify-pending` path, and a repo declaring
  `commands.verify: "none"`. The exemption is keyed on `commands.verify` alone — deliberately
  narrower than `isNoTestRepo()`, because a repo with only `commands.test: "none"` can still
  run a real feature verify and must keep arming the door.

**Red-first proof (high-risk lane, D2)**

Tests written first, run red once at the real ship path and kept: `1 failed` —
`wc1-out-absent (output field omitted entirely): expected trace.proof "unrecorded", got undefined`.
After the implementation: `138 passed, 0 failed` in `test_cells.mjs`, and `40 passed, 0 failed`
in the sibling `test_cli_cells.mjs` (which caps through the CLI, including the no-test path).
These runs are development iterations — the cell's `verify` is owned by main at the feature
boundary, and this cell capped through `--feature-verify-pending` with no per-cell evidence.

**Negative controls (D7)** — the `"unrecorded"` shape buys no door bypass: a `security` tiny cell
and a `behavior` high-risk cell still refuse without `red_failure_evidence` (`cells.mjs:2135`),
and a new test file still refuses without `new_suite_reason` (`:1999`). Each refused cap leaves
the cell `claimed`, with no partial write.

## Consults

1 consult — advisor **fable** (`advisor-consult wc-1: fable`), read-only design review before cap.

- **Ask:** is the exemption keyed right (`commands.verify` alone vs `isNoTestRepo`), is the stamp
  reachable for any lane but `tiny` today, and does any consumer break on a new `trace.proof` key?
- **Answer:** AGREE on all three — the narrow key transcribes D12 literally and closes a real hole
  (a `commands.test: "none"` repo can still run a feature verify); nothing in `lib/` or `tests/`
  does trace set-equality or schema validation on a capped trace, and `state.mjs` already reads
  `trace.proof === "unrecorded"` deliberately (wc-2's door). **One correction adopted:** lane
  `spike` is in `LANES` (`cells.mjs:91`) but absent from the Decision 0004 door's lane list
  (`:2158`), so a spike cap is markable today exactly like `tiny` — a `wc1-out-spike` row was
  added to the first table to pin it.

## For the orchestrator

- **wc-3 coverage obligation.** Once wc-3 makes `:1956` and `:2164` non-blocking, two populations
  become stampable that no test here can reach through `capCell` — small/standard/high-risk caps
  with neither output nor evidence, and `bc: true` caps with no evidence. wc-2's door tests
  hand-write `trace.proof` onto fixtures, so that "refusal removed ⇒ marker appears" seam is
  proven nowhere yet. wc-3's rows should cover it end-to-end.
- **Known seam, deliberately untouched.** A repo declaring only `commands.test: "none"` with a
  cell whose `verify` is the sentinel hits `noTestWaiver` (`:1908`, keyed on the broader
  `isNoTestRepo`), gets the auto-waiver note as evidence (`:1909-1911`), and caps unmarked — even
  though such a repo can run a real feature verify. That is decision 55b951e1's locked semantics
  ("the auto note stands in as recorded evidence"), so wc-1 left it alone; worth recording in
  CONTEXT.md as a known seam rather than leaving it implicit.
- **Cosmetic nit, not taken:** `readConfig(root)` is now read twice inside the same lock
  (`:1908` and the stamp predicate). The second read is short-circuited to the rare markable path;
  hoisting it would change evaluation order at `:1908`, so it was left for a cell that owns that line.

Full trace and evidence: `.bee/cells/wc-1.json`.
