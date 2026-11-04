---
type: bee.delivery
title: release-2-66-0 — delivery
description: "Delivery record proposed by bee knowledge promote for work item release-2-66-0: 1 capped cell(s), 1 recorded deviation(s)."
timestamp: 2026-10-29
bee:
  id: release-2-66-0-delivery
  lifecycle: active
  required_context: [docs/history/release-2-66-0/CONTEXT.md, docs/history/release-2-66-0/plan.md]
  sources: [docs/history/release-2-66-0/CONTEXT.md, docs/history/release-2-66-0/plan.md, .bee/cells/rel660-1.json]
---

# release-2-66-0 — Delivery

## What shipped

- **rel660-1** — bee 2.66.0 is live: tag pushed, release-binaries green, release carries 5 binaries and SHA256SUMS (3 file(s) changed)

## Verify

Each cell below was capped only against a recorded passing verify result — bee refuses a cap without one.

- **rel660-1** — `bash scripts/release.sh 2.66.0 && bee dev release-manifest --check` — script printed OK after the declared cargo suite (40 test binaries ok, opencode suite allowed to skip as in 2.60.0-2.65.0), tag push, green release-binaries run https://github.com/thanhsmind/beehive/…

## Deviations

- **rel660-1** — release.sh ran inline in the leader under permit d26d5e5d, as releases 2.58.0-2.65.0 did

## Provenance

Proposed by `bee knowledge promote --work release-2-66-0` from 1 capped cell trace(s) in `.bee/cells/` and the anchor `docs/history/release-2-66-0/CONTEXT.md`, `docs/history/release-2-66-0/plan.md`. Every line above is copied from a trace or from the work item; nothing here is curated truth until a human or agent accepts it.
