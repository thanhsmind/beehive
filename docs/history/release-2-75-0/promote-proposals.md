promote proposal for work item "release-2-75-0" (docs/history/release-2-75-0/CONTEXT.md + docs/history/release-2-75-0/plan.md) — 1 capped cell(s): rel750-1
anchor: history — docs/history/release-2-75-0/CONTEXT.md, docs/history/release-2-75-0/plan.md
PROPOSAL ONLY — nothing was written. Applying any section below is a human or agent decision.

(a) DELIVERY DRAFT — save as docs/knowledge/work/release-2-75-0/delivery.md

---
type: bee.delivery
title: release-2-75-0 — delivery
description: "Delivery record proposed by bee knowledge promote for work item release-2-75-0: 1 capped cell(s), 1 recorded deviation(s)."
timestamp: 2026-11-05
bee:
  id: release-2-75-0-delivery
  lifecycle: active
  required_context: [docs/history/release-2-75-0/CONTEXT.md, docs/history/release-2-75-0/plan.md]
  sources: [docs/history/release-2-75-0/CONTEXT.md, docs/history/release-2-75-0/plan.md, .bee/cells/rel750-1.json]
---

# release-2-75-0 — Delivery

## What shipped

- **rel750-1** — bee 2.75.0 is live (3 file(s) changed)

## Verify

Each cell below was capped only against a recorded passing verify result — bee refuses a cap without one.

- **rel750-1** — `bash scripts/release.sh 2.75.0 && bee dev release-manifest --check` — script printed release OK bee 2.75.0 is live after its full test gate; manifest check 600 files match; CI https://github.com/thanhsmind/beehive/actions/runs/41985682555

## Deviations

- **rel750-1** — ran the suite with CI-pinned opencode-ai@1.18.16 on PATH instead of the user-installed opencode 2.2.5, because the registry derivation cannot read 2.2.5; backlog row filed

## Provenance

Proposed by `bee knowledge promote --work release-2-75-0` from 1 capped cell trace(s) in `.bee/cells/` and the anchor `docs/history/release-2-75-0/CONTEXT.md`, `docs/history/release-2-75-0/plan.md`. Every line above is copied from a trace or from the work item; nothing here is curated truth until a human or agent accepts it.

(b) AREA UPDATES — candidate spec-sync bullets, each citing its cell

None: the work item declares no bee.areas, so there is no area to sync (D19).

(c) PATTERN CANDIDATES — candidate bee.pattern concepts, bee.polarity pitfall

from cell rel750-1 — save as docs/knowledge/patterns/release-2-75-0-rel750-1-pitfall.md

---
type: bee.pattern
title: release-2-75-0 cell rel750-1 — pitfall candidate
description: "Pitfall candidate mined from cell rel750-1's capped trace: ran the suite with CI-pinned opencode-ai@1.18.16 on PATH instead of the user-installed opencode 2.2.5, because the registry derivation cannot read 2.2.5; backl…"
timestamp: 2026-11-05
bee:
  id: release-2-75-0-rel750-1-pitfall
  lifecycle: draft
  sources: [.bee/cells/rel750-1.json]
  polarity: pitfall
---

# release-2-75-0 cell rel750-1 — pitfall candidate

## What the cell did

bee 2.75.0 is live

## Recorded evidence (verbatim from .bee/cells/rel750-1.json)

- **deviation** — ran the suite with CI-pinned opencode-ai@1.18.16 on PATH instead of the user-installed opencode 2.2.5, because the registry derivation cannot read 2.2.5; backlog row filed

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

knowledge promote: 1 capped cell(s) mined, 1 delivery draft, 0 area bullet(s), 1 pattern candidate(s), 0 file(s) written.