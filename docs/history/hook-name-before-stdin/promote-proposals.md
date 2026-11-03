promote proposal for work item "hook-name-before-stdin" (.bee/lanes/hook-name-before-stdin.json) — 1 capped cell(s): hnbs-check-name-first
anchor: ledger — .bee/lanes/hook-name-before-stdin.json
PROPOSAL ONLY — nothing was written. Applying any section below is a human or agent decision.

(a) DELIVERY DRAFT — save as docs/knowledge/work/hook-name-before-stdin/delivery.md

---
type: bee.delivery
title: hook-name-before-stdin — delivery
description: "Delivery record proposed by bee knowledge promote for work item hook-name-before-stdin: 1 capped cell(s), 1 recorded deviation(s)."
timestamp: 2026-08-30
bee:
  id: hook-name-before-stdin-delivery
  lifecycle: active
  required_context: [.bee/lanes/hook-name-before-stdin.json]
  sources: [.bee/lanes/hook-name-before-stdin.json, .bee/cells/hnbs-check-name-first.json]
---

# hook-name-before-stdin — Delivery

## What shipped

- **hnbs-check-name-first** — unknown hook names are refused before the stdin read; the hang path is gone (1 file(s) changed)

## Verify

Each cell below was capped only against a recorded passing verify result — bee refuses a cap without one.

- **hnbs-check-name-first** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml --bin bee hooks::tests`

## Deviations

- **hnbs-check-name-first** — followed the plan

## Provenance

Proposed by `bee knowledge promote --work hook-name-before-stdin` from 1 capped cell trace(s) in `.bee/cells/` and the anchor `.bee/lanes/hook-name-before-stdin.json`. Every line above is copied from a trace or from the work item; nothing here is curated truth until a human or agent accepts it.

(b) AREA UPDATES — candidate spec-sync bullets, each citing its cell

None: the work item declares no bee.areas, so there is no area to sync (D19).

(c) PATTERN CANDIDATES — candidate bee.pattern concepts, bee.polarity pitfall

from cell hnbs-check-name-first — save as docs/knowledge/patterns/hook-name-before-stdin-hnbs-check-name-first-pitfall.md

---
type: bee.pattern
title: hook-name-before-stdin cell hnbs-check-name-first — pitfall candidate
description: "Pitfall candidate mined from cell hnbs-check-name-first's capped trace: followed the plan"
timestamp: 2026-08-30
bee:
  id: hook-name-before-stdin-hnbs-check-name-first-pitfall
  lifecycle: draft
  sources: [.bee/cells/hnbs-check-name-first.json]
  polarity: pitfall
---

# hook-name-before-stdin cell hnbs-check-name-first — pitfall candidate

## What the cell did

unknown hook names are refused before the stdin read; the hang path is gone

## Recorded evidence (verbatim from .bee/cells/hnbs-check-name-first.json)

- **deviation** — followed the plan

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

knowledge promote: 1 capped cell(s) mined, 1 delivery draft, 0 area bullet(s), 1 pattern candidate(s), 0 file(s) written.