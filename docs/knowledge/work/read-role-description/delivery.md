---
type: bee.delivery
title: read-role-description — delivery
description: "Delivery record proposed by bee knowledge promote for work item read-role-description: 1 capped cell(s), 1 recorded deviation(s)."
timestamp: 2026-09-10
bee:
  id: read-role-description-delivery
  lifecycle: active
  areas: [doctrine-layer]
  required_context: [.bee/lanes/read-role-description.json]
  sources: [.bee/lanes/read-role-description.json, .bee/cells/archive/read-role-description/rrd-1.json]
---

# read-role-description — Delivery

## What shipped

- **rrd-1** — read role's seeded description now names the narrow read job and points a broad scan at a role-less --kind gather; routing untouched (3 file(s) changed)

## Verify

Each cell below was capped only against a recorded passing verify result — bee refuses a cap without one.

- **rrd-1** — `cargo test --release --manifest-path packages/bee-rs/Cargo.toml onboard and drivers tests green; bee dispatch prepare --runtime claude --kind gather --role read still returns the bee-extract payload.`

## Deviations

- **rrd-1** — bee config set is not built into this binary (its own help says so); .bee/config.json was edited directly, which is the CLI-only rule taking its named fallback

## Provenance

Proposed by `bee knowledge promote --work read-role-description` from 1 capped cell trace(s) in `.bee/cells/` and the anchor `.bee/lanes/read-role-description.json`. Every line above is copied from a trace or from the work item; nothing here is curated truth until a human or agent accepts it.
