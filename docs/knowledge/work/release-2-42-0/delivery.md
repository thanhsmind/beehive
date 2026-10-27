---
type: bee.delivery
title: release-2-42-0 — delivery
description: "Delivery record proposed by bee knowledge promote for work item release-2-42-0: 1 capped cell(s), 1 recorded deviation(s)."
timestamp: 2026-09-21
bee:
  id: release-2-42-0-delivery
  lifecycle: active
  required_context: [docs/history/release-2-42-0/CONTEXT.md, docs/history/release-2-42-0/plan.md]
  sources: [docs/history/release-2-42-0/CONTEXT.md, docs/history/release-2-42-0/plan.md, .bee/cells/rel420-1.json]
---

# release-2-42-0 — Delivery

## What shipped

- **rel420-1** — Publish release 2.42.0 through the sanctioned script (0 file(s) changed)

## Verify

Each cell below was capped only against a recorded passing verify result — bee refuses a cap without one.

- **rel420-1** — `bash scripts/release.sh 2.42.0 && bee dev release-manifest --check` — script printed 'release OK bee 2.42.0 is live — installers now serve v2.42.0'; leader confirmed the remote tag, CI conclusion success, and 3 release assets itself; manifest check 376 files match

## Deviations

- **rel420-1** — The release commit is made by scripts/release.sh and carries no cell trailer, so the cap uses --commit-pending, as every release lane does.

## Provenance

Proposed by `bee knowledge promote --work release-2-42-0` from 1 capped cell trace(s) in `.bee/cells/` and the anchor `docs/history/release-2-42-0/CONTEXT.md`, `docs/history/release-2-42-0/plan.md`. Every line above is copied from a trace or from the work item; nothing here is curated truth until a human or agent accepts it.
