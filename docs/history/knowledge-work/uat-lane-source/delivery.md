---
type: bee.delivery
title: uat-lane-source — delivery
description: "Delivery record proposed by bee knowledge promote for work item uat-lane-source: 1 capped cell(s), 2 recorded deviation(s)."
timestamp: 2026-08-28
bee:
  id: uat-lane-source-delivery
  lifecycle: active
  areas: [worktree-parallelism]
  required_context: [.bee/logs/scribing-runs.jsonl, .bee/lanes/uat-lane-source.json, docs/history/uat-lane-source/promote-proposals.md]
  sources: [.bee/logs/scribing-runs.jsonl, .bee/lanes/uat-lane-source.json, docs/history/uat-lane-source/promote-proposals.md, .bee/cells/archive/uat-lane-source/uls-1.json]
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

Proposed by `bee knowledge promote --work uat-lane-source` from 1 capped cell trace(s) in `.bee/cells/` and the anchor `.bee/logs/scribing-runs.jsonl`, `.bee/lanes/uat-lane-source.json`, `docs/history/uat-lane-source/promote-proposals.md`. Every line above is copied from a trace or from the work item; nothing here is curated truth until a human or agent accepts it.
