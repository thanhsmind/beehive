# wc-3 advisor consult — fable

**Ask:** Is part 5 of the net-behavior story ("no bypass level passes any of it") a genuine
coverage hole, or would authoring per-door bypass rows duplicate a property already pinned?
Also: regrade parts 1-4, and judge the smallest honest shape for any gap.

**Answer (digest):**

1. Genuine hole, but HALF the size claimed — my inventory had a factual error. I missed
   `test_bee_cli.mjs:2274` ("p1-3(F3): gate_bypass total and --waive-scribing-debt both fail to
   lift the feature-SWAP door"), which crosses the feature-swap door under bypass total with the
   pending marker. Corrected: phase-departure crossed (`:2209`, `:2608`, `:2900`), start-feature
   crossed (`:2608`, `:2900`), feature-swap crossed (`:2274`) — **scribing-run crossed by
   nothing**. One door naked, not two. The surviving threat is a handler-level bypass
   short-circuit sitting above `guardFeatureDebt`; the structural check at `:2767` cannot see it.
2. Generate over `DEBT_DOORS` (4 rows) for inheritance, on two conditions: the block comment must
   state the TRUE provenance (three doors already covered piecemeal, only scribing-run naked — in
   this file the comments are the audit trail), and use ONE marker (`unrecorded`) only. A
   doors × markers × kinds cube is the row inflation D4 exists to stop, because the handler-level
   short-circuit sits above the seam where markers and kinds become distinguishable.
3. Keep the debt-free open half — no debt-free door crossing under bypass exists anywhere, so
   "`:2886`/`:3244` make it redundant" is false (those run on a default config). But the stated
   non-vacuity rationale was wrong: if config seeding silently failed, BOTH halves still pass.
   Add an explicit assert that bypass is live (`status --json` exposes `gate_bypass_level`) — the
   existing hand-written bypass rows all share this weakness; this block can be the first without it.
4. Parts 1-4 COVERED grades all stand after re-reading every citation. One nuance, not a
   downgrade: in the part-4 status loop the start-feature door is deliberately exempted from the
   strict wording assert (`:3279-3294`) because its own nonterminal-cells guard answers first —
   deliberate, and wc-3 must not re-open it.

**Adopted:** all four points. The judgment was corrected to "3 of 4 doors already crossed under
bypass; scribing-run never", and one generated `DEBT_DOORS` block was authored on the unrecorded
marker only, with a truthful provenance comment and a live-bypass assert in every row.
