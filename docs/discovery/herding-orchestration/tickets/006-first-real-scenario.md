---
type: grilling
status: closed
claimed-by: none
blocked-by: none
---

## Question

What is the ONE real scenario that proves the destination?

## Answer

A spawn-and-brief wave with collection: open N agents, each in its own
worktree, wait for each to become ready, hand each its brief, wait on
all of them concurrently, collect what each produced, and aggregate the
failures.

It was chosen over the smaller "fan out a question to N already-running
panes" because it runs the whole five-phase choreography AND forces the
one repair that blocks everything else — the dead spawn line. On herdr
0.8.0, `herdr agent start … --cwd …` returns `unknown option: --cwd`;
`agent start` now takes `<NAME> --kind <KIND> --pane <ID>` and never
creates layout, so the spawn becomes split-then-start. Until that is
fixed, no scenario that opens an agent can run at all.

The scenario must exercise at least one failure path for the proof to
mean anything — a target that is busy at preflight, or a send that
fails mid-fan-out while earlier agents keep working.

Logged as D06.
