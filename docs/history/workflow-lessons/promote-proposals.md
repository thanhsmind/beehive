promote proposal for work item "workflow-lessons" (docs/history/workflow-lessons/plan.md) — 5 capped cell(s): wfl-1, wfl-2, wfl-3, wfl-4, wfl-5
anchor: history — docs/history/workflow-lessons/plan.md
PROPOSAL ONLY — nothing was written. Applying any section below is a human or agent decision.

(a) DELIVERY DRAFT — save as docs/knowledge/work/workflow-lessons/delivery.md

---
type: bee.delivery
title: workflow-lessons — delivery
description: "Delivery record proposed by bee knowledge promote for work item workflow-lessons: 5 capped cell(s), 1 recorded deviation(s)."
timestamp: 2026-08-11
bee:
  id: workflow-lessons-delivery
  lifecycle: active
  areas: [workflow-state]
  required_context: [docs/history/workflow-lessons/plan.md]
  sources: [docs/history/workflow-lessons/plan.md, .bee/cells/wfl-1.json, .bee/cells/wfl-2.json, .bee/cells/wfl-3.json, .bee/cells/wfl-4.json, .bee/cells/wfl-5.json]
---

# workflow-lessons — Delivery

## What shipped

- **wfl-1** — Worker Result-form JSON in worker-cell.md and cells finish --report validation storing trace.report (5 file(s) changed)
- **wfl-2** — Added bee dev regen (render-skill-trees -> onboard --apply -> release-manifest --write), stopping on first red and naming the step; REGEN_OBLIGATION fix text now routes to the verb (4 file(s) changed)
- **wfl-3** — Add a judge-debt close door for standard/high-risk routes; unjudged capped behavior_change cells refuse close, judge-on-smell lanes never grow the door (2 file(s) changed)
- **wfl-4** — Added bee dispatch wave: batches claim+reserve+payload prepare over the current schedule wave, one refusal skips its cell instead of aborting the batch (5 file(s) changed)
- **wfl-5** — Fixed judge-debt door to key off route.lane instead of lane.mode, fixed wfl-3 fixtures to the live lane-record shape, and documented --report on the cells.finish registry entry (4 file(s) changed)

## Verify

Each cell below was capped only against a recorded passing verify result — bee refuses a cap without one.

- **wfl-1** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml`
- **wfl-2** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml`
- **wfl-3** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml`
- **wfl-4** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml`
- **wfl-5** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml`

## Deviations

- **wfl-1** — handlers_close.rs (CapFlags struct/CLI table/trace write) and knowledge/tests.rs (one CapFlags literal) touched beyond wfl-1's file list -- required for the new report field to compile; both reserved under w1/wfl-1 first

## Provenance

Proposed by `bee knowledge promote --work workflow-lessons` from 5 capped cell trace(s) in `.bee/cells/` and the anchor `docs/history/workflow-lessons/plan.md`. Every line above is copied from a trace or from the work item; nothing here is curated truth until a human or agent accepts it.

(b) AREA UPDATES — candidate spec-sync bullets, each citing its cell

areas: from the scribing stamp for "workflow-lessons" — .bee/logs/scribing-runs.jsonl's most recent entry (2026-08-11T14:52:40.861Z), the work item declares no bee.areas.

area workflow-state:
  - [wfl-1] Worker Result-form JSON in worker-cell.md and cells finish --report validation storing trace.report — feature-wide sync per the scribing stamp, 5 file(s) changed (trace .bee/cells/wfl-1.json)
  - [wfl-2] Added bee dev regen (render-skill-trees -> onboard --apply -> release-manifest --write), stopping on first red and naming the step; REGEN_OBLIGATION fix text now routes to the verb — feature-wide sync per the scribing stamp, 4 file(s) changed (trace .bee/cells/wfl-2.json)
  - [wfl-3] Add a judge-debt close door for standard/high-risk routes; unjudged capped behavior_change cells refuse close, judge-on-smell lanes never grow the door — feature-wide sync per the scribing stamp, 2 file(s) changed (trace .bee/cells/wfl-3.json)
  - [wfl-4] Added bee dispatch wave: batches claim+reserve+payload prepare over the current schedule wave, one refusal skips its cell instead of aborting the batch — feature-wide sync per the scribing stamp, 5 file(s) changed (trace .bee/cells/wfl-4.json)
  - [wfl-5] Fixed judge-debt door to key off route.lane instead of lane.mode, fixed wfl-3 fixtures to the live lane-record shape, and documented --report on the cells.finish registry entry — feature-wide sync per the scribing stamp, 4 file(s) changed (trace .bee/cells/wfl-5.json)

(c) PATTERN CANDIDATES — candidate bee.pattern concepts, bee.polarity pitfall

from cell wfl-1 — save as docs/knowledge/patterns/workflow-lessons-wfl-1-pitfall.md

---
type: bee.pattern
title: workflow-lessons cell wfl-1 — pitfall candidate
description: "Pitfall candidate mined from cell wfl-1's capped trace: handlers_close.rs (CapFlags struct/CLI table/trace write) and knowledge/tests.rs (one CapFlags literal) touched beyond wfl-1's file list -- required for the ne…"
timestamp: 2026-08-11
bee:
  id: workflow-lessons-wfl-1-pitfall
  lifecycle: draft
  areas: [workflow-state]
  sources: [.bee/cells/wfl-1.json]
  polarity: pitfall
---

# workflow-lessons cell wfl-1 — pitfall candidate

## What the cell did

Worker Result-form JSON in worker-cell.md and cells finish --report validation storing trace.report

## Recorded evidence (verbatim from .bee/cells/wfl-1.json)

- **deviation** — handlers_close.rs (CapFlags struct/CLI table/trace write) and knowledge/tests.rs (one CapFlags literal) touched beyond wfl-1's file list -- required for the new report field to compile; both reserved under w1/wfl-1 first

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

knowledge promote: 5 capped cell(s) mined, 1 delivery draft, 5 area bullet(s), 1 pattern candidate(s), 0 file(s) written.