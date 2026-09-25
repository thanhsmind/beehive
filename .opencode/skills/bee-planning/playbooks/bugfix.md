# bugfix

1. Reproduce the symptom on the real interface.
2. Watch that reproduction FAIL before the fix. This step's rule already has a
   home — `bee-swarming/references/worker-details.md:33-35`
   ("red-before-green is craft, applied by judgment and enforced by review,
   not by flags"). Read it there; it is deliberately not copied here, so a
   cold execution worker never has to open a planning reference.
3. Find the mechanism, not the symptom — trace the bad value back to where
   it was made (`bee-principle-crash-site-versus-fault-site`).
4. Fix the mechanism.
5. Re-run the same reproduction, on the same interface. An inconclusive run,
   or a run on a different surface, is not a pass.

**When the cause is not known.** List the candidate causes. Each pass, test
the split that removes the most candidates, with runtime evidence, not a
reading of the code (`.bee/expertise/tests.md` ("Instrument before
guessing")). Revert every edit that a refuted hypothesis motivated. A
one-line fix whose cause is already known keeps the short path.

Proof line: the reproduction's output, red and then green, verbatim.
