promote proposal for work item "deploy-role-agy-flash" (.bee/lanes/deploy-role-agy-flash.json) — 1 capped cell(s): dragy-1
anchor: ledger — .bee/lanes/deploy-role-agy-flash.json
PROPOSAL ONLY — nothing was written. Applying any section below is a human or agent decision.

(a) DELIVERY DRAFT — save as docs/knowledge/work/deploy-role-agy-flash/delivery.md

---
type: bee.delivery
title: deploy-role-agy-flash — delivery
description: "Delivery record proposed by bee knowledge promote for work item deploy-role-agy-flash: 1 capped cell(s), 1 recorded deviation(s)."
timestamp: 2026-09-14
bee:
  id: deploy-role-agy-flash-delivery
  lifecycle: active
  required_context: [.bee/lanes/deploy-role-agy-flash.json]
  sources: [.bee/lanes/deploy-role-agy-flash.json, .bee/cells/archive/deploy-role-agy-flash/dragy-1.json]
---

# deploy-role-agy-flash — Delivery

## What shipped

- **dragy-1** — Added team.pi.deploy slot in .bee/config.json routing release work through agy-flash (1 file(s) changed)

## Verify

Each cell below was capped only against a recorded passing verify result — bee refuses a cap without one.

- **dragy-1** — `.bee/bin/bee team show --runtime pi --json | jq -e 'any(.runtimes[0].roles[]; .role == "deploy" and .transport == "herding: agy-flash" and .description == "publish an approved release from main, wait for CI, and verify release assets")'` — verified team.pi.deploy configuration

## Deviations

- **dragy-1** — followed the plan

## Provenance

Proposed by `bee knowledge promote --work deploy-role-agy-flash` from 1 capped cell trace(s) in `.bee/cells/` and the anchor `.bee/lanes/deploy-role-agy-flash.json`. Every line above is copied from a trace or from the work item; nothing here is curated truth until a human or agent accepts it.

(b) AREA UPDATES — candidate spec-sync bullets, each citing its cell

None: the work item declares no bee.areas, so there is no area to sync (D19).

(c) PATTERN CANDIDATES — candidate bee.pattern concepts, bee.polarity pitfall

from cell dragy-1 — save as docs/knowledge/patterns/deploy-role-agy-flash-dragy-1-pitfall.md

---
type: bee.pattern
title: deploy-role-agy-flash cell dragy-1 — pitfall candidate
description: "Pitfall candidate mined from cell dragy-1's capped trace: followed the plan"
timestamp: 2026-09-14
bee:
  id: deploy-role-agy-flash-dragy-1-pitfall
  lifecycle: draft
  sources: [.bee/cells/archive/deploy-role-agy-flash/dragy-1.json]
  polarity: pitfall
---

# deploy-role-agy-flash cell dragy-1 — pitfall candidate

## What the cell did

Added team.pi.deploy slot in .bee/config.json routing release work through agy-flash

## Recorded evidence (verbatim from .bee/cells/archive/deploy-role-agy-flash/dragy-1.json)

- **deviation** — followed the plan

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

knowledge promote: 1 capped cell(s) mined, 1 delivery draft, 0 area bullet(s), 1 pattern candidate(s), 0 file(s) written.