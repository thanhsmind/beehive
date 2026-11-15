# bug-fix

You own this task. Plan, review, verify. Dispatch the investigation and the
fix through `bee dispatch prepare` and stay in the lead.

Be scientific. Every shipped line traces to runtime evidence. A
belt-and-suspenders line that "might help" is a hypothesis, not a fix; it does
not ship. When evidence refutes a hypothesis, revert what it motivated. The
smallest change the evidence justifies ships, nothing more. The same
discipline holds for a perf defect, where the evidence is the trace
(`playbooks/perf.md`).

1. Reproduce it yourself on the matching surface, because the user met the
   symptom there, not in a unit. Drive that surface through the project's
   `verify-app` skill when it has one, otherwise through the real interface
   itself. Do not hand the repro to the user. A debug or instrumentation
   protocol that says to ask the user does not override this; you drive the
   instrumented runtime. Ask the user only with a stated, specific reason the
   surface cannot reach the target, and only after driving it as far as it
   goes. A bug that will not reproduce directly is forced: synthesize the
   trigger, tighten the conditions, or instrument until it fires. A bug you
   cannot reproduce, you cannot prove fixed.
2. Watch that reproduction FAIL before the fix. This step's rule already has a
   home — `bee-swarming/references/worker-details.md:33-35`
   ("red-before-green is craft, applied by judgment and enforced by review,
   not by flags"). Read it there; it is deliberately not copied here, so a
   cold execution worker never has to open a planning reference.
3. Binary-search the cause, and find the mechanism, not the symptom — trace
   the bad value back to where it was made
   (`bee-principle-crash-site-versus-fault-site`). Form the candidate
   hypotheses, then rule them out until one survives. Seed them with
   `bee-how` over the affected subsystem and `bee-why` for its regression
   history, run as two parallel gathers (`bee dispatch prepare --kind
   gather`). Each pass, test the split that removes the most candidates, with
   runtime evidence, not a reading of the code (`.bee/expertise/tests.md`
   ("Instrument before guessing")). When program state is unclear, add
   instrumentation or logging and read it as the code runs. Do not guess.
   Confirm the surviving mechanism with runtime evidence before the
   `bee-architect` and `bee-interrogate` fan-out of step 4; a design grounded
   on a plausible but unconfirmed cause can be unanimously wrong while the real
   cause sits one subsystem over.
4. Plan the fix, then fix the mechanism — a guard at the symptom leaves the
   cause free to strike again. When the fix crosses a function boundary, run
   `bee-architect` first and write down the contract the fix keeps, because a
   boundary change decided in the diff alone is decided without a second
   design to compare. Run `bee-interrogate` over the plan when it is
   contested, because independent reviewers catch the cause the planner
   anchored on. Dispatch the implementation as a cell
   (`bee dispatch prepare --kind cell`) with a specific scope, and review the
   diff.
5. Re-run the same reproduction, on the same surface, because only that run
   proves this bug is gone. An inconclusive run, or a run on a different
   surface, is not a pass; flag it. Unit tests show branch behavior, not bug
   absence.
6. Stage the commits so the failing repro lands before the fix in git
   history; the diff tells the story. See `bee-tdd` for the failing-test-first
   cadence when the bug has a cheap local test path, because the test written
   first is the repro that shows the fix is the cause; skip it when the test
   would be expensive, integration-heavy or unclear.

**When the cause is not known.** A symptom that turns on event order — a late,
doubled or reordered message, a retry, a stuck flow — takes the `statechart`
model of `feature/feature.md` step 3 as one candidate split. A fix in code that a
project-local formal verify skill covers reruns that skill (`feature/feature.md`
step 8). Revert every edit that a refuted hypothesis motivated. A one-line fix
whose cause is already known keeps the short path. If the deciding fact took a
deep dig to find, propose the cheap, always-on signal that would have shown it
at first look.

Reply: what was broken, the root cause, the fix, how you verified.

Proof line: the reproduction's output, red and then green, verbatim.
Proof floor: the cap refuses a result weaker than this class's floor and names it; the table is the cap's own (`bee cells finish --help`).
