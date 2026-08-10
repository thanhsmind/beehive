---
type: bee.pattern
title: "Cross-cell contracts and census carriers are plan-authoring work, not validation work"
description: "Cross-cell contracts and census carriers are plan-authoring work, not validation work"
tags: [process, planning, cells, verify-authoring, census]
timestamp: 2026-07-12
bee:
  id: pattern-20260712-cross-cell-contracts-and-census-carriers-are-plan
  lifecycle: active
  sources: ["docs/history/learnings/critical-patterns.md#PAT25", "original feature: review-on-demand", docs/history/learnings/20260712-review-on-demand.md]
  polarity: pitfall
  critical: false
---

# Cross-cell contracts and census carriers are plan-authoring work, not validation work

Recurred twice in one feature, in different shapes: a cell read a ledger field its upstream cell
never wrote; a whole-token verify ban collided with a line the same cell declared protected. And a
removal census scoped as "sweep the strays" missed the one file carrying the exact retired phrase.
At plan-authoring time, mechanically: (1) grep every value a cell READS against the sibling cell
that WRITES it, verbatim; (2) grep every whole-token negative-grep ban against every line the plan
promises to leave untouched; (3) for a census cell, run the real repo-root grep and write file:line
carriers into the cell — and if the tested artifact is self-referential (repo AGENTS.md, anything a
suite only fixtures), the verify greps the LIVE file. Independent reviewers converging is the
backstop, not the mechanism.
**Full entry:** docs/history/learnings/20260712-review-on-demand.md
