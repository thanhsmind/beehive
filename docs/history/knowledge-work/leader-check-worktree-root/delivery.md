---
type: bee.delivery
title: leader-check-worktree-root — delivery
description: "Delivery record proposed by bee knowledge promote for work item leader-check-worktree-root: 1 capped cell(s), 1 recorded deviation(s)."
timestamp: 2026-10-06
bee:
  id: leader-check-worktree-root-delivery
  lifecycle: active
  areas: [worktree-parallelism]
  required_context: [docs/history/leader-check-worktree-root/plan.md]
  sources: [docs/history/leader-check-worktree-root/plan.md, .bee/cells/archive/leader-check-worktree-root/lcwr-1.json]
---

# leader-check-worktree-root — Delivery

## What shipped

- **lcwr-1** — leader-check resolves artifacts against the feature worktree root (2 file(s) changed)

## Verify

Each cell below was capped only against a recorded passing verify result — bee refuses a cap without one.

- **lcwr-1** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml --bin bee leader_check` — 27 leader_check tests incl. 3 new worktree cases, red first for the reported reasons; full --bin bee 4130 passed, integration tests not run

## Deviations

- **lcwr-1** — sync-ack: no owned skill describes where leader-check resolves artifact paths; routing-and-contracts.md:181 (deferral is the named escape) stays true

## Provenance

Proposed by `bee knowledge promote --work leader-check-worktree-root` from 1 capped cell trace(s) in `.bee/cells/` and the anchor `docs/history/leader-check-worktree-root/plan.md`. Every line above is copied from a trace or from the work item; nothing here is curated truth until a human or agent accepts it.
