---
type: bee.pattern
title: Two correct cells can leave the join between them unbuilt
description: "When every cell in a slice meets its own must_haves and passes its own judge, the join between them — the writer of a state one cell introduces and the reader another points at — can go unbuilt with all checks green, because the defect lives between artifacts and no cell-scoped check can see it. Measured on herding-orchestration's wave ledger: three individually correct cells shipped a ledger nothing writes on the dispatch path, no ledger file on disk, and an occupancy read returning a confident non-degraded zero forever, so a four-slot cap can never engage."
timestamp: 2026-08-19
bee:
  id: pattern-20260819-untested-join-at-slice-scale
  lifecycle: active
  areas: [workflow-state, bee-herding]
  sources: ["capture stub 58f58424 (herding-orchestration, captured in its worktree)", skills/bee-herding/references/role-dispatch.md, packages/bee-rs/crates/bee/src/herding/wave_ledger.rs]
---

Sixth instance of the untested-join shape, and the first at
architecture scale: two things each built, the join between them not —
but this time the two things are cells, not functions. One cell built
an append-only ledger
(packages/bee-rs/crates/bee/src/herding/wave_ledger.rs). A second
built the verb that writes rows into it. A third pointed the control
role's occupancy read at it. Every cell met its own must_haves and
passed its own judge, and nothing wrote a row on the path the control
role actually takes, because the writer lives inside a verb the role
never calls. Measured: no ledger file on disk, and the occupancy read
returning a confident non-degraded zero forever, so a four-slot cap
can never engage — strictly worse than the pane counting it replaced,
which at least saw real panes.

The transferable part is the planning defect. The plan said the
occupancy check moves to the ledger and never asked who writes rows
on the dispatch path. A cell-level must_have cannot catch this,
because each cell was individually correct; only a question asked of
the slice can. The same asymmetry appeared in review: a judge asked
to read a document as a cold agent, start to finish, found what six
mutation-driven judges did not — some defects live between artifacts
and are invisible to any check scoped to one.

**The rule:** for any state a plan introduces, name its writer and
its reader as separate obligations in the plan itself, and make the
last cell of the slice prove that a value written by one is read by
the other. This is the function-level untested join
(docs/knowledge/patterns/20260819-the-join-between-two-tested-parts-is-what-nobody-tests.md)
one level up, and a cell-scoped judge structurally cannot see it.
