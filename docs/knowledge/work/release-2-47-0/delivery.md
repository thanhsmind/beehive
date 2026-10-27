---
type: bee.delivery
title: release-2-47-0 — delivery
description: "Delivery record proposed by bee knowledge promote for work item release-2-47-0: 1 capped cell(s), 1 recorded deviation(s)."
timestamp: 2026-10-02
bee:
  id: release-2-47-0-delivery
  lifecycle: active
  required_context: [docs/history/release-2-47-0/CONTEXT.md, docs/history/release-2-47-0/plan.md]
  sources: [docs/history/release-2-47-0/CONTEXT.md, docs/history/release-2-47-0/plan.md, .bee/cells/rel470-1.json]
---

# release-2-47-0 — Delivery

## What shipped

- **rel470-1** — Publish release 2.47.0 through the sanctioned script (0 file(s) changed)

## Verify

Each cell below was capped only against a recorded passing verify result — bee refuses a cap without one.

- **rel470-1** — `bash scripts/release.sh 2.47.0 && bee dev release-manifest --check` — release.sh ran the full declared suite (37 test binaries ok, 0 FAILED), tagged v2.47.0, release-binaries https://github.com/thanhsmind/beehive/actions/runs/37023471297 green, 5 binaries + SHA256SUMS …

## Deviations

- **rel470-1** — the claude deploy role returns a read-only gather payload that cannot run the script, so the leader ran release.sh itself under dispatch 37fd7e0f

## Provenance

Proposed by `bee knowledge promote --work release-2-47-0` from 1 capped cell trace(s) in `.bee/cells/` and the anchor `docs/history/release-2-47-0/CONTEXT.md`, `docs/history/release-2-47-0/plan.md`. Every line above is copied from a trace or from the work item; nothing here is curated truth until a human or agent accepts it.
