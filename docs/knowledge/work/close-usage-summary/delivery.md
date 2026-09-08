---
type: bee.delivery
title: close-usage-summary — delivery
description: "Delivery record proposed by bee knowledge promote for work item close-usage-summary: 1 capped cell(s), 1 recorded deviation(s)."
timestamp: 2026-08-30
bee:
  id: close-usage-summary-delivery
  lifecycle: active
  required_context: [.bee/lanes/close-usage-summary.json]
  sources: [.bee/lanes/close-usage-summary.json, .bee/cells/archive/close-usage-summary/cus-close-usage-section.json]
---

# close-usage-summary — Delivery

## What shipped

- **cus-close-usage-section** — bee close prints a token-usage section (sessions + subagents + total) and inserts a usage object into its JSON result (2 file(s) changed)

## Verify

Each cell below was capped only against a recorded passing verify result — bee refuses a cap without one.

- **cus-close-usage-section** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --no-fail-fast --manifest-path packages/bee-rs/Cargo.toml usage`

## Deviations

- **cus-close-usage-section** — followed the plan

## Provenance

Proposed by `bee knowledge promote --work close-usage-summary` from 1 capped cell trace(s) in `.bee/cells/` and the anchor `.bee/lanes/close-usage-summary.json`. Every line above is copied from a trace or from the work item; nothing here is curated truth until a human or agent accepts it.
