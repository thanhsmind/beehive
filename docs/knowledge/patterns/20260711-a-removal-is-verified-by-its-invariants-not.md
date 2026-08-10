---
type: bee.pattern
title: "A removal is verified by its invariants, not the names it deletes"
description: "A removal is verified by its invariants, not the names it deletes"
tags: [failure, removal-census, derived-constants, verification]
timestamp: 2026-07-11
bee:
  id: pattern-20260711-a-removal-is-verified-by-its-invariants-not
  lifecycle: active
  sources: ["docs/history/learnings/critical-patterns.md#PAT18", "original feature: learnings-pair-relocation", docs/history/learnings/20260711-learnings-pair-relocation.md]
  polarity: pitfall
  critical: false
---

# A removal is verified by its invariants, not the names it deletes

Removing a named entity and grepping the name is not enough — two P1s slipped a small-lane
census that way. When censusing a removal: grep from the **repo root** (exclude only declared
archaeology), include **bare-token variants** of the removed names, and re-derive **every
numeric constant computed from the removed thing's size** (caps, counts, "N reviewers",
table totals) — put the recomputed number in the positive verify grep. A capacity constant
that encodes the old roster size will silently refill the freed slots.

**Full entry:** docs/history/learnings/20260711-learnings-pair-relocation.md
