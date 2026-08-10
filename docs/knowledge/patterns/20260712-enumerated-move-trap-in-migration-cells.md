---
type: bee.pattern
title: Enumerated-move trap in migration cells
description: Enumerated-move trap in migration cells
tags: [failure, planning, filesystem, validation]
timestamp: 2026-07-12
bee:
  id: pattern-20260712-enumerated-move-trap-in-migration-cells
  lifecycle: active
  sources: ["docs/history/learnings/critical-patterns.md#PAT24", "original feature: bee-footprint"]
  polarity: pitfall
  critical: false
---

# Enumerated-move trap in migration cells

Exhaustive/destructive ops over a mutable directory (move-all, delete-all, "must end empty")
glob the children at execution time — never enumerate a fixed name list. Validation's own
artifacts (spikes, probes) may occupy that namespace by the time the cell runs; the cell
reviewer caught a deterministic verify failure this would have shipped.
