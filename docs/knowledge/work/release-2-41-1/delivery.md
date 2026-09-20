---
type: bee.delivery
title: release-2-41-1 — delivery
description: "Delivery record proposed by bee knowledge promote for work item release-2-41-1: 1 capped cell(s), 1 recorded deviation(s)."
timestamp: 2026-09-17
bee:
  id: release-2-41-1-delivery
  lifecycle: active
  required_context: [docs/history/release-2-41-1/CONTEXT.md, docs/history/release-2-41-1/plan.md]
  sources: [docs/history/release-2-41-1/CONTEXT.md, docs/history/release-2-41-1/plan.md, .bee/cells/archive/release-2-41-1/rel411-1.json]
---

# release-2-41-1 — Delivery

## What shipped

- **rel411-1** — Released bee 2.41.1 through scripts/release.sh (3 file(s) changed)

## Verify

Each cell below was capped only against a recorded passing verify result — bee refuses a cap without one.

- **rel411-1** — `bash scripts/release.sh 2.41.1 && bee dev release-manifest --check` — the script ran the full declared suite before tagging; git ls-remote shows v2.41.1 at ebfc0422, gh run 35217169787 success, gh release lists linux, windows and SHA256SUMS

## Deviations

- **rel411-1** — the first deployment dispatch was a cell dispatch and the second was refused on a moved main commit; both refused before any mutation

## Provenance

Proposed by `bee knowledge promote --work release-2-41-1` from 1 capped cell trace(s) in `.bee/cells/` and the anchor `docs/history/release-2-41-1/CONTEXT.md`, `docs/history/release-2-41-1/plan.md`. Every line above is copied from a trace or from the work item; nothing here is curated truth until a human or agent accepts it.
