promote proposal for work item "dispatch-cell-id-flag" (docs/history/dispatch-cell-id-flag/CONTEXT.md) — 1 capped cell(s): dcif-1
anchor: history — docs/history/dispatch-cell-id-flag/CONTEXT.md
PROPOSAL ONLY — nothing was written. Applying any section below is a human or agent decision.

(a) DELIVERY DRAFT — save as docs/knowledge/work/dispatch-cell-id-flag/delivery.md

---
type: bee.delivery
title: dispatch-cell-id-flag — delivery
description: "Delivery record proposed by bee knowledge promote for work item dispatch-cell-id-flag: 1 capped cell(s), 2 recorded deviation(s)."
timestamp: 2026-09-20
bee:
  id: dispatch-cell-id-flag-delivery
  lifecycle: active
  areas: [dispatch]
  required_context: [docs/history/dispatch-cell-id-flag/CONTEXT.md]
  sources: [docs/history/dispatch-cell-id-flag/CONTEXT.md, .bee/cells/dcif-1.json]
---

# dispatch-cell-id-flag — Delivery

## What shipped

- **dcif-1** — Pass --cell-id on door-prepared cell dispatches, making the cap refusal reachable (2 file(s) changed)

## Verify

Each cell below was capped only against a recorded passing verify result — bee refuses a cap without one.

- **dcif-1** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee --bin bee drivers` — 415 passed, 0 failed, 8 ignored. Proven red first: with the --cell-id block stripped from prepare.rs the new test fails at tests.rs:1546 with left ".bee/bin/bee herding run --task-file - --json" vs r…

## Deviations

- **dcif-1** — followed the plan
- **dcif-1** — sync-ack: No skill text is stale: this is the dispatch door's own command composition. bee-swarming already tells the orchestrator to run exactly the tool and payload `dispatch prepare` returns, and nothing in any skill spells out that command's flags — an agent never types it by hand.

## Provenance

Proposed by `bee knowledge promote --work dispatch-cell-id-flag` from 1 capped cell trace(s) in `.bee/cells/` and the anchor `docs/history/dispatch-cell-id-flag/CONTEXT.md`. Every line above is copied from a trace or from the work item; nothing here is curated truth until a human or agent accepts it.

(b) AREA UPDATES — candidate spec-sync bullets, each citing its cell

areas: from the scribing stamp for "dispatch-cell-id-flag" — .bee/logs/scribing-runs.jsonl's most recent entry (2026-09-20T23:18:32.688Z), the work item declares no bee.areas.

area dispatch:
  - [dcif-1] Pass --cell-id on door-prepared cell dispatches, making the cap refusal reachable — feature-wide sync per the scribing stamp, 2 file(s) changed (trace .bee/cells/dcif-1.json)

(c) PATTERN CANDIDATES — candidate bee.pattern concepts, bee.polarity pitfall

from cell dcif-1 — save as docs/knowledge/patterns/dispatch-cell-id-flag-dcif-1-pitfall.md

---
type: bee.pattern
title: dispatch-cell-id-flag cell dcif-1 — pitfall candidate
description: "Pitfall candidate mined from cell dcif-1's capped trace: followed the plan"
timestamp: 2026-09-20
bee:
  id: dispatch-cell-id-flag-dcif-1-pitfall
  lifecycle: draft
  areas: [dispatch]
  sources: [.bee/cells/dcif-1.json]
  polarity: pitfall
---

# dispatch-cell-id-flag cell dcif-1 — pitfall candidate

## What the cell did

Pass --cell-id on door-prepared cell dispatches, making the cap refusal reachable

## Recorded evidence (verbatim from .bee/cells/dcif-1.json)

- **deviation** — followed the plan
- **deviation** — sync-ack: No skill text is stale: this is the dispatch door's own command composition. bee-swarming already tells the orchestrator to run exactly the tool and payload `dispatch prepare` returns, and nothing in any skill spells out that command's flags — an agent never types it by hand.

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

knowledge promote: 1 capped cell(s) mined, 1 delivery draft, 1 area bullet(s), 1 pattern candidate(s), 0 file(s) written.