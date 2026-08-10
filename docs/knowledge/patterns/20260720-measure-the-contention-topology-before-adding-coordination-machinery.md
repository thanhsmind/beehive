---
type: bee.pattern
title: Measure the contention topology before adding coordination machinery
description: Measure the contention topology before adding coordination machinery
tags: [process, parallelism, contention, test-topology, discovery-over-registry]
timestamp: 2026-07-20
bee:
  id: pattern-20260720-measure-the-contention-topology-before-adding-coordination-machinery
  lifecycle: active
  sources: ["docs/history/learnings/critical-patterns.md#PAT44", "original feature: contention-split"]
  polarity: practice
  critical: false
---

# Measure the contention topology before adding coordination machinery

The "sessions wait for each other" complaint was 90% artificial: a 9.6k-line
test monolith every feature edited, a hand-written suite registry every
feature registered into, and an unlocked whole-tree regen. Splitting tests
per module + convention-based discovery + a locked tmp-swap render removed
the collisions outright — coordination machinery (holds ledger) is only for
the residue of REAL same-module overlap. Corollary proven the same day: the
hand registry had silently never run 4 existing test files, one of which was
red against the live hook (discovery > registration, for correctness too).
