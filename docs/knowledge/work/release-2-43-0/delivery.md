---
type: bee.delivery
title: release-2-43-0 — delivery
description: "Delivery record proposed by bee knowledge promote for work item release-2-43-0: 1 capped cell(s), 1 recorded deviation(s)."
timestamp: 2026-09-21
bee:
  id: release-2-43-0-delivery
  lifecycle: active
  required_context: [docs/history/release-2-43-0/CONTEXT.md, docs/history/release-2-43-0/plan.md]
  sources: [docs/history/release-2-43-0/CONTEXT.md, docs/history/release-2-43-0/plan.md, .bee/cells/rel430-1.json]
---

# release-2-43-0 — Delivery

## What shipped

- **rel430-1** — released 2.43.0 (tag v2.43.0, assets verified, OK) (3 file(s) changed)

## Verify

Each cell below was capped only against a recorded passing verify result — bee refuses a cap without one.

- **rel430-1** — `bash scripts/release.sh 2.43.0 && bee dev release-manifest --check` — the script ran the full declared suite (4195 passed, 0 failed) before tagging and printed its final OK

## Deviations

- **rel430-1** — the leader ran scripts/release.sh under the prepared deployment permit f1c8d7cf instead of a dispatched agent — the deploy role on the native opus runtime returned a read-only gather payload that cannot run the script — hit an unforeseen obstacle

## Provenance

Proposed by `bee knowledge promote --work release-2-43-0` from 1 capped cell trace(s) in `.bee/cells/` and the anchor `docs/history/release-2-43-0/CONTEXT.md`, `docs/history/release-2-43-0/plan.md`. Every line above is copied from a trace or from the work item; nothing here is curated truth until a human or agent accepts it.
