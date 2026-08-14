promote proposal for work item "retire-collation-guard" (docs/history/retire-collation-guard/CONTEXT.md + docs/history/retire-collation-guard/plan.md) — 1 capped cell(s): rcg-1
anchor: history — docs/history/retire-collation-guard/CONTEXT.md, docs/history/retire-collation-guard/plan.md
PROPOSAL ONLY — nothing was written. Applying any section below is a human or agent decision.

(a) DELIVERY DRAFT — save as docs/knowledge/work/retire-collation-guard/delivery.md

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
  sources: [docs/history/retire-collation-guard/CONTEXT.md, docs/history/retire-collation-guard/plan.md, .bee/cells/rcg-1.json]
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

Proposed by `bee knowledge promote --work retire-collation-guard` from 1 capped cell trace(s) in `.bee/cells/` and the anchor `docs/history/retire-collation-guard/CONTEXT.md`, `docs/history/retire-collation-guard/plan.md`. Every line above is copied from a trace or from the work item; nothing here is curated truth until a human or agent accepts it.

(b) AREA UPDATES — candidate spec-sync bullets, each citing its cell

areas: from the scribing stamp for "retire-collation-guard" — .bee/logs/scribing-runs.jsonl's most recent entry (2026-08-14T00:31:01.487Z), the work item declares no bee.areas.

area rust-runtime:
  - [rcg-1] Retired collation_safe (both copies) and id_sort_safe with their call-site guards; inverted the four tests that asserted the defect and deleted the two direct unit tests of the retired functions. bee decisions render and bee backlog pbi list (no --status) now run instead of refusing. — feature-wide sync per the scribing stamp, 3 file(s) changed (trace .bee/cells/rcg-1.json)

(c) PATTERN CANDIDATES — candidate bee.pattern concepts, bee.polarity pitfall

None: no capped cell trace carries a deviation or a failure signature.

knowledge promote: 1 capped cell(s) mined, 1 delivery draft, 1 area bullet(s), 0 pattern candidate(s), 0 file(s) written.