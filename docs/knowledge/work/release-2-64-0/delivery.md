---
type: bee.delivery
title: release-2-64-0 — delivery
description: "Delivery record proposed by bee knowledge promote for work item release-2-64-0: 1 capped cell(s), 2 recorded deviation(s)."
timestamp: 2026-10-27
bee:
  id: release-2-64-0-delivery
  lifecycle: active
  required_context: [docs/history/release-2-64-0/CONTEXT.md, docs/history/release-2-64-0/plan.md]
  sources: [docs/history/release-2-64-0/CONTEXT.md, docs/history/release-2-64-0/plan.md, .bee/cells/rel640-1.json]
---

# release-2-64-0 — Delivery

## What shipped

- **rel640-1** — Publish release 2.64.0 through the sanctioned script (3 file(s) changed)

## Verify

Each cell below was capped only against a recorded passing verify result — bee refuses a cap without one.

- **rel640-1** — `bash scripts/release.sh 2.64.0 && bee dev release-manifest --check` — release cell, evidence https://github.com/thanhsmind/beehive/actions/runs/40528796003 and /mnt/data/Projects/goglbe/beehive/.bee/mailbox/job-1793113776906-873705-1/report-1.md; the script ran the ful…

## Deviations

- **rel640-1** — BEE_OPENCODE_SUITE_ALLOW_SKIP=1 added to the deploy dispatch env, as in releases 2.60.0 to 2.63.0, because the local opencode is a wrapper script
- **rel640-1** — herding ceiling raised from the door 1800 s to 5400 s because the full declared suite exceeded 1800 s earlier the same day

## Provenance

Proposed by `bee knowledge promote --work release-2-64-0` from 1 capped cell trace(s) in `.bee/cells/` and the anchor `docs/history/release-2-64-0/CONTEXT.md`, `docs/history/release-2-64-0/plan.md`. Every line above is copied from a trace or from the work item; nothing here is curated truth until a human or agent accepts it.
