---
type: bee.delivery
title: release-2-63-0 — delivery
description: "Delivery record proposed by bee knowledge promote for work item release-2-63-0: 1 capped cell(s), 1 recorded deviation(s)."
timestamp: 2026-10-27
bee:
  id: release-2-63-0-delivery
  lifecycle: active
  required_context: [docs/history/release-2-63-0/CONTEXT.md, docs/history/release-2-63-0/plan.md]
  sources: [docs/history/release-2-63-0/CONTEXT.md, docs/history/release-2-63-0/plan.md, .bee/cells/rel630-1.json]
---

# release-2-63-0 — Delivery

## What shipped

- **rel630-1** — Publish release 2.63.0 through the sanctioned script (0 file(s) changed)

## Verify

Each cell below was capped only against a recorded passing verify result — bee refuses a cap without one.

- **rel630-1** — `bash scripts/release.sh 2.63.0 && bee dev release-manifest --check` — release cell, evidence https://github.com/thanhsmind/beehive/actions/runs/40469699223 and /mnt/data/Projects/goglbe/beehive/.bee/mailbox/job-1793087259677-3516293-1/report-1.md

## Deviations

- **rel630-1** — BEE_OPENCODE_SUITE_ALLOW_SKIP=1 added to the deploy dispatch env, as in releases 2.60.0 to 2.62.0, because the local opencode is a wrapper script

## Provenance

Proposed by `bee knowledge promote --work release-2-63-0` from 1 capped cell trace(s) in `.bee/cells/` and the anchor `docs/history/release-2-63-0/CONTEXT.md`, `docs/history/release-2-63-0/plan.md`. Every line above is copied from a trace or from the work item; nothing here is curated truth until a human or agent accepts it.
