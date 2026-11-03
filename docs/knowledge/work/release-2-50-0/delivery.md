---
type: bee.delivery
title: release-2-50-0 — delivery
description: "Delivery record proposed by bee knowledge promote for work item release-2-50-0: 1 capped cell(s), 1 recorded deviation(s)."
timestamp: 2026-10-06
bee:
  id: release-2-50-0-delivery
  lifecycle: active
  required_context: [docs/history/release-2-50-0/CONTEXT.md, docs/history/release-2-50-0/plan.md]
  sources: [docs/history/release-2-50-0/CONTEXT.md, docs/history/release-2-50-0/plan.md, .bee/cells/rel500-1.json]
---

# release-2-50-0 — Delivery

## What shipped

- **rel500-1** — Publish release 2.50.0 through the sanctioned script (0 file(s) changed)

## Verify

Each cell below was capped only against a recorded passing verify result — bee refuses a cap without one.

- **rel500-1** — `bash scripts/release.sh 2.50.0 && bee dev release-manifest --check` — script printed OK after the full suite (4494 passed, 0 failed); CI https://github.com/thanhsmind/beehive/actions/runs/37415025003 success; release https://github.com/thanhsmind/beehive/releases/tag/v…

## Deviations

- **rel500-1** — deploy role on the claude runtime returns a read-only gather payload that cannot run the script; the leader ran scripts/release.sh itself with the prepared dispatch id 6a80f163

## Provenance

Proposed by `bee knowledge promote --work release-2-50-0` from 1 capped cell trace(s) in `.bee/cells/` and the anchor `docs/history/release-2-50-0/CONTEXT.md`, `docs/history/release-2-50-0/plan.md`. Every line above is copied from a trace or from the work item; nothing here is curated truth until a human or agent accepts it.
