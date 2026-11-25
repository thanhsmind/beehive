---
type: bee.pattern
title: "Superseded: a pinned read-only worker type and a cell-execution dispatch are different jobs"
description: "Retired 2026-11-26. The type names it warned about (bee-gather, bee-extract, bee-review) no longer exist, and its remedy — hand-pick a model param and a general subagent type — is now refused by the dispatch door. Read the resolver pattern that superseded it."
tags: [swarming, agent-dispatch, tier]
timestamp: 2026-07-23
bee:
  id: pattern-20260723-pinned-readonly-type-vs-execution-dispatch
  lifecycle: superseded
  areas: [workflow-state]
  sources: [docs/history/learnings/20260723-backlog-auto-commit.md, "original feature: backlog-auto-commit (P78, cell backlog-auto-commit-1)", .claude/agents/bee-worker.md, .claude/agents/bee-forager.md, docs/history/dispatch-one-door/CONTEXT.md]
  polarity: pitfall
  critical: false
  superseded_by: pattern-20260831-prose-that-recommends-a-shape-drifts-from-the-resolver-that-computes-it
---

# Superseded: a pinned read-only worker type and a cell-execution dispatch are different jobs

Read when: an old note or skill names bee-gather, bee-extract or bee-review as
a worker type, or tells you to pass a `model` param with a general subagent
type.

This pattern was true on 2026-07-23 and is retired. Its lesson still holds:
a read-only gather type never runs a cell. Its type names and its remedy do
not.

What changed:

- The read-only gather types are now `bee-forager` and `bee-scout`, and the
  review type is `bee-guard`. A cell runs as `bee-worker` (or `bee-comb` for
  rendered UI). Each one's contract is in `.claude/agents/`.
- The old remedy — "use a `model` param and the runtime's general subagent
  type" — is now the mistake. The dispatch door picks the type and the model:
  run `bee dispatch prepare --kind cell|gather|reviewer|advisor --json` and
  run the exact tool and payload it returns (dispatch-one-door D1). A
  hand-picked type or model is refused or rewritten by the model guard.

The current rule lives in
`patterns/20260831-prose-that-recommends-a-shape-drifts-from-the-resolver-that-computes-it.md`.
