---
type: bee.delivery
title: merge-door-precision — delivery
description: "Delivery record proposed by bee knowledge promote for work item merge-door-precision: 1 capped cell(s), 3 recorded deviation(s)."
timestamp: 2026-08-28
bee:
  id: merge-door-precision-delivery
  lifecycle: active
  areas: [worktree-parallelism]
  required_context: [.bee/logs/scribing-runs.jsonl, .bee/lanes/merge-door-precision.json, docs/history/merge-door-precision/promote-proposals.md]
  sources: [.bee/logs/scribing-runs.jsonl, .bee/lanes/merge-door-precision.json, docs/history/merge-door-precision/promote-proposals.md, .bee/cells/archive/merge-door-precision/mdp-1.json]
---

# merge-door-precision — Delivery

## What shipped

- **mdp-1** — The merge door now refuses only for dirt it can collide with: tracked dirt unchanged, untracked dirt blocks only inside the branch's merge-base changed-file set (3 file(s) changed)

## Verify

Each cell below was capped only against a recorded passing verify result — bee refuses a cap without one.

- **mdp-1** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee --bins worktree && PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee --bins merge`

## Deviations

- **mdp-1** — Deleted git_status_porcelain_excluding_untracked_all instead of keeping it beside the new -z splitter — it had exactly four callers, all in the code this cell rewrites, and leaving it would have been dead code in a non-test build; its `?? docs/history/<theirs>/plan.md` rationale moved verbatim onto the new reader — found a better route
- **mdp-1** — Wrote the two new refusal tests against plain git commands rather than the new helpers, so the red run failed on behaviour instead of on a missing symbol — found a better route
- **mdp-1** — docs/knowledge/areas/worktree-parallelism/returning-and-the-merge-gate.md lines 197-201 still state the old rule ("any dirty path outside those two roots still refuses exactly as before") and I left it untouched — it is not in the cell's files, so the sync is the orchestrator's scope call — hit an unforeseen obstacle

## Provenance

Proposed by `bee knowledge promote --work merge-door-precision` from 1 capped cell trace(s) in `.bee/cells/` and the anchor `.bee/logs/scribing-runs.jsonl`, `.bee/lanes/merge-door-precision.json`, `docs/history/merge-door-precision/promote-proposals.md`. Every line above is copied from a trace or from the work item; nothing here is curated truth until a human or agent accepts it.
