promote proposal for work item "worktree-entry-routing" (docs/history/worktree-entry-routing/CONTEXT.md) — 1 capped cell(s): wer-1
anchor: history — docs/history/worktree-entry-routing/CONTEXT.md
PROPOSAL ONLY — nothing was written. Applying any section below is a human or agent decision.

(a) DELIVERY DRAFT — save as docs/knowledge/work/worktree-entry-routing/delivery.md

---
type: bee.delivery
title: worktree-entry-routing — delivery
description: "Delivery record proposed by bee knowledge promote for work item worktree-entry-routing: 1 capped cell(s), 2 recorded deviation(s)."
timestamp: 2026-09-05
bee:
  id: worktree-entry-routing-delivery
  lifecycle: active
  required_context: [docs/history/worktree-entry-routing/CONTEXT.md]
  sources: [docs/history/worktree-entry-routing/CONTEXT.md, .bee/cells/wer-1.json]
---

# worktree-entry-routing — Delivery

## What shipped

- **wer-1** — Separated native vs external transport in worktree dispatch instructions (2 file(s) changed)

## Verify

Each cell below was capped only against a recorded passing verify result — bee refuses a cap without one.

- **wer-1** — `grep -q 'external.*cwd' skills/bee-hive/references/routing-and-contracts.md || grep -q 'herding.*cwd' skills/bee-hive/references/routing-and-contracts.md`

## Deviations

- **wer-1** — Edited AGENTS.md instead of routing-and-contracts.md — the problematic instruction was in AGENTS.md, routing-and-contracts.md had no such content — found a better route
- **wer-1** — sync-ack: Cell predicted routing-and-contracts.md but actual fix was in AGENTS.md where the instruction lived

## Provenance

Proposed by `bee knowledge promote --work worktree-entry-routing` from 1 capped cell trace(s) in `.bee/cells/` and the anchor `docs/history/worktree-entry-routing/CONTEXT.md`. Every line above is copied from a trace or from the work item; nothing here is curated truth until a human or agent accepts it.

(b) AREA UPDATES — candidate spec-sync bullets, each citing its cell

None: the work item declares no bee.areas, so there is no area to sync (D19).

(c) PATTERN CANDIDATES — candidate bee.pattern concepts, bee.polarity pitfall

from cell wer-1 — save as docs/knowledge/patterns/worktree-entry-routing-wer-1-pitfall.md

---
type: bee.pattern
title: worktree-entry-routing cell wer-1 — pitfall candidate
description: "Pitfall candidate mined from cell wer-1's capped trace: Edited AGENTS.md instead of routing-and-contracts.md — the problematic instruction was in AGENTS.md, routing-and-contracts.md had no such content — found a bet…"
timestamp: 2026-09-05
bee:
  id: worktree-entry-routing-wer-1-pitfall
  lifecycle: draft
  sources: [.bee/cells/wer-1.json]
  polarity: pitfall
---

# worktree-entry-routing cell wer-1 — pitfall candidate

## What the cell did

Separated native vs external transport in worktree dispatch instructions

## Recorded evidence (verbatim from .bee/cells/wer-1.json)

- **deviation** — Edited AGENTS.md instead of routing-and-contracts.md — the problematic instruction was in AGENTS.md, routing-and-contracts.md had no such content — found a better route
- **deviation** — sync-ack: Cell predicted routing-and-contracts.md but actual fix was in AGENTS.md where the instruction lived

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

knowledge promote: 1 capped cell(s) mined, 1 delivery draft, 0 area bullet(s), 1 pattern candidate(s), 0 file(s) written.