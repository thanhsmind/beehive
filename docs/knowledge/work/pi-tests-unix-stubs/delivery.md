---
type: bee.delivery
title: pi-tests-unix-stubs — delivery
description: "Delivery record proposed by bee knowledge promote for work item pi-tests-unix-stubs: 1 capped cell(s), 1 recorded deviation(s)."
timestamp: 2026-10-07
bee:
  id: pi-tests-unix-stubs-delivery
  lifecycle: active
  areas: [workflow-state]
  required_context: [docs/history/pi-tests-unix-stubs/plan.md]
  sources: [docs/history/pi-tests-unix-stubs/plan.md, .bee/cells/pts-1.json]
---

# pi-tests-unix-stubs — Delivery

## What shipped

- **pts-1** — Gated the eight sh-stub pi contract tests and their two helpers with cfg(unix) (2 file(s) changed)

## Verify

Each cell below was capped only against a recorded passing verify result — bee refuses a cap without one.

- **pts-1** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml --test pi_paseo_heartbeat_contracts --test pi_worker_guard_contracts` — the two touched test files, 26 + 7 passed on Linux, none filtered; Windows not run locally, CI verify-windows is the check

## Deviations

- **pts-1** — Also gated belt_dir and IDENTITY_STUBS_JS in pi_paseo_heartbeat_contracts.rs — only the gated tests use them, so Windows would see dead code — something else had to be fixed first

## Provenance

Proposed by `bee knowledge promote --work pi-tests-unix-stubs` from 1 capped cell trace(s) in `.bee/cells/` and the anchor `docs/history/pi-tests-unix-stubs/plan.md`. Every line above is copied from a trace or from the work item; nothing here is curated truth until a human or agent accepts it.
