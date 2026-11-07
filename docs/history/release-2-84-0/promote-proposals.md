promote proposal for work item "release-2-84-0" (docs/history/release-2-84-0/CONTEXT.md + docs/history/release-2-84-0/plan.md) — 1 capped cell(s): rel840-1
anchor: history — docs/history/release-2-84-0/CONTEXT.md, docs/history/release-2-84-0/plan.md
PROPOSAL ONLY — nothing was written. Applying any section below is a human or agent decision.

(a) DELIVERY DRAFT — save as docs/knowledge/work/release-2-84-0/delivery.md

---
type: bee.delivery
title: release-2-84-0 — delivery
description: "Delivery record proposed by bee knowledge promote for work item release-2-84-0: 1 capped cell(s), 2 recorded deviation(s)."
timestamp: 2026-11-07
bee:
  id: release-2-84-0-delivery
  lifecycle: active
  required_context: [docs/history/release-2-84-0/CONTEXT.md, docs/history/release-2-84-0/plan.md]
  sources: [docs/history/release-2-84-0/CONTEXT.md, docs/history/release-2-84-0/plan.md, .bee/cells/archive/release-2-84-0/rel840-1.json]
---

# release-2-84-0 — Delivery

## What shipped

- **rel840-1** — 2.84.0 published: tag v2.84.0 on a9c1c3d11, release-binaries run 42346202194 green, 6 assets (3 file(s) changed)

## Verify

Each cell below was capped only against a recorded passing verify result — bee refuses a cap without one.

- **rel840-1** — `bash scripts/release.sh 2.84.0 && bee dev release-manifest --check` — the script ran the declared suite before tagging and printed release OK bee 2.84.0 is live; release-manifest --check 605 files match; assets at https://github.com/thanhsmind/beehive/releases/tag/v2.8…

## Deviations

- **rel840-1** — The herded deploy worker was refused deploy_authorization_wrong_session before any change; the leader ran scripts/release.sh inline in the issuing session under a fresh permit, the accepted path named by decision 50187668 and pattern-20261027
- **rel840-1** — PATH carried the CI-pinned opencode 1.18.16 dir ahead of the rest

## Provenance

Proposed by `bee knowledge promote --work release-2-84-0` from 1 capped cell trace(s) in `.bee/cells/` and the anchor `docs/history/release-2-84-0/CONTEXT.md`, `docs/history/release-2-84-0/plan.md`. Every line above is copied from a trace or from the work item; nothing here is curated truth until a human or agent accepts it.

(b) AREA UPDATES — candidate spec-sync bullets, each citing its cell

None: the work item declares no bee.areas, so there is no area to sync (D19).

(c) PATTERN CANDIDATES — candidate bee.pattern concepts, bee.polarity pitfall

from cell rel840-1 — save as docs/knowledge/patterns/release-2-84-0-rel840-1-pitfall.md

---
type: bee.pattern
title: release-2-84-0 cell rel840-1 — pitfall candidate
description: "Pitfall candidate mined from cell rel840-1's capped trace: The herded deploy worker was refused deploy_authorization_wrong_session before any change; the leader ran scripts/release.sh inline in the issuing session unde…"
timestamp: 2026-11-07
bee:
  id: release-2-84-0-rel840-1-pitfall
  lifecycle: draft
  sources: [.bee/cells/archive/release-2-84-0/rel840-1.json]
  polarity: pitfall
---

# release-2-84-0 cell rel840-1 — pitfall candidate

## What the cell did

2.84.0 published: tag v2.84.0 on a9c1c3d11, release-binaries run 42346202194 green, 6 assets

## Recorded evidence (verbatim from .bee/cells/archive/release-2-84-0/rel840-1.json)

- **deviation** — The herded deploy worker was refused deploy_authorization_wrong_session before any change; the leader ran scripts/release.sh inline in the issuing session under a fresh permit, the accepted path named by decision 50187668 and pattern-20261027
- **deviation** — PATH carried the CI-pinned opencode 1.18.16 dir ahead of the rest

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

knowledge promote: 1 capped cell(s) mined, 1 delivery draft, 0 area bullet(s), 1 pattern candidate(s), 0 file(s) written.