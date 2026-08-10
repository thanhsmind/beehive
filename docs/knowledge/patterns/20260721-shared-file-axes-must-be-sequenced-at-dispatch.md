---
type: bee.pattern
title: "Shared-file axes must be sequenced at dispatch time; a worker's \"watcher\" dies with its turn"
description: "Shared-file axes must be sequenced at dispatch time; a worker's \"watcher\" dies with its turn"
tags: [process, orchestration, deadlock, reservations, subagents]
timestamp: 2026-07-21
bee:
  id: pattern-20260721-shared-file-axes-must-be-sequenced-at-dispatch
  lifecycle: active
  sources: ["docs/history/learnings/critical-patterns.md#PAT45", "original feature: hardening-1-7-9"]
  polarity: pitfall
  critical: false
---

# Shared-file axes must be sequenced at dispatch time; a worker's "watcher" dies with its turn

Two lessons from a real circular wait (worker A held the dispatcher file B
needed; B held the test files A wanted quiet): (1) when several cells share
one hotspot file, pick the order UP FRONT and tell each worker who precedes
it — polite polling from both sides deadlocks. (2) A returned subagent's
"background monitor" does not survive its turn: the orchestrator owns
resumption — track who waits on what, and SendMessage the waiter the moment
its blocker clears. Corollary CLI gap (filed): reservation release scopes by
agent/cell only, so releasing ONE path drops all holds.
