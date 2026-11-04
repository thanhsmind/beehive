---
type: bee.delivery
title: wave-run — delivery
description: "Delivery record proposed by bee knowledge promote for work item wave-run: 1 capped cell(s), 2 recorded deviation(s)."
timestamp: 2026-11-03
bee:
  id: wave-run-delivery
  lifecycle: active
  areas: [workflow-state]
  required_context: [docs/history/wave-run/CONTEXT.md]
  sources: [docs/history/wave-run/CONTEXT.md, .bee/cells/wvr-1.json]
---

# wave-run — Delivery

## What shipped

- **wvr-1** — bee dispatch wave gains --run: the herding half of a prepared wave now executes in parallel in one call and returns {ran, native, skipped}, while the prepare-only path stays byte-identical (commit 64fb311a4). (7 file(s) changed)

## Verify

Each cell below was capped only against a recorded passing verify result — bee refuses a cap without one.

- **wvr-1** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee dispatch_wave && PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee --test registry_contracts` — 9 dispatch_wave tests incl. 2 new dispatch_wave_run tests and 12 registry_contracts, rerun by the leader

## Deviations

- **wvr-1** — touched reservations/flags.rs (FLAG_ALONE_BOOLEANS gets run) and catalog.rs (PINNED_FLAG_COUNT 234 to 235), both required by any new boolean flag
- **wvr-1** — sync-ack: the bee-swarming line that teaches dispatch wave --run lands with the knowledge sync of this feature, after the code merge

## Provenance

Proposed by `bee knowledge promote --work wave-run` from 1 capped cell trace(s) in `.bee/cells/` and the anchor `docs/history/wave-run/CONTEXT.md`. Every line above is copied from a trace or from the work item; nothing here is curated truth until a human or agent accepts it.
