promote proposal for work item "herding-worker-standalone" (docs/history/herding-worker-standalone/CONTEXT.md) — 1 capped cell(s): hws-1
anchor: history — docs/history/herding-worker-standalone/CONTEXT.md
PROPOSAL ONLY — nothing was written. Applying any section below is a human or agent decision.

(a) DELIVERY DRAFT — save as docs/knowledge/work/herding-worker-standalone/delivery.md

---
type: bee.delivery
title: herding-worker-standalone — delivery
description: "Delivery record proposed by bee knowledge promote for work item herding-worker-standalone: 1 capped cell(s), 0 recorded deviation(s)."
timestamp: 2026-08-20
bee:
  id: herding-worker-standalone-delivery
  lifecycle: active
  areas: [bee-herding]
  required_context: [docs/history/herding-worker-standalone/CONTEXT.md]
  sources: [docs/history/herding-worker-standalone/CONTEXT.md, .bee/cells/hws-1.json]
---

# herding-worker-standalone — Delivery

## What shipped

- **hws-1** — Herded workers stay out of the bee flow: brief contract, always-on env marker, hook kill-switch (3 file(s) changed)

## Verify

Each cell below was capped only against a recorded passing verify result — bee refuses a cap without one.

- **hws-1** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml herding hooks`

## Deviations

None recorded in the capped cell traces.

## Provenance

Proposed by `bee knowledge promote --work herding-worker-standalone` from 1 capped cell trace(s) in `.bee/cells/` and the anchor `docs/history/herding-worker-standalone/CONTEXT.md`. Every line above is copied from a trace or from the work item; nothing here is curated truth until a human or agent accepts it.

(b) AREA UPDATES — candidate spec-sync bullets, each citing its cell

areas: from the scribing stamp for "herding-worker-standalone" — .bee/logs/scribing-runs.jsonl's most recent entry (2026-08-20T11:32:05.944Z), the work item declares no bee.areas.

area bee-herding:
  - [hws-1] Herded workers stay out of the bee flow: brief contract, always-on env marker, hook kill-switch — feature-wide sync per the scribing stamp, 3 file(s) changed (trace .bee/cells/hws-1.json)

(c) PATTERN CANDIDATES — candidate bee.pattern concepts, bee.polarity pitfall

None: no capped cell trace carries a deviation or a failure signature.

knowledge promote: 1 capped cell(s) mined, 1 delivery draft, 1 area bullet(s), 0 pattern candidate(s), 0 file(s) written.