---
type: bee.delivery
title: baseline-release-clear — delivery
description: "Delivery record proposed by bee knowledge promote for work item baseline-release-clear: 1 capped cell(s), 2 recorded deviation(s)."
timestamp: 2026-09-20
bee:
  id: baseline-release-clear-delivery
  lifecycle: active
  areas: [workflow-state]
  required_context: [.bee/logs/scribing-runs.jsonl, .bee/lanes/baseline-release-clear.json]
  sources: [.bee/logs/scribing-runs.jsonl, .bee/lanes/baseline-release-clear.json, .bee/cells/brc-1.json]
---

# baseline-release-clear — Delivery

## What shipped

- **brc-1** — Clear the cap baseline when a cell is released, proven red first (2 file(s) changed)

## Verify

Each cell below was capped only against a recorded passing verify result — bee refuses a cap without one.

- **brc-1** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee --bin bee cells` — 378 passed, 0 failed; and proven red first: with "baseline" removed from release_trace the new test fails on 'a released cell must not carry the previous run's baseline' (exit 101), with it restored …

## Deviations

- **brc-1** — followed the plan
- **brc-1** — sync-ack: No skill text is stale: this clears an internal trace field on release. worker-details.md already tells a worker the baseline is optional and what it means, and nothing in any skill describes what a RELEASED cell keeps — that is plumbing an agent never reads. The instruction half of this feature shipped with proof-honesty.

## Provenance

Proposed by `bee knowledge promote --work baseline-release-clear` from 1 capped cell trace(s) in `.bee/cells/` and the anchor `.bee/logs/scribing-runs.jsonl`, `.bee/lanes/baseline-release-clear.json`. Every line above is copied from a trace or from the work item; nothing here is curated truth until a human or agent accepts it.
