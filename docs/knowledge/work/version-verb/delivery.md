---
type: bee.delivery
title: version-verb — delivery
description: "Delivery record proposed by bee knowledge promote for work item version-verb: 1 capped cell(s), 0 recorded deviation(s)."
timestamp: 2026-08-31
bee:
  id: version-verb-delivery
  lifecycle: active
  required_context: [docs/history/version-verb/CONTEXT.md]
  sources: [docs/history/version-verb/CONTEXT.md, .bee/cells/archive/version-verb/vv-1.json]
---

# version-verb — Delivery

## What shipped

- **vv-1** — bee version / --version / -V served rootless from the router; registry entry + PORTED line + black-box tests (0 file(s) changed)

## Verify

Each cell below was capped only against a recorded passing verify result — bee refuses a cap without one.

- **vv-1** — `cargo test --release --manifest-path packages/bee-rs/Cargo.toml`

## Deviations

None recorded in the capped cell traces.

## Provenance

Proposed by `bee knowledge promote --work version-verb` from 1 capped cell trace(s) in `.bee/cells/` and the anchor `docs/history/version-verb/CONTEXT.md`. Every line above is copied from a trace or from the work item; nothing here is curated truth until a human or agent accepts it.
