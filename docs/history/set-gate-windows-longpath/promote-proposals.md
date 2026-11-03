promote proposal for work item "set-gate-windows-longpath" (docs/history/set-gate-windows-longpath/plan.md) — 1 capped cell(s): sgl-1
anchor: history — docs/history/set-gate-windows-longpath/plan.md
PROPOSAL ONLY — nothing was written. Applying any section below is a human or agent decision.

(a) DELIVERY DRAFT — save as docs/knowledge/work/set-gate-windows-longpath/delivery.md

---
type: bee.delivery
title: set-gate-windows-longpath — delivery
description: "Delivery record proposed by bee knowledge promote for work item set-gate-windows-longpath: 1 capped cell(s), 2 recorded deviation(s)."
timestamp: 2026-10-07
bee:
  id: set-gate-windows-longpath-delivery
  lifecycle: active
  areas: [workflow-state]
  required_context: [docs/history/set-gate-windows-longpath/plan.md]
  sources: [docs/history/set-gate-windows-longpath/plan.md, .bee/cells/sgl-1.json]
---

# set-gate-windows-longpath — Delivery

## What shipped

- **sgl-1** — both control-plane assertions compare against the canonical main path (1 file(s) changed)

## Verify

Each cell below was capped only against a recorded passing verify result — bee refuses a cap without one.

- **sgl-1** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml --bin bee verbs::state_group::set_gate` — 65 set_gate tests on Linux; the Windows-only red cannot run here, Windows CI is the proof

## Deviations

- **sgl-1** — followed the plan
- **sgl-1** — sync-ack: test-only change

## Provenance

Proposed by `bee knowledge promote --work set-gate-windows-longpath` from 1 capped cell trace(s) in `.bee/cells/` and the anchor `docs/history/set-gate-windows-longpath/plan.md`. Every line above is copied from a trace or from the work item; nothing here is curated truth until a human or agent accepts it.

(b) AREA UPDATES — candidate spec-sync bullets, each citing its cell

areas: from the scribing stamp for "set-gate-windows-longpath" — .bee/logs/scribing-runs.jsonl's most recent entry (2026-10-07T07:02:19.238Z), the work item declares no bee.areas.

area workflow-state:
  (no capped behavior_change cell exists for this feature)

(c) PATTERN CANDIDATES — candidate bee.pattern concepts, bee.polarity pitfall

from cell sgl-1 — save as docs/knowledge/patterns/set-gate-windows-longpath-sgl-1-pitfall.md

---
type: bee.pattern
title: set-gate-windows-longpath cell sgl-1 — pitfall candidate
description: "Pitfall candidate mined from cell sgl-1's capped trace: followed the plan"
timestamp: 2026-10-07
bee:
  id: set-gate-windows-longpath-sgl-1-pitfall
  lifecycle: draft
  areas: [workflow-state]
  sources: [.bee/cells/sgl-1.json]
  polarity: pitfall
---

# set-gate-windows-longpath cell sgl-1 — pitfall candidate

## What the cell did

both control-plane assertions compare against the canonical main path

## Recorded evidence (verbatim from .bee/cells/sgl-1.json)

- **deviation** — followed the plan
- **deviation** — sync-ack: test-only change

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

knowledge promote: 1 capped cell(s) mined, 1 delivery draft, 0 area bullet(s), 1 pattern candidate(s), 0 file(s) written.