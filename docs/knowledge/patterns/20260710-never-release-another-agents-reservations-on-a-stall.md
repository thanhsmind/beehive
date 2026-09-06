---
type: bee.pattern
title: "Never release another agent's reservations on a stall signal"
description: "Never release another agent's reservations on a stall signal"
tags: [failure, swarming, reservations, orchestrator]
timestamp: 2026-07-10
bee:
  id: pattern-20260710-never-release-another-agents-reservations-on-a-stall
  lifecycle: active
  areas: [worktree-parallelism]
  sources: ["docs/history/learnings/critical-patterns.md#PAT13", "original feature: evolving-loop"]
  polarity: pitfall
  critical: true
---

# Never release another agent's reservations on a stall signal

A "stalled/killed" notification was trusted; the orchestrator released a live worker's reservations,
reset its claimed cell, and dispatched a duplicate. Nothing corrupted — the first worker finished and
the second returned `[NOOP]` — but the reservation guard was defeated by the orchestrator, not by a
race. Before declaring a worker dead, check for progress on disk over an interval. The lock did its
job; the person with the key opened the door.
