---
type: bee.pattern
title: bee dev regen's blast radius exceeds the files a cell names
description: Regen renders every vendored skill tree, not just the ones the plan lists — reserve and commit the full render set, and leave leader-owned planning artifacts to the orchestrator
timestamp: 2026-08-30
bee:
  id: proactive-leader-intake-pli-3-pitfall
  lifecycle: draft
  areas: [advisor-protocol, doctrine-layer]
  sources: [.bee/cells/pli-3.json]
  polarity: pitfall
---

# bee dev regen's blast radius exceeds the files a cell names

## What happened

Cell pli-3's file list named only `.claude/skills` and `.claude-plugin`, but
`bee dev regen` also renders `.agents` and `.codex-plugin` from the same
source edit — both had to be reserved before committing. The cell also left
`docs/history/proactive-leader-intake/*.md` and a temp dispatch file
unstaged (leader-owned planning artifacts, not files the cell names), and
deliberately did not touch a still-stale R16b sentence in a different area
spec outside its file list, flagging that edit as an orchestrator-scope
call rather than silently taking it.

## The lesson

A cell that edits any source `bee dev regen` renders must expect to reserve
every vendored projection regen touches, not only the ones the plan
enumerated — check the regen output's changed-file list before committing.
Separately: leave a file plainly outside the cell's own scope untouched and
named, rather than either skipping the sync or overreaching into another
cell's or the orchestrator's territory.

## Status

Candidate only. Naming the pattern, generalizing it beyond this cell, and
moving `bee.lifecycle` to `active` are a human or agent decision.
