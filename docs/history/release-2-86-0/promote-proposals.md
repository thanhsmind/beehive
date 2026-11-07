promote proposal for work item "release-2-86-0" (docs/history/release-2-86-0/CONTEXT.md + docs/history/release-2-86-0/plan.md) — 1 capped cell(s): rel860-1
anchor: history — docs/history/release-2-86-0/CONTEXT.md, docs/history/release-2-86-0/plan.md
PROPOSAL ONLY — nothing was written. Applying any section below is a human or agent decision.

(a) DELIVERY DRAFT — save as docs/knowledge/work/release-2-86-0/delivery.md

---
type: bee.delivery
title: release-2-86-0 — delivery
description: "Delivery record proposed by bee knowledge promote for work item release-2-86-0: 1 capped cell(s), 1 recorded deviation(s)."
timestamp: 2026-11-07
bee:
  id: release-2-86-0-delivery
  lifecycle: active
  required_context: [docs/history/release-2-86-0/CONTEXT.md, docs/history/release-2-86-0/plan.md]
  sources: [docs/history/release-2-86-0/CONTEXT.md, docs/history/release-2-86-0/plan.md, .bee/cells/rel860-1.json]
---

# release-2-86-0 — Delivery

## What shipped

- **rel860-1** — Published 2.86.0 through scripts/release.sh under permit e31550b2 (3 file(s) changed)

## Verify

Each cell below was capped only against a recorded passing verify result — bee refuses a cap without one.

- **rel860-1** — `bash scripts/release.sh 2.86.0 && bee dev release-manifest --check` — suite ran before tagging, v2.86.0 pushed, release-binaries run https://github.com/thanhsmind/beehive/actions/runs/42360182845 green, assets verified, final OK line printed, manifest matches

## Deviations

- **rel860-1** — first two runs refused before any change: dirty tree, then a permit bound to an older main commit; a fresh permit e31550b2 on current main

## Provenance

Proposed by `bee knowledge promote --work release-2-86-0` from 1 capped cell trace(s) in `.bee/cells/` and the anchor `docs/history/release-2-86-0/CONTEXT.md`, `docs/history/release-2-86-0/plan.md`. Every line above is copied from a trace or from the work item; nothing here is curated truth until a human or agent accepts it.

(b) AREA UPDATES — candidate spec-sync bullets, each citing its cell

None: the work item declares no bee.areas, so there is no area to sync (D19).

(c) PATTERN CANDIDATES — candidate bee.pattern concepts, bee.polarity pitfall

from cell rel860-1 — save as docs/knowledge/patterns/release-2-86-0-rel860-1-pitfall.md

---
type: bee.pattern
title: release-2-86-0 cell rel860-1 — pitfall candidate
description: "Pitfall candidate mined from cell rel860-1's capped trace: first two runs refused before any change: dirty tree, then a permit bound to an older main commit; a fresh permit e31550b2 on current main"
timestamp: 2026-11-07
bee:
  id: release-2-86-0-rel860-1-pitfall
  lifecycle: draft
  sources: [.bee/cells/rel860-1.json]
  polarity: pitfall
---

# release-2-86-0 cell rel860-1 — pitfall candidate

## What the cell did

Published 2.86.0 through scripts/release.sh under permit e31550b2

## Recorded evidence (verbatim from .bee/cells/rel860-1.json)

- **deviation** — first two runs refused before any change: dirty tree, then a permit bound to an older main commit; a fresh permit e31550b2 on current main

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

knowledge promote: 1 capped cell(s) mined, 1 delivery draft, 0 area bullet(s), 1 pattern candidate(s), 0 file(s) written.