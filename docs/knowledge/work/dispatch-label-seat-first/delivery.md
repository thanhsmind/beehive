---
type: bee.delivery
title: dispatch-label-seat-first — delivery
description: "Delivery record proposed by bee knowledge promote for work item dispatch-label-seat-first: 1 capped cell(s), 1 recorded deviation(s)."
timestamp: 2026-09-02
bee:
  id: dispatch-label-seat-first-delivery
  lifecycle: active
  required_context: [.bee/lanes/dispatch-label-seat-first.json]
  sources: [.bee/lanes/dispatch-label-seat-first.json, .bee/cells/archive/dispatch-label-seat-first/dlsf-1.json]
---

# dispatch-label-seat-first — Delivery

## What shipped

- **dlsf-1** — The dispatch label leads with the asked role when --role is given; role-less dispatches keep today's bytes (3 file(s) changed)

## Verify

Each cell below was capped only against a recorded passing verify result — bee refuses a cap without one.

- **dlsf-1** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -- drivers prepare`

## Deviations

- **dlsf-1** — Edited docs/product-description/delegation/dispatch.md line 211 (the label rule) though the cell listed it only under affects_specs — the sentence named <kind>: <purpose> as the rule and became false the moment the lead changed — something else had to be fixed first

## Provenance

Proposed by `bee knowledge promote --work dispatch-label-seat-first` from 1 capped cell trace(s) in `.bee/cells/` and the anchor `.bee/lanes/dispatch-label-seat-first.json`. Every line above is copied from a trace or from the work item; nothing here is curated truth until a human or agent accepts it.
