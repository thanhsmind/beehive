promote proposal for work item "knowledge-show-new" (.bee/lanes/knowledge-show-new.json + docs/history/knowledge-show-new/promote-proposals.md) — 1 capped cell(s): ksn-1
anchor: ledger — .bee/lanes/knowledge-show-new.json, docs/history/knowledge-show-new/promote-proposals.md
PROPOSAL ONLY — nothing was written. Applying any section below is a human or agent decision.

(a) DELIVERY DRAFT — save as docs/knowledge/work/knowledge-show-new/delivery.md

---
type: bee.delivery
title: knowledge-show-new — delivery
description: "Delivery record proposed by bee knowledge promote for work item knowledge-show-new: 1 capped cell(s), 0 recorded deviation(s)."
timestamp: 2026-09-06
bee:
  id: knowledge-show-new-delivery
  lifecycle: active
  required_context: [.bee/lanes/knowledge-show-new.json, docs/history/knowledge-show-new/promote-proposals.md]
  sources: [.bee/lanes/knowledge-show-new.json, docs/history/knowledge-show-new/promote-proposals.md, .bee/cells/archive/knowledge-show-new/ksn-1.json]
---

# knowledge-show-new — Delivery

## What shipped

- **ksn-1** — knowledge show and knowledge new verbs landed (4 file(s) changed)

## Verify

Each cell below was capped only against a recorded passing verify result — bee refuses a cap without one.

- **ksn-1** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --no-fail-fast --manifest-path packages/bee-rs/Cargo.toml knowledge -- --nocapture 2>&1 | tail -20; then cargo test registry_contracts and catalog; then .bee/bin/bee knowledge check on the repo bundle stays OK`

## Deviations

None recorded in the capped cell traces.

## Provenance

Proposed by `bee knowledge promote --work knowledge-show-new` from 1 capped cell trace(s) in `.bee/cells/` and the anchor `.bee/lanes/knowledge-show-new.json`, `docs/history/knowledge-show-new/promote-proposals.md`. Every line above is copied from a trace or from the work item; nothing here is curated truth until a human or agent accepts it.

(b) AREA UPDATES — candidate spec-sync bullets, each citing its cell

None: the work item declares no bee.areas, so there is no area to sync (D19).

(c) PATTERN CANDIDATES — candidate bee.pattern concepts, bee.polarity pitfall

None: no capped cell trace carries a deviation or a failure signature.

knowledge promote: 1 capped cell(s) mined, 1 delivery draft, 0 area bullet(s), 0 pattern candidate(s), 0 file(s) written.