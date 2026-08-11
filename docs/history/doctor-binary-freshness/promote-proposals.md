promote proposal for work item "doctor-binary-freshness" (.bee/logs/scribing-runs.jsonl + .bee/lanes/doctor-binary-freshness.json) — 1 capped cell(s): dbf-1
anchor: ledger — .bee/logs/scribing-runs.jsonl, .bee/lanes/doctor-binary-freshness.json
PROPOSAL ONLY — nothing was written. Applying any section below is a human or agent decision.

(a) DELIVERY DRAFT — save as docs/knowledge/work/doctor-binary-freshness/delivery.md

---
type: bee.delivery
title: doctor-binary-freshness — delivery
description: "Delivery record proposed by bee knowledge promote for work item doctor-binary-freshness: 1 capped cell(s), 0 recorded deviation(s)."
timestamp: 2026-08-11
bee:
  id: doctor-binary-freshness-delivery
  lifecycle: active
  areas: [hook-runtime]
  required_context: [.bee/logs/scribing-runs.jsonl, .bee/lanes/doctor-binary-freshness.json]
  sources: [.bee/logs/scribing-runs.jsonl, .bee/lanes/doctor-binary-freshness.json, .bee/cells/dbf-1.json]
---

# doctor-binary-freshness — Delivery

## What shipped

- **dbf-1** — Added binary_freshness doctor row (source-checkout only): version-parity + mtime-freshness checks, unknown-without-binary, absent-on-host; 5 new tests (2 file(s) changed)

## Verify

Each cell below was capped only against a recorded passing verify result — bee refuses a cap without one.

- **dbf-1** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml`

## Deviations

None recorded in the capped cell traces.

## Provenance

Proposed by `bee knowledge promote --work doctor-binary-freshness` from 1 capped cell trace(s) in `.bee/cells/` and the anchor `.bee/logs/scribing-runs.jsonl`, `.bee/lanes/doctor-binary-freshness.json`. Every line above is copied from a trace or from the work item; nothing here is curated truth until a human or agent accepts it.

(b) AREA UPDATES — candidate spec-sync bullets, each citing its cell

areas: from the scribing stamp for "doctor-binary-freshness" — .bee/logs/scribing-runs.jsonl's most recent entry (2026-08-11T15:32:31.009Z), the work item declares no bee.areas.

area hook-runtime:
  - [dbf-1] Added binary_freshness doctor row (source-checkout only): version-parity + mtime-freshness checks, unknown-without-binary, absent-on-host; 5 new tests — feature-wide sync per the scribing stamp, 2 file(s) changed (trace .bee/cells/dbf-1.json)

(c) PATTERN CANDIDATES — candidate bee.pattern concepts, bee.polarity pitfall

None: no capped cell trace carries a deviation or a failure signature.

knowledge promote: 1 capped cell(s) mined, 1 delivery draft, 1 area bullet(s), 0 pattern candidate(s), 0 file(s) written.