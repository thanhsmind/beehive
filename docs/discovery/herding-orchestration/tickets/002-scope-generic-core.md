---
type: grilling
status: closed
claimed-by: none
blocked-by: none
---

## Question

Is the orchestrator bee-specific, or a generic coordination core that
bee-herding merely uses?

## Answer

A generic core. It knows workers, tasks, waiting, result collection and
failure aggregation — not cells, lanes, or worktrees. bee-herding
becomes its first client. Chosen by the owner over a bee-specialised
build, on two grounds: it matches "sau có thể phục vụ nhiều việc", and
it separates the part that must be proven on Windows from the part that
is bound to herdr.

Consequence carried forward: the core needs a backend seam (ticket 007)
and a way to describe a scenario (still fog).

Logged as D02.
