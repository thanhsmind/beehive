---
type: bee.pattern
title: A pinned read-only worker type and a cell-execution dispatch are different jobs — never spawn one for the other
description: "bee-gather/bee-extract/bee-review are read-only by hard system contract; a cell that edits, runs git, or runs verify must never be dispatched as one of them, even when a skill doc offers it as a valid tier-matched choice."
tags: [swarming, agent-dispatch, bee-gather, tier]
timestamp: 2026-07-23
bee:
  id: pattern-20260723-pinned-readonly-type-vs-execution-dispatch
  lifecycle: active
  areas: [workflow-state]
  sources: [docs/history/learnings/20260723-backlog-auto-commit.md, "original feature: backlog-auto-commit (P78, cell backlog-auto-commit-1)", .bee/backlog.jsonl friction 2026-07-20 (dispatch prepare --kind cell) and 2026-07-23 (this recurrence)]
  polarity: pitfall
  critical: true
---

# A pinned read-only worker type and a cell-execution dispatch are different jobs — never spawn one for the other

A skill doc's "tier-matched pinned type" table (bee-gather for generation, bee-extract for
extraction, bee-review for review) is written for I/O-offload gather dispatches — narrow lookups
that only read and report a digest. It is not written for cell execution, even when the prose
doesn't say so explicitly. bee-gather's own system contract is unconditionally read-only: no
writes, no edits, no mutating commands, ever — no prompt can expand that scope. A cell-execution
worker (implement within `files`, run git, run the cell's `verify` command, cap the cell) spawned
as `subagent_type: "bee-gather"` is refused outright, at the cost of a full dispatch round-trip.
This has now recurred twice through two different code paths (`dispatch prepare --kind cell`, and
bee-swarming's own single-execution-worker spawn instructions), which is the signal that it's a
durable rule, not a one-off misreading. Before spawning any worker that will edit, run commands
with side effects, or run verify: use a `model` param (or the bare ceiling marker) and the
runtime's default/general subagent type — never a pinned bee-gather/bee-extract/bee-review type,
regardless of what a skill's spawn table appears to offer.
