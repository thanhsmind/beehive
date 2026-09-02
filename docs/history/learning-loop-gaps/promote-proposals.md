promote proposal for work item "learning-loop-gaps" (docs/history/learning-loop-gaps/CONTEXT.md + docs/history/learning-loop-gaps/plan.md) — 3 capped cell(s): llg-1, llg-2, llg-3
anchor: history — docs/history/learning-loop-gaps/CONTEXT.md, docs/history/learning-loop-gaps/plan.md
PROPOSAL ONLY — nothing was written. Applying any section below is a human or agent decision.

(a) DELIVERY DRAFT — save as docs/knowledge/work/learning-loop-gaps/delivery.md

---
type: bee.delivery
title: learning-loop-gaps — delivery
description: "Delivery record proposed by bee knowledge promote for work item learning-loop-gaps: 3 capped cell(s), 4 recorded deviation(s)."
timestamp: 2026-09-02
bee:
  id: learning-loop-gaps-delivery
  lifecycle: active
  required_context: [docs/history/learning-loop-gaps/CONTEXT.md, docs/history/learning-loop-gaps/plan.md]
  sources: [docs/history/learning-loop-gaps/CONTEXT.md, docs/history/learning-loop-gaps/plan.md, .bee/cells/llg-1.json, .bee/cells/llg-2.json, .bee/cells/llg-3.json]
---

# learning-loop-gaps — Delivery

## What shipped

- **llg-1** — The mining offer covers a clean session; the skill no longer routes through the unbuilt recovery-window verb (3 file(s) changed)
- **llg-2** — A non-mechanizable promotion routes into a skill the run actually opened, or tunes the description of one that should have fired (3 file(s) changed)
- **llg-3** — The crashed-session bullet uses the locked name clean-end trio, the miner prompt bars every edit, and the RED reports no longer claim results their commands did not produce (3 file(s) changed)

## Verify

Each cell below was capped only against a recorded passing verify result — bee refuses a cap without one.

- **llg-1** — `rg -q 'recovery window' skills/bee-hive/references/scout-and-ticks.md && echo 'FAIL: still cites the unbuilt verb as the route' && exit 1; rg -q 'asked' skills/bee-hive/references/scout-and-ticks.md && rg -q 'capture add --source mined' skills/bee-hive/references/scout-and-ticks.md`
- **llg-2** — `rg -q 'tune description' skills/bee-capturing/references/promotion.md && rg -c 'all three' skills/bee-capturing/references/promotion.md skills/bee-capturing/SKILL.md`
- **llg-3** — `rg -q 'clean-end trio' skills/bee-hive/references/scout-and-ticks.md && ! rg -q 'clean-end sequence' skills/bee-hive/references/scout-and-ticks.md && ! rg -q 'no code edits' skills/bee-hive/references/scout-and-ticks.md`

## Deviations

- **llg-1** — The worker left the cell claimed rather than capping it — a herding pane worker is bee-ignorant by contract and never runs a bee verb, so the orchestrator caps for it — followed the plan
- **llg-2** — The durable-owner backlog row the cell asked for was filed by the orchestrator, not the worker — the cell was dispatched to a herding pane, which is bee-ignorant by contract and runs no bee verb, so the obligation was never the worker's to meet — the plan was wrong about a fact
- **llg-2** — The worker left the cell claimed rather than capping it, for the same bee-ignorant-by-contract reason — followed the plan
- **llg-3** — Left the two pre-existing bee status spellings at scout-and-ticks.md:28 and :237 alone and matched only the line this feature introduced — the file was already mixed before the feature and unifying it whole is scope creep — found a better route

## Provenance

Proposed by `bee knowledge promote --work learning-loop-gaps` from 3 capped cell trace(s) in `.bee/cells/` and the anchor `docs/history/learning-loop-gaps/CONTEXT.md`, `docs/history/learning-loop-gaps/plan.md`. Every line above is copied from a trace or from the work item; nothing here is curated truth until a human or agent accepts it.

