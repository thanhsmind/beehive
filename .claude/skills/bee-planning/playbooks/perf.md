# perf

1. Before the first run, write down the result that would change the
   decision and the noise floor — the run-to-run spread on an unchanged
   build. Stop when the spread is larger than the effect: a number with no
   decision written before it gets fitted to whatever it shows.
2. Capture a baseline with the real command or trace, and record the number —
   without it, no later number means anything.
3. State the hypothesis — a mechanism the baseline shows. First ask whether
   the slow path must exist at all: a deleted path beats a faster one.
4. Change ONE thing, so that the delta has one cause.
5. Re-measure the same way, with the same command, so that the delta measures
   the change and not the method.
6. Keep the win, discard the loss, and record BOTH numbers — a loss on record
   stops the next attempt from trying it again.

**When the ask is a sustained metric target.** Prove the harness separates
the target case, then freeze it. Report the median of several runs. Set the
stop rule before the first attempt: the target reached, plus a minimum
number of attempts. Log one row per attempt. Revert an attempt that does
not clear the noise. Never loosen the stop rule. A plateau is not a stop —
it means pivot the hypothesis, not give up.

"It feels faster" is not a result (per D2, decision `1593e365`).

Proof line: the baseline, the after, the delta, the command, and the
artifact path.
