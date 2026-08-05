promote proposal for work item "ledger-anchor" (.bee/logs/scribing-runs.jsonl + .bee/lanes/ledger-anchor.json) — 1 capped cell(s): la-1
anchor: ledger — .bee/logs/scribing-runs.jsonl, .bee/lanes/ledger-anchor.json
PROPOSAL ONLY — nothing was written. Applying any section below is a human or agent decision.

(a) DELIVERY DRAFT — save as docs/knowledge/work/ledger-anchor/delivery.md

---
type: bee.delivery
title: ledger-anchor — delivery
description: "Delivery record proposed by bee knowledge promote for work item ledger-anchor: 1 capped cell(s), 0 recorded deviation(s)."
timestamp: 2026-08-05
bee:
  id: ledger-anchor-delivery
  lifecycle: active
  areas: [okf-profile]
  required_context: [.bee/logs/scribing-runs.jsonl, .bee/lanes/ledger-anchor.json]
  sources: [.bee/logs/scribing-runs.jsonl, .bee/lanes/ledger-anchor.json, .bee/cells/la-1.json]
---

# ledger-anchor — Delivery

## What shipped

- **la-1** — resolve_anchor gains a third ledger arm reached when no work-item or docs/history anchor exists; both demos resolve promote-reach through it (5 file(s) changed)

## Verify

Each cell below was capped only against a recorded passing verify result — bee refuses a cap without one.

- **la-1** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml`

## Deviations

None recorded in the capped cell traces.

## Provenance

Proposed by `bee knowledge promote --work ledger-anchor` from 1 capped cell trace(s) in `.bee/cells/` and the anchor `.bee/logs/scribing-runs.jsonl`, `.bee/lanes/ledger-anchor.json`. Every line above is copied from a trace or from the work item; nothing here is curated truth until a human or agent accepts it.

(b) AREA UPDATES — candidate spec-sync bullets, each citing its cell

areas: from the scribing stamp for "ledger-anchor" — .bee/logs/scribing-runs.jsonl's most recent entry (2026-08-05T07:28:56.350Z), the work item declares no bee.areas.

area okf-profile:
  - [la-1] resolve_anchor gains a third ledger arm reached when no work-item or docs/history anchor exists; both demos resolve promote-reach through it — feature-wide sync per the scribing stamp, 5 file(s) changed (trace .bee/cells/la-1.json)

(c) PATTERN CANDIDATES — candidate bee.pattern concepts, bee.polarity pitfall

None: no capped cell trace carries a deviation or a failure signature.

knowledge promote: 1 capped cell(s) mined, 1 delivery draft, 1 area bullet(s), 0 pattern candidate(s), 0 file(s) written.