---
type: bee.delivery
title: set-gate-windows-json — delivery
description: "Delivery record proposed by bee knowledge promote for work item set-gate-windows-json: 1 capped cell(s), 2 recorded deviation(s)."
timestamp: 2026-10-07
bee:
  id: set-gate-windows-json-delivery
  lifecycle: active
  areas: [workflow-state]
  required_context: [docs/history/set-gate-windows-json/plan.md]
  sources: [docs/history/set-gate-windows-json/plan.md, .bee/cells/sgw-1.json]
---

# set-gate-windows-json — Delivery

## What shipped

- **sgw-1** — control_root assertion compares against the JSON-escaped main path (1 file(s) changed)

## Verify

Each cell below was capped only against a recorded passing verify result — bee refuses a cap without one.

- **sgw-1** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml --bin bee verbs::state_group::set_gate` — 65 set_gate tests on Linux; the Windows-only red cannot run here, Windows CI is the proof

## Deviations

- **sgw-1** — followed the plan
- **sgw-1** — sync-ack: test-only change

## Provenance

Proposed by `bee knowledge promote --work set-gate-windows-json` from 1 capped cell trace(s) in `.bee/cells/` and the anchor `docs/history/set-gate-windows-json/plan.md`. Every line above is copied from a trace or from the work item; nothing here is curated truth until a human or agent accepts it.
