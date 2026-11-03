---
type: bee.delivery
title: stale-worktree-lock — delivery
description: "Delivery record proposed by bee knowledge promote for work item stale-worktree-lock: 1 capped cell(s), 1 recorded deviation(s)."
timestamp: 2026-08-26
bee:
  id: stale-worktree-lock-delivery
  lifecycle: active
  required_context: [.bee/lanes/stale-worktree-lock.json]
  sources: [.bee/lanes/stale-worktree-lock.json, .bee/cells/archive/stale-worktree-lock/swl-1.json]
---

# stale-worktree-lock — Delivery

## What shipped

- **swl-1** — Released this session's expired worktree lock and pruned the merged human-mailbox worktree, reclaiming 2.1 GB (0 file(s) changed)

## Verify

Each cell below was capped only against a recorded passing verify result — bee refuses a cap without one.

- **swl-1** — `git worktree list | grep -c human-mailbox`

## Deviations

- **swl-1** — The lock and the idle gate formed a small deadlock: the lock could only be released by a session doing bee work, and releasing it was not itself bee work. Opened a tiny cell rather than disabling guards.idle_gate, which the refusal itself offers as a last resort. Kind: hit an unforeseen obstacle.

## Provenance

Proposed by `bee knowledge promote --work stale-worktree-lock` from 1 capped cell trace(s) in `.bee/cells/` and the anchor `.bee/lanes/stale-worktree-lock.json`. Every line above is copied from a trace or from the work item; nothing here is curated truth until a human or agent accepts it.

