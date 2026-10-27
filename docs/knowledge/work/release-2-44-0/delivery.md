---
type: bee.delivery
title: release-2-44-0 — delivery
description: "Delivery record proposed by bee knowledge promote for work item release-2-44-0: 1 capped cell(s), 1 recorded deviation(s)."
timestamp: 2026-09-22
bee:
  id: release-2-44-0-delivery
  lifecycle: active
  required_context: [docs/history/release-2-44-0/CONTEXT.md, docs/history/release-2-44-0/plan.md]
  sources: [docs/history/release-2-44-0/CONTEXT.md, docs/history/release-2-44-0/plan.md, .bee/cells/rel440-1.json]
---

# release-2-44-0 — Delivery

## What shipped

- **rel440-1** — Released 2.44.0 (tag v2.44.0, five binaries verified, OK) (3 file(s) changed)

## Verify

Each cell below was capped only against a recorded passing verify result — bee refuses a cap without one.

- **rel440-1** — `bash scripts/release.sh 2.44.0 && bee dev release-manifest --check` — the script ran the full declared suite (4205 passed, 0 failed) before tagging, release-binaries run 35680422522 went green on all five rows including the three new macOS/ARM runners, and it printed i…

## Deviations

- **rel440-1** — the leader ran scripts/release.sh under the prepared deployment permit 996ea8f2 instead of a dispatched agent — the deploy role on the native opus runtime returned a read-only gather payload that cannot run the script (same as release-2-43-0) — hit an unforeseen obstacle

## Provenance

Proposed by `bee knowledge promote --work release-2-44-0` from 1 capped cell trace(s) in `.bee/cells/` and the anchor `docs/history/release-2-44-0/CONTEXT.md`, `docs/history/release-2-44-0/plan.md`. Every line above is copied from a trace or from the work item; nothing here is curated truth until a human or agent accepts it.
