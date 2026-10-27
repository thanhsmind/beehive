---
type: bee.delivery
title: occupancy-test-pane-fixture — delivery
description: "Delivery record proposed by bee knowledge promote for work item occupancy-test-pane-fixture: 1 capped cell(s), 1 recorded deviation(s)."
timestamp: 2026-10-06
bee:
  id: occupancy-test-pane-fixture-delivery
  lifecycle: active
  areas: [bee-herding]
  required_context: [docs/history/occupancy-test-pane-fixture/plan.md]
  sources: [docs/history/occupancy-test-pane-fixture/plan.md, .bee/cells/otpf-1.json]
---

# occupancy-test-pane-fixture — Delivery

## What shipped

- **otpf-1** — paseo occupancy test passes an explicit empty pane set (1 file(s) changed)

## Verify

Each cell below was capped only against a recorded passing verify result — bee refuses a cap without one.

- **otpf-1** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml --bin bee herding::wave::tests::occupancy` — 10 occupancy tests; red reproduced first by running the test binary with PATH=/nonexistent (Fallback(1) vs Live(1)), green after with and without pane tools; full suite left to CI

## Deviations

- **otpf-1** — sync-ack: test-only change; no owned skill describes this test fixture

## Provenance

Proposed by `bee knowledge promote --work occupancy-test-pane-fixture` from 1 capped cell trace(s) in `.bee/cells/` and the anchor `docs/history/occupancy-test-pane-fixture/plan.md`. Every line above is copied from a trace or from the work item; nothing here is curated truth until a human or agent accepts it.
