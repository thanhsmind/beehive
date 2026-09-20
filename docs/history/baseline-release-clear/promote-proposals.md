promote proposal for work item "baseline-release-clear" (.bee/logs/scribing-runs.jsonl + .bee/lanes/baseline-release-clear.json) — 1 capped cell(s): brc-1
anchor: ledger — .bee/logs/scribing-runs.jsonl, .bee/lanes/baseline-release-clear.json
PROPOSAL ONLY — nothing was written. Applying any section below is a human or agent decision.

(a) DELIVERY DRAFT — save as docs/knowledge/work/baseline-release-clear/delivery.md

---
type: bee.delivery
title: baseline-release-clear — delivery
description: "Delivery record proposed by bee knowledge promote for work item baseline-release-clear: 1 capped cell(s), 2 recorded deviation(s)."
timestamp: 2026-09-20
bee:
  id: baseline-release-clear-delivery
  lifecycle: active
  areas: [workflow-state]
  required_context: [.bee/logs/scribing-runs.jsonl, .bee/lanes/baseline-release-clear.json]
  sources: [.bee/logs/scribing-runs.jsonl, .bee/lanes/baseline-release-clear.json, .bee/cells/brc-1.json]
---

# baseline-release-clear — Delivery

## What shipped

- **brc-1** — Clear the cap baseline when a cell is released, proven red first (2 file(s) changed)

## Verify

Each cell below was capped only against a recorded passing verify result — bee refuses a cap without one.

- **brc-1** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee --bin bee cells` — 378 passed, 0 failed; and proven red first: with "baseline" removed from release_trace the new test fails on 'a released cell must not carry the previous run's baseline' (exit 101), with it restored …

## Deviations

- **brc-1** — followed the plan
- **brc-1** — sync-ack: No skill text is stale: this clears an internal trace field on release. worker-details.md already tells a worker the baseline is optional and what it means, and nothing in any skill describes what a RELEASED cell keeps — that is plumbing an agent never reads. The instruction half of this feature shipped with proof-honesty.

## Provenance

Proposed by `bee knowledge promote --work baseline-release-clear` from 1 capped cell trace(s) in `.bee/cells/` and the anchor `.bee/logs/scribing-runs.jsonl`, `.bee/lanes/baseline-release-clear.json`. Every line above is copied from a trace or from the work item; nothing here is curated truth until a human or agent accepts it.

(b) AREA UPDATES — candidate spec-sync bullets, each citing its cell

areas: from the scribing stamp for "baseline-release-clear" — .bee/logs/scribing-runs.jsonl's most recent entry (2026-09-20T18:56:22.473Z), the work item declares no bee.areas.

area workflow-state:
  - [brc-1] Clear the cap baseline when a cell is released, proven red first — feature-wide sync per the scribing stamp, 2 file(s) changed (trace .bee/cells/brc-1.json)

(c) PATTERN CANDIDATES — candidate bee.pattern concepts, bee.polarity pitfall

from cell brc-1 — save as docs/knowledge/patterns/baseline-release-clear-brc-1-pitfall.md

---
type: bee.pattern
title: baseline-release-clear cell brc-1 — pitfall candidate
description: "Pitfall candidate mined from cell brc-1's capped trace: followed the plan"
timestamp: 2026-09-20
bee:
  id: baseline-release-clear-brc-1-pitfall
  lifecycle: draft
  areas: [workflow-state]
  sources: [.bee/cells/brc-1.json]
  polarity: pitfall
---

# baseline-release-clear cell brc-1 — pitfall candidate

## What the cell did

Clear the cap baseline when a cell is released, proven red first

## Recorded evidence (verbatim from .bee/cells/brc-1.json)

- **deviation** — followed the plan
- **deviation** — sync-ack: No skill text is stale: this clears an internal trace field on release. worker-details.md already tells a worker the baseline is optional and what it means, and nothing in any skill describes what a RELEASED cell keeps — that is plumbing an agent never reads. The instruction half of this feature shipped with proof-honesty.

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

knowledge promote: 1 capped cell(s) mined, 1 delivery draft, 1 area bullet(s), 1 pattern candidate(s), 0 file(s) written.