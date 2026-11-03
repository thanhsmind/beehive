promote proposal for work item "release-2-71-0" (docs/history/release-2-71-0/CONTEXT.md + docs/history/release-2-71-0/plan.md) — 1 capped cell(s): rel710-1
anchor: history — docs/history/release-2-71-0/CONTEXT.md, docs/history/release-2-71-0/plan.md
PROPOSAL ONLY — nothing was written. Applying any section below is a human or agent decision.

(a) DELIVERY DRAFT — save as docs/knowledge/work/release-2-71-0/delivery.md

---
type: bee.delivery
title: release-2-71-0 — delivery
description: "Delivery record proposed by bee knowledge promote for work item release-2-71-0: 1 capped cell(s), 1 recorded deviation(s)."
timestamp: 2026-11-03
bee:
  id: release-2-71-0-delivery
  lifecycle: active
  required_context: [docs/history/release-2-71-0/CONTEXT.md, docs/history/release-2-71-0/plan.md]
  sources: [docs/history/release-2-71-0/CONTEXT.md, docs/history/release-2-71-0/plan.md, .bee/cells/rel710-1.json]
---

# release-2-71-0 — Delivery

## What shipped

- **rel710-1** — Released 2.71.0: tag pushed, release-binaries green, assets published (0 file(s) changed)

## Verify

Each cell below was capped only against a recorded passing verify result — bee refuses a cap without one.

- **rel710-1** — `bash scripts/release.sh 2.71.0 && bee dev release-manifest --check` — full declared suite ran before tagging; CI https://github.com/thanhsmind/beehive/actions/runs/41578503377 green; release https://github.com/thanhsmind/beehive/releases/tag/v2.71.0 carries 5 binaries …

## Deviations

- **rel710-1** — the leader ran the script inline under deploy dispatch e0d32069 instead of a herding worker, because a herded worker may not push; the first permit 450cee7a carried no release version and was refused before any change

## Provenance

Proposed by `bee knowledge promote --work release-2-71-0` from 1 capped cell trace(s) in `.bee/cells/` and the anchor `docs/history/release-2-71-0/CONTEXT.md`, `docs/history/release-2-71-0/plan.md`. Every line above is copied from a trace or from the work item; nothing here is curated truth until a human or agent accepts it.

(b) AREA UPDATES — candidate spec-sync bullets, each citing its cell

None: the work item declares no bee.areas, so there is no area to sync (D19).

(c) PATTERN CANDIDATES — candidate bee.pattern concepts, bee.polarity pitfall

from cell rel710-1 — save as docs/knowledge/patterns/release-2-71-0-rel710-1-pitfall.md

---
type: bee.pattern
title: release-2-71-0 cell rel710-1 — pitfall candidate
description: "Pitfall candidate mined from cell rel710-1's capped trace: the leader ran the script inline under deploy dispatch e0d32069 instead of a herding worker, because a herded worker may not push; the first permit 450cee7a ca…"
timestamp: 2026-11-03
bee:
  id: release-2-71-0-rel710-1-pitfall
  lifecycle: draft
  sources: [.bee/cells/rel710-1.json]
  polarity: pitfall
---

# release-2-71-0 cell rel710-1 — pitfall candidate

## What the cell did

Released 2.71.0: tag pushed, release-binaries green, assets published

## Recorded evidence (verbatim from .bee/cells/rel710-1.json)

- **deviation** — the leader ran the script inline under deploy dispatch e0d32069 instead of a herding worker, because a herded worker may not push; the first permit 450cee7a carried no release version and was refused before any change

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

knowledge promote: 1 capped cell(s) mined, 1 delivery draft, 0 area bullet(s), 1 pattern candidate(s), 0 file(s) written.