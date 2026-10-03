promote proposal for work item "release-red-2-49-0" (docs/history/release-red-2-49-0/plan.md) — 1 capped cell(s): rr49-1
anchor: history — docs/history/release-red-2-49-0/plan.md
PROPOSAL ONLY — nothing was written. Applying any section below is a human or agent decision.

(a) DELIVERY DRAFT — save as docs/knowledge/work/release-red-2-49-0/delivery.md

---
type: bee.delivery
title: release-red-2-49-0 — delivery
description: "Delivery record proposed by bee knowledge promote for work item release-red-2-49-0: 1 capped cell(s), 2 recorded deviation(s)."
timestamp: 2026-10-03
bee:
  id: release-red-2-49-0-delivery
  lifecycle: active
  required_context: [docs/history/release-red-2-49-0/plan.md]
  sources: [docs/history/release-red-2-49-0/plan.md, .bee/cells/rr49-1.json]
---

# release-red-2-49-0 — Delivery

## What shipped

- **rr49-1** — Five closed-plan prose spans join KNOWN_HISTORICAL_EXCEPTIONS; the shipped-spelling test is green (1 file(s) changed)

## Verify

Each cell below was capped only against a recorded passing verify result — bee refuses a cap without one.

- **rr49-1** — `cd packages/bee-rs && cargo test --release -p bee --bin bee cli_shape` — 39 passed including no_shipped_command_spelling_is_refused_by_the_widened_guard, red in the 2.49.0 release suite before

## Deviations

- **rr49-1** — followed the plan
- **rr49-1** — sync-ack: test-only pin list; no owned skill quotes these spans

## Provenance

Proposed by `bee knowledge promote --work release-red-2-49-0` from 1 capped cell trace(s) in `.bee/cells/` and the anchor `docs/history/release-red-2-49-0/plan.md`. Every line above is copied from a trace or from the work item; nothing here is curated truth until a human or agent accepts it.

(b) AREA UPDATES — candidate spec-sync bullets, each citing its cell

None: the work item declares no bee.areas, so there is no area to sync (D19).

(c) PATTERN CANDIDATES — candidate bee.pattern concepts, bee.polarity pitfall

from cell rr49-1 — save as docs/knowledge/patterns/release-red-2-49-0-rr49-1-pitfall.md

---
type: bee.pattern
title: release-red-2-49-0 cell rr49-1 — pitfall candidate
description: "Pitfall candidate mined from cell rr49-1's capped trace: followed the plan"
timestamp: 2026-10-03
bee:
  id: release-red-2-49-0-rr49-1-pitfall
  lifecycle: draft
  sources: [.bee/cells/rr49-1.json]
  polarity: pitfall
---

# release-red-2-49-0 cell rr49-1 — pitfall candidate

## What the cell did

Five closed-plan prose spans join KNOWN_HISTORICAL_EXCEPTIONS; the shipped-spelling test is green

## Recorded evidence (verbatim from .bee/cells/rr49-1.json)

- **deviation** — followed the plan
- **deviation** — sync-ack: test-only pin list; no owned skill quotes these spans

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

knowledge promote: 1 capped cell(s) mined, 1 delivery draft, 0 area bullet(s), 1 pattern candidate(s), 0 file(s) written.