promote proposal for work item "piggery-eval" (docs/history/piggery-eval/CONTEXT.md + docs/history/piggery-eval/plan.md) — 1 capped cell(s): pe-1
anchor: history — docs/history/piggery-eval/CONTEXT.md, docs/history/piggery-eval/plan.md
PROPOSAL ONLY — nothing was written. Applying any section below is a human or agent decision.

(a) DELIVERY DRAFT — save as docs/knowledge/work/piggery-eval/delivery.md

---
type: bee.delivery
title: piggery-eval — delivery
description: "Delivery record proposed by bee knowledge promote for work item piggery-eval: 1 capped cell(s), 1 recorded deviation(s)."
timestamp: 2026-10-16
bee:
  id: piggery-eval-delivery
  lifecycle: active
  required_context: [docs/history/piggery-eval/CONTEXT.md, docs/history/piggery-eval/plan.md]
  sources: [docs/history/piggery-eval/CONTEXT.md, docs/history/piggery-eval/plan.md, .bee/cells/pe-1.json]
---

# piggery-eval — Delivery

## What shipped

- **pe-1** — Landed the piggery xia brief and spec drop sup-20261016-a506; draft removed (5 file(s) changed)

## Verify

Each cell below was capped only against a recorded passing verify result — bee refuses a cap without one.

- **pe-1** — `Both files exist; every verdict row cites a repo path that exists (rg/fd); the brief never gives bee's gate piggery's meaning.` — docs pointer check via python3 -I .bee/tmp/pe/check.py: brief checked=29 missing=2 (docs/guide.md and docs/reference.md are piggery-relative, both exist under /mnt/data/Projects/refs/slp/piggery/docs…

## Deviations

- **pe-1** — the draft removal shows no deletion in the commit — the draft was never tracked, so removing it before staging left nothing to record — the plan was wrong about a fact

## Provenance

Proposed by `bee knowledge promote --work piggery-eval` from 1 capped cell trace(s) in `.bee/cells/` and the anchor `docs/history/piggery-eval/CONTEXT.md`, `docs/history/piggery-eval/plan.md`. Every line above is copied from a trace or from the work item; nothing here is curated truth until a human or agent accepts it.

(b) AREA UPDATES — candidate spec-sync bullets, each citing its cell

None: the work item declares no bee.areas, so there is no area to sync (D19).

(c) PATTERN CANDIDATES — candidate bee.pattern concepts, bee.polarity pitfall

from cell pe-1 — save as docs/knowledge/patterns/piggery-eval-pe-1-pitfall.md

---
type: bee.pattern
title: piggery-eval cell pe-1 — pitfall candidate
description: "Pitfall candidate mined from cell pe-1's capped trace: the draft removal shows no deletion in the commit — the draft was never tracked, so removing it before staging left nothing to record — the plan was wrong abou…"
timestamp: 2026-10-16
bee:
  id: piggery-eval-pe-1-pitfall
  lifecycle: draft
  sources: [.bee/cells/pe-1.json]
  polarity: pitfall
---

# piggery-eval cell pe-1 — pitfall candidate

## What the cell did

Landed the piggery xia brief and spec drop sup-20261016-a506; draft removed

## Recorded evidence (verbatim from .bee/cells/pe-1.json)

- **deviation** — the draft removal shows no deletion in the commit — the draft was never tracked, so removing it before staging left nothing to record — the plan was wrong about a fact

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

knowledge promote: 1 capped cell(s) mined, 1 delivery draft, 0 area bullet(s), 1 pattern candidate(s), 0 file(s) written.