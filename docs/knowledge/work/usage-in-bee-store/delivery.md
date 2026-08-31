---
type: bee.delivery
title: usage-in-bee-store — delivery
description: "Delivery record proposed by bee knowledge promote for work item usage-in-bee-store: 1 capped cell(s), 2 recorded deviation(s)."
timestamp: 2026-08-30
bee:
  id: usage-in-bee-store-delivery
  lifecycle: active
  required_context: [.bee/lanes/usage-in-bee-store.json]
  sources: [.bee/lanes/usage-in-bee-store.json, .bee/cells/uibs-move-usage-record.json]
---

# usage-in-bee-store — Delivery

## What shipped

- **uibs-move-usage-record** — Close writes its token-usage record to .bee/usage/<feature>.json under the control root; close's own bookkeeping commit lands it (1 file(s) changed)

## Verify

Each cell below was capped only against a recorded passing verify result — bee refuses a cap without one.

- **uibs-move-usage-record** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --no-fail-fast --manifest-path packages/bee-rs/Cargo.toml --bin bee usage`

## Deviations

- **uibs-move-usage-record** — Rewrote clean_store_green_close_reports_reason_clean as green_close_commits_the_usage_record_it_just_wrote — a green close now always writes .bee/usage/<feature>.json, so .bee is never clean at that point; the test now pins that the record is committed and asserts reason clean one step later — the plan was wrong about a fact
- **uibs-move-usage-record** — Widened green_close_commits_only_dirty_bee_paths_leaving_unrelated_dirt_uncommitted to expect .bee/usage/demo.json beside .bee/config.json — the record is a new .bee path the same path-scoped git add stages — something else had to be fixed first

## Provenance

Proposed by `bee knowledge promote --work usage-in-bee-store` from 1 capped cell trace(s) in `.bee/cells/` and the anchor `.bee/lanes/usage-in-bee-store.json`. Every line above is copied from a trace or from the work item; nothing here is curated truth until a human or agent accepts it.
