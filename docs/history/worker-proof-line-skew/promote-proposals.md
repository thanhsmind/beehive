promote proposal for work item "worker-proof-line-skew" (.bee/lanes/worker-proof-line-skew.json) — 1 capped cell(s): wpls-1
anchor: ledger — .bee/lanes/worker-proof-line-skew.json
PROPOSAL ONLY — nothing was written. Applying any section below is a human or agent decision.

(a) DELIVERY DRAFT — save as docs/knowledge/work/worker-proof-line-skew/delivery.md

---
type: bee.delivery
title: worker-proof-line-skew — delivery
description: "Delivery record proposed by bee knowledge promote for work item worker-proof-line-skew: 1 capped cell(s), 0 recorded deviation(s)."
timestamp: 2026-08-21
bee:
  id: worker-proof-line-skew-delivery
  lifecycle: active
  required_context: [.bee/lanes/worker-proof-line-skew.json]
  sources: [.bee/lanes/worker-proof-line-skew.json, .bee/cells/wpls-1.json]
---

# worker-proof-line-skew — Delivery

## What shipped

- **wpls-1** — worker prompt now states finish records proof and runs no tests; planners author narrow verify commands (3 file(s) changed)

## Verify

Each cell below was capped only against a recorded passing verify result — bee refuses a cap without one.

- **wpls-1** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee prompt && .bee/bin/bee dev release-manifest --check`

## Deviations

None recorded in the capped cell traces.

## Provenance

Proposed by `bee knowledge promote --work worker-proof-line-skew` from 1 capped cell trace(s) in `.bee/cells/` and the anchor `.bee/lanes/worker-proof-line-skew.json`. Every line above is copied from a trace or from the work item; nothing here is curated truth until a human or agent accepts it.

(b) AREA UPDATES — candidate spec-sync bullets, each citing its cell

None: the work item declares no bee.areas, so there is no area to sync (D19).

(c) PATTERN CANDIDATES — candidate bee.pattern concepts, bee.polarity pitfall

None: no capped cell trace carries a deviation or a failure signature.

knowledge promote: 1 capped cell(s) mined, 1 delivery draft, 0 area bullet(s), 0 pattern candidate(s), 0 file(s) written.