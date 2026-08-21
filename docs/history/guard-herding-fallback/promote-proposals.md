promote proposal for work item "guard-herding-fallback" (.bee/lanes/guard-herding-fallback.json) — 1 capped cell(s): hgf-1
anchor: ledger — .bee/lanes/guard-herding-fallback.json
PROPOSAL ONLY — nothing was written. Applying any section below is a human or agent decision.

(a) DELIVERY DRAFT — save as docs/knowledge/work/guard-herding-fallback/delivery.md

---
type: bee.delivery
title: guard-herding-fallback — delivery
description: "Delivery record proposed by bee knowledge promote for work item guard-herding-fallback: 1 capped cell(s), 1 recorded deviation(s)."
timestamp: 2026-08-21
bee:
  id: guard-herding-fallback-delivery
  lifecycle: active
  required_context: [.bee/lanes/guard-herding-fallback.json]
  sources: [.bee/lanes/guard-herding-fallback.json, .bee/cells/hgf-1.json]
---

# guard-herding-fallback — Delivery

## What shipped

- **hgf-1** — The model-guard now admits exactly the models dispatch prepare can publish for a herding slot carrying fallback:default - generation and review only - so the two doors no longer contradict each other. (1 file(s) changed)

## Verify

Each cell below was capped only against a recorded passing verify result — bee refuses a cap without one.

- **hgf-1** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml model_guard`

## Deviations

- **hgf-1** — Cell lane raised from small to standard at creation time so the judge-required hooks/ root took a real independent read instead of a lane-small acknowledgement.

## Provenance

Proposed by `bee knowledge promote --work guard-herding-fallback` from 1 capped cell trace(s) in `.bee/cells/` and the anchor `.bee/lanes/guard-herding-fallback.json`. Every line above is copied from a trace or from the work item; nothing here is curated truth until a human or agent accepts it.

(b) AREA UPDATES — candidate spec-sync bullets, each citing its cell

None: the work item declares no bee.areas, so there is no area to sync (D19).

(c) PATTERN CANDIDATES — candidate bee.pattern concepts, bee.polarity pitfall

from cell hgf-1 — save as docs/knowledge/patterns/guard-herding-fallback-hgf-1-pitfall.md

---
type: bee.pattern
title: guard-herding-fallback cell hgf-1 — pitfall candidate
description: "Pitfall candidate mined from cell hgf-1's capped trace: Cell lane raised from small to standard at creation time so the judge-required hooks/ root took a real independent read instead of a lane-small acknowledgement."
timestamp: 2026-08-21
bee:
  id: guard-herding-fallback-hgf-1-pitfall
  lifecycle: draft
  sources: [.bee/cells/hgf-1.json]
  polarity: pitfall
---

# guard-herding-fallback cell hgf-1 — pitfall candidate

## What the cell did

The model-guard now admits exactly the models dispatch prepare can publish for a herding slot carrying fallback:default - generation and review only - so the two doors no longer contradict each other.

## Recorded evidence (verbatim from .bee/cells/hgf-1.json)

- **deviation** — Cell lane raised from small to standard at creation time so the judge-required hooks/ root took a real independent read instead of a lane-small acknowledgement.

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

knowledge promote: 1 capped cell(s) mined, 1 delivery draft, 0 area bullet(s), 1 pattern candidate(s), 0 file(s) written.