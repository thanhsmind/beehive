---
type: bee.delivery
title: code-shape-doctrine — delivery
description: "Delivery record proposed by bee knowledge promote for work item code-shape-doctrine: 1 capped cell(s), 1 recorded deviation(s)."
timestamp: 2026-09-01
bee:
  id: code-shape-doctrine-delivery
  lifecycle: active
  required_context: [docs/history/code-shape-doctrine/CONTEXT.md]
  sources: [docs/history/code-shape-doctrine/CONTEXT.md, .bee/cells/archive/code-shape-doctrine/csdoc-1.json]
---

# code-shape-doctrine — Delivery

## What shipped

- **csdoc-1** — Added the four code-shape rules as one contract bullet in the worker brief source and the bee-build agent template, stated as judgment-and-review with no refusal (7 file(s) changed)

## Verify

Each cell below was capped only against a recorded passing verify result — bee refuses a cap without one.

- **csdoc-1** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --no-fail-fast --manifest-path packages/bee-rs/Cargo.toml && .bee/bin/bee onboard --repo-root . --json && .bee/bin/bee dev release-manifest --check`

## Deviations

- **csdoc-1** — followed the plan

## Provenance

Proposed by `bee knowledge promote --work code-shape-doctrine` from 1 capped cell trace(s) in `.bee/cells/` and the anchor `docs/history/code-shape-doctrine/CONTEXT.md`. Every line above is copied from a trace or from the work item; nothing here is curated truth until a human or agent accepts it.
