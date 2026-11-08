---
type: bee.delivery
title: release-2-68-0 — delivery
description: "Delivery record proposed by bee knowledge promote for work item release-2-68-0: 1 capped cell(s), 1 recorded deviation(s)."
timestamp: 2026-10-31
bee:
  id: release-2-68-0-delivery
  lifecycle: active
  required_context: [docs/history/release-2-68-0/CONTEXT.md, docs/history/release-2-68-0/plan.md]
  sources: [docs/history/release-2-68-0/CONTEXT.md, docs/history/release-2-68-0/plan.md, cell rel680-1 (trace purged from .bee/cells/)]
---

# release-2-68-0 — Delivery

## What shipped

- **rel680-1** — bee 2.68.0 is live (3 file(s) changed)

## Verify

Each cell below was capped only against a recorded passing verify result — bee refuses a cap without one.

- **rel680-1** — `bash scripts/release.sh 2.68.0 && bee dev release-manifest --check` — script printed release OK bee 2.68.0 is live after the full declared cargo suite (5016 passed, 0 failed, CI-pinned opencode 1.18.16 first on PATH per the logged decision), tag v2.68.0 pushed, green r…

## Deviations

- **rel680-1** — First run under permit 4cf8335d went red only on opencode_plugin_contracts because this machine has opencode 2.0.32 (backlog p-624e2d7d); the re-run put the CI-pinned 1.18.16 first on PATH instead of skipping the suite — something else had to be fixed first

## Provenance

Proposed by `bee knowledge promote --work release-2-68-0` from 1 capped cell trace(s) in `.bee/cells/` and the anchor `docs/history/release-2-68-0/CONTEXT.md`, `docs/history/release-2-68-0/plan.md`. Every line above is copied from a trace or from the work item; nothing here is curated truth until a human or agent accepts it.
