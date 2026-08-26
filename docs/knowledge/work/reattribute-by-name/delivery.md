---
type: bee.delivery
title: reattribute-by-name — delivery
description: "Delivery record proposed by bee knowledge promote for work item reattribute-by-name: 1 capped cell(s), 1 recorded deviation(s)."
timestamp: 2026-08-26
bee:
  id: reattribute-by-name-delivery
  lifecycle: active
  areas: [decision-memory]
  required_context: [.bee/logs/scribing-runs.jsonl, .bee/lanes/reattribute-by-name.json]
  sources: [.bee/logs/scribing-runs.jsonl, .bee/lanes/reattribute-by-name.json, .bee/cells/archive/reattribute-by-name/rbn-1.json]
---

# reattribute-by-name — Delivery

## What shipped

- **rbn-1** — the human-named reattribution door lands and the five prompt-work-record records now carry their own feature; a store-wide re-check also caught and re-fixed 18 merge-resurrected stamps (3 file(s) changed)

## Verify

Each cell below was capped only against a recorded passing verify result — bee refuses a cap without one.

- **rbn-1** — `cargo test --release --no-fail-fast --manifest-path packages/bee-rs/Cargo.toml`

## Deviations

- **rbn-1** — Executed inline; reason on trace.inline_reason.

## Provenance

Proposed by `bee knowledge promote --work reattribute-by-name` from 1 capped cell trace(s) in `.bee/cells/` and the anchor `.bee/logs/scribing-runs.jsonl`, `.bee/lanes/reattribute-by-name.json`. Every line above is copied from a trace or from the work item; nothing here is curated truth until a human or agent accepts it.

