---
type: bee.delivery
title: no-midslice-ask — delivery
description: "Delivery record proposed by bee knowledge promote for work item no-midslice-ask: 1 capped cell(s), 1 recorded deviation(s)."
timestamp: 2026-08-26
bee:
  id: no-midslice-ask-delivery
  lifecycle: active
  required_context: [.bee/lanes/no-midslice-ask.json]
  sources: [.bee/lanes/no-midslice-ask.json, .bee/cells/archive/no-midslice-ask/nma-1.json]
---

# no-midslice-ask — Delivery

## What shipped

- **nma-1** — Always-loaded layer now forbids mid-work continue-asks; slice boundary continues in the same turn (0 file(s) changed)

## Verify

Each cell below was capped only against a recorded passing verify result — bee refuses a cap without one.

- **nma-1** — `bee dev regen green; bee dev release-manifest --check clean; rg 'in the same turn' AGENTS.md skills/bee-swarming/SKILL.md packages/bee/AGENTS.block.md confirms the promoted rule in all three`

## Deviations

- **nma-1** — sync-ack: AGENTS.md touched only inside the agents-one-next-action rule body; the flagged rule agents-capture-line-at-close and its applied_at files are untouched by this diff

## Provenance

Proposed by `bee knowledge promote --work no-midslice-ask` from 1 capped cell trace(s) in `.bee/cells/` and the anchor `.bee/lanes/no-midslice-ask.json`. Every line above is copied from a trace or from the work item; nothing here is curated truth until a human or agent accepts it.

