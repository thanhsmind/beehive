promote proposal for work item "dispatch-label-seat-first" (.bee/lanes/dispatch-label-seat-first.json) — 1 capped cell(s): dlsf-1
anchor: ledger — .bee/lanes/dispatch-label-seat-first.json
PROPOSAL ONLY — nothing was written. Applying any section below is a human or agent decision.

(a) DELIVERY DRAFT — save as docs/knowledge/work/dispatch-label-seat-first/delivery.md

---
type: bee.delivery
title: dispatch-label-seat-first — delivery
description: "Delivery record proposed by bee knowledge promote for work item dispatch-label-seat-first: 1 capped cell(s), 1 recorded deviation(s)."
timestamp: 2026-09-02
bee:
  id: dispatch-label-seat-first-delivery
  lifecycle: active
  required_context: [.bee/lanes/dispatch-label-seat-first.json]
  sources: [.bee/lanes/dispatch-label-seat-first.json, .bee/cells/dlsf-1.json]
---

# dispatch-label-seat-first — Delivery

## What shipped

- **dlsf-1** — The dispatch label leads with the asked role when --role is given; role-less dispatches keep today's bytes (3 file(s) changed)

## Verify

Each cell below was capped only against a recorded passing verify result — bee refuses a cap without one.

- **dlsf-1** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -- drivers prepare`

## Deviations

- **dlsf-1** — Edited docs/product-description/delegation/dispatch.md line 211 (the label rule) though the cell listed it only under affects_specs — the sentence named <kind>: <purpose> as the rule and became false the moment the lead changed — something else had to be fixed first

## Provenance

Proposed by `bee knowledge promote --work dispatch-label-seat-first` from 1 capped cell trace(s) in `.bee/cells/` and the anchor `.bee/lanes/dispatch-label-seat-first.json`. Every line above is copied from a trace or from the work item; nothing here is curated truth until a human or agent accepts it.

(b) AREA UPDATES — candidate spec-sync bullets, each citing its cell

None: the work item declares no bee.areas, so there is no area to sync (D19).

(c) PATTERN CANDIDATES — candidate bee.pattern concepts, bee.polarity pitfall

from cell dlsf-1 — save as docs/knowledge/patterns/dispatch-label-seat-first-dlsf-1-pitfall.md

---
type: bee.pattern
title: dispatch-label-seat-first cell dlsf-1 — pitfall candidate
description: "Pitfall candidate mined from cell dlsf-1's capped trace: Edited docs/product-description/delegation/dispatch.md line 211 (the label rule) though the cell listed it only under affects_specs — the sentence named <kind>…"
timestamp: 2026-09-02
bee:
  id: dispatch-label-seat-first-dlsf-1-pitfall
  lifecycle: draft
  sources: [.bee/cells/dlsf-1.json]
  polarity: pitfall
---

# dispatch-label-seat-first cell dlsf-1 — pitfall candidate

## What the cell did

The dispatch label leads with the asked role when --role is given; role-less dispatches keep today's bytes

## Recorded evidence (verbatim from .bee/cells/dlsf-1.json)

- **deviation** — Edited docs/product-description/delegation/dispatch.md line 211 (the label rule) though the cell listed it only under affects_specs — the sentence named <kind>: <purpose> as the rule and became false the moment the lead changed — something else had to be fixed first

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

knowledge promote: 1 capped cell(s) mined, 1 delivery draft, 0 area bullet(s), 1 pattern candidate(s), 0 file(s) written.