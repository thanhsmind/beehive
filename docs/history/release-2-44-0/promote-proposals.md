promote proposal for work item "release-2-44-0" (docs/history/release-2-44-0/CONTEXT.md + docs/history/release-2-44-0/plan.md) — 1 capped cell(s): rel440-1
anchor: history — docs/history/release-2-44-0/CONTEXT.md, docs/history/release-2-44-0/plan.md
PROPOSAL ONLY — nothing was written. Applying any section below is a human or agent decision.

(a) DELIVERY DRAFT — save as docs/knowledge/work/release-2-44-0/delivery.md

---
type: bee.delivery
title: release-2-44-0 — delivery
description: "Delivery record proposed by bee knowledge promote for work item release-2-44-0: 1 capped cell(s), 1 recorded deviation(s)."
timestamp: 2026-09-22
bee:
  id: release-2-44-0-delivery
  lifecycle: active
  required_context: [docs/history/release-2-44-0/CONTEXT.md, docs/history/release-2-44-0/plan.md]
  sources: [docs/history/release-2-44-0/CONTEXT.md, docs/history/release-2-44-0/plan.md, .bee/cells/rel440-1.json]
---

# release-2-44-0 — Delivery

## What shipped

- **rel440-1** — Released 2.44.0 (tag v2.44.0, five binaries verified, OK) (3 file(s) changed)

## Verify

Each cell below was capped only against a recorded passing verify result — bee refuses a cap without one.

- **rel440-1** — `bash scripts/release.sh 2.44.0 && bee dev release-manifest --check` — the script ran the full declared suite (4205 passed, 0 failed) before tagging, release-binaries run 35680422522 went green on all five rows including the three new macOS/ARM runners, and it printed i…

## Deviations

- **rel440-1** — the leader ran scripts/release.sh under the prepared deployment permit 996ea8f2 instead of a dispatched agent — the deploy role on the native opus runtime returned a read-only gather payload that cannot run the script (same as release-2-43-0) — hit an unforeseen obstacle

## Provenance

Proposed by `bee knowledge promote --work release-2-44-0` from 1 capped cell trace(s) in `.bee/cells/` and the anchor `docs/history/release-2-44-0/CONTEXT.md`, `docs/history/release-2-44-0/plan.md`. Every line above is copied from a trace or from the work item; nothing here is curated truth until a human or agent accepts it.

(b) AREA UPDATES — candidate spec-sync bullets, each citing its cell

None: the work item declares no bee.areas, so there is no area to sync (D19).

(c) PATTERN CANDIDATES — candidate bee.pattern concepts, bee.polarity pitfall

from cell rel440-1 — save as docs/knowledge/patterns/release-2-44-0-rel440-1-pitfall.md

---
type: bee.pattern
title: release-2-44-0 cell rel440-1 — pitfall candidate
description: "Pitfall candidate mined from cell rel440-1's capped trace: the leader ran scripts/release.sh under the prepared deployment permit 996ea8f2 instead of a dispatched agent — the deploy role on the native opus runtime retu…"
timestamp: 2026-09-22
bee:
  id: release-2-44-0-rel440-1-pitfall
  lifecycle: draft
  sources: [.bee/cells/rel440-1.json]
  polarity: pitfall
---

# release-2-44-0 cell rel440-1 — pitfall candidate

## What the cell did

Released 2.44.0 (tag v2.44.0, five binaries verified, OK)

## Recorded evidence (verbatim from .bee/cells/rel440-1.json)

- **deviation** — the leader ran scripts/release.sh under the prepared deployment permit 996ea8f2 instead of a dispatched agent — the deploy role on the native opus runtime returned a read-only gather payload that cannot run the script (same as release-2-43-0) — hit an unforeseen obstacle

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

knowledge promote: 1 capped cell(s) mined, 1 delivery draft, 0 area bullet(s), 1 pattern candidate(s), 0 file(s) written.