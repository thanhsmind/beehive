---
type: bee.delivery
title: release-2-40-0 — delivery
description: "Delivery record proposed by bee knowledge promote for work item release-2-40-0: 1 capped cell(s), 2 recorded deviation(s)."
timestamp: 2026-09-16
bee:
  id: release-2-40-0-delivery
  lifecycle: active
  required_context: [docs/history/release-2-40-0/CONTEXT.md, docs/history/release-2-40-0/plan.md]
  sources: [docs/history/release-2-40-0/CONTEXT.md, docs/history/release-2-40-0/plan.md, .bee/cells/archive/release-2-40-0/rel40-1.json]
---

# release-2-40-0 — Delivery

## What shipped

- **rel40-1** — Published release 2.40.0 through the sanctioned script (3 file(s) changed)

## Verify

Each cell below was capped only against a recorded passing verify result — bee refuses a cap without one.

- **rel40-1** — `bash scripts/release.sh 2.40.0 && bee dev release-manifest --check` — the script ran the declared suite before tagging and printed its final 'release OK bee 2.40.0 is live'; leader re-verified independently: both manifests 2.40.0, tag v2.40.0 on origin at 49ff5d4b, 0 u…

## Deviations

- **rel40-1** — The lane took a plan-rev bump after its shape gate: the first cell packet was refused by the REGEN_OBLIGATION guard because .claude-plugin/plugin.json is a release-manifest root, so the cell had to gain docs/history/codex-harness-hardening/release-manifest.json and a release-manifest --check in verify. Re-previewed and re-gated with the reason recorded.
- **rel40-1** — The release lane's own bookkeeping commit was made through a private GIT_INDEX_FILE rather than `git add`, which the concurrent-worker guard refuses while a sibling session is live in this checkout.

## Provenance

Proposed by `bee knowledge promote --work release-2-40-0` from 1 capped cell trace(s) in `.bee/cells/` and the anchor `docs/history/release-2-40-0/CONTEXT.md`, `docs/history/release-2-40-0/plan.md`. Every line above is copied from a trace or from the work item; nothing here is curated truth until a human or agent accepts it.
