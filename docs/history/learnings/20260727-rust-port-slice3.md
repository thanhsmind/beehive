---
date: 2026-07-27
feature: rust-port
categories: [proof-discipline, instrumentation, planning, review-layering]
severity: high
tags: [read-dedup, counters, budget-supersession, frozen-plan, executed-falsification]
---

# rust-port Slice 3 — the defects were in the planning artifacts, and only execution found the one that mattered

Three cells: build a read-accounting instrument and characterise today's reads with it; remove the duplicated reads; measure and settle the budget. All three capped. Status assembly went from roughly 52 ms to roughly 24.5 ms at the 95th percentile against a node baseline near 180 ms, with each store now read once per invocation instead of four, six and two times.

## What Happened

**Eleven findings landed before a line of code was written.** A plan-checker raised five blockers, a cold-pickup reviewer raised five criticals, and the advisor raised a sixth blocker — all against artifacts I authored. The most instructive:

- The plan **reproduced a prior decision's read counts instead of re-deriving them**, and inherited that decision's omissions: one scan site was missing entirely and another was marked unconditional when it is feature-gated. The total came out right because the two errors cancelled. The cell then *forbade correcting the numbers*, which would have sent a worker hunting a phantom instrument bug the first time a fixture disagreed.
- The archive-fallback trap paragraph **named the wrong path** — the real layout has a feature subdirectory. The required guard test, built from that text, would have gone red against unchanged code, and the natural repair would have been to loosen the test.
- The budget cell's honest-failure branch was **unsatisfiable**: the gate is a strict less-than, so setting the budget to the measured value fails on the run that set it, while the same cell forbade widening.
- The advisor's blocker was visible only between cells: **every instrument measured the headline command while the signature change landed on two per-event hooks**, where an eager shared load would have made a conditional scan unconditional with every check still green.

**Then execution found what reading could not.** After the first cell capped clean, the goal-check judge injected two real journal reads and a real directory scan at the level the next cell would hoist to. The counters did not move and the test passed: three real reads invisible. The instrument built to judge the deduplication was gameable by that very deduplication, and it was one judge pass from becoming the ground truth the dependent cell built against.

**The remaining passes were all won by execution too**: reverting the counter placement to prove the new regression test discriminates; forcing the shared memo eager to prove laziness is what keeps the hook path cheap; resolving dependencies against the active-only inventory to prove the archive guard bites; deleting a binary to prove a verify leg self-heals.

## Root Cause

1. **A cited number is a claim, not a fact.** The decision's counts were accurate for one fixture and were copied as though they were fixture-independent. Nothing in the pipeline re-derives a quoted anchor against current source, so an inherited omission propagates silently and gains authority with each restatement.
2. **Reading has a ceiling, and this slice hit it.** Three independent reading passes each found things the previous pass's own stated scope should have covered — and none found the gameable counter, because an instrument's gameability is a property of execution. The passes are not cleanly partitioned checks; they are three overlapping attempts at "do these claims match reality", and their success came from stacking attempts rather than any one being complete.
3. **A must-have can be satisfied by evidence that does not establish it.** The placement requirement was checked by a test showing a primitive call and a reader call agree — trivially true when both route through the same frame, and silent about the lower path that bypassed the counter.

## Recommendation

1. **Re-derive every number you cite, from source, in the artifact that cites it.** A decision's summary figures are evidence of what was true when measured, on the fixture measured. Quote the derivation (call sites, which are unconditional, what gates the rest), not the total — and never write a prohibition against correcting a number you did not re-derive.
2. **Give every instrument an injection test that survives the change it judges.** Add a real extra operation at the level the refactor will use and require the count to move; keep it permanently. Ask the survival question per counter, not per instrument: in this slice four counters survived a hoist and one did not, and only naming them one at a time surfaced which.
3. **Budget one adversarial execution per review layer, not one more reading pass.** A reviewer that can mutate a scratch copy and watch a specific test go red produces verdicts a reader cannot. Where the environment blocks that, fix the environment — the weaker round in the previous slice was the one whose judge could not run experiments.
4. **State what an instrument cannot see, in its own output.** The dependency-read counter bundles the archive fallback, so losing that fallback moves nothing; the printed table now says so. An instrument that names its blind spots keeps the next worker from trusting it past its edge.
5. **Encode arithmetic rules as checks, not prose.** The budget is now derived (round the measured percentile up to the next 5 ms step, add one more, and carry three times that to CI). That is a one-line invariant a test can assert on every re-measurement instead of relying on a human redoing it correctly.
6. **A frozen document that has been superseded needs a forward pointer.** The plan stayed stale while the cells were repaired around it, and a plan-first reader meets the wrong numbers with nothing pointing forward. The freeze protects what was approved; it should not require a reader to reconstruct the correction from three cell records. Filed as a bee change rather than resolved unilaterally.

## What shipped besides the code

The budget is settled at 30 ms with the 20 ms target explicitly superseded rather than left standing — the slice was allowed to conclude "not reached" and did. The remaining time is dominated by the crash-recovery block and the work-item listing, neither of which is a repeated-read problem, so the next reduction means doing less work rather than reading fewer times.

## Method note

This close ran one analyst (failure lens) rather than three. The pattern and decision lenses were covered from first-hand session evidence, and the deviation is recorded here rather than left silent.
