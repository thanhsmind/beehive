---
type: bee.delivery
title: herding-cap-check — delivery
description: "Delivery record proposed by bee knowledge promote for work item herding-cap-check: 1 capped cell(s), 2 recorded deviation(s)."
timestamp: 2026-09-20
bee:
  id: herding-cap-check-delivery
  lifecycle: active
  areas: [bee-herding]
  required_context: [docs/history/herding-cap-check/CONTEXT.md]
  sources: [docs/history/herding-cap-check/CONTEXT.md, .bee/cells/hcapc-1.json]
---

# herding-cap-check — Delivery

## What shipped

- **hcapc-1** — Refuse herding run when worker claimed success and left cell uncapped (1 file(s) changed)

## Verify

Each cell below was capped only against a recorded passing verify result — bee refuses a cap without one.

- **hcapc-1** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee --bin bee herding::run` — touched run.rs

## Deviations

- **hcapc-1** — followed the plan
- **hcapc-1** — sync-ack: cell hcapc-1 scoped only to run.rs without skill edits as approved in cell definition

## Provenance

Proposed by `bee knowledge promote --work herding-cap-check` from 1 capped cell trace(s) in `.bee/cells/` and the anchor `docs/history/herding-cap-check/CONTEXT.md`. Every line above is copied from a trace or from the work item; nothing here is curated truth until a human or agent accepts it.
