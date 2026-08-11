promote proposal for work item "schedule-regen-awareness" (.bee/logs/scribing-runs.jsonl) — 2 capped cell(s): sra-1, sra-2
anchor: ledger — .bee/logs/scribing-runs.jsonl
PROPOSAL ONLY — nothing was written. Applying any section below is a human or agent decision.

(a) DELIVERY DRAFT — save as docs/knowledge/work/schedule-regen-awareness/delivery.md

---
type: bee.delivery
title: schedule-regen-awareness — delivery
description: "Delivery record proposed by bee knowledge promote for work item schedule-regen-awareness: 2 capped cell(s), 1 recorded deviation(s)."
timestamp: 2026-08-11
bee:
  id: schedule-regen-awareness-delivery
  lifecycle: active
  areas: [workflow-state]
  required_context: [.bee/logs/scribing-runs.jsonl]
  sources: [.bee/logs/scribing-runs.jsonl, .bee/cells/sra-1.json, .bee/cells/sra-2.json]
---

# schedule-regen-awareness — Delivery

## What shipped

- **sra-1** — compute_schedule derives obligated roots via derive_regen_guards (the cells-add authority) and serializes shared-root cells like file overlaps; obligation_conflicts recorded on the Schedule struct (2 file(s) changed)
- **sra-2** — cells schedule renders obligation conflicts: json obligation_conflicts array + one text line per conflict naming both cells and the shared root; empty case byte-identical; command-level tests out-of-process (2 file(s) changed)

## Verify

Each cell below was capped only against a recorded passing verify result — bee refuses a cap without one.

- **sra-1** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml`
- **sra-2** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml`

## Deviations

- **sra-1** — conflict rendering in the cells schedule command was out of the declared file scope - completed by follow-up cell sra-2

## Provenance

Proposed by `bee knowledge promote --work schedule-regen-awareness` from 2 capped cell trace(s) in `.bee/cells/` and the anchor `.bee/logs/scribing-runs.jsonl`. Every line above is copied from a trace or from the work item; nothing here is curated truth until a human or agent accepts it.

(b) AREA UPDATES — candidate spec-sync bullets, each citing its cell

areas: from the scribing stamp for "schedule-regen-awareness" — .bee/logs/scribing-runs.jsonl's most recent entry (2026-08-11T04:56:28.993Z), the work item declares no bee.areas.

area workflow-state:
  - [sra-1] compute_schedule derives obligated roots via derive_regen_guards (the cells-add authority) and serializes shared-root cells like file overlaps; obligation_conflicts recorded on the Schedule struct — feature-wide sync per the scribing stamp, 2 file(s) changed (trace .bee/cells/sra-1.json)
  - [sra-2] cells schedule renders obligation conflicts: json obligation_conflicts array + one text line per conflict naming both cells and the shared root; empty case byte-identical; command-level tests out-of-process — feature-wide sync per the scribing stamp, 2 file(s) changed (trace .bee/cells/sra-2.json)

(c) PATTERN CANDIDATES — candidate bee.pattern concepts, bee.polarity pitfall

from cell sra-1 — save as docs/knowledge/patterns/schedule-regen-awareness-sra-1-pitfall.md

---
type: bee.pattern
title: schedule-regen-awareness cell sra-1 — pitfall candidate
description: "Pitfall candidate mined from cell sra-1's capped trace: conflict rendering in the cells schedule command was out of the declared file scope - completed by follow-up cell sra-2"
timestamp: 2026-08-11
bee:
  id: schedule-regen-awareness-sra-1-pitfall
  lifecycle: draft
  areas: [workflow-state]
  sources: [.bee/cells/sra-1.json]
  polarity: pitfall
---

# schedule-regen-awareness cell sra-1 — pitfall candidate

## What the cell did

compute_schedule derives obligated roots via derive_regen_guards (the cells-add authority) and serializes shared-root cells like file overlaps; obligation_conflicts recorded on the Schedule struct

## Recorded evidence (verbatim from .bee/cells/sra-1.json)

- **deviation** — conflict rendering in the cells schedule command was out of the declared file scope - completed by follow-up cell sra-2

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

knowledge promote: 2 capped cell(s) mined, 1 delivery draft, 2 area bullet(s), 1 pattern candidate(s), 0 file(s) written.