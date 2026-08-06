promote proposal for work item "route-identity" (.bee/logs/scribing-runs.jsonl) — 1 capped cell(s): rti-1
anchor: ledger — .bee/logs/scribing-runs.jsonl
PROPOSAL ONLY — nothing was written. Applying any section below is a human or agent decision.

(a) DELIVERY DRAFT — save as docs/knowledge/work/route-identity/delivery.md

---
type: bee.delivery
title: route-identity — delivery
description: "Delivery record proposed by bee knowledge promote for work item route-identity: 1 capped cell(s), 0 recorded deviation(s)."
timestamp: 2026-08-06
bee:
  id: route-identity-delivery
  lifecycle: active
  areas: [workflow-state]
  required_context: [.bee/logs/scribing-runs.jsonl]
  sources: [.bee/logs/scribing-runs.jsonl, .bee/cells/rti-1.json]
---

# route-identity — Delivery

## What shipped

- **rti-1** — Route now carries the feature it was recorded for; start_default drops a stale route; run_route only checks demotion against a same-feature route (3 file(s) changed)

## Verify

Each cell below was capped only against a recorded passing verify result — bee refuses a cap without one.

- **rti-1** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml`

## Deviations

None recorded in the capped cell traces.

## Provenance

Proposed by `bee knowledge promote --work route-identity` from 1 capped cell trace(s) in `.bee/cells/` and the anchor `.bee/logs/scribing-runs.jsonl`. Every line above is copied from a trace or from the work item; nothing here is curated truth until a human or agent accepts it.

(b) AREA UPDATES — candidate spec-sync bullets, each citing its cell

areas: from the scribing stamp for "route-identity" — .bee/logs/scribing-runs.jsonl's most recent entry (2026-08-06T03:21:59.084Z), the work item declares no bee.areas.

area workflow-state:
  - [rti-1] Route now carries the feature it was recorded for; start_default drops a stale route; run_route only checks demotion against a same-feature route — feature-wide sync per the scribing stamp, 3 file(s) changed (trace .bee/cells/rti-1.json)

(c) PATTERN CANDIDATES — candidate bee.pattern concepts, bee.polarity pitfall

None: no capped cell trace carries a deviation or a failure signature.

knowledge promote: 1 capped cell(s) mined, 1 delivery draft, 1 area bullet(s), 0 pattern candidate(s), 0 file(s) written.