---
type: research
status: closed
claimed-by:
blocked-by: none
---

## Question

Where does the supervisor heartbeat RUN, and on what model? Options to
cost out: (a) a cron/scheduled session (bee triggers, external cron,
paseo heartbeat), (b) a dedicated pane in the herding cockpit, (c) a
`bee herding`-style role invoked per tick. Model side: the open
fall-through role set (model-role-split decision 06e49368) should make
a cheap `supervisor` role plus a stronger escalation role pure
configuration — verify that, and name what (if any) new machinery a
15-minute heartbeat needs. Constraints R2/R3/R4 hold.

## Answer

Option (c): the supervisor runs as a new `--role supervisor` of the
existing native control loop (`bee herding control-loop --interval
900`), spawning COLD each tick — no persistent session, no context
bloat. Model is pure configuration thanks to the open fall-through
role set (`06e49368`): `models.claude.supervisor` on a cheap model,
with the existing `advisor` role as the semantic escalation path.
Tool surface stays enumerated read/query only, so R2/R3/R4 hold.
Cron/external schedulers rejected (no repo/lifecycle awareness,
platform-split); a persistent cockpit pane rejected (context
accumulation) — a pane merely HOSTING the loop is fine.
Full findings: docs/history/research/slp-supervisor-placement.md.
Logged as decision — see MAP "Decisions so far".
