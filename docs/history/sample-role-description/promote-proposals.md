promote proposal for work item "sample-role-description" (.bee/lanes/sample-role-description.json) — 1 capped cell(s): csd-1
anchor: ledger — .bee/lanes/sample-role-description.json
PROPOSAL ONLY — nothing was written. Applying any section below is a human or agent decision.

(a) DELIVERY DRAFT — save as docs/knowledge/work/sample-role-description/delivery.md

---
type: bee.delivery
title: sample-role-description — delivery
description: "Delivery record proposed by bee knowledge promote for work item sample-role-description: 1 capped cell(s), 1 recorded deviation(s)."
timestamp: 2026-08-26
bee:
  id: sample-role-description-delivery
  lifecycle: active
  required_context: [.bee/lanes/sample-role-description.json]
  sources: [.bee/lanes/sample-role-description.json, .bee/cells/csd-1.json]
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

(b) AREA UPDATES — candidate spec-sync bullets, each citing its cell

None: the work item declares no bee.areas, so there is no area to sync (D19).

(c) PATTERN CANDIDATES — candidate bee.pattern concepts, bee.polarity pitfall

from cell csd-1 — save as docs/knowledge/patterns/sample-role-description-csd-1-pitfall.md

---
type: bee.pattern
title: sample-role-description cell csd-1 — pitfall candidate
description: "Pitfall candidate mined from cell csd-1's capped trace: followed the plan"
timestamp: 2026-08-26
bee:
  id: sample-role-description-csd-1-pitfall
  lifecycle: draft
  sources: [.bee/cells/csd-1.json]
  polarity: pitfall
---

# sample-role-description cell csd-1 — pitfall candidate

## What the cell did

config-sample.json documents the optional role-slot description field (_doc slot_shapes entry + live generation example)

## Recorded evidence (verbatim from .bee/cells/csd-1.json)

- **deviation** — followed the plan

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

knowledge promote: 1 capped cell(s) mined, 1 delivery draft, 0 area bullet(s), 1 pattern candidate(s), 0 file(s) written.