promote proposal for work item "pi-worker-surface" (docs/history/pi-worker-surface/CONTEXT.md + docs/history/pi-worker-surface/plan.md) — 2 capped cell(s): pws-1, pws-2
anchor: history — docs/history/pi-worker-surface/CONTEXT.md, docs/history/pi-worker-surface/plan.md
PROPOSAL ONLY — nothing was written. Applying any section below is a human or agent decision.

(a) DELIVERY DRAFT — save as docs/knowledge/work/pi-worker-surface/delivery.md

---
type: bee.delivery
title: pi-worker-surface — delivery
description: "Delivery record proposed by bee knowledge promote for work item pi-worker-surface: 2 capped cell(s), 2 recorded deviation(s)."
timestamp: 2026-09-20
bee:
  id: pi-worker-surface-delivery
  lifecycle: active
  areas: [hook-runtime]
  required_context: [docs/history/pi-worker-surface/CONTEXT.md, docs/history/pi-worker-surface/plan.md]
  sources: [docs/history/pi-worker-surface/CONTEXT.md, docs/history/pi-worker-surface/plan.md, .bee/cells/pws-1.json, .bee/cells/pws-2.json]
---

# pi-worker-surface — Delivery

## What shipped

- **pws-1** — Registered terminating verdict tool on Pi belt and routed to write-guard (4 file(s) changed)
- **pws-2** — Draw in-flight workers in a widget below the editor with seat-named rows, D3 clearing, and no second timer (4 file(s) changed)

## Verify

Each cell below was capped only against a recorded passing verify result — bee refuses a cap without one.

- **pws-1** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee --test pi_plugin_contracts && .bee/bin/bee dev release-manifest --check` — touched bee-guard.ts and pi_plugin_contracts.rs
- **pws-2** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee --test pi_plugin_contracts && .bee/bin/bee dev release-manifest --check` — leader re-ran it independently after the merge: 87 passed, 0 failed (81 before this cell); node --check on the belt and release-manifest --check both exit 0

## Deviations

- **pws-1** — followed the plan
- **pws-2** — followed the plan

## Provenance

Proposed by `bee knowledge promote --work pi-worker-surface` from 2 capped cell trace(s) in `.bee/cells/` and the anchor `docs/history/pi-worker-surface/CONTEXT.md`, `docs/history/pi-worker-surface/plan.md`. Every line above is copied from a trace or from the work item; nothing here is curated truth until a human or agent accepts it.

(b) AREA UPDATES — candidate spec-sync bullets, each citing its cell

areas: from the scribing stamp for "pi-worker-surface" — .bee/logs/scribing-runs.jsonl's most recent entry (2026-09-20T10:47:33.994Z), the work item declares no bee.areas.

area hook-runtime:
  - [pws-1] Registered terminating verdict tool on Pi belt and routed to write-guard — feature-wide sync per the scribing stamp, 4 file(s) changed (trace .bee/cells/pws-1.json)
  - [pws-2] Draw in-flight workers in a widget below the editor with seat-named rows, D3 clearing, and no second timer — feature-wide sync per the scribing stamp, 4 file(s) changed (trace .bee/cells/pws-2.json)

(c) PATTERN CANDIDATES — candidate bee.pattern concepts, bee.polarity pitfall

from cell pws-1 — save as docs/knowledge/patterns/pi-worker-surface-pws-1-pitfall.md

---
type: bee.pattern
title: pi-worker-surface cell pws-1 — pitfall candidate
description: "Pitfall candidate mined from cell pws-1's capped trace: followed the plan"
timestamp: 2026-09-20
bee:
  id: pi-worker-surface-pws-1-pitfall
  lifecycle: draft
  areas: [hook-runtime]
  sources: [.bee/cells/pws-1.json]
  polarity: pitfall
---

# pi-worker-surface cell pws-1 — pitfall candidate

## What the cell did

Registered terminating verdict tool on Pi belt and routed to write-guard

## Recorded evidence (verbatim from .bee/cells/pws-1.json)

- **deviation** — followed the plan

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

from cell pws-2 — save as docs/knowledge/patterns/pi-worker-surface-pws-2-pitfall.md

---
type: bee.pattern
title: pi-worker-surface cell pws-2 — pitfall candidate
description: "Pitfall candidate mined from cell pws-2's capped trace: followed the plan"
timestamp: 2026-09-20
bee:
  id: pi-worker-surface-pws-2-pitfall
  lifecycle: draft
  areas: [hook-runtime]
  sources: [.bee/cells/pws-2.json]
  polarity: pitfall
---

# pi-worker-surface cell pws-2 — pitfall candidate

## What the cell did

Draw in-flight workers in a widget below the editor with seat-named rows, D3 clearing, and no second timer

## Recorded evidence (verbatim from .bee/cells/pws-2.json)

- **deviation** — followed the plan

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

knowledge promote: 2 capped cell(s) mined, 1 delivery draft, 2 area bullet(s), 2 pattern candidate(s), 0 file(s) written.