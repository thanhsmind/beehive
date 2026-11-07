---
type: bee.delivery
title: release-2-72-0 — delivery
description: "Delivery record proposed by bee knowledge promote for work item release-2-72-0: 1 capped cell(s), 1 recorded deviation(s)."
timestamp: 2026-11-03
bee:
  id: release-2-72-0-delivery
  lifecycle: active
  required_context: [docs/history/release-2-72-0/CONTEXT.md, docs/history/release-2-72-0/plan.md]
  sources: [docs/history/release-2-72-0/CONTEXT.md, docs/history/release-2-72-0/plan.md, cell rel720-1 (trace purged from .bee/cells/)]
---

# release-2-72-0 — Delivery

## What shipped

- **rel720-1** — Released 2.72.0: tag pushed, release-binaries green, assets published (0 file(s) changed)

## Verify

Each cell below was capped only against a recorded passing verify result — bee refuses a cap without one.

- **rel720-1** — `bash scripts/release.sh 2.72.0 && bee dev release-manifest --check` — full declared suite ran before tagging; CI https://github.com/thanhsmind/beehive/actions/runs/41615050777 green; release https://github.com/thanhsmind/beehive/releases/tag/v2.72.0 carries 5 binaries …

## Deviations

- **rel720-1** — the leader ran the script inline under deploy dispatch ddf6bcb7 because a herded worker may not push; the first permit b3c001d2 was bound to an older main commit and was refused before any change

## Provenance

Proposed by `bee knowledge promote --work release-2-72-0` from 1 capped cell trace(s) in `.bee/cells/` and the anchor `docs/history/release-2-72-0/CONTEXT.md`, `docs/history/release-2-72-0/plan.md`. Every line above is copied from a trace or from the work item; nothing here is curated truth until a human or agent accepts it.
