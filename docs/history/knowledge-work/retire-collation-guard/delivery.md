---
type: bee.delivery
title: retire-collation-guard — delivery
description: "Delivery record proposed by bee knowledge promote for work item retire-collation-guard: 1 capped cell(s), 0 recorded deviation(s)."
timestamp: 2026-08-14
bee:
  id: retire-collation-guard-delivery
  lifecycle: active
  areas: [rust-runtime]
  required_context: [docs/history/retire-collation-guard/CONTEXT.md, docs/history/retire-collation-guard/plan.md]
  sources: [docs/history/retire-collation-guard/CONTEXT.md, docs/history/retire-collation-guard/plan.md, .bee/cells/archive/retire-collation-guard/rcg-1.json]
---

# retire-collation-guard — Delivery

## What shipped

- **rcg-1** — Retired collation_safe (both copies) and id_sort_safe with their call-site guards; inverted the four tests that asserted the defect and deleted the two direct unit tests of the retired functions. bee decisions render and bee backlog pbi list (no --status) now run instead of refusing. (3 file(s) changed)

## Verify

Each cell below was capped only against a recorded passing verify result — bee refuses a cap without one.

- **rcg-1** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml`

## Deviations

None recorded in the capped cell traces.

## Provenance

Proposed by `bee knowledge promote --work retire-collation-guard` from 1 capped cell trace(s) in `.bee/cells/` and the anchor `docs/history/retire-collation-guard/CONTEXT.md`, `docs/history/retire-collation-guard/plan.md`. Every line above is copied from a trace or from the work item; Applied 2026-08-16 from docs/history/retire-collation-guard/promote-proposals.md; area bullets declined (feature-wide scribing sync already stamped), no pattern candidates survived review.
