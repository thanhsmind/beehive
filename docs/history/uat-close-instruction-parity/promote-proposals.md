promote proposal for work item "uat-close-instruction-parity" (.bee/lanes/uat-close-instruction-parity.json) — 1 capped cell(s): ucip-1
anchor: ledger — .bee/lanes/uat-close-instruction-parity.json
PROPOSAL ONLY — nothing was written. Applying any section below is a human or agent decision.

(a) DELIVERY DRAFT — save as docs/knowledge/work/uat-close-instruction-parity/delivery.md

---
type: bee.delivery
title: uat-close-instruction-parity — delivery
description: "Delivery record proposed by bee knowledge promote for work item uat-close-instruction-parity: 1 capped cell(s), 3 recorded deviation(s)."
timestamp: 2026-08-26
bee:
  id: uat-close-instruction-parity-delivery
  lifecycle: active
  required_context: [.bee/lanes/uat-close-instruction-parity.json]
  sources: [.bee/lanes/uat-close-instruction-parity.json, .bee/cells/ucip-1.json]
---

# uat-close-instruction-parity — Delivery

## What shipped

- **ucip-1** — Make uat_stop close placement read merge-first across the instruction layer (0 file(s) changed)

## Verify

Each cell below was capped only against a recorded passing verify result — bee refuses a cap without one.

- **ucip-1** — `bee dev regen green, then bee dev release-manifest --check clean; rg confirms no instruction file still says the uat door defaults to merge time`

## Deviations

- **ucip-1** — docs-lane ran in the MAIN checkout while a sibling session was live (models-show-verb): file sets fully disjoint (skills/docs vs packages/bee-rs), commit path-scoped through the concurrent-worker guard
- **ucip-1** — docs-lane in MAIN with a live sibling session; disjoint files, path-scoped commit
- **ucip-1** — sync-ack: AGENTS.md touched only in the worktree-first rule tail (uat_stop placement); the flagged rule agents-capture-line-at-close and its applied_at files are untouched by this diff

## Provenance

Proposed by `bee knowledge promote --work uat-close-instruction-parity` from 1 capped cell trace(s) in `.bee/cells/` and the anchor `.bee/lanes/uat-close-instruction-parity.json`. Every line above is copied from a trace or from the work item; nothing here is curated truth until a human or agent accepts it.

(b) AREA UPDATES — candidate spec-sync bullets, each citing its cell

None: the work item declares no bee.areas, so there is no area to sync (D19).

(c) PATTERN CANDIDATES — candidate bee.pattern concepts, bee.polarity pitfall

from cell ucip-1 — save as docs/knowledge/patterns/uat-close-instruction-parity-ucip-1-pitfall.md

---
type: bee.pattern
title: uat-close-instruction-parity cell ucip-1 — pitfall candidate
description: "Pitfall candidate mined from cell ucip-1's capped trace: docs-lane ran in the MAIN checkout while a sibling session was live (models-show-verb): file sets fully disjoint (skills/docs vs packages/bee-rs), commit path-…"
timestamp: 2026-08-26
bee:
  id: uat-close-instruction-parity-ucip-1-pitfall
  lifecycle: draft
  sources: [.bee/cells/ucip-1.json]
  polarity: pitfall
---

# uat-close-instruction-parity cell ucip-1 — pitfall candidate

## What the cell did

Make uat_stop close placement read merge-first across the instruction layer

## Recorded evidence (verbatim from .bee/cells/ucip-1.json)

- **deviation** — docs-lane ran in the MAIN checkout while a sibling session was live (models-show-verb): file sets fully disjoint (skills/docs vs packages/bee-rs), commit path-scoped through the concurrent-worker guard
- **deviation** — docs-lane in MAIN with a live sibling session; disjoint files, path-scoped commit
- **deviation** — sync-ack: AGENTS.md touched only in the worktree-first rule tail (uat_stop placement); the flagged rule agents-capture-line-at-close and its applied_at files are untouched by this diff

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

knowledge promote: 1 capped cell(s) mined, 1 delivery draft, 0 area bullet(s), 1 pattern candidate(s), 0 file(s) written.