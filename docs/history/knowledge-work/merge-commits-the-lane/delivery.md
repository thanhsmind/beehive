---
type: bee.delivery
title: merge-commits-the-lane — delivery
description: "Delivery record proposed by bee knowledge promote for work item merge-commits-the-lane: 1 capped cell(s), 0 recorded deviation(s)."
timestamp: 2026-08-18
bee:
  id: merge-commits-the-lane-delivery
  lifecycle: active
  areas: [worktree-parallelism]
  required_context: [.bee/logs/scribing-runs.jsonl, .bee/lanes/merge-commits-the-lane.json]
  sources: [.bee/logs/scribing-runs.jsonl, .bee/lanes/merge-commits-the-lane.json, .bee/cells/archive/merge-commits-the-lane/mct-1.json]
---

# merge-commits-the-lane — Delivery

## What shipped

- **mct-1** — worktree merge: commit the lane rewrite instead of leaving it as dirt (2 file(s) changed)

## Verify

Each cell below was capped only against a recorded passing verify result — bee refuses a cap without one.

- **mct-1** — `PATH="$HOME/.cargo/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml worktree`

## Deviations

None recorded in the capped cell traces.

## Provenance

Proposed by `bee knowledge promote --work merge-commits-the-lane` from 1 capped cell trace(s) in `.bee/cells/` (since archived to `.bee/cells/archive/merge-commits-the-lane/`) and the anchor `.bee/logs/scribing-runs.jsonl`, `.bee/lanes/merge-commits-the-lane.json`. Every line above is copied from a trace or from the work item.

Accepted 2026-08-18 during a capture flush. The proposal's area-update bullet for `worktree-parallelism` was reviewed and NOT applied: `areas/worktree-parallelism/returning-and-the-merge-gate.md` already states the shipped behavior under "The lane rewrite is committed, not left as dirt (merge-commits-the-lane D1)", so the bullet would have restated an existing line. The proposal carried no pattern candidates.
