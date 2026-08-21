promote proposal for work item "worker-brief-expertise" (.bee/logs/scribing-runs.jsonl) — 3 capped cell(s): wbe-1, wbe-2, wbe-3
anchor: ledger — .bee/logs/scribing-runs.jsonl
PROPOSAL ONLY — nothing was written. Applying any section below is a human or agent decision.

(a) DELIVERY DRAFT — save as docs/knowledge/work/worker-brief-expertise/delivery.md

---
type: bee.delivery
title: worker-brief-expertise — delivery
description: "Delivery record proposed by bee knowledge promote for work item worker-brief-expertise: 3 capped cell(s), 0 recorded deviation(s)."
timestamp: 2026-08-20
bee:
  id: worker-brief-expertise-delivery
  lifecycle: active
  areas: [bee-herding]
  required_context: [.bee/logs/scribing-runs.jsonl]
  sources: [.bee/logs/scribing-runs.jsonl, .bee/cells/wbe-1.json, .bee/cells/wbe-2.json, .bee/cells/wbe-3.json]
---

# worker-brief-expertise — Delivery

## What shipped

- **wbe-1** — herding run --expertise plumbs dispatcher-picked expertise into the worker brief; ignore clause rescoped to workflow-only (4 file(s) changed)
- **wbe-2** — dispatch prepare --expertise feeds a dispatcher-picked Expertise block into the worker-cell prompt (9 file(s) changed)
- **wbe-3** — Skill prose: dispatcher composes Expertise entries leader-style; herding run flag list gains --expertise (4 file(s) changed)

## Verify

Each cell below was capped only against a recorded passing verify result — bee refuses a cap without one.

- **wbe-1** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml`
- **wbe-2** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml && .bee/bin/bee dev release-manifest --check`
- **wbe-3** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml && .bee/bin/bee dev release-manifest --check`

## Deviations

None recorded in the capped cell traces.

## Provenance

Proposed by `bee knowledge promote --work worker-brief-expertise` from 3 capped cell trace(s) in `.bee/cells/` and the anchor `.bee/logs/scribing-runs.jsonl`. Every line above is copied from a trace or from the work item; nothing here is curated truth until a human or agent accepts it.

(b) AREA UPDATES — candidate spec-sync bullets, each citing its cell

areas: from the scribing stamp for "worker-brief-expertise" — .bee/logs/scribing-runs.jsonl's most recent entry (2026-08-21T00:00:49.736Z), the work item declares no bee.areas.

area bee-herding:
  - [wbe-1] herding run --expertise plumbs dispatcher-picked expertise into the worker brief; ignore clause rescoped to workflow-only — feature-wide sync per the scribing stamp, 4 file(s) changed (trace .bee/cells/wbe-1.json)
  - [wbe-2] dispatch prepare --expertise feeds a dispatcher-picked Expertise block into the worker-cell prompt — feature-wide sync per the scribing stamp, 9 file(s) changed (trace .bee/cells/wbe-2.json)

(c) PATTERN CANDIDATES — candidate bee.pattern concepts, bee.polarity pitfall

None: no capped cell trace carries a deviation or a failure signature.

knowledge promote: 3 capped cell(s) mined, 1 delivery draft, 2 area bullet(s), 0 pattern candidate(s), 0 file(s) written.