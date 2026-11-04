---
type: bee.delivery
title: release-2-67-0 — delivery
description: "Delivery record proposed by bee knowledge promote for work item release-2-67-0: 1 capped cell(s), 2 recorded deviation(s)."
timestamp: 2026-10-30
bee:
  id: release-2-67-0-delivery
  lifecycle: active
  areas: [onboarding]
  required_context: [docs/history/release-2-67-0/CONTEXT.md, docs/history/release-2-67-0/plan.md]
  sources: [docs/history/release-2-67-0/CONTEXT.md, docs/history/release-2-67-0/plan.md, .bee/cells/rel670-1.json]
---

# release-2-67-0 — Delivery

## What shipped

- **rel670-1** — bee 2.67.0 is live (3 file(s) changed)

## Verify

Each cell below was capped only against a recorded passing verify result — bee refuses a cap without one.

- **rel670-1** — `bash scripts/release.sh 2.67.0 && bee dev release-manifest --check` — script printed OK after the declared cargo suite (opencode suite allowed to skip as in 2.60.0-2.66.0), tag push, green release-binaries run https://github.com/thanhsmind/beehive/actions/runs/41045668…

## Deviations

- **rel670-1** — release.sh ran inline in the leader under permit 243d147d, as releases 2.58.0-2.66.0 did — the paseo deploy worker is slow and had timed out earlier in the day — found a better route
- **rel670-1** — First run under permit ecd0cee0 went red on the pinned flag-vocabulary test; PINNED_FLAG_COUNT moved to 234 with decision ea32ab90 and a fresh permit was issued — something else had to be fixed first

## Provenance

Proposed by `bee knowledge promote --work release-2-67-0` from 1 capped cell trace(s) in `.bee/cells/` and the anchor `docs/history/release-2-67-0/CONTEXT.md`, `docs/history/release-2-67-0/plan.md`. Every line above is copied from a trace or from the work item; nothing here is curated truth until a human or agent accepts it.
