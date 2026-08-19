promote proposal for work item "uat-approval-reaches-the-door" (docs/history/uat-approval-reaches-the-door/plan.md) — 2 capped cell(s): uad-1, uad-2
anchor: history — docs/history/uat-approval-reaches-the-door/plan.md
PROPOSAL ONLY — nothing was written. Applying any section below is a human or agent decision.

(a) DELIVERY DRAFT — save as docs/knowledge/work/uat-approval-reaches-the-door/delivery.md

---
type: bee.delivery
title: uat-approval-reaches-the-door — delivery
description: "Delivery record proposed by bee knowledge promote for work item uat-approval-reaches-the-door: 2 capped cell(s), 4 recorded deviation(s)."
timestamp: 2026-08-18
bee:
  id: uat-approval-reaches-the-door-delivery
  lifecycle: active
  areas: [worktree-parallelism, workflow-state]
  required_context: [docs/history/uat-approval-reaches-the-door/plan.md]
  sources: [docs/history/uat-approval-reaches-the-door/plan.md, .bee/cells/uad-1.json, .bee/cells/uad-2.json]
---

# uat-approval-reaches-the-door — Delivery

## What shipped

- **uad-1** — One uat resolver in uat.rs serves both doors, with the lane record as a second source (4 file(s) changed)
- **uad-2** — bee gate says so when a lane approval cannot reach the durable workflow stamp (1 file(s) changed)

## Verify

Each cell below was capped only against a recorded passing verify result — bee refuses a cap without one.

- **uad-1** — `PATH="$HOME/.cargo/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml`
- **uad-2** — `PATH="$HOME/.cargo/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml`

## Deviations

- **uad-1** — SHIPPED SHAPE DIFFERS FROM THE FROZEN PLAN, ratified by decision uat-approval-reaches-the-door D1 (8ca2378f). The plan's Approach step 3 named a strict cascade — live record, else lane record, else default state. The shipped resolver in uat.rs is live record, else (lane OR default). The cascade is unimplementable as written: spread_gates stamps uat:false onto every merged record and read_lane_display returns that merged shape, so a lane record silent on uat is byte-identical at the decision point to one saying false; a presence-based cascade would read every silent lane record as a veto and would have broken the pre-existing passing test uat_door_does_not_block_once_uat_is_approved.
- **uad-1** — WHAT THE OR GIVES UP, stated so it is not discovered later: an explicit lane-side false cannot veto a stale default-state true for the same feature, so a revocation after an earlier --no-lane approval leaves the door open. Equally true before this change, so not a regression — filed as its own backlog row rather than folded in here.
- **uad-1** — Cell truth 3 amended to name the default-state clause the OR makes explicit, and a truth added stating the approved-input set is exactly the pre-change set plus a literal true at the lane record. The original cap recorded trace.deviations as empty; this re-cap is a record repair with no code change, and the commit is unchanged.
- **uad-1** — An independent judge verified the security property separately: the set of inputs the resolver approves is the pre-change set plus a literal true at the lane record's approved_gates.uat, written only by bee gate, which still refuses --actor auto for uat. No input approves without an owner approval naming that same feature.

## Provenance

Proposed by `bee knowledge promote --work uat-approval-reaches-the-door` from 2 capped cell trace(s) in `.bee/cells/` and the anchor `docs/history/uat-approval-reaches-the-door/plan.md`. Every line above is copied from a trace or from the work item; nothing here is curated truth until a human or agent accepts it.

(b) AREA UPDATES — candidate spec-sync bullets, each citing its cell

areas: from the scribing stamp for "uat-approval-reaches-the-door" — .bee/logs/scribing-runs.jsonl's most recent entry (2026-08-18T15:49:11.446Z), the work item declares no bee.areas.

area worktree-parallelism:
  - [uad-1] One uat resolver in uat.rs serves both doors, with the lane record as a second source — feature-wide sync per the scribing stamp, 4 file(s) changed (trace .bee/cells/uad-1.json)
  - [uad-2] bee gate says so when a lane approval cannot reach the durable workflow stamp — feature-wide sync per the scribing stamp, 1 file(s) changed (trace .bee/cells/uad-2.json)

area workflow-state:
  - [uad-1] One uat resolver in uat.rs serves both doors, with the lane record as a second source — feature-wide sync per the scribing stamp, 4 file(s) changed (trace .bee/cells/uad-1.json)
  - [uad-2] bee gate says so when a lane approval cannot reach the durable workflow stamp — feature-wide sync per the scribing stamp, 1 file(s) changed (trace .bee/cells/uad-2.json)

(c) PATTERN CANDIDATES — candidate bee.pattern concepts, bee.polarity pitfall

from cell uad-1 — save as docs/knowledge/patterns/uat-approval-reaches-the-door-uad-1-pitfall.md

---
type: bee.pattern
title: uat-approval-reaches-the-door cell uad-1 — pitfall candidate
description: "Pitfall candidate mined from cell uad-1's capped trace: SHIPPED SHAPE DIFFERS FROM THE FROZEN PLAN, ratified by decision uat-approval-reaches-the-door D1 (8ca2378f). The plan's Approach step 3 named a strict cascade…"
timestamp: 2026-08-18
bee:
  id: uat-approval-reaches-the-door-uad-1-pitfall
  lifecycle: draft
  areas: [worktree-parallelism, workflow-state]
  sources: [.bee/cells/uad-1.json]
  polarity: pitfall
---

# uat-approval-reaches-the-door cell uad-1 — pitfall candidate

## What the cell did

One uat resolver in uat.rs serves both doors, with the lane record as a second source

## Recorded evidence (verbatim from .bee/cells/uad-1.json)

- **deviation** — SHIPPED SHAPE DIFFERS FROM THE FROZEN PLAN, ratified by decision uat-approval-reaches-the-door D1 (8ca2378f). The plan's Approach step 3 named a strict cascade — live record, else lane record, else default state. The shipped resolver in uat.rs is live record, else (lane OR default). The cascade is unimplementable as written: spread_gates stamps uat:false onto every merged record and read_lane_display returns that merged shape, so a lane record silent on uat is byte-identical at the decision point to one saying false; a presence-based cascade would read every silent lane record as a veto and would have broken the pre-existing passing test uat_door_does_not_block_once_uat_is_approved.
- **deviation** — WHAT THE OR GIVES UP, stated so it is not discovered later: an explicit lane-side false cannot veto a stale default-state true for the same feature, so a revocation after an earlier --no-lane approval leaves the door open. Equally true before this change, so not a regression — filed as its own backlog row rather than folded in here.
- **deviation** — Cell truth 3 amended to name the default-state clause the OR makes explicit, and a truth added stating the approved-input set is exactly the pre-change set plus a literal true at the lane record. The original cap recorded trace.deviations as empty; this re-cap is a record repair with no code change, and the commit is unchanged.
- **deviation** — An independent judge verified the security property separately: the set of inputs the resolver approves is the pre-change set plus a literal true at the lane record's approved_gates.uat, written only by bee gate, which still refuses --actor auto for uat. No input approves without an owner approval naming that same feature.

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

knowledge promote: 2 capped cell(s) mined, 1 delivery draft, 4 area bullet(s), 1 pattern candidate(s), 0 file(s) written.