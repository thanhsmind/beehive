promote proposal for work item "dispatch-door-upfront" (docs/history/dispatch-door-upfront/CONTEXT.md) — 3 capped cell(s): ddu-1, ddu-2, ddu-3
anchor: history — docs/history/dispatch-door-upfront/CONTEXT.md
PROPOSAL ONLY — nothing was written. Applying any section below is a human or agent decision.

(a) DELIVERY DRAFT — save as docs/knowledge/work/dispatch-door-upfront/delivery.md

---
type: bee.delivery
title: dispatch-door-upfront — delivery
description: "Delivery record proposed by bee knowledge promote for work item dispatch-door-upfront: 3 capped cell(s), 2 recorded deviation(s)."
timestamp: 2026-08-22
bee:
  id: dispatch-door-upfront-delivery
  lifecycle: active
  areas: [hook-runtime]
  required_context: [docs/history/dispatch-door-upfront/CONTEXT.md]
  sources: [docs/history/dispatch-door-upfront/CONTEXT.md, .bee/cells/archive/dispatch-door-upfront/ddu-1.json, .bee/cells/archive/dispatch-door-upfront/ddu-2.json, .bee/cells/archive/dispatch-door-upfront/ddu-3.json]
---

# dispatch-door-upfront — Delivery

## What shipped

- **ddu-1** — AGENTS.md names dispatch prepare as the one door before any agent name, marker or model param (1 file(s) changed)
- **ddu-2** — Preamble and compaction capsule render a Dispatch door block with the prepare command and resolved claude tier slots via a model_guard helper (4 file(s) changed)
- **ddu-3** — Render dispatch door slots from shared models resolver and update tests with same-source proof (3 file(s) changed)

## Verify

Each cell below was capped only against a recorded passing verify result — bee refuses a cap without one.

- **ddu-1** — `rg -n 'dispatch prepare' AGENTS.md`
- **ddu-2** — `PATH="$HOME/.cargo/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml session_preamble`
- **ddu-3** — `PATH="$HOME/.cargo/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml session_preamble`

## Deviations

- **ddu-1** — worker committed without the cell trailer and did not run cells finish; orchestrator rewrote the trailer via commit-tree and capped
- **ddu-2** — worker committed without the cell trailer and did not run cells finish; orchestrator rewrote the trailer via commit-tree and capped

## Provenance

Proposed by `bee knowledge promote --work dispatch-door-upfront` from 3 capped cell trace(s) in `.bee/cells/` and the anchor `docs/history/dispatch-door-upfront/CONTEXT.md`. Every line above is copied from a trace or from the work item; nothing here is curated truth until a human or agent accepts it.

(b) AREA UPDATES — candidate spec-sync bullets, each citing its cell

areas: from the scribing stamp for "dispatch-door-upfront" — .bee/logs/scribing-runs.jsonl's most recent entry (2026-08-22T04:41:43.345Z), the work item declares no bee.areas.

area hook-runtime:
  - [ddu-2] Preamble and compaction capsule render a Dispatch door block with the prepare command and resolved claude tier slots via a model_guard helper — feature-wide sync per the scribing stamp, 4 file(s) changed (trace .bee/cells/archive/dispatch-door-upfront/ddu-2.json)
  - [ddu-3] Render dispatch door slots from shared models resolver and update tests with same-source proof — feature-wide sync per the scribing stamp, 3 file(s) changed (trace .bee/cells/archive/dispatch-door-upfront/ddu-3.json)

(c) PATTERN CANDIDATES — candidate bee.pattern concepts, bee.polarity pitfall

from cell ddu-1 — save as docs/knowledge/patterns/dispatch-door-upfront-ddu-1-pitfall.md

---
type: bee.pattern
title: dispatch-door-upfront cell ddu-1 — pitfall candidate
description: "Pitfall candidate mined from cell ddu-1's capped trace: worker committed without the cell trailer and did not run cells finish; orchestrator rewrote the trailer via commit-tree and capped"
timestamp: 2026-08-22
bee:
  id: dispatch-door-upfront-ddu-1-pitfall
  lifecycle: draft
  areas: [hook-runtime]
  sources: [.bee/cells/archive/dispatch-door-upfront/ddu-1.json]
  polarity: pitfall
---

# dispatch-door-upfront cell ddu-1 — pitfall candidate

## What the cell did

AGENTS.md names dispatch prepare as the one door before any agent name, marker or model param

## Recorded evidence (verbatim from .bee/cells/archive/dispatch-door-upfront/ddu-1.json)

- **deviation** — worker committed without the cell trailer and did not run cells finish; orchestrator rewrote the trailer via commit-tree and capped

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

from cell ddu-2 — save as docs/knowledge/patterns/dispatch-door-upfront-ddu-2-pitfall.md

---
type: bee.pattern
title: dispatch-door-upfront cell ddu-2 — pitfall candidate
description: "Pitfall candidate mined from cell ddu-2's capped trace: worker committed without the cell trailer and did not run cells finish; orchestrator rewrote the trailer via commit-tree and capped"
timestamp: 2026-08-22
bee:
  id: dispatch-door-upfront-ddu-2-pitfall
  lifecycle: draft
  areas: [hook-runtime]
  sources: [.bee/cells/archive/dispatch-door-upfront/ddu-2.json]
  polarity: pitfall
---

# dispatch-door-upfront cell ddu-2 — pitfall candidate

## What the cell did

Preamble and compaction capsule render a Dispatch door block with the prepare command and resolved claude tier slots via a model_guard helper

## Recorded evidence (verbatim from .bee/cells/archive/dispatch-door-upfront/ddu-2.json)

- **deviation** — worker committed without the cell trailer and did not run cells finish; orchestrator rewrote the trailer via commit-tree and capped

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

knowledge promote: 3 capped cell(s) mined, 1 delivery draft, 2 area bullet(s), 2 pattern candidate(s), 0 file(s) written.