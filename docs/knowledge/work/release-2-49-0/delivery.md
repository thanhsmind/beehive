---
type: bee.delivery
title: release-2-49-0 — delivery
description: "Delivery record proposed by bee knowledge promote for work item release-2-49-0: 1 capped cell(s), 2 recorded deviation(s)."
timestamp: 2026-10-03
bee:
  id: release-2-49-0-delivery
  lifecycle: active
  required_context: [docs/history/release-2-49-0/CONTEXT.md, docs/history/release-2-49-0/plan.md]
  sources: [docs/history/release-2-49-0/CONTEXT.md, docs/history/release-2-49-0/plan.md, .bee/cells/rel490-1.json]
---

# release-2-49-0 — Delivery

## What shipped

- **rel490-1** — bee 2.49.0 is live: tag v2.49.0 pushed, release-binaries green, five binaries and SHA256SUMS published (3 file(s) changed)

## Verify

Each cell below was capped only against a recorded passing verify result — bee refuses a cap without one.

- **rel490-1** — `bash scripts/release.sh 2.49.0 && bee dev release-manifest --check` — script printed release OK; run https://github.com/thanhsmind/beehive/actions/runs/37111227421; release https://github.com/thanhsmind/beehive/releases/tag/v2.49.0; manifest check 418 files match

## Deviations

- **rel490-1** — on the claude runtime the deploy role returns a read-only gather payload, so the leader ran scripts/release.sh under the authorized dispatch id — something else had to be fixed first
- **rel490-1** — the first run went red on no_shipped_command_spelling_is_refused_by_the_widened_guard; fix-first lane release-red-2-49-0 pinned five closed-plan spans, then a fresh permit and a second run went green — hit an unforeseen obstacle

## Provenance

Proposed by `bee knowledge promote --work release-2-49-0` from 1 capped cell trace(s) in `.bee/cells/` and the anchor `docs/history/release-2-49-0/CONTEXT.md`, `docs/history/release-2-49-0/plan.md`. Every line above is copied from a trace or from the work item; nothing here is curated truth until a human or agent accepts it.
