promote proposal for work item "release-2-47-0" (docs/history/release-2-47-0/CONTEXT.md + docs/history/release-2-47-0/plan.md) — 1 capped cell(s): rel470-1
anchor: history — docs/history/release-2-47-0/CONTEXT.md, docs/history/release-2-47-0/plan.md
PROPOSAL ONLY — nothing was written. Applying any section below is a human or agent decision.

(a) DELIVERY DRAFT — save as docs/knowledge/work/release-2-47-0/delivery.md

---
type: bee.delivery
title: release-2-47-0 — delivery
description: "Delivery record proposed by bee knowledge promote for work item release-2-47-0: 1 capped cell(s), 1 recorded deviation(s)."
timestamp: 2026-10-02
bee:
  id: release-2-47-0-delivery
  lifecycle: active
  required_context: [docs/history/release-2-47-0/CONTEXT.md, docs/history/release-2-47-0/plan.md]
  sources: [docs/history/release-2-47-0/CONTEXT.md, docs/history/release-2-47-0/plan.md, .bee/cells/rel470-1.json]
---

# release-2-47-0 — Delivery

## What shipped

- **rel470-1** — Publish release 2.47.0 through the sanctioned script (0 file(s) changed)

## Verify

Each cell below was capped only against a recorded passing verify result — bee refuses a cap without one.

- **rel470-1** — `bash scripts/release.sh 2.47.0 && bee dev release-manifest --check` — release.sh ran the full declared suite (37 test binaries ok, 0 FAILED), tagged v2.47.0, release-binaries https://github.com/thanhsmind/beehive/actions/runs/37023471297 green, 5 binaries + SHA256SUMS …

## Deviations

- **rel470-1** — the claude deploy role returns a read-only gather payload that cannot run the script, so the leader ran release.sh itself under dispatch 37fd7e0f

## Provenance

Proposed by `bee knowledge promote --work release-2-47-0` from 1 capped cell trace(s) in `.bee/cells/` and the anchor `docs/history/release-2-47-0/CONTEXT.md`, `docs/history/release-2-47-0/plan.md`. Every line above is copied from a trace or from the work item; nothing here is curated truth until a human or agent accepts it.

(b) AREA UPDATES — candidate spec-sync bullets, each citing its cell

None: the work item declares no bee.areas, so there is no area to sync (D19).

(c) PATTERN CANDIDATES — candidate bee.pattern concepts, bee.polarity pitfall

from cell rel470-1 — save as docs/knowledge/patterns/release-2-47-0-rel470-1-pitfall.md

---
type: bee.pattern
title: release-2-47-0 cell rel470-1 — pitfall candidate
description: "Pitfall candidate mined from cell rel470-1's capped trace: the claude deploy role returns a read-only gather payload that cannot run the script, so the leader ran release.sh itself under dispatch 37fd7e0f"
timestamp: 2026-10-02
bee:
  id: release-2-47-0-rel470-1-pitfall
  lifecycle: draft
  sources: [.bee/cells/rel470-1.json]
  polarity: pitfall
---

# release-2-47-0 cell rel470-1 — pitfall candidate

## What the cell did

Publish release 2.47.0 through the sanctioned script

## Recorded evidence (verbatim from .bee/cells/rel470-1.json)

- **deviation** — the claude deploy role returns a read-only gather payload that cannot run the script, so the leader ran release.sh itself under dispatch 37fd7e0f

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

knowledge promote: 1 capped cell(s) mined, 1 delivery draft, 0 area bullet(s), 1 pattern candidate(s), 0 file(s) written.