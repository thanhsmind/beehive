promote proposal for work item "trigger-cache-path" (.bee/lanes/trigger-cache-path.json) — 1 capped cell(s): tcp-1
anchor: ledger — .bee/lanes/trigger-cache-path.json
PROPOSAL ONLY — nothing was written. Applying any section below is a human or agent decision.

(a) DELIVERY DRAFT — save as docs/knowledge/work/trigger-cache-path/delivery.md

---
type: bee.delivery
title: trigger-cache-path — delivery
description: "Delivery record proposed by bee knowledge promote for work item trigger-cache-path: 1 capped cell(s), 1 recorded deviation(s)."
timestamp: 2026-09-21
bee:
  id: trigger-cache-path-delivery
  lifecycle: active
  required_context: [.bee/lanes/trigger-cache-path.json]
  sources: [.bee/lanes/trigger-cache-path.json, .bee/cells/tcp-1.json]
---

# trigger-cache-path — Delivery

## What shipped

- **tcp-1** — prompt trigger HEAD cache moved to .bee/cache/triggers-last-eval-head (1 file(s) changed)

## Verify

Each cell below was capped only against a recorded passing verify result — bee refuses a cap without one.

- **tcp-1** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee --bin bee triggers` — 17 triggers tests; due_count_for_prompt test seen red at the new path first

## Deviations

- **tcp-1** — followed the plan

## Provenance

Proposed by `bee knowledge promote --work trigger-cache-path` from 1 capped cell trace(s) in `.bee/cells/` and the anchor `.bee/lanes/trigger-cache-path.json`. Every line above is copied from a trace or from the work item; nothing here is curated truth until a human or agent accepts it.

(b) AREA UPDATES — candidate spec-sync bullets, each citing its cell

None: the work item declares no bee.areas, so there is no area to sync (D19).

(c) PATTERN CANDIDATES — candidate bee.pattern concepts, bee.polarity pitfall

from cell tcp-1 — save as docs/knowledge/patterns/trigger-cache-path-tcp-1-pitfall.md

---
type: bee.pattern
title: trigger-cache-path cell tcp-1 — pitfall candidate
description: "Pitfall candidate mined from cell tcp-1's capped trace: followed the plan"
timestamp: 2026-09-21
bee:
  id: trigger-cache-path-tcp-1-pitfall
  lifecycle: draft
  sources: [.bee/cells/tcp-1.json]
  polarity: pitfall
---

# trigger-cache-path cell tcp-1 — pitfall candidate

## What the cell did

prompt trigger HEAD cache moved to .bee/cache/triggers-last-eval-head

## Recorded evidence (verbatim from .bee/cells/tcp-1.json)

- **deviation** — followed the plan

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

knowledge promote: 1 capped cell(s) mined, 1 delivery draft, 0 area bullet(s), 1 pattern candidate(s), 0 file(s) written.