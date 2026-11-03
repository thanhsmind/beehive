promote proposal for work item "release-2-66-0" (docs/history/release-2-66-0/CONTEXT.md + docs/history/release-2-66-0/plan.md) — 1 capped cell(s): rel660-1
anchor: history — docs/history/release-2-66-0/CONTEXT.md, docs/history/release-2-66-0/plan.md
PROPOSAL ONLY — nothing was written. Applying any section below is a human or agent decision.

(a) DELIVERY DRAFT — save as docs/knowledge/work/release-2-66-0/delivery.md

---
type: bee.delivery
title: release-2-66-0 — delivery
description: "Delivery record proposed by bee knowledge promote for work item release-2-66-0: 1 capped cell(s), 1 recorded deviation(s)."
timestamp: 2026-10-29
bee:
  id: release-2-66-0-delivery
  lifecycle: active
  required_context: [docs/history/release-2-66-0/CONTEXT.md, docs/history/release-2-66-0/plan.md]
  sources: [docs/history/release-2-66-0/CONTEXT.md, docs/history/release-2-66-0/plan.md, .bee/cells/rel660-1.json]
---

# release-2-66-0 — Delivery

## What shipped

- **rel660-1** — bee 2.66.0 is live: tag pushed, release-binaries green, release carries 5 binaries and SHA256SUMS (3 file(s) changed)

## Verify

Each cell below was capped only against a recorded passing verify result — bee refuses a cap without one.

- **rel660-1** — `bash scripts/release.sh 2.66.0 && bee dev release-manifest --check` — script printed OK after the declared cargo suite (40 test binaries ok, opencode suite allowed to skip as in 2.60.0-2.65.0), tag push, green release-binaries run https://github.com/thanhsmind/beehive/…

## Deviations

- **rel660-1** — release.sh ran inline in the leader under permit d26d5e5d, as releases 2.58.0-2.65.0 did

## Provenance

Proposed by `bee knowledge promote --work release-2-66-0` from 1 capped cell trace(s) in `.bee/cells/` and the anchor `docs/history/release-2-66-0/CONTEXT.md`, `docs/history/release-2-66-0/plan.md`. Every line above is copied from a trace or from the work item; nothing here is curated truth until a human or agent accepts it.

(b) AREA UPDATES — candidate spec-sync bullets, each citing its cell

None: the work item declares no bee.areas, so there is no area to sync (D19).

(c) PATTERN CANDIDATES — candidate bee.pattern concepts, bee.polarity pitfall

from cell rel660-1 — save as docs/knowledge/patterns/release-2-66-0-rel660-1-pitfall.md

---
type: bee.pattern
title: release-2-66-0 cell rel660-1 — pitfall candidate
description: "Pitfall candidate mined from cell rel660-1's capped trace: release.sh ran inline in the leader under permit d26d5e5d, as releases 2.58.0-2.65.0 did"
timestamp: 2026-10-29
bee:
  id: release-2-66-0-rel660-1-pitfall
  lifecycle: draft
  sources: [.bee/cells/rel660-1.json]
  polarity: pitfall
---

# release-2-66-0 cell rel660-1 — pitfall candidate

## What the cell did

bee 2.66.0 is live: tag pushed, release-binaries green, release carries 5 binaries and SHA256SUMS

## Recorded evidence (verbatim from .bee/cells/rel660-1.json)

- **deviation** — release.sh ran inline in the leader under permit d26d5e5d, as releases 2.58.0-2.65.0 did

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

knowledge promote: 1 capped cell(s) mined, 1 delivery draft, 0 area bullet(s), 1 pattern candidate(s), 0 file(s) written.