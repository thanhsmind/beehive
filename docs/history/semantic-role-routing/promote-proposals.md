promote proposal for work item "semantic-role-routing" (.bee/lanes/semantic-role-routing.json) — 1 capped cell(s): slr-1
anchor: ledger — .bee/lanes/semantic-role-routing.json
PROPOSAL ONLY — nothing was written. Applying any section below is a human or agent decision.

(a) DELIVERY DRAFT — save as docs/knowledge/work/semantic-role-routing/delivery.md

---
type: bee.delivery
title: semantic-role-routing — delivery
description: "Delivery record proposed by bee knowledge promote for work item semantic-role-routing: 1 capped cell(s), 2 recorded deviation(s)."
timestamp: 2026-09-14
bee:
  id: semantic-role-routing-delivery
  lifecycle: active
  required_context: [.bee/lanes/semantic-role-routing.json]
  sources: [.bee/lanes/semantic-role-routing.json, .bee/cells/slr-1.json]
---

# semantic-role-routing — Delivery

## What shipped

- **slr-1** — Planning SKILL.md updated with role assignment step; planning-reference has Role assignments section; AGENTS.md has dispatch-follows-plan rule; swarming-reference cites cell role as plan-approved assignment (4 file(s) changed)

## Verify

Each cell below was capped only against a recorded passing verify result — bee refuses a cap without one.

- **slr-1** — `grep -q 'role assignment' .agents/skills/bee-planning/SKILL.md && grep -q 're-route' AGENTS.md` — parity/pointer checks on 4 doc files

## Deviations

- **slr-1** — followed the plan
- **slr-1** — sync-ack: cell slr-1 touches rule home AGENTS.md only; applied_at files (bee-capturing, bee-hive, routing-and-contracts) not in scope

## Provenance

Proposed by `bee knowledge promote --work semantic-role-routing` from 1 capped cell trace(s) in `.bee/cells/` and the anchor `.bee/lanes/semantic-role-routing.json`. Every line above is copied from a trace or from the work item; nothing here is curated truth until a human or agent accepts it.

(b) AREA UPDATES — candidate spec-sync bullets, each citing its cell

None: the work item declares no bee.areas, so there is no area to sync (D19).

(c) PATTERN CANDIDATES — candidate bee.pattern concepts, bee.polarity pitfall

from cell slr-1 — save as docs/knowledge/patterns/semantic-role-routing-slr-1-pitfall.md

---
type: bee.pattern
title: semantic-role-routing cell slr-1 — pitfall candidate
description: "Pitfall candidate mined from cell slr-1's capped trace: followed the plan"
timestamp: 2026-09-14
bee:
  id: semantic-role-routing-slr-1-pitfall
  lifecycle: draft
  sources: [.bee/cells/slr-1.json]
  polarity: pitfall
---

# semantic-role-routing cell slr-1 — pitfall candidate

## What the cell did

Planning SKILL.md updated with role assignment step; planning-reference has Role assignments section; AGENTS.md has dispatch-follows-plan rule; swarming-reference cites cell role as plan-approved assignment

## Recorded evidence (verbatim from .bee/cells/slr-1.json)

- **deviation** — followed the plan
- **deviation** — sync-ack: cell slr-1 touches rule home AGENTS.md only; applied_at files (bee-capturing, bee-hive, routing-and-contracts) not in scope

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

knowledge promote: 1 capped cell(s) mined, 1 delivery draft, 0 area bullet(s), 1 pattern candidate(s), 0 file(s) written.