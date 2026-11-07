---
type: bee.delivery
title: worktree-entry-routing — delivery
description: "Delivery record proposed by bee knowledge promote for work item worktree-entry-routing: 1 capped cell(s), 2 recorded deviation(s)."
timestamp: 2026-09-05
bee:
  id: worktree-entry-routing-delivery
  lifecycle: active
  required_context: [docs/history/worktree-entry-routing/CONTEXT.md]
  sources: [docs/history/worktree-entry-routing/CONTEXT.md, .bee/cells/archive/worktree-entry-routing/wer-1.json]
---

# worktree-entry-routing — Delivery

## What shipped

- **wer-1** — Separated native vs external transport in worktree dispatch instructions (2 file(s) changed)

## Verify

Each cell below was capped only against a recorded passing verify result — bee refuses a cap without one.

- **wer-1** — `grep -q 'external.*cwd' skills/bee-hive/references/routing-and-contracts.md || grep -q 'herding.*cwd' skills/bee-hive/references/routing-and-contracts.md`

## Deviations

- **wer-1** — Edited AGENTS.md instead of routing-and-contracts.md — the problematic instruction was in AGENTS.md, routing-and-contracts.md had no such content — found a better route
- **wer-1** — sync-ack: Cell predicted routing-and-contracts.md but actual fix was in AGENTS.md where the instruction lived

## Provenance

Proposed by `bee knowledge promote --work worktree-entry-routing` from 1 capped cell trace(s) in `.bee/cells/` and the anchor `docs/history/worktree-entry-routing/CONTEXT.md`. Every line above is copied from a trace or from the work item; nothing here is curated truth until a human or agent accepts it.
