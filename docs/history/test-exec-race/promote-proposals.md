promote proposal for work item "test-exec-race" (.bee/lanes/test-exec-race.json) — 1 capped cell(s): ter-1
anchor: ledger — .bee/lanes/test-exec-race.json
PROPOSAL ONLY — nothing was written. Applying any section below is a human or agent decision.

(a) DELIVERY DRAFT — save as docs/knowledge/work/test-exec-race/delivery.md

---
type: bee.delivery
title: test-exec-race — delivery
description: "Delivery record proposed by bee knowledge promote for work item test-exec-race: 1 capped cell(s), 1 recorded deviation(s)."
timestamp: 2026-08-25
bee:
  id: test-exec-race-delivery
  lifecycle: active
  required_context: [.bee/lanes/test-exec-race.json]
  sources: [.bee/lanes/test-exec-race.json, .bee/cells/ter-1.json]
---

# test-exec-race — Delivery

## What shipped

- **ter-1** — Doctor test helpers install their exec target by rename, so no process ever holds it open for writing and the ETXTBSY race has no precondition (1 file(s) changed)

## Verify

Each cell below was capped only against a recorded passing verify result — bee refuses a cap without one.

- **ter-1** — `cargo test --release --manifest-path packages/bee-rs/Cargo.toml doctor::`

## Deviations

- **ter-1** — Ran inline rather than through a dispatched worker: tiny lane, one test file, no product code, and the change plus its remedy were fully specified before execution

## Provenance

Proposed by `bee knowledge promote --work test-exec-race` from 1 capped cell trace(s) in `.bee/cells/` and the anchor `.bee/lanes/test-exec-race.json`. Every line above is copied from a trace or from the work item; nothing here is curated truth until a human or agent accepts it.

(b) AREA UPDATES — candidate spec-sync bullets, each citing its cell

None: the work item declares no bee.areas, so there is no area to sync (D19).

(c) PATTERN CANDIDATES — candidate bee.pattern concepts, bee.polarity pitfall

from cell ter-1 — save as docs/knowledge/patterns/test-exec-race-ter-1-pitfall.md

---
type: bee.pattern
title: test-exec-race cell ter-1 — pitfall candidate
description: "Pitfall candidate mined from cell ter-1's capped trace: Ran inline rather than through a dispatched worker: tiny lane, one test file, no product code, and the change plus its remedy were fully specified before execu…"
timestamp: 2026-08-25
bee:
  id: test-exec-race-ter-1-pitfall
  lifecycle: draft
  sources: [.bee/cells/ter-1.json]
  polarity: pitfall
---

# test-exec-race cell ter-1 — pitfall candidate

## What the cell did

Doctor test helpers install their exec target by rename, so no process ever holds it open for writing and the ETXTBSY race has no precondition

## Recorded evidence (verbatim from .bee/cells/ter-1.json)

- **deviation** — Ran inline rather than through a dispatched worker: tiny lane, one test file, no product code, and the change plus its remedy were fully specified before execution

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

knowledge promote: 1 capped cell(s) mined, 1 delivery draft, 0 area bullet(s), 1 pattern candidate(s), 0 file(s) written.