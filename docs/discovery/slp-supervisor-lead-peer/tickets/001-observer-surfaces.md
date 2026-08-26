---
type: research
status: closed
claimed-by:
blocked-by: none
---

## Question

What can a supervisor/detector actually READ in bee today, and which
of the spec's seven signal types (self-correction, struggling-loop,
boundary-approach, big-decision, test-on-unstable-contract, budget-80,
danger-op) have a bee-native observable? Candidate surfaces: tmux
cockpit pane transcripts, the herding activity hook, waiting-on marks,
`bee state session list` heartbeats, the wave ledger, cells/claims,
decisions stream. Name each surface with a file/command anchor and say
what it can and cannot see.

## Answer

bee already exposes SEVEN read surfaces a supervisor/detector can
poll: pane transcripts + screen classifier, the activity hook records
(5 states, tool failures, content-free), waiting-on marks, the session
registry with 90s liveness, the wave ledger + occupancy, cells/claims
with retry budgets, and the decisions stream + triggers. Of the
spec's seven signals, THREE are observable day 1 — struggling-loop
(cell budgets, PostToolUseFailure bursts), big-decision (decisions
stream, gates, waiting-on), danger-op (write-guard refusals, blocked
screens, secret scrub). FOUR need new machinery: self-correction
(needs a scrollback/transcript scanner — the true Detector),
boundary-approach (needs StopAndAsk discipline, ticket 005),
test-on-unstable-contract (needs the contract_status label, ticket
007), budget-80 (only 100% exhaustion exists; an 80% threshold event
is new telemetry). Full findings with anchors:
docs/history/research/slp-observer-surfaces.md.
