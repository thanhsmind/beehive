---
type: bee.delivery
title: worker-guard-stub-stdin — delivery
description: "Delivery record proposed by bee knowledge promote for work item worker-guard-stub-stdin: 1 capped cell(s), 2 recorded deviation(s)."
timestamp: 2026-10-07
bee:
  id: worker-guard-stub-stdin-delivery
  lifecycle: active
  areas: [hook-runtime]
  required_context: [docs/history/worker-guard-stub-stdin/plan.md]
  sources: [docs/history/worker-guard-stub-stdin/plan.md, .bee/cells/wgs-1.json]
---

# worker-guard-stub-stdin — Delivery

## What shipped

- **wgs-1** — worker-guard stubs drain stdin; the Linux EPIPE flake is gone under stress (1 file(s) changed)

## Verify

Each cell below was capped only against a recorded passing verify result — bee refuses a cap without one.

- **wgs-1** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml --test pi_worker_guard_contracts` — 7 passed; stress 8 parallel loops x 15 runs: allow test 0/360 failures after the fix (1/120 before), deny test 0/120

## Deviations

- **wgs-1** — drained stdin in all four stubs of the file, not only the allow and deny stubs — the noenv stub also receives write-guard JSON and could hit the same EPIPE — found a better route
- **wgs-1** — sync-ack: test-only change

## Provenance

Proposed by `bee knowledge promote --work worker-guard-stub-stdin` from 1 capped cell trace(s) in `.bee/cells/` and the anchor `docs/history/worker-guard-stub-stdin/plan.md`. Every line above is copied from a trace or from the work item; nothing here is curated truth until a human or agent accepts it.
