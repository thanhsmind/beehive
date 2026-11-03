promote proposal for work item "release-pointer-fix" (.bee/lanes/release-pointer-fix.json) — 1 capped cell(s): rpf-1
anchor: ledger — .bee/lanes/release-pointer-fix.json
PROPOSAL ONLY — nothing was written. Applying any section below is a human or agent decision.

(a) DELIVERY DRAFT — save as docs/knowledge/work/release-pointer-fix/delivery.md

---
type: bee.delivery
title: release-pointer-fix — delivery
description: "Delivery record proposed by bee knowledge promote for work item release-pointer-fix: 1 capped cell(s), 1 recorded deviation(s)."
timestamp: 2026-09-06
bee:
  id: release-pointer-fix-delivery
  lifecycle: active
  required_context: [.bee/lanes/release-pointer-fix.json]
  sources: [.bee/lanes/release-pointer-fix.json, .bee/cells/rpf-1.json]
---

# release-pointer-fix — Delivery

## What shipped

- **rpf-1** — Cross-skill pointer now bee-hive/references/routing-and-contracts.md; overview.md links are bare paths; regen green (4 file(s) changed)

## Verify

Each cell below was capped only against a recorded passing verify result — bee refuses a cap without one.

- **rpf-1** — `bee dev release-manifest --check — green:static — regenerated skill copies and manifest match`

## Deviations

- **rpf-1** — sync-ack: skills/bee-swarming/SKILL.md changed only in pointer form; regen copies refreshed

## Provenance

Proposed by `bee knowledge promote --work release-pointer-fix` from 1 capped cell trace(s) in `.bee/cells/` and the anchor `.bee/lanes/release-pointer-fix.json`. Every line above is copied from a trace or from the work item; nothing here is curated truth until a human or agent accepts it.

(b) AREA UPDATES — candidate spec-sync bullets, each citing its cell

None: the work item declares no bee.areas, so there is no area to sync (D19).

(c) PATTERN CANDIDATES — candidate bee.pattern concepts, bee.polarity pitfall

from cell rpf-1 — save as docs/knowledge/patterns/release-pointer-fix-rpf-1-pitfall.md

---
type: bee.pattern
title: release-pointer-fix cell rpf-1 — pitfall candidate
description: "Pitfall candidate mined from cell rpf-1's capped trace: sync-ack: skills/bee-swarming/SKILL.md changed only in pointer form; regen copies refreshed"
timestamp: 2026-09-06
bee:
  id: release-pointer-fix-rpf-1-pitfall
  lifecycle: draft
  sources: [.bee/cells/rpf-1.json]
  polarity: pitfall
---

# release-pointer-fix cell rpf-1 — pitfall candidate

## What the cell did

Cross-skill pointer now bee-hive/references/routing-and-contracts.md; overview.md links are bare paths; regen green

## Recorded evidence (verbatim from .bee/cells/rpf-1.json)

- **deviation** — sync-ack: skills/bee-swarming/SKILL.md changed only in pointer form; regen copies refreshed

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

knowledge promote: 1 capped cell(s) mined, 1 delivery draft, 0 area bullet(s), 1 pattern candidate(s), 0 file(s) written.