---
type: bee.delivery
title: release-2-74-0 — delivery
description: "Delivery record proposed by bee knowledge promote for work item release-2-74-0: 1 capped cell(s), 1 recorded deviation(s)."
timestamp: 2026-11-05
bee:
  id: release-2-74-0-delivery
  lifecycle: active
  required_context: [docs/history/release-2-74-0/CONTEXT.md, docs/history/release-2-74-0/plan.md]
  sources: [docs/history/release-2-74-0/CONTEXT.md, docs/history/release-2-74-0/plan.md, .bee/cells/rel740-1.json]
---

# release-2-74-0 — Delivery

## What shipped

- **rel740-1** — bee 2.74.0 is live: tag pushed, CI green, 6 assets published (3 file(s) changed)

## Verify

Each cell below was capped only against a recorded passing verify result — bee refuses a cap without one.

- **rel740-1** — `bash scripts/release.sh 2.74.0 && bee dev release-manifest --check` — script printed release OK bee 2.74.0 is live after its full test gate; CI https://github.com/thanhsmind/beehive/actions/runs/41960335609

## Deviations

- **rel740-1** — permit came from a deployment-stage gather dispatch carrying --release-version, not the cell dispatch, whose permit had no version

## Provenance

Proposed by `bee knowledge promote --work release-2-74-0` from 1 capped cell trace(s) in `.bee/cells/` and the anchor `docs/history/release-2-74-0/CONTEXT.md`, `docs/history/release-2-74-0/plan.md`. Every line above is copied from a trace or from the work item; nothing here is curated truth until a human or agent accepts it.
