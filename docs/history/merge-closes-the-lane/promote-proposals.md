promote proposal for work item "merge-closes-the-lane" (docs/history/merge-closes-the-lane/plan.md) — 3 capped cell(s): mcl-1, mcl-2, mcl-3
anchor: history — docs/history/merge-closes-the-lane/plan.md
PROPOSAL ONLY — nothing was written. Applying any section below is a human or agent decision.

(a) DELIVERY DRAFT — save as docs/knowledge/work/merge-closes-the-lane/delivery.md

---
type: bee.delivery
title: merge-closes-the-lane — delivery
description: "Delivery record proposed by bee knowledge promote for work item merge-closes-the-lane: 3 capped cell(s), 0 recorded deviation(s)."
timestamp: 2026-08-18
bee:
  id: merge-closes-the-lane-delivery
  lifecycle: active
  areas: [worktree-parallelism]
  required_context: [docs/history/merge-closes-the-lane/plan.md]
  sources: [docs/history/merge-closes-the-lane/plan.md, .bee/cells/mcl-1.json, .bee/cells/mcl-2.json, .bee/cells/mcl-3.json]
---

# merge-closes-the-lane — Delivery

## What shipped

- **mcl-1** — Widened waiting-on clear to accept a closed workflow and null the lane waiting_on/run_state pair (2 file(s) changed)
- **mcl-2** — Move worktree merge lane write past the post-commit dirty guard (2 file(s) changed)
- **mcl-3** — close: a green non-dry-run close sets the lane terminal (1 file(s) changed)

## Verify

Each cell below was capped only against a recorded passing verify result — bee refuses a cap without one.

- **mcl-1** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml waiting_on`
- **mcl-2** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml worktree`
- **mcl-3** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml close`

## Deviations

None recorded in the capped cell traces.

## Provenance

Proposed by `bee knowledge promote --work merge-closes-the-lane` from 3 capped cell trace(s) in `.bee/cells/` and the anchor `docs/history/merge-closes-the-lane/plan.md`. Every line above is copied from a trace or from the work item; nothing here is curated truth until a human or agent accepts it.

(b) AREA UPDATES — candidate spec-sync bullets, each citing its cell

areas: from the scribing stamp for "merge-closes-the-lane" — .bee/logs/scribing-runs.jsonl's most recent entry (2026-08-18T11:09:56.436Z), the work item declares no bee.areas.

area worktree-parallelism:
  - [mcl-1] Widened waiting-on clear to accept a closed workflow and null the lane waiting_on/run_state pair — feature-wide sync per the scribing stamp, 2 file(s) changed (trace .bee/cells/mcl-1.json)
  - [mcl-2] Move worktree merge lane write past the post-commit dirty guard — feature-wide sync per the scribing stamp, 2 file(s) changed (trace .bee/cells/mcl-2.json)
  - [mcl-3] close: a green non-dry-run close sets the lane terminal — feature-wide sync per the scribing stamp, 1 file(s) changed (trace .bee/cells/mcl-3.json)

(c) PATTERN CANDIDATES — candidate bee.pattern concepts, bee.polarity pitfall

from cell mcl-2 — save as docs/knowledge/patterns/merge-closes-the-lane-mcl-2-pitfall.md

---
type: bee.pattern
title: merge-closes-the-lane cell mcl-2 — pitfall candidate
description: "Pitfall candidate mined from cell mcl-2's capped trace: merge_lane_write_placed_before_post_commit_dirty_guard_fires_verify_mutated_tracked_files_on_every_green_merge"
timestamp: 2026-08-18
bee:
  id: merge-closes-the-lane-mcl-2-pitfall
  lifecycle: draft
  areas: [worktree-parallelism]
  sources: [.bee/cells/mcl-2.json]
  polarity: pitfall
---

# merge-closes-the-lane cell mcl-2 — pitfall candidate

## What the cell did

Move worktree merge lane write past the post-commit dirty guard

## Recorded evidence (verbatim from .bee/cells/mcl-2.json)

- **failure_signature** — merge_lane_write_placed_before_post_commit_dirty_guard_fires_verify_mutated_tracked_files_on_every_green_merge

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

knowledge promote: 3 capped cell(s) mined, 1 delivery draft, 3 area bullet(s), 1 pattern candidate(s), 0 file(s) written.