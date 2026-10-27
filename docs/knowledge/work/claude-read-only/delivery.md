---
type: bee.delivery
title: claude-read-only — delivery
description: "Delivery record proposed by bee knowledge promote for work item claude-read-only: 1 capped cell(s), 1 recorded deviation(s)."
timestamp: 2026-10-12
bee:
  id: claude-read-only-delivery
  lifecycle: active
  required_context: [docs/history/claude-read-only/CONTEXT.md, docs/history/claude-read-only/plan.md]
  sources: [docs/history/claude-read-only/CONTEXT.md, docs/history/claude-read-only/plan.md, .bee/cells/cro-1.json]
---

# claude-read-only — Delivery

## What shipped

- **cro-1** — Read-only worker-guard accepts a redirect to exactly /dev/null; other targets outside the mailbox stay denied (1 file(s) changed)

## Verify

Each cell below was capped only against a recorded passing verify result — bee refuses a cap without one.

- **cro-1** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --manifest-path packages/bee-rs/Cargo.toml -p bee hook_manifests && PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --manifest-path packages/bee-rs/Cargo.toml -p bee hooks_wiring && PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --manifest-path packages/bee-rs/Cargo.toml -p bee worker_guard && PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --manifest-path packages/bee-rs/Cargo.toml -p bee execute_paseo_read_only` — cell verify filters 8, 24, 11 and 5 passed plus 1 integration row; read_only_table was red before the fix (10 passed, 1 failed); full suite not run

## Deviations

- **cro-1** — followed the plan

## Provenance

Proposed by `bee knowledge promote --work claude-read-only` from 1 capped cell trace(s) in `.bee/cells/` and the anchor `docs/history/claude-read-only/CONTEXT.md`, `docs/history/claude-read-only/plan.md`. Every line above is copied from a trace or from the work item; nothing here is curated truth until a human or agent accepts it.
