---
type: bee.delivery
title: uat-close-instruction-parity — delivery
description: "Delivery record proposed by bee knowledge promote for work item uat-close-instruction-parity: 1 capped cell(s), 3 recorded deviation(s)."
timestamp: 2026-08-26
bee:
  id: uat-close-instruction-parity-delivery
  lifecycle: active
  required_context: [.bee/lanes/uat-close-instruction-parity.json]
  sources: [.bee/lanes/uat-close-instruction-parity.json, .bee/cells/archive/uat-close-instruction-parity/ucip-1.json]
---

# uat-close-instruction-parity — Delivery

## What shipped

- **ucip-1** — Make uat_stop close placement read merge-first across the instruction layer (0 file(s) changed)

## Verify

Each cell below was capped only against a recorded passing verify result — bee refuses a cap without one.

- **ucip-1** — `bee dev regen green, then bee dev release-manifest --check clean; rg confirms no instruction file still says the uat door defaults to merge time`

## Deviations

- **ucip-1** — docs-lane ran in the MAIN checkout while a sibling session was live (models-show-verb): file sets fully disjoint (skills/docs vs packages/bee-rs), commit path-scoped through the concurrent-worker guard
- **ucip-1** — docs-lane in MAIN with a live sibling session; disjoint files, path-scoped commit
- **ucip-1** — sync-ack: AGENTS.md touched only in the worktree-first rule tail (uat_stop placement); the flagged rule agents-capture-line-at-close and its applied_at files are untouched by this diff

## Provenance

Proposed by `bee knowledge promote --work uat-close-instruction-parity` from 1 capped cell trace(s) in `.bee/cells/` and the anchor `.bee/lanes/uat-close-instruction-parity.json`. Every line above is copied from a trace or from the work item; nothing here is curated truth until a human or agent accepts it.

