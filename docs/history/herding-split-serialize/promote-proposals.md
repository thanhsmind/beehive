promote proposal for work item "herding-split-serialize" (.bee/logs/scribing-runs.jsonl + .bee/lanes/herding-split-serialize.json + docs/history/herding-split-serialize/promote-proposals.md) — 2 capped cell(s): hss-1, hss-2
anchor: ledger — .bee/logs/scribing-runs.jsonl, .bee/lanes/herding-split-serialize.json, docs/history/herding-split-serialize/promote-proposals.md
PROPOSAL ONLY — nothing was written. Applying any section below is a human or agent decision.

(a) DELIVERY DRAFT — save as docs/knowledge/work/herding-split-serialize/delivery.md

---
type: bee.delivery
title: herding-split-serialize — delivery
description: "Delivery record proposed by bee knowledge promote for work item herding-split-serialize: 2 capped cell(s), 0 recorded deviation(s)."
timestamp: 2026-08-21
bee:
  id: herding-split-serialize-delivery
  lifecycle: active
  areas: [bee-herding]
  required_context: [.bee/logs/scribing-runs.jsonl, .bee/lanes/herding-split-serialize.json, docs/history/herding-split-serialize/promote-proposals.md]
  sources: [.bee/logs/scribing-runs.jsonl, .bee/lanes/herding-split-serialize.json, docs/history/herding-split-serialize/promote-proposals.md, .bee/cells/archive/herding-split-serialize/hss-1.json, .bee/cells/archive/herding-split-serialize/hss-2.json]
---

# herding-split-serialize — Delivery

## What shipped

- **hss-1** — Cross-process pane-split file lock module added, with acquire wait budget, two-tier stale takeover, identity-checked Drop release, and 5 unit tests (2 file(s) changed)
- **hss-2** — Pane split now holds a cross-process lock across the layout read and the split, so concurrent spawns stop all splitting right (2 file(s) changed)

## Verify

Each cell below was capped only against a recorded passing verify result — bee refuses a cap without one.

- **hss-1** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml herding::split_lock`
- **hss-2** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml herding::`

## Deviations

None recorded in the capped cell traces.

## Provenance

Proposed by `bee knowledge promote --work herding-split-serialize` from 2 capped cell trace(s) in `.bee/cells/` and the anchor `.bee/logs/scribing-runs.jsonl`, `.bee/lanes/herding-split-serialize.json`, `docs/history/herding-split-serialize/promote-proposals.md`. Every line above is copied from a trace or from the work item; nothing here is curated truth until a human or agent accepts it.

(b) AREA UPDATES — candidate spec-sync bullets, each citing its cell

areas: from the scribing stamp for "herding-split-serialize" — .bee/logs/scribing-runs.jsonl's most recent entry (2026-08-21T13:59:28.075Z), the work item declares no bee.areas.

area bee-herding:
  - [hss-2] Pane split now holds a cross-process lock across the layout read and the split, so concurrent spawns stop all splitting right — feature-wide sync per the scribing stamp, 2 file(s) changed (trace .bee/cells/archive/herding-split-serialize/hss-2.json)

(c) PATTERN CANDIDATES — candidate bee.pattern concepts, bee.polarity pitfall

None: no capped cell trace carries a deviation or a failure signature.

knowledge promote: 2 capped cell(s) mined, 1 delivery draft, 1 area bullet(s), 0 pattern candidate(s), 0 file(s) written.