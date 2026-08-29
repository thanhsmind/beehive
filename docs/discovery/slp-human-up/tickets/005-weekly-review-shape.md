---
type: grilling
status: closed
claimed-by: none
blocked-by: none
---

## Question

The weekly review (28a75c87) needs its ritual fixed: (a) trigger — a
schedule/cron, a human gesture, or the supervisor's own cadence? (b) output
home — a waggledance page, a letter in each repo's mailbox, or both? (c) the
reader — human only, or do repo leads consume their slice automatically?
(d) how a learning flows back — a capture stub pushed into the owning repo's
queue, or a backlog item the repo's lead triages?

## Answer

Resolved 2026-08-29 by the user (decision 30799303): (a) trigger = a weekly
schedule owned by the deterministic layer; (b) output home = the cockpit repo
(per 12be1c0b), waggledance renders it; (c) reader = the human; (d) each
learning lands as a PROPOSED backlog item dropped into its related repo via
that repo's own bee CLI, triaged accept-or-decline by that repo's lead.
Companion decisions settled in the same round: 423871d7 (the supervisor is a
cold tick under an external scheduler — a machine shutdown pauses ticks,
nothing to resume, nothing lost) and 3cfd9980 (poor-work signals →
intervention record recommends an advisor consult; the struggling repo's own
lead summons it).
