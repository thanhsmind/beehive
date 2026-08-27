---
type: grilling
status: closed
claimed-by:
blocked-by: none (002 closed — supervisor is a cold control-loop tick, so the channel must survive between ticks)
---

## Question

How does a supervisor's open question REACH a live working session,
and does the human see interventions in real time or only in reports
(spec §10 f)? Candidates: bee mailbox, SendMessage between sessions,
pane injection in the cockpit. Also: the frequency cap (spec: never
twice on the same point; second time = escalate) — where is that state
kept?

## Answer

(D c80debd7) Interventions are FILE RECORDS in a mailbox the target
session reads at its next turn boundary — never pane injection
mid-turn. The record is the source of truth and carries the
frequency-cap state (same point twice = escalate, never repeat).
Ordinary interventions reach the human only through the daily/wake
reports; danger-class UrgentAlerts notify immediately. This suits the
cold-tick supervisor (322695d6): a persistent record is what survives
between ticks. Spec §10 (f) thereby resolved: not real-time.
