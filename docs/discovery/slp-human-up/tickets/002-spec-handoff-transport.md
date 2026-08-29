---
type: research
status: open
claimed-by: none
blocked-by: none
---

## Question

What is the concrete transport for a cross-repo spec drop (6f039742,
5bed1c01)? Facts needed from the waggledance and bee codebases: (a) exact
semantics of `waggledance_dispatch`/`waggledance_await`/`waggledance_runs` —
opt-in surface, busy-pane behavior, delivery guarantee, payload shape;
(b) how a dropped file + backlog item becomes work bee's route/claim actually
picks up in the receiving repo (backlog add? PBI? mailbox letter?); (c) where
the correlation id and provenance line live on each side without cross-repo
storage; (d) what herdr's dispatch role needs so a fresh spec item qualifies
as "safe backlog work" it may ignite.

## Answer

(open)
