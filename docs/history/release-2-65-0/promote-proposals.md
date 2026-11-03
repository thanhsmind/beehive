promote proposal for work item "release-2-65-0" (docs/history/release-2-65-0/CONTEXT.md + docs/history/release-2-65-0/plan.md) — 1 capped cell(s): rel650-1
anchor: history — docs/history/release-2-65-0/CONTEXT.md, docs/history/release-2-65-0/plan.md
PROPOSAL ONLY — nothing was written. Applying any section below is a human or agent decision.

(a) DELIVERY DRAFT — save as docs/knowledge/work/release-2-65-0/delivery.md

---
type: bee.delivery
title: release-2-65-0 — delivery
description: "Delivery record proposed by bee knowledge promote for work item release-2-65-0: 1 capped cell(s), 1 recorded deviation(s)."
timestamp: 2026-10-28
bee:
  id: release-2-65-0-delivery
  lifecycle: active
  required_context: [docs/history/release-2-65-0/CONTEXT.md, docs/history/release-2-65-0/plan.md]
  sources: [docs/history/release-2-65-0/CONTEXT.md, docs/history/release-2-65-0/plan.md, .bee/cells/rel650-1.json]
---

# release-2-65-0 — Delivery

## What shipped

- **rel650-1** — Released 2.65.0 (3 file(s) changed)

## Verify

Each cell below was capped only against a recorded passing verify result — bee refuses a cap without one.

- **rel650-1** — `bash scripts/release.sh 2.65.0 && bee dev release-manifest --check` — script printed OK (test gate 4998 passed, 0 failed, opencode registry test skipped by BEE_OPENCODE_SUITE_ALLOW_SKIP=1 as for 2.60.0-2.64.0); release run https://github.com/thanhsmind/beehive/actions/…

## Deviations

- **rel650-1** — ran the gate with BEE_OPENCODE_SUITE_ALLOW_SKIP=1 — the local opencode is a mise wrapper the test cannot read; CI runs that suite — hit an unforeseen obstacle

## Provenance

Proposed by `bee knowledge promote --work release-2-65-0` from 1 capped cell trace(s) in `.bee/cells/` and the anchor `docs/history/release-2-65-0/CONTEXT.md`, `docs/history/release-2-65-0/plan.md`. Every line above is copied from a trace or from the work item; nothing here is curated truth until a human or agent accepts it.

(b) AREA UPDATES — candidate spec-sync bullets, each citing its cell

None: the work item declares no bee.areas, so there is no area to sync (D19).

(c) PATTERN CANDIDATES — candidate bee.pattern concepts, bee.polarity pitfall

from cell rel650-1 — save as docs/knowledge/patterns/release-2-65-0-rel650-1-pitfall.md

---
type: bee.pattern
title: release-2-65-0 cell rel650-1 — pitfall candidate
description: "Pitfall candidate mined from cell rel650-1's capped trace: ran the gate with BEE_OPENCODE_SUITE_ALLOW_SKIP=1 — the local opencode is a mise wrapper the test cannot read; CI runs that suite — hit an unforeseen obstacle"
timestamp: 2026-10-28
bee:
  id: release-2-65-0-rel650-1-pitfall
  lifecycle: draft
  sources: [.bee/cells/rel650-1.json]
  polarity: pitfall
---

# release-2-65-0 cell rel650-1 — pitfall candidate

## What the cell did

Released 2.65.0

## Recorded evidence (verbatim from .bee/cells/rel650-1.json)

- **deviation** — ran the gate with BEE_OPENCODE_SUITE_ALLOW_SKIP=1 — the local opencode is a mise wrapper the test cannot read; CI runs that suite — hit an unforeseen obstacle

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

knowledge promote: 1 capped cell(s) mined, 1 delivery draft, 0 area bullet(s), 1 pattern candidate(s), 0 file(s) written.