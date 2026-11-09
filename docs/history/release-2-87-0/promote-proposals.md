promote proposal for work item "release-2-87-0" (docs/history/release-2-87-0/CONTEXT.md + docs/history/release-2-87-0/plan.md) — 1 capped cell(s): rel870-1
anchor: history — docs/history/release-2-87-0/CONTEXT.md, docs/history/release-2-87-0/plan.md
PROPOSAL ONLY — nothing was written. Applying any section below is a human or agent decision.

This proposal covers (b) area updates and (c) pattern candidates only; it proposes no delivery record.

(b) AREA UPDATES — candidate spec-sync bullets, each citing its cell

None: the work item declares no bee.areas, so there is no area to sync (D19).

(c) PATTERN CANDIDATES — candidate bee.pattern concepts, bee.polarity pitfall

from cell rel870-1 — save as docs/knowledge/patterns/release-2-87-0-rel870-1-pitfall.md

---
type: bee.pattern
title: release-2-87-0 cell rel870-1 — pitfall candidate
description: "Pitfall candidate mined from cell rel870-1's capped trace: the deploy worker ran from the bee dispatch prepare payload, which sets no opencode PATH or private CARGO_TARGET_DIR; the suite was green anyway"
timestamp: 2026-11-09
bee:
  id: release-2-87-0-rel870-1-pitfall
  lifecycle: draft
  sources: [.bee/cells/rel870-1.json]
  polarity: pitfall
---

# release-2-87-0 cell rel870-1 — pitfall candidate

## What the cell did

Published 2.87.0 through scripts/release.sh under permit bf83d0d2

## Recorded evidence (verbatim from .bee/cells/rel870-1.json)

- **deviation** — the deploy worker ran from the bee dispatch prepare payload, which sets no opencode PATH or private CARGO_TARGET_DIR; the suite was green anyway
- **deviation** — sync-ack: release version bump only: scripts/release.sh rewrites version stamps and the manifest; no behavior an anchored concept describes changed

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

knowledge promote: 1 capped cell(s) mined, 0 area bullet(s), 1 pattern candidate(s), 0 file(s) written.