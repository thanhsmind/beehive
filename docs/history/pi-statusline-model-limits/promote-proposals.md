promote proposal for work item "pi-statusline-model-limits" (.bee/lanes/pi-statusline-model-limits.json) — 1 capped cell(s): psml-1
anchor: ledger — .bee/lanes/pi-statusline-model-limits.json
PROPOSAL ONLY — nothing was written. Applying any section below is a human or agent decision.

(a) DELIVERY DRAFT — save as docs/knowledge/work/pi-statusline-model-limits/delivery.md

---
type: bee.delivery
title: pi-statusline-model-limits — delivery
description: "Delivery record proposed by bee knowledge promote for work item pi-statusline-model-limits: 1 capped cell(s), 0 recorded deviation(s)."
timestamp: 2026-09-18
bee:
  id: pi-statusline-model-limits-delivery
  lifecycle: active
  required_context: [.bee/lanes/pi-statusline-model-limits.json]
  sources: [.bee/lanes/pi-statusline-model-limits.json, .bee/cells/psml-1.json]
---

# pi-statusline-model-limits — Delivery

## What shipped

- **psml-1** — Show 5h and weekly model rate limits in Pi extension statusline (2 file(s) changed)

## Verify

Each cell below was capped only against a recorded passing verify result — bee refuses a cap without one.

- **psml-1** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee --test pi_plugin_contracts model_usage_status` — touched .pi/extensions/bee-guard.ts

## Deviations

None recorded in the capped cell traces.

## Provenance

Proposed by `bee knowledge promote --work pi-statusline-model-limits` from 1 capped cell trace(s) in `.bee/cells/` and the anchor `.bee/lanes/pi-statusline-model-limits.json`. Every line above is copied from a trace or from the work item; nothing here is curated truth until a human or agent accepts it.

(b) AREA UPDATES — candidate spec-sync bullets, each citing its cell

None: the work item declares no bee.areas, so there is no area to sync (D19).

(c) PATTERN CANDIDATES — candidate bee.pattern concepts, bee.polarity pitfall

None: no capped cell trace carries a deviation or a failure signature.

knowledge promote: 1 capped cell(s) mined, 1 delivery draft, 0 area bullet(s), 0 pattern candidate(s), 0 file(s) written.