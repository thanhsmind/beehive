promote proposal for work item "evolving-rank-freshness" (.bee/logs/scribing-runs.jsonl) — 1 capped cell(s): erf-1
anchor: ledger — .bee/logs/scribing-runs.jsonl
PROPOSAL ONLY — nothing was written. Applying any section below is a human or agent decision.

(a) DELIVERY DRAFT — save as docs/knowledge/work/evolving-rank-freshness/delivery.md

---
type: bee.delivery
title: evolving-rank-freshness — delivery
description: "Delivery record proposed by bee knowledge promote for work item evolving-rank-freshness: 1 capped cell(s), 1 recorded deviation(s)."
timestamp: 2026-08-11
bee:
  id: evolving-rank-freshness-delivery
  lifecycle: active
  areas: [feedback-digest]
  required_context: [.bee/logs/scribing-runs.jsonl]
  sources: [.bee/logs/scribing-runs.jsonl, .bee/cells/erf-1.json]
---

# evolving-rank-freshness — Delivery

## What shipped

- **erf-1** — clusters with any closed-kind entry leave the ranking into a retired array; surviving ranks byte-identical; retire convention = append-only same-title backlog-closed row (1 file(s) changed)

## Verify

Each cell below was capped only against a recorded passing verify result — bee refuses a cap without one.

- **erf-1** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml`

## Deviations

- **erf-1** — rank --json payload shape changed from bare array to {ranked, retired} - an array cannot carry the named sibling; consumers indexing the bare array must move to .ranked

## Provenance

Proposed by `bee knowledge promote --work evolving-rank-freshness` from 1 capped cell trace(s) in `.bee/cells/` and the anchor `.bee/logs/scribing-runs.jsonl`. Every line above is copied from a trace or from the work item; nothing here is curated truth until a human or agent accepts it.

(b) AREA UPDATES — candidate spec-sync bullets, each citing its cell

areas: from the scribing stamp for "evolving-rank-freshness" — .bee/logs/scribing-runs.jsonl's most recent entry (2026-08-11T03:26:17.387Z), the work item declares no bee.areas.

area feedback-digest:
  - [erf-1] clusters with any closed-kind entry leave the ranking into a retired array; surviving ranks byte-identical; retire convention = append-only same-title backlog-closed row — feature-wide sync per the scribing stamp, 1 file(s) changed (trace .bee/cells/erf-1.json)

(c) PATTERN CANDIDATES — candidate bee.pattern concepts, bee.polarity pitfall

from cell erf-1 — save as docs/knowledge/patterns/evolving-rank-freshness-erf-1-pitfall.md

---
type: bee.pattern
title: evolving-rank-freshness cell erf-1 — pitfall candidate
description: "Pitfall candidate mined from cell erf-1's capped trace: rank --json payload shape changed from bare array to {ranked, retired} - an array cannot carry the named sibling; consumers indexing the bare array must move t…"
timestamp: 2026-08-11
bee:
  id: evolving-rank-freshness-erf-1-pitfall
  lifecycle: draft
  areas: [feedback-digest]
  sources: [.bee/cells/erf-1.json]
  polarity: pitfall
---

# evolving-rank-freshness cell erf-1 — pitfall candidate

## What the cell did

clusters with any closed-kind entry leave the ranking into a retired array; surviving ranks byte-identical; retire convention = append-only same-title backlog-closed row

## Recorded evidence (verbatim from .bee/cells/erf-1.json)

- **deviation** — rank --json payload shape changed from bare array to {ranked, retired} - an array cannot carry the named sibling; consumers indexing the bare array must move to .ranked

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

knowledge promote: 1 capped cell(s) mined, 1 delivery draft, 1 area bullet(s), 1 pattern candidate(s), 0 file(s) written.