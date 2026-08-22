promote proposal for work item "deviation-one-list" (.bee/lanes/deviation-one-list.json) — 1 capped cell(s): dol-1
anchor: ledger — .bee/lanes/deviation-one-list.json
PROPOSAL ONLY — nothing was written. Applying any section below is a human or agent decision.

(a) DELIVERY DRAFT — save as docs/knowledge/work/deviation-one-list/delivery.md

---
type: bee.delivery
title: deviation-one-list — delivery
description: "Delivery record proposed by bee knowledge promote for work item deviation-one-list: 1 capped cell(s), 1 recorded deviation(s)."
timestamp: 2026-08-22
bee:
  id: deviation-one-list-delivery
  lifecycle: active
  required_context: [.bee/lanes/deviation-one-list.json]
  sources: [.bee/lanes/deviation-one-list.json, .bee/cells/dol-1.json]
---

# deviation-one-list — Delivery

## What shipped

- **dol-1** — A deviation a worker records in its report now reaches trace.deviations with no hand-copying, in string and non-string form alike, deduped by the same rendering the miner uses. (2 file(s) changed)

## Verify

Each cell below was capped only against a recorded passing verify result — bee refuses a cap without one.

- **dol-1** — `PATH="$HOME/.cargo/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml cells`

## Deviations

- **dol-1** — Judge raised two residuals across three rounds: a report-side non-string deviation was silently dropped while the same shape from a deviations-file was mined, and the first dedup test passed against the pre-fix code because a skip leaves the same array a dedup does. Both closed and independently reproduced.

## Provenance

Proposed by `bee knowledge promote --work deviation-one-list` from 1 capped cell trace(s) in `.bee/cells/` and the anchor `.bee/lanes/deviation-one-list.json`. Every line above is copied from a trace or from the work item; nothing here is curated truth until a human or agent accepts it.

(b) AREA UPDATES — candidate spec-sync bullets, each citing its cell

None: the work item declares no bee.areas, so there is no area to sync (D19).

(c) PATTERN CANDIDATES — candidate bee.pattern concepts, bee.polarity pitfall

from cell dol-1 — save as docs/knowledge/patterns/deviation-one-list-dol-1-pitfall.md

---
type: bee.pattern
title: deviation-one-list cell dol-1 — pitfall candidate
description: "Pitfall candidate mined from cell dol-1's capped trace: Judge raised two residuals across three rounds: a report-side non-string deviation was silently dropped while the same shape from a deviations-file was mined, …"
timestamp: 2026-08-22
bee:
  id: deviation-one-list-dol-1-pitfall
  lifecycle: draft
  sources: [.bee/cells/dol-1.json]
  polarity: pitfall
---

# deviation-one-list cell dol-1 — pitfall candidate

## What the cell did

A deviation a worker records in its report now reaches trace.deviations with no hand-copying, in string and non-string form alike, deduped by the same rendering the miner uses.

## Recorded evidence (verbatim from .bee/cells/dol-1.json)

- **deviation** — Judge raised two residuals across three rounds: a report-side non-string deviation was silently dropped while the same shape from a deviations-file was mined, and the first dedup test passed against the pre-fix code because a skip leaves the same array a dedup does. Both closed and independently reproduced.

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

knowledge promote: 1 capped cell(s) mined, 1 delivery draft, 0 area bullet(s), 1 pattern candidate(s), 0 file(s) written.