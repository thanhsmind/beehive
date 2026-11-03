---
type: bee.delivery
title: run-git-stdin-thread — delivery
description: "Delivery record proposed by bee knowledge promote for work item run-git-stdin-thread: 1 capped cell(s), 1 recorded deviation(s)."
timestamp: 2026-10-17
bee:
  id: run-git-stdin-thread-delivery
  lifecycle: active
  areas: [worktree-parallelism]
  required_context: [docs/history/run-git-stdin-thread/CONTEXT.md, docs/history/run-git-stdin-thread/plan.md]
  sources: [docs/history/run-git-stdin-thread/CONTEXT.md, docs/history/run-git-stdin-thread/plan.md, .bee/cells/rgs-1.json]
---

# run-git-stdin-thread — Delivery

## What shipped

- **rgs-1** — run_git_stdin writes stdin from a thread; no deadlock on a long patch-id stream; typo fixed (2 file(s) changed)

## Verify

Each cell below was capped only against a recorded passing verify result — bee refuses a cap without one.

- **rgs-1** — ``PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee verbs::worktree::tests` is green; `rg -c 'A clean rebase is not stale proof' docs/knowledge/areas/worktree-parallelism/returning-and-the-merge-gate.md` prints 1; `.bee/bin/bee dev comment-baseline --check` is clean.` — worktree tests 199 passed 0 failed after the change; rg printed 1; comment-baseline 253 files at or below baseline; no new test per D2 (an 800-commit fixture is not worth its runtime)

## Deviations

- **rgs-1** — followed the plan

## Provenance

Proposed by `bee knowledge promote --work run-git-stdin-thread` from 1 capped cell trace(s) in `.bee/cells/` and the anchor `docs/history/run-git-stdin-thread/CONTEXT.md`, `docs/history/run-git-stdin-thread/plan.md`. Every line above is copied from a trace or from the work item; nothing here is curated truth until a human or agent accepts it.
