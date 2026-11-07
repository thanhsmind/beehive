---
type: bee.delivery
title: hook-name-before-stdin — delivery
description: "Delivery record proposed by bee knowledge promote for work item hook-name-before-stdin: 1 capped cell(s), 1 recorded deviation(s)."
timestamp: 2026-08-30
bee:
  id: hook-name-before-stdin-delivery
  lifecycle: active
  required_context: [.bee/lanes/hook-name-before-stdin.json]
  sources: [.bee/lanes/hook-name-before-stdin.json, .bee/cells/archive/hook-name-before-stdin/hnbs-check-name-first.json]
---

# hook-name-before-stdin — Delivery

## What shipped

- **hnbs-check-name-first** — unknown hook names are refused before the stdin read; the hang path is gone (1 file(s) changed)

## Verify

Each cell below was capped only against a recorded passing verify result — bee refuses a cap without one.

- **hnbs-check-name-first** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml --bin bee hooks::tests`

## Deviations

- **hnbs-check-name-first** — followed the plan

## Provenance

Proposed by `bee knowledge promote --work hook-name-before-stdin` from 1 capped cell trace(s) in `.bee/cells/` and the anchor `.bee/lanes/hook-name-before-stdin.json`. Every line above is copied from a trace or from the work item; nothing here is curated truth until a human or agent accepts it.
