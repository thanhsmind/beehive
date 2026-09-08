---
type: bee.delivery
title: release-pointer-fix — delivery
description: "Delivery record proposed by bee knowledge promote for work item release-pointer-fix: 1 capped cell(s), 1 recorded deviation(s)."
timestamp: 2026-09-06
bee:
  id: release-pointer-fix-delivery
  lifecycle: active
  required_context: [.bee/lanes/release-pointer-fix.json]
  sources: [.bee/lanes/release-pointer-fix.json, .bee/cells/archive/release-pointer-fix/rpf-1.json]
---

# release-pointer-fix — Delivery

## What shipped

- **rpf-1** — Cross-skill pointer now bee-hive/references/routing-and-contracts.md; overview.md links are bare paths; regen green (4 file(s) changed)

## Verify

Each cell below was capped only against a recorded passing verify result — bee refuses a cap without one.

- **rpf-1** — `bee dev release-manifest --check — green:static — regenerated skill copies and manifest match`

## Deviations

- **rpf-1** — sync-ack: skills/bee-swarming/SKILL.md changed only in pointer form; regen copies refreshed

## Provenance

Proposed by `bee knowledge promote --work release-pointer-fix` from 1 capped cell trace(s) in `.bee/cells/` and the anchor `.bee/lanes/release-pointer-fix.json`. Every line above is copied from a trace or from the work item; nothing here is curated truth until a human or agent accepts it.
