promote proposal for work item "design-check-motion-frames" (.bee/lanes/design-check-motion-frames.json) — 1 capped cell(s): dcm-1
anchor: ledger — .bee/lanes/design-check-motion-frames.json
PROPOSAL ONLY — nothing was written. Applying any section below is a human or agent decision.

(a) DELIVERY DRAFT — save as docs/knowledge/work/design-check-motion-frames/delivery.md

---
type: bee.delivery
title: design-check-motion-frames — delivery
description: "Delivery record proposed by bee knowledge promote for work item design-check-motion-frames: 1 capped cell(s), 1 recorded deviation(s)."
timestamp: 2026-11-02
bee:
  id: design-check-motion-frames-delivery
  lifecycle: active
  required_context: [.bee/lanes/design-check-motion-frames.json]
  sources: [.bee/lanes/design-check-motion-frames.json, .bee/cells/dcm-1.json]
---

# design-check-motion-frames — Delivery

## What shipped

- **dcm-1** — bee-design Check step runs sheet and jumps on a moving page, hand-over gains MOTION (2 file(s) changed)

## Verify

Each cell below was capped only against a recorded passing verify result — bee refuses a cap without one.

- **dcm-1** — `rg -n "jumps.mjs" packages/bee/design/bee-design/SKILL.md && bee dev release-manifest --check` — docs-only change; rg hit line 108, manifest 593 files match

## Deviations

- **dcm-1** — followed the plan

## Provenance

Proposed by `bee knowledge promote --work design-check-motion-frames` from 1 capped cell trace(s) in `.bee/cells/` and the anchor `.bee/lanes/design-check-motion-frames.json`. Every line above is copied from a trace or from the work item; nothing here is curated truth until a human or agent accepts it.

(b) AREA UPDATES — candidate spec-sync bullets, each citing its cell

None: the work item declares no bee.areas, so there is no area to sync (D19).

(c) PATTERN CANDIDATES — candidate bee.pattern concepts, bee.polarity pitfall

from cell dcm-1 — save as docs/knowledge/patterns/design-check-motion-frames-dcm-1-pitfall.md

---
type: bee.pattern
title: design-check-motion-frames cell dcm-1 — pitfall candidate
description: "Pitfall candidate mined from cell dcm-1's capped trace: followed the plan"
timestamp: 2026-11-02
bee:
  id: design-check-motion-frames-dcm-1-pitfall
  lifecycle: draft
  sources: [.bee/cells/dcm-1.json]
  polarity: pitfall
---

# design-check-motion-frames cell dcm-1 — pitfall candidate

## What the cell did

bee-design Check step runs sheet and jumps on a moving page, hand-over gains MOTION

## Recorded evidence (verbatim from .bee/cells/dcm-1.json)

- **deviation** — followed the plan

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

knowledge promote: 1 capped cell(s) mined, 1 delivery draft, 0 area bullet(s), 1 pattern candidate(s), 0 file(s) written.