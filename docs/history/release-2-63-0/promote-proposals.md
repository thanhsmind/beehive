promote proposal for work item "release-2-63-0" (docs/history/release-2-63-0/CONTEXT.md + docs/history/release-2-63-0/plan.md) — 1 capped cell(s): rel630-1
anchor: history — docs/history/release-2-63-0/CONTEXT.md, docs/history/release-2-63-0/plan.md
PROPOSAL ONLY — nothing was written. Applying any section below is a human or agent decision.

(a) DELIVERY DRAFT — save as docs/knowledge/work/release-2-63-0/delivery.md

---
type: bee.delivery
title: release-2-63-0 — delivery
description: "Delivery record proposed by bee knowledge promote for work item release-2-63-0: 1 capped cell(s), 1 recorded deviation(s)."
timestamp: 2026-10-27
bee:
  id: release-2-63-0-delivery
  lifecycle: active
  required_context: [docs/history/release-2-63-0/CONTEXT.md, docs/history/release-2-63-0/plan.md]
  sources: [docs/history/release-2-63-0/CONTEXT.md, docs/history/release-2-63-0/plan.md, .bee/cells/rel630-1.json]
---

# release-2-63-0 — Delivery

## What shipped

- **rel630-1** — Publish release 2.63.0 through the sanctioned script (0 file(s) changed)

## Verify

Each cell below was capped only against a recorded passing verify result — bee refuses a cap without one.

- **rel630-1** — `bash scripts/release.sh 2.63.0 && bee dev release-manifest --check` — release cell, evidence https://github.com/thanhsmind/beehive/actions/runs/40469699223 and /mnt/data/Projects/goglbe/beehive/.bee/mailbox/job-1793087259677-3516293-1/report-1.md

## Deviations

- **rel630-1** — BEE_OPENCODE_SUITE_ALLOW_SKIP=1 added to the deploy dispatch env, as in releases 2.60.0 to 2.62.0, because the local opencode is a wrapper script

## Provenance

Proposed by `bee knowledge promote --work release-2-63-0` from 1 capped cell trace(s) in `.bee/cells/` and the anchor `docs/history/release-2-63-0/CONTEXT.md`, `docs/history/release-2-63-0/plan.md`. Every line above is copied from a trace or from the work item; nothing here is curated truth until a human or agent accepts it.

(b) AREA UPDATES — candidate spec-sync bullets, each citing its cell

None: the work item declares no bee.areas, so there is no area to sync (D19).

(c) PATTERN CANDIDATES — candidate bee.pattern concepts, bee.polarity pitfall

from cell rel630-1 — save as docs/knowledge/patterns/release-2-63-0-rel630-1-pitfall.md

---
type: bee.pattern
title: release-2-63-0 cell rel630-1 — pitfall candidate
description: "Pitfall candidate mined from cell rel630-1's capped trace: BEE_OPENCODE_SUITE_ALLOW_SKIP=1 added to the deploy dispatch env, as in releases 2.60.0 to 2.62.0, because the local opencode is a wrapper script"
timestamp: 2026-10-27
bee:
  id: release-2-63-0-rel630-1-pitfall
  lifecycle: draft
  sources: [.bee/cells/rel630-1.json]
  polarity: pitfall
---

# release-2-63-0 cell rel630-1 — pitfall candidate

## What the cell did

Publish release 2.63.0 through the sanctioned script

## Recorded evidence (verbatim from .bee/cells/rel630-1.json)

- **deviation** — BEE_OPENCODE_SUITE_ALLOW_SKIP=1 added to the deploy dispatch env, as in releases 2.60.0 to 2.62.0, because the local opencode is a wrapper script

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

knowledge promote: 1 capped cell(s) mined, 1 delivery draft, 0 area bullet(s), 1 pattern candidate(s), 0 file(s) written.