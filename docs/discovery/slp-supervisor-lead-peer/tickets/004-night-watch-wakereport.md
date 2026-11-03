---
type: grilling
status: closed
claimed-by:
blocked-by: none
---

## Question

What does night watch mean in bee terms? Concretely: what does
Presence(away) map to (a gate_bypass level? a waiting-on kind? a new
flag), which items QUEUE for the human (gate questions, uat doors,
escalations) versus which still run, what the single WakeReport
contains (spec: ≤10 lines, act-on-it-now), and what counts as an
UrgentAlert that skips the queue.

## Answer

(D 9f5cd250) Presence is an explicit lightweight away/back mark with
exactly two effects: it defines the report accumulation window, and
non-urgent asks queue silently while away. Gate behavior and bypass
levels are NOT touched by presence — permission control never hides
in a presence flag. On back: exactly one WakeReport — a markdown file
of at most 10 lines, four sections (what happened / what was decided /
what needs you / next action) — plus a single push notification.
UrgentAlerts skip the queue as always. Voice rendering is out of
scope for this effort.

Post-close narrowing: c706053e adds a NARROW opt-in silence-is-consent
mode — with it enabled, a non-gate queued ask may auto-proceed after a
user-set timeout, logged and prominent in the WakeReport; gates and
one-way low-confidence asks still always wait. a8f4b8ab sorts the queue
by the confidence×door predicate; 66c4c251 sorts the WakeReport's
assumptions by impact-if-wrong descending.

slp-human-up's **83baf03f** later generalizes c706053e's silence-is-consent
mode into a delegated-decision tier: the supervisor may decide a matter
on the human's behalf, not just auto-proceed a queued ask, when scope
is small, the action is reversible, the observation is proven, and it
stays inside protocol.
