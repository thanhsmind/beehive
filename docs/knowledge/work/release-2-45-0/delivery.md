---
type: bee.delivery
title: release-2-45-0 — delivery
description: "Delivery record proposed by bee knowledge promote for work item release-2-45-0: 1 capped cell(s), 1 recorded deviation(s)."
timestamp: 2026-09-22
bee:
  id: release-2-45-0-delivery
  lifecycle: active
  required_context: [docs/history/release-2-45-0/CONTEXT.md, docs/history/release-2-45-0/plan.md]
  sources: [docs/history/release-2-45-0/CONTEXT.md, docs/history/release-2-45-0/plan.md, .bee/cells/rel450-1.json]
---

# release-2-45-0 — Delivery

## What shipped

- **rel450-1** — Published bee 2.45.0 through scripts/release.sh; the script printed its final OK line (3 file(s) changed)

## Verify

Each cell below was capped only against a recorded passing verify result — bee refuses a cap without one.

- **rel450-1** — `bash scripts/release.sh 2.45.0 && bee dev release-manifest --check` — the script ran the full declared suite before tagging, tag v2.45.0 pushed, release-binaries run 35716171894 green, five binaries plus SHA256SUMS verified, manifest check exit 0; log at scratchpad/rel…

## Deviations

- **rel450-1** — The deploy role's dispatch on the claude runtime is a read-only gather payload that cannot run the script; the leader ran scripts/release.sh itself with BEE_DISPATCH_ID=c94240a4-8099-4db2-b79d-c8124aadd019 exported, as release-2-44-0 did

## Provenance

Proposed by `bee knowledge promote --work release-2-45-0` from 1 capped cell trace(s) in `.bee/cells/` and the anchor `docs/history/release-2-45-0/CONTEXT.md`, `docs/history/release-2-45-0/plan.md`. Every line above is copied from a trace or from the work item; nothing here is curated truth until a human or agent accepts it.
