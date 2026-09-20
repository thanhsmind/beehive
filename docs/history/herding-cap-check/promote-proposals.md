promote proposal for work item "herding-cap-check" (docs/history/herding-cap-check/CONTEXT.md) — 1 capped cell(s): hcapc-1
anchor: history — docs/history/herding-cap-check/CONTEXT.md
PROPOSAL ONLY — nothing was written. Applying any section below is a human or agent decision.

(a) DELIVERY DRAFT — save as docs/knowledge/work/herding-cap-check/delivery.md

---
type: bee.delivery
title: herding-cap-check — delivery
description: "Delivery record proposed by bee knowledge promote for work item herding-cap-check: 1 capped cell(s), 2 recorded deviation(s)."
timestamp: 2026-09-20
bee:
  id: herding-cap-check-delivery
  lifecycle: active
  areas: [bee-herding]
  required_context: [docs/history/herding-cap-check/CONTEXT.md]
  sources: [docs/history/herding-cap-check/CONTEXT.md, .bee/cells/hcapc-1.json]
---

# herding-cap-check — Delivery

## What shipped

- **hcapc-1** — Refuse herding run when worker claimed success and left cell uncapped (1 file(s) changed)

## Verify

Each cell below was capped only against a recorded passing verify result — bee refuses a cap without one.

- **hcapc-1** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee --bin bee herding::run` — touched run.rs

## Deviations

- **hcapc-1** — followed the plan
- **hcapc-1** — sync-ack: cell hcapc-1 scoped only to run.rs without skill edits as approved in cell definition

## Provenance

Proposed by `bee knowledge promote --work herding-cap-check` from 1 capped cell trace(s) in `.bee/cells/` and the anchor `docs/history/herding-cap-check/CONTEXT.md`. Every line above is copied from a trace or from the work item; nothing here is curated truth until a human or agent accepts it.

(b) AREA UPDATES — candidate spec-sync bullets, each citing its cell

areas: from the scribing stamp for "herding-cap-check" — .bee/logs/scribing-runs.jsonl's most recent entry (2026-09-20T14:16:48.924Z), the work item declares no bee.areas.

area bee-herding:
  - [hcapc-1] Refuse herding run when worker claimed success and left cell uncapped — feature-wide sync per the scribing stamp, 1 file(s) changed (trace .bee/cells/hcapc-1.json)

(c) PATTERN CANDIDATES — candidate bee.pattern concepts, bee.polarity pitfall

from cell hcapc-1 — save as docs/knowledge/patterns/herding-cap-check-hcapc-1-pitfall.md

---
type: bee.pattern
title: herding-cap-check cell hcapc-1 — pitfall candidate
description: "Pitfall candidate mined from cell hcapc-1's capped trace: followed the plan"
timestamp: 2026-09-20
bee:
  id: herding-cap-check-hcapc-1-pitfall
  lifecycle: draft
  areas: [bee-herding]
  sources: [.bee/cells/hcapc-1.json]
  polarity: pitfall
---

# herding-cap-check cell hcapc-1 — pitfall candidate

## What the cell did

Refuse herding run when worker claimed success and left cell uncapped

## Recorded evidence (verbatim from .bee/cells/hcapc-1.json)

- **deviation** — followed the plan
- **deviation** — sync-ack: cell hcapc-1 scoped only to run.rs without skill edits as approved in cell definition

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

knowledge promote: 1 capped cell(s) mined, 1 delivery draft, 1 area bullet(s), 1 pattern candidate(s), 0 file(s) written.