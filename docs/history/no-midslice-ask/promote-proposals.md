promote proposal for work item "no-midslice-ask" (.bee/lanes/no-midslice-ask.json) — 1 capped cell(s): nma-1
anchor: ledger — .bee/lanes/no-midslice-ask.json
PROPOSAL ONLY — nothing was written. Applying any section below is a human or agent decision.

(a) DELIVERY DRAFT — save as docs/knowledge/work/no-midslice-ask/delivery.md

---
type: bee.delivery
title: no-midslice-ask — delivery
description: "Delivery record proposed by bee knowledge promote for work item no-midslice-ask: 1 capped cell(s), 1 recorded deviation(s)."
timestamp: 2026-08-26
bee:
  id: no-midslice-ask-delivery
  lifecycle: active
  required_context: [.bee/lanes/no-midslice-ask.json]
  sources: [.bee/lanes/no-midslice-ask.json, .bee/cells/nma-1.json]
---

# no-midslice-ask — Delivery

## What shipped

- **nma-1** — Always-loaded layer now forbids mid-work continue-asks; slice boundary continues in the same turn (0 file(s) changed)

## Verify

Each cell below was capped only against a recorded passing verify result — bee refuses a cap without one.

- **nma-1** — `bee dev regen green; bee dev release-manifest --check clean; rg 'in the same turn' AGENTS.md skills/bee-swarming/SKILL.md packages/bee/AGENTS.block.md confirms the promoted rule in all three`

## Deviations

- **nma-1** — sync-ack: AGENTS.md touched only inside the agents-one-next-action rule body; the flagged rule agents-capture-line-at-close and its applied_at files are untouched by this diff

## Provenance

Proposed by `bee knowledge promote --work no-midslice-ask` from 1 capped cell trace(s) in `.bee/cells/` and the anchor `.bee/lanes/no-midslice-ask.json`. Every line above is copied from a trace or from the work item; nothing here is curated truth until a human or agent accepts it.

(b) AREA UPDATES — candidate spec-sync bullets, each citing its cell

None: the work item declares no bee.areas, so there is no area to sync (D19).

(c) PATTERN CANDIDATES — candidate bee.pattern concepts, bee.polarity pitfall

from cell nma-1 — save as docs/knowledge/patterns/no-midslice-ask-nma-1-pitfall.md

---
type: bee.pattern
title: no-midslice-ask cell nma-1 — pitfall candidate
description: "Pitfall candidate mined from cell nma-1's capped trace: sync-ack: AGENTS.md touched only inside the agents-one-next-action rule body; the flagged rule agents-capture-line-at-close and its applied_at files are untouc…"
timestamp: 2026-08-26
bee:
  id: no-midslice-ask-nma-1-pitfall
  lifecycle: draft
  sources: [.bee/cells/nma-1.json]
  polarity: pitfall
---

# no-midslice-ask cell nma-1 — pitfall candidate

## What the cell did

Always-loaded layer now forbids mid-work continue-asks; slice boundary continues in the same turn

## Recorded evidence (verbatim from .bee/cells/nma-1.json)

- **deviation** — sync-ack: AGENTS.md touched only inside the agents-one-next-action rule body; the flagged rule agents-capture-line-at-close and its applied_at files are untouched by this diff

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

knowledge promote: 1 capped cell(s) mined, 1 delivery draft, 0 area bullet(s), 1 pattern candidate(s), 0 file(s) written.