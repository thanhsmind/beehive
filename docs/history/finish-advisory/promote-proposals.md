promote proposal for work item "finish-advisory" (.bee/logs/scribing-runs.jsonl) — 1 capped cell(s): fa-1
anchor: ledger — .bee/logs/scribing-runs.jsonl
PROPOSAL ONLY — nothing was written. Applying any section below is a human or agent decision.

(a) DELIVERY DRAFT — save as docs/knowledge/work/finish-advisory/delivery.md

---
type: bee.delivery
title: finish-advisory — delivery
description: "Delivery record proposed by bee knowledge promote for work item finish-advisory: 1 capped cell(s), 0 recorded deviation(s)."
timestamp: 2026-08-11
bee:
  id: finish-advisory-delivery
  lifecycle: active
  areas: [verify-pipeline]
  required_context: [.bee/logs/scribing-runs.jsonl]
  sources: [.bee/logs/scribing-runs.jsonl, .bee/cells/fa-1.json]
---

# finish-advisory — Delivery

## What shipped

- **fa-1** — Added a diff-vs-test advisory to cells finish's green-cap path (trace.warnings + one stderr line), config-gated and silent on any git failure (3 file(s) changed)

## Verify

Each cell below was capped only against a recorded passing verify result — bee refuses a cap without one.

- **fa-1** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml`

## Deviations

None recorded in the capped cell traces.

## Provenance

Proposed by `bee knowledge promote --work finish-advisory` from 1 capped cell trace(s) in `.bee/cells/` and the anchor `.bee/logs/scribing-runs.jsonl`. Every line above is copied from a trace or from the work item; nothing here is curated truth until a human or agent accepts it.

(b) AREA UPDATES — candidate spec-sync bullets, each citing its cell

areas: from the scribing stamp for "finish-advisory" — .bee/logs/scribing-runs.jsonl's most recent entry (2026-08-11T11:18:35.921Z), the work item declares no bee.areas.

area verify-pipeline:
  - [fa-1] Added a diff-vs-test advisory to cells finish's green-cap path (trace.warnings + one stderr line), config-gated and silent on any git failure — feature-wide sync per the scribing stamp, 3 file(s) changed (trace .bee/cells/fa-1.json)

(c) PATTERN CANDIDATES — candidate bee.pattern concepts, bee.polarity pitfall

None: no capped cell trace carries a deviation or a failure signature.

knowledge promote: 1 capped cell(s) mined, 1 delivery draft, 1 area bullet(s), 0 pattern candidate(s), 0 file(s) written.