---
type: research
status: closed
claimed-by: none
blocked-by: none
---

## Question

Is the source skill's Python/bash a complete coordination choreography
worth learning from — as opposed to a set of primitives already
superseded by herdr 0.8.0's native CLI verbs?

## Answer

Yes, decisively — and this corrects the first distill pass, which
judged the scripts as primitives and concluded they were superseded.
Read as a state machine rather than as API calls, `broadcast.sh` is a
five-phase choreography whose ORDERING carries properties no set of
send/wait primitives provides:

1. Resolve and dedupe every target to a canonical pane id. An
   unresolved target aborts everything; nothing is sent. Alias dedupe
   stops one pane being sent to twice.
2. Fail-closed status filter. A parse failure, a null field, or an
   off-enum value is "unverifiable", never coerced to safe.
3. Baseline snapshot of every target BEFORE any dispatch. This is the
   single most load-bearing ordering fact: an agent that finishes in
   under a second would otherwise produce no "new output since I
   started watching", and the wait would time out on a success.
4. Re-check status immediately before each individual send. Phases 2-3
   take time proportional to the number of targets — that window is
   exactly when a pane flips to working or blocked.
5. Dispatch, then wait on every target concurrently, then aggregate. A
   send failure mid-fan-out records the failure and keeps going;
   abandoning the loop would throw away the results of agents already
   working.

Eight properties come only from that ordering: the fast-completion
race; stale-marker rejection (a completion marker must be present now
AND absent from the baseline, so a previous task's marker cannot be
credited to this send); the time-of-check/time-of-use race at dispatch;
fail-closed status everywhere; partial-failure isolation; mixed-result
exit aggregation (a wave where every SENT target succeeded still fails
if any target was dropped); bounded working-to-done re-polling; and
dedupe-before-preflight.

The tests are the real evidence: 29 of them, several written from
recorded regressions — an off-enum numeric status that a truthy check
let through, a null `pane` field that must reject without leaking a
traceback, a blocked pane whose status lookup ALSO fails and must not
stabilise into false-ready, "unverifiable was omitted from the final
aggregation". The fake herdr harness needs atomic write-then-rename
because concurrent waiters read its state file mid-write — the harness
had to solve a concurrency hazard in order to test concurrency at all.

Verdict: take the choreography and the test corpus as the
specification. Take none of the code. Almost nothing in it is
inherently POSIX — the subshell / `wait` / tmpfile layer exists only
because bash cannot return structured values from a subshell, and that
layer disappears entirely in Rust.

Logged as D03. Full distill:
docs/history/research/herdr-orchestrator-distill.md
