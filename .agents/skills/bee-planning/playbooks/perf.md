# perf

1. Capture a baseline with the real command or trace, and record the number.
2. State the hypothesis — a mechanism the baseline shows. First ask whether
   the slow path must exist at all: a deleted path beats a faster one.
3. Change ONE thing.
4. Re-measure the same way, with the same command.
5. Keep the win, discard the loss, and record BOTH numbers.

**When the ask is a sustained metric target.** Prove the harness separates
the target case, then freeze it. Report the median of several runs. Set the
stop rule before the first attempt: the target reached, plus a minimum
number of attempts. Log one row per attempt. Revert an attempt that does
not clear the noise. Never loosen the stop rule. A plateau is not a stop —
it means pivot the hypothesis, not give up.

"It feels faster" is not a result (per D2, decision `1593e365`).

Proof line: the baseline, the after, the delta, the command, and the
artifact path.
