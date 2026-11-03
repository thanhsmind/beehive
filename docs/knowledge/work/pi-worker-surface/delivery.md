---
type: bee.delivery
title: pi-worker-surface — delivery
description: "Delivery record proposed by bee knowledge promote for work item pi-worker-surface: 2 capped cell(s), 2 recorded deviation(s)."
timestamp: 2026-09-20
bee:
  id: pi-worker-surface-delivery
  lifecycle: active
  areas: [hook-runtime]
  required_context: [docs/history/pi-worker-surface/CONTEXT.md, docs/history/pi-worker-surface/plan.md]
  sources: [docs/history/pi-worker-surface/CONTEXT.md, docs/history/pi-worker-surface/plan.md, .bee/cells/archive/pi-worker-surface/pws-1.json, .bee/cells/archive/pi-worker-surface/pws-2.json]
---

# pi-worker-surface — Delivery

## What shipped

- **pws-1** — Registered terminating verdict tool on Pi belt and routed to write-guard (4 file(s) changed)
- **pws-2** — Draw in-flight workers in a widget below the editor with seat-named rows, D3 clearing, and no second timer (4 file(s) changed)

## Verify

Each cell below was capped only against a recorded passing verify result — bee refuses a cap without one.

- **pws-1** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee --test pi_plugin_contracts && .bee/bin/bee dev release-manifest --check` — touched bee-guard.ts and pi_plugin_contracts.rs
- **pws-2** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee --test pi_plugin_contracts && .bee/bin/bee dev release-manifest --check` — leader re-ran it independently after the merge: 87 passed, 0 failed (81 before this cell); node --check on the belt and release-manifest --check both exit 0

## Deviations

- **pws-1** — followed the plan
- **pws-2** — followed the plan

## Provenance

Proposed by `bee knowledge promote --work pi-worker-surface` from 2 capped cell trace(s) in `.bee/cells/` and the anchor `docs/history/pi-worker-surface/CONTEXT.md`, `docs/history/pi-worker-surface/plan.md`. Every line above is copied from a trace or from the work item; nothing here is curated truth until a human or agent accepts it.
