---
type: bee.delivery
title: release-2-52-0 — delivery
description: "Delivery record proposed by bee knowledge promote for work item release-2-52-0: 1 capped cell(s), 0 recorded deviation(s)."
timestamp: 2026-10-13
bee:
  id: release-2-52-0-delivery
  lifecycle: active
  required_context: [docs/history/release-2-52-0/CONTEXT.md, docs/history/release-2-52-0/plan.md]
  sources: [docs/history/release-2-52-0/CONTEXT.md, docs/history/release-2-52-0/plan.md, .bee/cells/rel520-1.json]
---

# release-2-52-0 — Delivery

## What shipped

- **rel520-1** — Released bee 2.52.0: tag v2.52.0 at 313be241f, release-binaries green, five binaries and SHA256SUMS published (3 file(s) changed)

## Verify

Each cell below was capped only against a recorded passing verify result — bee refuses a cap without one.

- **rel520-1** — `bash scripts/release.sh 2.52.0 && bee dev release-manifest --check` — the script ran the full declared cargo suite before tagging and printed its final OK line, the manifest check matched 419 files; release https://github.com/thanhsmind/beehive/releases/tag/v2.52.0 car…

## Deviations

None recorded in the capped cell traces.

## Provenance

Proposed by `bee knowledge promote --work release-2-52-0` from 1 capped cell trace(s) in `.bee/cells/` and the anchor `docs/history/release-2-52-0/CONTEXT.md`, `docs/history/release-2-52-0/plan.md`. Every line above is copied from a trace or from the work item; nothing here is curated truth until a human or agent accepts it.
