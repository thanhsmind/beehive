---
type: bee.delivery
title: doctor-probe-etxtbsy — delivery
description: "Delivery record for work item doctor-probe-etxtbsy: 1 capped cell(s), 1 recorded deviation(s)."
timestamp: 2026-08-30
bee:
  id: doctor-probe-etxtbsy-delivery
  lifecycle: active
  areas: [rust-runtime]
  required_context: [.bee/logs/scribing-runs.jsonl, .bee/lanes/doctor-probe-etxtbsy.json]
  sources: [.bee/logs/scribing-runs.jsonl, .bee/lanes/doctor-probe-etxtbsy.json, .bee/cells/dpe-1.json]
---

# doctor-probe-etxtbsy — Delivery

## What shipped

- **dpe-1** — `ExecutableFileBusy` is retried 10x20ms; the freshness-unknown row now names the real failure reason instead of a generic unknown (1 file(s) changed)

## Verify

- **dpe-1** — `cargo test --release --no-fail-fast --manifest-path packages/bee-rs/Cargo.toml` — green.

## Deviations

- First wording of the new detail dropped the literal `rs-info` and broke `binary_freshness_is_unknown_when_the_probe_fails_and_nothing_is_newer` on the first stress run; reworded to keep it — the plan was wrong about a fact.

## Provenance

Mined from 1 capped cell trace in `.bee/cells/dpe-1.json`.
