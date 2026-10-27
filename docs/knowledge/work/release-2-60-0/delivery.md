---
type: bee.delivery
title: release-2-60-0 — delivery
description: "Delivery record proposed by bee knowledge promote for work item release-2-60-0: 1 capped cell(s), 1 recorded deviation(s)."
timestamp: 2026-10-24
bee:
  id: release-2-60-0-delivery
  lifecycle: active
  required_context: [docs/history/release-2-60-0/CONTEXT.md, docs/history/release-2-60-0/plan.md]
  sources: [docs/history/release-2-60-0/CONTEXT.md, docs/history/release-2-60-0/plan.md, .bee/cells/rel600-1.json]
---

# release-2-60-0 — Delivery

## What shipped

- **rel600-1** — Released bee 2.60.0: tag v2.60.0, release-binaries run 40089544028 green, five binaries plus SHA256SUMS published (3 file(s) changed)

## Verify

Each cell below was capped only against a recorded passing verify result — bee refuses a cap without one.

- **rel600-1** — `bash scripts/release.sh 2.60.0 && bee dev release-manifest --check` — the release script ran under deployment dispatch 5645be08 and printed release OK bee 2.60.0 is live; its test gate passed 4942/0 with BEE_OPENCODE_SUITE_ALLOW_SKIP=1 (decision logged, opencode on PAT…

## Deviations

- **rel600-1** — the first dispatch stopped at the dirty-tree precondition before spending its permit; bookkeeping was committed and a fresh permit prepared — something else had to be fixed first

## Provenance

Proposed by `bee knowledge promote --work release-2-60-0` from 1 capped cell trace(s) in `.bee/cells/` and the anchor `docs/history/release-2-60-0/CONTEXT.md`, `docs/history/release-2-60-0/plan.md`. Every line above is copied from a trace or from the work item; nothing here is curated truth until a human or agent accepts it.
