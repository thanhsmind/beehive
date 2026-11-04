---
type: bee.delivery
title: cli-output-spill — delivery
description: "Delivery record proposed by bee knowledge promote for work item cli-output-spill: 1 capped cell(s), 0 recorded deviation(s)."
timestamp: 2026-11-03
bee:
  id: cli-output-spill-delivery
  lifecycle: active
  areas: [rust-runtime]
  required_context: [docs/history/cli-output-spill/CONTEXT.md]
  sources: [docs/history/cli-output-spill/CONTEXT.md, .bee/cells/cos-1.json]
---

# cli-output-spill — Delivery

## What shipped

- **cos-1** — Added src/output_spill.rs wrapping router::try_native in main() so allowlisted read-verb output over config output_spill_bytes spills to .bee/tmp/output via a unix fd-1 redirect, printing head/marker/tail and keeping the file; 8 integration tests green; commit cac33a917. (3 file(s) changed)

## Verify

Each cell below was capped only against a recorded passing verify result — bee refuses a cap without one.

- **cos-1** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee --test output_spill` — 8/8 output_spill tests over a fixture repo, rerun by the leader

## Deviations

None recorded in the capped cell traces.

## Provenance

Proposed by `bee knowledge promote --work cli-output-spill` from 1 capped cell trace(s) in `.bee/cells/` and the anchor `docs/history/cli-output-spill/CONTEXT.md`. Every line above is copied from a trace or from the work item; nothing here is curated truth until a human or agent accepts it.
