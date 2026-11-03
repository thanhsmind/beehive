---
type: bee.delivery
title: prune-queue-copy — delivery
description: "Delivery record proposed by bee knowledge promote for work item prune-queue-copy: 1 capped cell(s), 1 recorded deviation(s)."
timestamp: 2026-10-15
bee:
  id: prune-queue-copy-delivery
  lifecycle: active
  areas: [worktree-parallelism]
  required_context: [docs/history/prune-queue-copy/CONTEXT.md, docs/history/prune-queue-copy/plan.md]
  sources: [docs/history/prune-queue-copy/CONTEXT.md, docs/history/prune-queue-copy/plan.md, .bee/cells/pqc-1.json]
---

# prune-queue-copy — Delivery

## What shipped

- **pqc-1** — Prune keeps a worktree for its capture queue only on a pending stub that main never names, and every doubt keeps (3 file(s) changed)

## Verify

Each cell below was capped only against a recorded passing verify result — bee refuses a cap without one.

- **pqc-1** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --manifest-path packages/bee-rs/Cargo.toml -p bee worktree::` — 213 passed 0 failed, the 12 new queue tests listed by name, scope is the worktree module the change lives in, bee dev comment-baseline --check also green, full suite not run, run with PATH=HOME/.carg…

## Deviations

- **pqc-1** — ran the verify with PATH=HOME/.cargo/bin instead of the CARGO_HOME expansion — the session isolation guard refused the variable expansion — same cargo binary and same test filter

## Provenance

Proposed by `bee knowledge promote --work prune-queue-copy` from 1 capped cell trace(s) in `.bee/cells/` and the anchor `docs/history/prune-queue-copy/CONTEXT.md`, `docs/history/prune-queue-copy/plan.md`. Every line above is copied from a trace or from the work item; nothing here is curated truth until a human or agent accepts it.
