promote proposal for work item "preamble-law-fix" (docs/history/preamble-law-fix/CONTEXT.md + docs/history/preamble-law-fix/plan.md) — 1 capped cell(s): plf-1
anchor: history — docs/history/preamble-law-fix/CONTEXT.md, docs/history/preamble-law-fix/plan.md
PROPOSAL ONLY — nothing was written. Applying any section below is a human or agent decision.

(a) DELIVERY DRAFT — save as docs/knowledge/work/preamble-law-fix/delivery.md

---
type: bee.delivery
title: preamble-law-fix — delivery
description: "Delivery record proposed by bee knowledge promote for work item preamble-law-fix: 1 capped cell(s), 1 recorded deviation(s)."
timestamp: 2026-10-21
bee:
  id: preamble-law-fix-delivery
  lifecycle: active
  required_context: [docs/history/preamble-law-fix/CONTEXT.md, docs/history/preamble-law-fix/plan.md]
  sources: [docs/history/preamble-law-fix/CONTEXT.md, docs/history/preamble-law-fix/plan.md, .bee/cells/plf-1.json]
---

# preamble-law-fix — Delivery

## What shipped

- **plf-1** — instruction_laws green again: the budget doc paragraphs swap so the not-instruction-text paragraph sits outside the 6-line scan window (1 file(s) changed)

## Verify

Each cell below was capped only against a recorded passing verify result — bee refuses a cap without one.

- **plf-1** — `cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee --test instruction_laws && cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee --bin bee comment_baseline` — 9 and 9 passed; the law test was red before this change on the same tree

## Deviations

- **plf-1** — followed the plan

## Provenance

Proposed by `bee knowledge promote --work preamble-law-fix` from 1 capped cell trace(s) in `.bee/cells/` and the anchor `docs/history/preamble-law-fix/CONTEXT.md`, `docs/history/preamble-law-fix/plan.md`. Every line above is copied from a trace or from the work item; nothing here is curated truth until a human or agent accepts it.

(b) AREA UPDATES — candidate spec-sync bullets, each citing its cell

None: the work item declares no bee.areas, so there is no area to sync (D19).

(c) PATTERN CANDIDATES — candidate bee.pattern concepts, bee.polarity pitfall

from cell plf-1 — save as docs/knowledge/patterns/preamble-law-fix-plf-1-pitfall.md

---
type: bee.pattern
title: preamble-law-fix cell plf-1 — pitfall candidate
description: "Pitfall candidate mined from cell plf-1's capped trace: followed the plan"
timestamp: 2026-10-21
bee:
  id: preamble-law-fix-plf-1-pitfall
  lifecycle: draft
  sources: [.bee/cells/plf-1.json]
  polarity: pitfall
---

# preamble-law-fix cell plf-1 — pitfall candidate

## What the cell did

instruction_laws green again: the budget doc paragraphs swap so the not-instruction-text paragraph sits outside the 6-line scan window

## Recorded evidence (verbatim from .bee/cells/plf-1.json)

- **deviation** — followed the plan

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

knowledge promote: 1 capped cell(s) mined, 1 delivery draft, 0 area bullet(s), 1 pattern candidate(s), 0 file(s) written.