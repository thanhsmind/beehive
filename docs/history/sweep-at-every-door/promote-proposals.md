promote proposal for work item "sweep-at-every-door" (docs/history/sweep-at-every-door/CONTEXT.md + docs/history/sweep-at-every-door/plan.md) — 2 capped cell(s): sad-1, sad-2
anchor: history — docs/history/sweep-at-every-door/CONTEXT.md, docs/history/sweep-at-every-door/plan.md
PROPOSAL ONLY — nothing was written. Applying any section below is a human or agent decision.

(a) DELIVERY DRAFT — save as docs/knowledge/work/sweep-at-every-door/delivery.md

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
  sources: [docs/history/sweep-at-every-door/CONTEXT.md, docs/history/sweep-at-every-door/plan.md, .bee/cells/sad-1.json, .bee/cells/sad-2.json]
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

Proposed by `bee knowledge promote --work sweep-at-every-door` from 2 capped cell trace(s) in `.bee/cells/` and the anchor `docs/history/sweep-at-every-door/CONTEXT.md`, `docs/history/sweep-at-every-door/plan.md`. Every line above is copied from a trace or from the work item; nothing here is curated truth until a human or agent accepts it.

(b) AREA UPDATES — candidate spec-sync bullets, each citing its cell

areas: from the scribing stamp for "sweep-at-every-door" — .bee/logs/scribing-runs.jsonl's most recent entry (2026-08-13T15:05:39.951Z), the work item declares no bee.areas.

area workflow-state:
  - [sad-1] sweep is now caller-aware (D6 self-exclusion), writes the D4 blocked verdict with trace.blocked_reason, and never crosses the store boundary (D5) — feature-wide sync per the scribing stamp, 3 file(s) changed (trace .bee/cells/sad-1.json)
  - [sad-2] bee orient now sweeps expired claims (D1/D6), self-excluding its own caller and declining when unresolvable; registry text corrected — feature-wide sync per the scribing stamp, 3 file(s) changed (trace .bee/cells/sad-2.json)

(c) PATTERN CANDIDATES — candidate bee.pattern concepts, bee.polarity pitfall

None: no capped cell trace carries a deviation or a failure signature.

knowledge promote: 2 capped cell(s) mined, 1 delivery draft, 2 area bullet(s), 0 pattern candidate(s), 0 file(s) written.