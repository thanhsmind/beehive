promote proposal for work item "two-flows" (docs/history/two-flows/CONTEXT.md) — 2 capped cell(s): tf-1, tf-2
anchor: history — docs/history/two-flows/CONTEXT.md
PROPOSAL ONLY — nothing was written. Applying any section below is a human or agent decision.

(a) DELIVERY DRAFT — save as docs/knowledge/work/two-flows/delivery.md

---
type: bee.delivery
title: two-flows — delivery
description: "Delivery record proposed by bee knowledge promote for work item two-flows: 2 capped cell(s), 0 recorded deviation(s)."
timestamp: 2026-08-18
bee:
  id: two-flows-delivery
  lifecycle: active
  required_context: [docs/history/two-flows/CONTEXT.md]
  sources: [docs/history/two-flows/CONTEXT.md, .bee/cells/tf-1.json, .bee/cells/tf-2.json]
---

# two-flows — Delivery

## What shipped

- **tf-1** — Named Main/Discovery flows in bee-hive's Route table, AGENTS.md (via its block source), and README's role table + workflow prose; regen chain green (5 file(s) changed)
- **tf-2** — Discovery flow made whole: probed and wrote the reservation-backed claim guard, LOGIC-page spike recipe, five-variant cap, and primary-source rule (5 file(s) changed)

## Verify

Each cell below was capped only against a recorded passing verify result — bee refuses a cap without one.

- **tf-1** — `.bee/bin/bee dev release-manifest --check`
- **tf-2** — `.bee/bin/bee dev release-manifest --check`

## Deviations

None recorded in the capped cell traces.

## Provenance

Proposed by `bee knowledge promote --work two-flows` from 2 capped cell trace(s) in `.bee/cells/` and the anchor `docs/history/two-flows/CONTEXT.md`. Every line above is copied from a trace or from the work item; nothing here is curated truth until a human or agent accepts it.

(b) AREA UPDATES — candidate spec-sync bullets, each citing its cell

None: the work item declares no bee.areas, so there is no area to sync (D19).

(c) PATTERN CANDIDATES — candidate bee.pattern concepts, bee.polarity pitfall

None: no capped cell trace carries a deviation or a failure signature.

knowledge promote: 2 capped cell(s) mined, 1 delivery draft, 0 area bullet(s), 0 pattern candidate(s), 0 file(s) written.