promote proposal for work item "uat-lane-source" (.bee/logs/scribing-runs.jsonl + .bee/lanes/uat-lane-source.json) — 1 capped cell(s): uls-1
anchor: ledger — .bee/logs/scribing-runs.jsonl, .bee/lanes/uat-lane-source.json
PROPOSAL ONLY — nothing was written. Applying any section below is a human or agent decision.

(a) DELIVERY DRAFT — save as docs/knowledge/work/uat-lane-source/delivery.md

---
type: bee.delivery
title: uat-lane-source — delivery
description: "Delivery record proposed by bee knowledge promote for work item uat-lane-source: 1 capped cell(s), 2 recorded deviation(s)."
timestamp: 2026-08-28
bee:
  id: uat-lane-source-delivery
  lifecycle: active
  areas: [worktree-parallelism]
  required_context: [.bee/logs/scribing-runs.jsonl, .bee/lanes/uat-lane-source.json]
  sources: [.bee/logs/scribing-runs.jsonl, .bee/lanes/uat-lane-source.json, .bee/cells/uls-1.json]
---

# uat-lane-source — Delivery

## What shipped

- **uls-1** — uat_lane_mode resolves the lane from route.lane, with mode kept only as a legacy fallback when its value is a real route-lane (1 file(s) changed)

## Verify

Each cell below was capped only against a recorded passing verify result — bee refuses a cap without one.

- **uls-1** — `PATH="$HOME/.cargo/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml uat && PATH="$HOME/.cargo/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml worktree && PATH="$HOME/.cargo/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml close`

## Deviations

- **uls-1** — Capped from main rather than from a worker run — the work already landed as commit c9c01ca7 and merged at 7ae59965; a dead session's swept claim left the record blocked while the work was done — found a better route
- **uls-1** — Capped from main rather than from a worker run — the work already landed as c9c01ca7 while a swept claim left the record blocked

## Provenance

Proposed by `bee knowledge promote --work uat-lane-source` from 1 capped cell trace(s) in `.bee/cells/` and the anchor `.bee/logs/scribing-runs.jsonl`, `.bee/lanes/uat-lane-source.json`. Every line above is copied from a trace or from the work item; nothing here is curated truth until a human or agent accepts it.

(b) AREA UPDATES — candidate spec-sync bullets, each citing its cell

areas: from the scribing stamp for "uat-lane-source" — .bee/logs/scribing-runs.jsonl's most recent entry (2026-08-28T05:34:00.379Z), the work item declares no bee.areas.

area worktree-parallelism:
  - [uls-1] uat_lane_mode resolves the lane from route.lane, with mode kept only as a legacy fallback when its value is a real route-lane — feature-wide sync per the scribing stamp, 1 file(s) changed (trace .bee/cells/uls-1.json)

(c) PATTERN CANDIDATES — candidate bee.pattern concepts, bee.polarity pitfall

from cell uls-1 — save as docs/knowledge/patterns/uat-lane-source-uls-1-pitfall.md

---
type: bee.pattern
title: uat-lane-source cell uls-1 — pitfall candidate
description: "Pitfall candidate mined from cell uls-1's capped trace: Capped from main rather than from a worker run — the work already landed as commit c9c01ca7 and merged at 7ae59965; a dead session's swept claim left the recor…"
timestamp: 2026-08-28
bee:
  id: uat-lane-source-uls-1-pitfall
  lifecycle: draft
  areas: [worktree-parallelism]
  sources: [.bee/cells/uls-1.json]
  polarity: pitfall
---

# uat-lane-source cell uls-1 — pitfall candidate

## What the cell did

uat_lane_mode resolves the lane from route.lane, with mode kept only as a legacy fallback when its value is a real route-lane

## Recorded evidence (verbatim from .bee/cells/uls-1.json)

- **deviation** — Capped from main rather than from a worker run — the work already landed as commit c9c01ca7 and merged at 7ae59965; a dead session's swept claim left the record blocked while the work was done — found a better route
- **deviation** — Capped from main rather than from a worker run — the work already landed as c9c01ca7 while a swept claim left the record blocked

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

knowledge promote: 1 capped cell(s) mined, 1 delivery draft, 1 area bullet(s), 1 pattern candidate(s), 0 file(s) written.