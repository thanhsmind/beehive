promote proposal for work item "onboard-root-resolution" (.bee/logs/scribing-runs.jsonl) — 1 capped cell(s): orr-1
anchor: ledger — .bee/logs/scribing-runs.jsonl
PROPOSAL ONLY — nothing was written. Applying any section below is a human or agent decision.

(a) DELIVERY DRAFT — save as docs/knowledge/work/onboard-root-resolution/delivery.md

---
type: bee.delivery
title: onboard-root-resolution — delivery
description: "Delivery record proposed by bee knowledge promote for work item onboard-root-resolution: 1 capped cell(s), 0 recorded deviation(s)."
timestamp: 2026-08-10
bee:
  id: onboard-root-resolution-delivery
  lifecycle: active
  areas: [onboarding]
  required_context: [.bee/logs/scribing-runs.jsonl]
  sources: [.bee/logs/scribing-runs.jsonl, .bee/cells/orr-1.json]
---

# onboard-root-resolution — Delivery

## What shipped

- **orr-1** — Engine::locate walks up from current_dir only (never current_exe); typed LocateError names invocation root + missing template; refusal payload carries both; real site source.rs not apply.rs, noted per instruction (3 file(s) changed)

## Verify

Each cell below was capped only against a recorded passing verify result — bee refuses a cap without one.

- **orr-1** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml`

## Deviations

None recorded in the capped cell traces.

## Provenance

Proposed by `bee knowledge promote --work onboard-root-resolution` from 1 capped cell trace(s) in `.bee/cells/` and the anchor `.bee/logs/scribing-runs.jsonl`. Every line above is copied from a trace or from the work item; nothing here is curated truth until a human or agent accepts it.

(b) AREA UPDATES — candidate spec-sync bullets, each citing its cell

areas: from the scribing stamp for "onboard-root-resolution" — .bee/logs/scribing-runs.jsonl's most recent entry (2026-08-10T15:41:21.647Z), the work item declares no bee.areas.

area onboarding:
  - [orr-1] Engine::locate walks up from current_dir only (never current_exe); typed LocateError names invocation root + missing template; refusal payload carries both; real site source.rs not apply.rs, noted per instruction — feature-wide sync per the scribing stamp, 3 file(s) changed (trace .bee/cells/orr-1.json)

(c) PATTERN CANDIDATES — candidate bee.pattern concepts, bee.polarity pitfall

None: no capped cell trace carries a deviation or a failure signature.

knowledge promote: 1 capped cell(s) mined, 1 delivery draft, 1 area bullet(s), 0 pattern candidate(s), 0 file(s) written.