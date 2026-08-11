promote proposal for work item "island-read-filter" (.bee/logs/scribing-runs.jsonl) — 1 capped cell(s): irf-1
anchor: ledger — .bee/logs/scribing-runs.jsonl
PROPOSAL ONLY — nothing was written. Applying any section below is a human or agent decision.

(a) DELIVERY DRAFT — save as docs/knowledge/work/island-read-filter/delivery.md

---
type: bee.delivery
title: island-read-filter — delivery
description: "Delivery record proposed by bee knowledge promote for work item island-read-filter: 1 capped cell(s), 1 recorded deviation(s)."
timestamp: 2026-08-11
bee:
  id: island-read-filter-delivery
  lifecycle: active
  areas: [worktree-parallelism]
  required_context: [.bee/logs/scribing-runs.jsonl]
  sources: [.bee/logs/scribing-runs.jsonl, .bee/cells/irf-1.json]
---

# island-read-filter — Delivery

## What shipped

- **irf-1** — island reads scope to the granted feature at both enumerators (cells/read.rs list_cells, status_full list+archive door); main reads pinned byte-identical; 9 new tests red-first (4 file(s) changed)

## Verify

Each cell below was capped only against a recorded passing verify result — bee refuses a cap without one.

- **irf-1** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml`

## Deviations

- **irf-1** — two enumerators exist (cells/read.rs and status_full/cells.rs), filtered at each lowest seam reusing read_worktree_feature; cells list/ready/claim-next CLI still refuse-delegate for granted worktrees at root resolution - widening those doors is a separate ask

## Provenance

Proposed by `bee knowledge promote --work island-read-filter` from 1 capped cell trace(s) in `.bee/cells/` and the anchor `.bee/logs/scribing-runs.jsonl`. Every line above is copied from a trace or from the work item; nothing here is curated truth until a human or agent accepts it.

(b) AREA UPDATES — candidate spec-sync bullets, each citing its cell

areas: from the scribing stamp for "island-read-filter" — .bee/logs/scribing-runs.jsonl's most recent entry (2026-08-11T04:37:52.389Z), the work item declares no bee.areas.

area worktree-parallelism:
  - [irf-1] island reads scope to the granted feature at both enumerators (cells/read.rs list_cells, status_full list+archive door); main reads pinned byte-identical; 9 new tests red-first — feature-wide sync per the scribing stamp, 4 file(s) changed (trace .bee/cells/irf-1.json)

(c) PATTERN CANDIDATES — candidate bee.pattern concepts, bee.polarity pitfall

from cell irf-1 — save as docs/knowledge/patterns/island-read-filter-irf-1-pitfall.md

---
type: bee.pattern
title: island-read-filter cell irf-1 — pitfall candidate
description: "Pitfall candidate mined from cell irf-1's capped trace: two enumerators exist (cells/read.rs and status_full/cells.rs), filtered at each lowest seam reusing read_worktree_feature; cells list/ready/claim-next CLI sti…"
timestamp: 2026-08-11
bee:
  id: island-read-filter-irf-1-pitfall
  lifecycle: draft
  areas: [worktree-parallelism]
  sources: [.bee/cells/irf-1.json]
  polarity: pitfall
---

# island-read-filter cell irf-1 — pitfall candidate

## What the cell did

island reads scope to the granted feature at both enumerators (cells/read.rs list_cells, status_full list+archive door); main reads pinned byte-identical; 9 new tests red-first

## Recorded evidence (verbatim from .bee/cells/irf-1.json)

- **deviation** — two enumerators exist (cells/read.rs and status_full/cells.rs), filtered at each lowest seam reusing read_worktree_feature; cells list/ready/claim-next CLI still refuse-delegate for granted worktrees at root resolution - widening those doors is a separate ask

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

knowledge promote: 1 capped cell(s) mined, 1 delivery draft, 1 area bullet(s), 1 pattern candidate(s), 0 file(s) written.