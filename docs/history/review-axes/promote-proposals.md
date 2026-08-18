promote proposal for work item "review-axes" (docs/history/review-axes/CONTEXT.md) — 1 capped cell(s): ra-1
anchor: history — docs/history/review-axes/CONTEXT.md
PROPOSAL ONLY — nothing was written. Applying any section below is a human or agent decision.

(a) DELIVERY DRAFT — save as docs/knowledge/work/review-axes/delivery.md

---
type: bee.delivery
title: review-axes — delivery
description: "Delivery record proposed by bee knowledge promote for work item review-axes: 1 capped cell(s), 1 recorded deviation(s)."
timestamp: 2026-08-18
bee:
  id: review-axes-delivery
  lifecycle: active
  required_context: [docs/history/review-axes/CONTEXT.md]
  sources: [docs/history/review-axes/CONTEXT.md, .bee/cells/ra-1.json]
---

# review-axes — Delivery

## What shipped

- **ra-1** — Added standards/spec axis grouping and the 12-smell vocabulary to reviewing; regen chain synced (4 file(s) changed)

## Verify

Each cell below was capped only against a recorded passing verify result — bee refuses a cap without one.

- **ra-1** — `.bee/bin/bee dev release-manifest --check`

## Deviations

- **ra-1** — Copied .bee/bin/bee from the main checkout into this fresh worktree before running the regen chain, gitignored build artifact absent from a fresh worktree, tooling prerequisite not a source change

## Provenance

Proposed by `bee knowledge promote --work review-axes` from 1 capped cell trace(s) in `.bee/cells/` and the anchor `docs/history/review-axes/CONTEXT.md`. Every line above is copied from a trace or from the work item; nothing here is curated truth until a human or agent accepts it.

(b) AREA UPDATES — candidate spec-sync bullets, each citing its cell

None: the work item declares no bee.areas, so there is no area to sync (D19).

(c) PATTERN CANDIDATES — candidate bee.pattern concepts, bee.polarity pitfall

from cell ra-1 — save as docs/knowledge/patterns/review-axes-ra-1-pitfall.md

---
type: bee.pattern
title: review-axes cell ra-1 — pitfall candidate
description: "Pitfall candidate mined from cell ra-1's capped trace: Copied .bee/bin/bee from the main checkout into this fresh worktree before running the regen chain, gitignored build artifact absent from a fresh worktree, too…"
timestamp: 2026-08-18
bee:
  id: review-axes-ra-1-pitfall
  lifecycle: draft
  sources: [.bee/cells/ra-1.json]
  polarity: pitfall
---

# review-axes cell ra-1 — pitfall candidate

## What the cell did

Added standards/spec axis grouping and the 12-smell vocabulary to reviewing; regen chain synced

## Recorded evidence (verbatim from .bee/cells/ra-1.json)

- **deviation** — Copied .bee/bin/bee from the main checkout into this fresh worktree before running the regen chain, gitignored build artifact absent from a fresh worktree, tooling prerequisite not a source change

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

knowledge promote: 1 capped cell(s) mined, 1 delivery draft, 0 area bullet(s), 1 pattern candidate(s), 0 file(s) written.