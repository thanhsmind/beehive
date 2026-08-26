---
type: bee.delivery
title: sample-role-description — delivery
description: "Delivery record proposed by bee knowledge promote for work item sample-role-description: 1 capped cell(s), 1 recorded deviation(s)."
timestamp: 2026-08-26
bee:
  id: sample-role-description-delivery
  lifecycle: active
  required_context: [.bee/lanes/sample-role-description.json]
  sources: [.bee/lanes/sample-role-description.json, .bee/cells/archive/sample-role-description/csd-1.json]
---

# sample-role-description — Delivery

## What shipped

- **csd-1** — config-sample.json documents the optional role-slot description field (_doc slot_shapes entry + live generation example) (1 file(s) changed)

## Verify

Each cell below was capped only against a recorded passing verify result — bee refuses a cap without one.

- **csd-1** — `cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee onboard -- --nocapture: the embedded config-sample.json parse/shape tests stay green over the edited file`

## Deviations

- **csd-1** — followed the plan

## Provenance

Proposed by `bee knowledge promote --work sample-role-description` from 1 capped cell trace(s) in `.bee/cells/` and the anchor `.bee/lanes/sample-role-description.json`. Every line above is copied from a trace or from the work item; nothing here is curated truth until a human or agent accepts it.

