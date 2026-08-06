promote proposal for work item "feature-swap-door" (.bee/logs/scribing-runs.jsonl) — 1 capped cell(s): fsd-1
anchor: ledger — .bee/logs/scribing-runs.jsonl
PROPOSAL ONLY — nothing was written. Applying any section below is a human or agent decision.

(a) DELIVERY DRAFT — save as docs/knowledge/work/feature-swap-door/delivery.md

---
type: bee.delivery
title: feature-swap-door — delivery
description: "Delivery record proposed by bee knowledge promote for work item feature-swap-door: 1 capped cell(s), 0 recorded deviation(s)."
timestamp: 2026-08-06
bee:
  id: feature-swap-door-delivery
  lifecycle: active
  areas: [workflow-state]
  required_context: [.bee/logs/scribing-runs.jsonl]
  sources: [.bee/logs/scribing-runs.jsonl, .bee/cells/fsd-1.json]
---

# feature-swap-door — Delivery

## What shipped

- **fsd-1** — Ported the feature-swap scribing-debt door natively (scribing_debt_swap_door), replacing both Node-delegate returns; 7 new tests cover refusal, the three non-swap shapes, the waiver, the capture-deferral escape, and the two lane-path regressions. (2 file(s) changed)

## Verify

Each cell below was capped only against a recorded passing verify result — bee refuses a cap without one.

- **fsd-1** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml`

## Deviations

None recorded in the capped cell traces.

## Provenance

Proposed by `bee knowledge promote --work feature-swap-door` from 1 capped cell trace(s) in `.bee/cells/` and the anchor `.bee/logs/scribing-runs.jsonl`. Every line above is copied from a trace or from the work item; nothing here is curated truth until a human or agent accepts it.

(b) AREA UPDATES — candidate spec-sync bullets, each citing its cell

areas: from the scribing stamp for "feature-swap-door" — .bee/logs/scribing-runs.jsonl's most recent entry (2026-08-06T02:36:08.226Z), the work item declares no bee.areas.

area workflow-state:
  - [fsd-1] Ported the feature-swap scribing-debt door natively (scribing_debt_swap_door), replacing both Node-delegate returns; 7 new tests cover refusal, the three non-swap shapes, the waiver, the capture-deferral escape, and the two lane-path regressions. — feature-wide sync per the scribing stamp, 2 file(s) changed (trace .bee/cells/fsd-1.json)

(c) PATTERN CANDIDATES — candidate bee.pattern concepts, bee.polarity pitfall

None: no capped cell trace carries a deviation or a failure signature.

knowledge promote: 1 capped cell(s) mined, 1 delivery draft, 1 area bullet(s), 0 pattern candidate(s), 0 file(s) written.