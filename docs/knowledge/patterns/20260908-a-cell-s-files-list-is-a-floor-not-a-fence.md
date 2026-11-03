---
type: bee.pattern
title: "A cell's files list is a floor, not a fence"
description: "A behavior change drags callers, test fixtures, quoted docs and vendored twins with it, so the files list is the minimum a cell touches and the widening is reserved and recorded."
tags: [planning, cells, drift, reservations, docs]
timestamp: 2026-09-08
bee:
  id: pattern-20260908-a-cell-s-files-list-is-a-floor-not-a-fence
  lifecycle: active
  areas: [workflow-state]
---

Across seven features the same deviation is recorded in different words: the
worker had to reserve and edit a file the cell never named. Not once was the
worker wrong to do it. The planner simply cannot see, from the plan, every
place a behavior is restated.

Four kinds of file follow a behavior change and are missed the same way every
time:

- **Callers and exhaustive literals.** A new struct field makes every
  construction site red. `rbl-1` edited four unnamed files for exactly this.
- **Test fixtures that model the old contract.** A fixture is a claim about
  behavior; when the behavior closes, the fixture is now a claim the product
  refuses. `pse-1` found its cap fixture in a knowledge test file.
- **Docs and product-description pages that quote the changed string.** A
  quoted pattern, a refusal message, a flag spelling — each becomes false at
  the moment of the change. `scor-2` found the tag pattern quoted verbatim in
  a memory page.
- **Vendored twins and generated trees.** The prompts under `.bee/bin/prompts`,
  `.bee/onboarding.json`, the release manifest: a regen rewrites them, and a
  commit that leaves them out ships a stale record. `scor-1`, `pg-2` and
  `psa-6` each hit a different one.

## What to do

- Treat the files list as the minimum the cell is known to touch. Reserve the
  extra path before writing it — the reservation is what keeps a widened scope
  honest and visible to siblings — and record the widening as a deviation with
  its reason.
- Prefer the widening to the alternative. `pse-1` left one listed file
  unedited *because* editing it alone would go red, and recorded that too; the
  judgment call is which direction keeps the tree green, not whether to obey
  the list.
- A planner who wants a tighter list searches for the string being changed,
  not for the module being changed.

## What this is not

It is not permission to redraw a cell's scope. Editing a file another live
cell reserved is still a conflict, and taking a sibling's work is still
taking a sibling's work — `pse-2` and `psa-1` both stopped at that line and
said so.