(b) AREA UPDATES — candidate spec-sync bullets, each citing its cell

None: the work item declares no bee.areas, so there is no area to sync (D19).

(c) PATTERN CANDIDATES — candidate bee.pattern concepts, bee.polarity pitfall

from cell llg-1 — save as docs/knowledge/patterns/learning-loop-gaps-llg-1-pitfall.md

---
type: bee.pattern
title: learning-loop-gaps cell llg-1 — pitfall candidate
description: "Pitfall candidate mined from cell llg-1's capped trace: The worker left the cell claimed rather than capping it — a herding pane worker is bee-ignorant by contract and never runs a bee verb, so the orchestrator caps…"
timestamp: 2026-09-02
bee:
  id: learning-loop-gaps-llg-1-pitfall
  lifecycle: draft
  sources: [.bee/cells/llg-1.json]
  polarity: pitfall
---

# learning-loop-gaps cell llg-1 — pitfall candidate

## What the cell did

The mining offer covers a clean session; the skill no longer routes through the unbuilt recovery-window verb

## Recorded evidence (verbatim from .bee/cells/llg-1.json)

- **deviation** — The worker left the cell claimed rather than capping it — a herding pane worker is bee-ignorant by contract and never runs a bee verb, so the orchestrator caps for it — followed the plan

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

from cell llg-2 — save as docs/knowledge/patterns/learning-loop-gaps-llg-2-pitfall.md

---
type: bee.pattern
title: learning-loop-gaps cell llg-2 — pitfall candidate
description: "Pitfall candidate mined from cell llg-2's capped trace: The durable-owner backlog row the cell asked for was filed by the orchestrator, not the worker — the cell was dispatched to a herding pane, which is bee-ignora…"
timestamp: 2026-09-02
bee:
  id: learning-loop-gaps-llg-2-pitfall
  lifecycle: draft
  sources: [.bee/cells/llg-2.json]
  polarity: pitfall
---

# learning-loop-gaps cell llg-2 — pitfall candidate

## What the cell did

A non-mechanizable promotion routes into a skill the run actually opened, or tunes the description of one that should have fired

## Recorded evidence (verbatim from .bee/cells/llg-2.json)

- **deviation** — The durable-owner backlog row the cell asked for was filed by the orchestrator, not the worker — the cell was dispatched to a herding pane, which is bee-ignorant by contract and runs no bee verb, so the obligation was never the worker's to meet — the plan was wrong about a fact
- **deviation** — The worker left the cell claimed rather than capping it, for the same bee-ignorant-by-contract reason — followed the plan

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

from cell llg-3 — save as docs/knowledge/patterns/learning-loop-gaps-llg-3-pitfall.md

---
type: bee.pattern
title: learning-loop-gaps cell llg-3 — pitfall candidate
description: "Pitfall candidate mined from cell llg-3's capped trace: Left the two pre-existing bee status spellings at scout-and-ticks.md:28 and :237 alone and matched only the line this feature introduced — the file was already…"
timestamp: 2026-09-02
bee:
  id: learning-loop-gaps-llg-3-pitfall
  lifecycle: draft
  sources: [.bee/cells/llg-3.json]
  polarity: pitfall
---

# learning-loop-gaps cell llg-3 — pitfall candidate

## What the cell did

The crashed-session bullet uses the locked name clean-end trio, the miner prompt bars every edit, and the RED reports no longer claim results their commands did not produce

## Recorded evidence (verbatim from .bee/cells/llg-3.json)

- **deviation** — Left the two pre-existing bee status spellings at scout-and-ticks.md:28 and :237 alone and matched only the line this feature introduced — the file was already mixed before the feature and unifying it whole is scope creep — found a better route

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

knowledge promote: 3 capped cell(s) mined, 1 delivery draft, 0 area bullet(s), 3 pattern candidate(s), 0 file(s) written.