promote proposal for work item "close-usage-summary" (.bee/lanes/close-usage-summary.json) — 1 capped cell(s): cus-close-usage-section
anchor: ledger — .bee/lanes/close-usage-summary.json
PROPOSAL ONLY — nothing was written. Applying any section below is a human or agent decision.

(a) DELIVERY DRAFT — save as docs/knowledge/work/close-usage-summary/delivery.md

---
type: bee.delivery
title: close-usage-summary — delivery
description: "Delivery record proposed by bee knowledge promote for work item close-usage-summary: 1 capped cell(s), 1 recorded deviation(s)."
timestamp: 2026-08-30
bee:
  id: close-usage-summary-delivery
  lifecycle: active
  required_context: [.bee/lanes/close-usage-summary.json]
  sources: [.bee/lanes/close-usage-summary.json, .bee/cells/cus-close-usage-section.json]
---

# close-usage-summary — Delivery

## What shipped

- **cus-close-usage-section** — bee close prints a token-usage section (sessions + subagents + total) and inserts a usage object into its JSON result (2 file(s) changed)

## Verify

Each cell below was capped only against a recorded passing verify result — bee refuses a cap without one.

- **cus-close-usage-section** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --no-fail-fast --manifest-path packages/bee-rs/Cargo.toml usage`

## Deviations

- **cus-close-usage-section** — followed the plan

## Provenance

Proposed by `bee knowledge promote --work close-usage-summary` from 1 capped cell trace(s) in `.bee/cells/` and the anchor `.bee/lanes/close-usage-summary.json`. Every line above is copied from a trace or from the work item; nothing here is curated truth until a human or agent accepts it.

(b) AREA UPDATES — candidate spec-sync bullets, each citing its cell

None: the work item declares no bee.areas, so there is no area to sync (D19).

(c) PATTERN CANDIDATES — candidate bee.pattern concepts, bee.polarity pitfall

from cell cus-close-usage-section — save as docs/knowledge/patterns/close-usage-summary-cus-close-usage-section-pitfall.md

---
type: bee.pattern
title: close-usage-summary cell cus-close-usage-section — pitfall candidate
description: "Pitfall candidate mined from cell cus-close-usage-section's capped trace: followed the plan"
timestamp: 2026-08-30
bee:
  id: close-usage-summary-cus-close-usage-section-pitfall
  lifecycle: draft
  sources: [.bee/cells/cus-close-usage-section.json]
  polarity: pitfall
---

# close-usage-summary cell cus-close-usage-section — pitfall candidate

## What the cell did

bee close prints a token-usage section (sessions + subagents + total) and inserts a usage object into its JSON result

## Recorded evidence (verbatim from .bee/cells/cus-close-usage-section.json)

- **deviation** — followed the plan

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

knowledge promote: 1 capped cell(s) mined, 1 delivery draft, 0 area bullet(s), 1 pattern candidate(s), 0 file(s) written.