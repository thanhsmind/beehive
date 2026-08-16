---
type: bee.delivery
title: sweep-at-every-door — delivery
description: "Delivery record proposed by bee knowledge promote for work item sweep-at-every-door: 2 capped cell(s), 0 recorded deviation(s)."
timestamp: 2026-08-13
bee:
  id: sweep-at-every-door-delivery
  lifecycle: active
  areas: [workflow-state]
  required_context: [docs/history/sweep-at-every-door/CONTEXT.md, docs/history/sweep-at-every-door/plan.md]
  sources: [docs/history/sweep-at-every-door/CONTEXT.md, docs/history/sweep-at-every-door/plan.md, .bee/cells/archive/sweep-at-every-door/sad-1.json, .bee/cells/archive/sweep-at-every-door/sad-2.json]
---

# sweep-at-every-door — Delivery

## What shipped

- **sad-1** — sweep is now caller-aware (D6 self-exclusion), writes the D4 blocked verdict with trace.blocked_reason, and never crosses the store boundary (D5) (3 file(s) changed)
- **sad-2** — bee orient now sweeps expired claims (D1/D6), self-excluding its own caller and declining when unresolvable; registry text corrected (3 file(s) changed)

## Verify

Each cell below was capped only against a recorded passing verify result — bee refuses a cap without one.

- **sad-1** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml`
- **sad-2** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml`

## Deviations

None recorded in the capped cell traces.

## Provenance

Proposed by `bee knowledge promote --work sweep-at-every-door` from 2 capped cell trace(s) in `.bee/cells/` and the anchor `docs/history/sweep-at-every-door/CONTEXT.md`, `docs/history/sweep-at-every-door/plan.md`. Every line above is copied from a trace or from the work item; Applied 2026-08-16 from docs/history/sweep-at-every-door/promote-proposals.md; area bullets declined (feature-wide scribing sync already stamped), no pattern candidates survived review.
