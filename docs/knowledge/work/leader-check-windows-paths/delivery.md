---
type: bee.delivery
title: leader-check-windows-paths — delivery
description: "Delivery record proposed by bee knowledge promote for work item leader-check-windows-paths: 1 capped cell(s), 1 recorded deviation(s)."
timestamp: 2026-10-06
bee:
  id: leader-check-windows-paths-delivery
  lifecycle: active
  areas: [worktree-parallelism]
  required_context: [docs/history/leader-check-windows-paths/plan.md]
  sources: [docs/history/leader-check-windows-paths/plan.md, .bee/cells/lcwp-1.json]
---

# leader-check-windows-paths — Delivery

## What shipped

- **lcwp-1** — leader-check canonicalizes on a failed strip so aliased worktree paths match (2 file(s) changed)

## Verify

Each cell below was capped only against a recorded passing verify result — bee refuses a cap without one.

- **lcwp-1** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml --bin bee leader_check` — 28 leader_check tests; new unix symlink test red first with the Windows failure shape; full --bin bee 4131 passed on Linux; Windows itself not run locally, left to Windows CI

## Deviations

- **lcwp-1** — sync-ack: no owned skill describes leader-check path matching

## Provenance

Proposed by `bee knowledge promote --work leader-check-windows-paths` from 1 capped cell trace(s) in `.bee/cells/` and the anchor `docs/history/leader-check-windows-paths/plan.md`. Every line above is copied from a trace or from the work item; nothing here is curated truth until a human or agent accepts it.
