promote proposal for work item "orient-merged-worktree" (docs/history/orient-merged-worktree/plan.md) — 1 capped cell(s): omw-1
anchor: history — docs/history/orient-merged-worktree/plan.md
PROPOSAL ONLY — nothing was written. Applying any section below is a human or agent decision.

(a) DELIVERY DRAFT — save as docs/knowledge/work/orient-merged-worktree/delivery.md

---
type: bee.delivery
title: orient-merged-worktree — delivery
description: "Delivery record proposed by bee knowledge promote for work item orient-merged-worktree: 1 capped cell(s), 2 recorded deviation(s)."
timestamp: 2026-10-03
bee:
  id: orient-merged-worktree-delivery
  lifecycle: active
  required_context: [docs/history/orient-merged-worktree/plan.md]
  sources: [docs/history/orient-merged-worktree/plan.md, .bee/cells/omw-1.json]
---

# orient-merged-worktree — Delivery

## What shipped

- **omw-1** — orient and the per-turn hint skip the worktree enter step once the branch is merged into main (4 file(s) changed)

## Verify

Each cell below was capped only against a recorded passing verify result — bee refuses a cap without one.

- **omw-1** — `cd packages/bee-rs && cargo test --release -p bee --bin bee verbs::status_full && cargo test --release -p bee --bin bee prompt_context` — status_full 117 (116 before), prompt_context 38; merged and fresh branch fixtures

## Deviations

- **omw-1** — followed the plan
- **omw-1** — sync-ack: no owned skill documents the per-turn hint or orient's next step; the knowledge concept carries the rule

## Provenance

Proposed by `bee knowledge promote --work orient-merged-worktree` from 1 capped cell trace(s) in `.bee/cells/` and the anchor `docs/history/orient-merged-worktree/plan.md`. Every line above is copied from a trace or from the work item; nothing here is curated truth until a human or agent accepts it.

(b) AREA UPDATES — candidate spec-sync bullets, each citing its cell

None: the work item declares no bee.areas, so there is no area to sync (D19).

(c) PATTERN CANDIDATES — candidate bee.pattern concepts, bee.polarity pitfall

from cell omw-1 — save as docs/knowledge/patterns/orient-merged-worktree-omw-1-pitfall.md

---
type: bee.pattern
title: orient-merged-worktree cell omw-1 — pitfall candidate
description: "Pitfall candidate mined from cell omw-1's capped trace: followed the plan"
timestamp: 2026-10-03
bee:
  id: orient-merged-worktree-omw-1-pitfall
  lifecycle: draft
  sources: [.bee/cells/omw-1.json]
  polarity: pitfall
---

# orient-merged-worktree cell omw-1 — pitfall candidate

## What the cell did

orient and the per-turn hint skip the worktree enter step once the branch is merged into main

## Recorded evidence (verbatim from .bee/cells/omw-1.json)

- **deviation** — followed the plan
- **deviation** — sync-ack: no owned skill documents the per-turn hint or orient's next step; the knowledge concept carries the rule

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

knowledge promote: 1 capped cell(s) mined, 1 delivery draft, 0 area bullet(s), 1 pattern candidate(s), 0 file(s) written.