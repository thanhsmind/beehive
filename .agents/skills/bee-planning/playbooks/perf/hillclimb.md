# hillclimb

You own the metric and the experiment's integrity. Supervise and review;
dispatch the attempts through `bee dispatch prepare`. This is for sustained,
iterative improvement of one measurable thing against a target ("hillclimb on
X", "make startup 50% faster", "systematically drive down <metric>", "keep
trying until <metric> improves by N%"). A one-off fix is `bug-fix` or
`perf-issue`; this is the loop.

Core discipline: one change, one measurement, keep or revert. Never stack
untested changes, and never claim a win from code inspection, because the data
decides and a stacked change has no single cause.

1. Ground the workload and architecture before choosing the ruler. Run
   `bee-how` over the target as a gather (`bee dispatch prepare --kind
   gather`), name the realistic workload dimensions that can move the result
   (data size, history, state, concurrency), and select a case that reproduces
   the user's complaint. If no case reproduces it, fix the repro instead of
   hillclimbing, because a ruler that cannot see the complaint climbs the
   wrong hill. Then fix one metric, the direction that counts as better, and a
   checkable stop predicate that pairs a target with a floor on attempts so a
   lucky early win cannot end the run (the example "at least 50% better than
   baseline and at least 10 iterations" is this shape). Use the user's numbers
   when given, otherwise agree them.
2. Build the measurement harness, prove its sensitivity, then freeze it
   (`bee-principle-build-the-lever`). Run contrasting realistic workloads and
   confirm the target case reproduces the symptom while easier cases separate
   as expected. If the ruler cannot distinguish them, revise the workload or
   metric. Once frozen, one repeatable command emits the metric, sampled
   enough to clear the noise (the median of several runs, not a single run);
   changing it invalidates every earlier number. Record the baseline metric
   and a green run of the regression gate (the tests that must keep passing)
   before any change, so a later red is known to be the change's.
3. Open the decision trail under `docs/history/<feature>/`: one row per
   attempt with id, hypothesis, change, before, after, delta, tests, verdict
   (kept or reverted) and note. This is the run's memory. Read it before each
   attempt so the search accumulates instead of circling. Keep the trail file
   outside the files an attempt touches, so a revert never removes it.
4. Ground each hypothesis in the architecture model from step 1, so it names a
   specific mechanism ("defer X off the boot path because it blocks first
   paint"), not "try memoizing something", because a named mechanism can be
   refuted and a vague try cannot.
5. Loop, one hypothesis per iteration:
   - Dispatch the change as a cell (`bee dispatch prepare --kind cell`) on the
     code role's configured model with a tight scope; supervise and review the
     diff rather than typing it, so the lead's context stays on the metric.
     When several independent hypotheses are live, fan them to parallel cells,
     each in its own worktree so they cannot collide
     (`bee-principle-separate-before-serializing-shared-state`).
   - Measure before and after with the frozen harness, and run the regression
     gate.
   - Accept only when the metric moves past noise and the gate stays green.
     Otherwise revert the change in full; a tweak that "might help" does not
     ride along. Revert an attempt that does not clear the noise.
   - One commit per accepted fix, staging only the files you changed (`git add
     <files>`, never `-A`), so each commit is one measured step. Log the row
     either way, kept or reverted.
   Each iteration ends in a check before the next begins, so a failure points
   at one attempt. If the run is unattended, borrow only the wake mechanism
   from the autonomous-run playbook (`playbooks/session/autonomous-run.md`),
   not its stop rule. This playbook's stop criteria below govern, so a plateau
   means pivot, not stop.
6. Push past the first plateau. A plateau is not a stop — it means pivot the
   hypothesis, not give up. On a stall, several rejects in a row, pivot
   category, combine near-misses, re-read the source, or try something more
   radical before concluding the hill is climbed. Correctness and simplicity
   outrank the number: revert a win that breaks behavior, and keep a
   simplification that holds the number.
7. When the ask is a sustained metric target, set the stop rule before the
   first attempt: the target reached, plus a minimum number of attempts. Stop
   when the predicate is met, or when the remaining ideas are genuinely
   marginal and not worth their cost. Never loosen the stop rule, and do not
   quit while cheap untried hypotheses remain, because a relaxed predicate
   turns the run into a report of effort. If you are stuck, surface it instead
   of spinning.
8. Land the accepted commits through the bee close and worktree merge path,
   in the order they landed so the metric's climb reads top to bottom: the work
   is capped, merged by `bee worktree merge`, and the `uat` acceptance at
   `bee close` stays the owner's.

"It feels faster" is not a result (per D2, decision `1593e365`).

Reply: the metric and target, baseline to final with the percent delta,
iterations run (kept vs reverted), each accepted fix on one line, the decision
trail path, and the best idea you would try next if pushed further.

Proof line: the baseline, the after, the delta, the command, and the artifact path.
Proof floor: the cap refuses a result weaker than this class's floor and names it; the table is the cap's own (`bee cells finish --help`).
