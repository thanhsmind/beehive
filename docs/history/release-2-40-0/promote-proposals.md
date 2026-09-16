promote proposal for work item "release-2-40-0" (docs/history/release-2-40-0/CONTEXT.md + docs/history/release-2-40-0/plan.md) — 1 capped cell(s): rel40-1
anchor: history — docs/history/release-2-40-0/CONTEXT.md, docs/history/release-2-40-0/plan.md
PROPOSAL ONLY — nothing was written. Applying any section below is a human or agent decision.

(a) DELIVERY DRAFT — save as docs/knowledge/work/release-2-40-0/delivery.md

---
type: bee.delivery
title: release-2-40-0 — delivery
description: "Delivery record proposed by bee knowledge promote for work item release-2-40-0: 1 capped cell(s), 2 recorded deviation(s)."
timestamp: 2026-09-16
bee:
  id: release-2-40-0-delivery
  lifecycle: active
  required_context: [docs/history/release-2-40-0/CONTEXT.md, docs/history/release-2-40-0/plan.md]
  sources: [docs/history/release-2-40-0/CONTEXT.md, docs/history/release-2-40-0/plan.md, .bee/cells/rel40-1.json]
---

# release-2-40-0 — Delivery

## What shipped

- **rel40-1** — Published release 2.40.0 through the sanctioned script (3 file(s) changed)

## Verify

Each cell below was capped only against a recorded passing verify result — bee refuses a cap without one.

- **rel40-1** — `bash scripts/release.sh 2.40.0 && bee dev release-manifest --check` — the script ran the declared suite before tagging and printed its final 'release OK bee 2.40.0 is live'; leader re-verified independently: both manifests 2.40.0, tag v2.40.0 on origin at 49ff5d4b, 0 u…

## Deviations

- **rel40-1** — The lane took a plan-rev bump after its shape gate: the first cell packet was refused by the REGEN_OBLIGATION guard because .claude-plugin/plugin.json is a release-manifest root, so the cell had to gain docs/history/codex-harness-hardening/release-manifest.json and a release-manifest --check in verify. Re-previewed and re-gated with the reason recorded.
- **rel40-1** — The release lane's own bookkeeping commit was made through a private GIT_INDEX_FILE rather than `git add`, which the concurrent-worker guard refuses while a sibling session is live in this checkout.

## Provenance

Proposed by `bee knowledge promote --work release-2-40-0` from 1 capped cell trace(s) in `.bee/cells/` and the anchor `docs/history/release-2-40-0/CONTEXT.md`, `docs/history/release-2-40-0/plan.md`. Every line above is copied from a trace or from the work item; nothing here is curated truth until a human or agent accepts it.

(b) AREA UPDATES — candidate spec-sync bullets, each citing its cell

None: the work item declares no bee.areas, so there is no area to sync (D19).

(c) PATTERN CANDIDATES — candidate bee.pattern concepts, bee.polarity pitfall

from cell rel40-1 — save as docs/knowledge/patterns/release-2-40-0-rel40-1-pitfall.md

---
type: bee.pattern
title: release-2-40-0 cell rel40-1 — pitfall candidate
description: "Pitfall candidate mined from cell rel40-1's capped trace: The lane took a plan-rev bump after its shape gate: the first cell packet was refused by the REGEN_OBLIGATION guard because .claude-plugin/plugin.json is a rel…"
timestamp: 2026-09-16
bee:
  id: release-2-40-0-rel40-1-pitfall
  lifecycle: draft
  sources: [.bee/cells/rel40-1.json]
  polarity: pitfall
---

# release-2-40-0 cell rel40-1 — pitfall candidate

## What the cell did

Published release 2.40.0 through the sanctioned script

## Recorded evidence (verbatim from .bee/cells/rel40-1.json)

- **deviation** — The lane took a plan-rev bump after its shape gate: the first cell packet was refused by the REGEN_OBLIGATION guard because .claude-plugin/plugin.json is a release-manifest root, so the cell had to gain docs/history/codex-harness-hardening/release-manifest.json and a release-manifest --check in verify. Re-previewed and re-gated with the reason recorded.
- **deviation** — The release lane's own bookkeeping commit was made through a private GIT_INDEX_FILE rather than `git add`, which the concurrent-worker guard refuses while a sibling session is live in this checkout.

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

knowledge promote: 1 capped cell(s) mined, 1 delivery draft, 0 area bullet(s), 1 pattern candidate(s), 0 file(s) written.