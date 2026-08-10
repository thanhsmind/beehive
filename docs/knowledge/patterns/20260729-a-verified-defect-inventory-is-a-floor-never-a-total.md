---
type: bee.pattern
title: "A verified defect inventory is a floor, never a total"
description: "A verified defect inventory is a floor, never a total"
tags: [failure, planning, scoping, discovery, cell-authoring]
timestamp: 2026-07-29
bee:
  id: pattern-20260729-a-verified-defect-inventory-is-a-floor-never-a-total
  lifecycle: active
  sources: ["original feature: budget-fence-removal", docs/history/learnings/20260729-budget-fence-removal.md]
  polarity: pitfall
  critical: false
---

# A verified defect inventory is a floor, never a total

An inventory whose every row was confirmed by direct read feels like a total. It is not. It is a
total only if the pass that produced it was exhaustive, and verification of the rows says nothing
about that.

`budget-fence-removal` enumerated 13 stale pointers, each verified. The cell was told to treat the
table as a floor and re-sweep; it found **18**. Two of the extra were rows already in the table
whose file the cell's scope had omitted — an orchestrator error the table could not surface, because
a table cannot report what was never looked at.

The tell was visible before the cell ran: one inventory row had been found only while verifying its
neighbours. A row discovered as a side effect of checking another row proves the pass that produced
the others was incomplete. When that tell appears, the enumeration is a starting point, and the cell
owes a fresh discovery run plus **its own count** — not the count it was handed.

A sweep that returns materially more than its inventory is a scope event, not a completeness
result: the shape was costed against the wrong number, so the finding goes back to the human before
the cell keeps executing.

**Full entry:** docs/history/learnings/20260729-budget-fence-removal.md
