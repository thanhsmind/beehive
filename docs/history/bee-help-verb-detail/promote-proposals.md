promote proposal for work item "bee-help-verb-detail" (.bee/logs/scribing-runs.jsonl) — 1 capped cell(s): bhv-1
anchor: ledger — .bee/logs/scribing-runs.jsonl
PROPOSAL ONLY — nothing was written. Applying any section below is a human or agent decision.

(a) DELIVERY DRAFT — save as docs/knowledge/work/bee-help-verb-detail/delivery.md

---
type: bee.delivery
title: bee-help-verb-detail — delivery
description: "Delivery record proposed by bee knowledge promote for work item bee-help-verb-detail: 1 capped cell(s), 0 recorded deviation(s)."
timestamp: 2026-08-21
bee:
  id: bee-help-verb-detail-delivery
  lifecycle: active
  areas: [rust-runtime]
  required_context: [.bee/logs/scribing-runs.jsonl]
  sources: [.bee/logs/scribing-runs.jsonl, .bee/cells/bhv-1.json]
---

# bee-help-verb-detail — Delivery

## What shipped

- **bhv-1** — Single-verb help prints per-flag descriptions with required markers; read-once rule lands in preamble and AGENTS.md (5 file(s) changed)

## Verify

Each cell below was capped only against a recorded passing verify result — bee refuses a cap without one.

- **bhv-1** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml`

## Deviations

None recorded in the capped cell traces.

## Provenance

Proposed by `bee knowledge promote --work bee-help-verb-detail` from 1 capped cell trace(s) in `.bee/cells/` and the anchor `.bee/logs/scribing-runs.jsonl`. Every line above is copied from a trace or from the work item; nothing here is curated truth until a human or agent accepts it.

(b) AREA UPDATES — candidate spec-sync bullets, each citing its cell

areas: from the scribing stamp for "bee-help-verb-detail" — .bee/logs/scribing-runs.jsonl's most recent entry (2026-08-21T01:50:00.495Z), the work item declares no bee.areas.

area rust-runtime:
  - [bhv-1] Single-verb help prints per-flag descriptions with required markers; read-once rule lands in preamble and AGENTS.md — feature-wide sync per the scribing stamp, 5 file(s) changed (trace .bee/cells/bhv-1.json)

(c) PATTERN CANDIDATES — candidate bee.pattern concepts, bee.polarity pitfall

None: no capped cell trace carries a deviation or a failure signature.

knowledge promote: 1 capped cell(s) mined, 1 delivery draft, 1 area bullet(s), 0 pattern candidate(s), 0 file(s) written.