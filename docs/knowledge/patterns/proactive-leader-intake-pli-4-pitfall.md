---
type: bee.pattern
title: An area-scoped edit re-renders the area index, not the root index
description: A plan that names docs/knowledge/index.md for an area-spec description change is naming the wrong generated file
timestamp: 2026-08-30
bee:
  id: proactive-leader-intake-pli-4-pitfall
  lifecycle: draft
  areas: [advisor-protocol, doctrine-layer]
  sources: [.bee/cells/pli-4.json]
  polarity: pitfall
---

# An area-scoped edit re-renders the area index, not the root index

## What happened

Cell pli-4's plan named `docs/knowledge/index.md` as the file to reserve
and commit alongside a changed area-spec description. The description
change actually re-renders `docs/knowledge/areas/doctrine-layer/index.md`
(the AREA index), not the root index — the cell reserved and committed the
correct one once this was noticed.

## The lesson

`bee knowledge index` renders one index per directory level that contains a
changed concept — a description edit inside an area spec regenerates that
area's own index.md, not the root one. Check which index actually changed
(`bee knowledge index --check`) rather than assuming the root index is
always the target.

## Status

Candidate only. Naming the pattern, generalizing it beyond this cell, and
moving `bee.lifecycle` to `active` are a human or agent decision.
