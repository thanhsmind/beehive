---
type: bee.delivery
title: herding-activity-hook — delivery
description: "Delivery record for work item herding-activity-hook: 2 capped cell(s), one contract-changing deviation, and the verify each cell capped against."
timestamp: 2026-08-23
bee:
  id: herding-activity-hook-delivery
  lifecycle: active
  areas: [bee-herding, hook-runtime]
  required_context: [docs/history/herding-activity-hook/CONTEXT.md]
  sources: [docs/history/herding-activity-hook/CONTEXT.md, .bee/cells/archive/herding-activity-hook/hact-1.json, .bee/cells/archive/herding-activity-hook/hact-2.json]
---

# herding-activity-hook — Delivery

## What shipped

- **hact-1** — The activity hook runs in a herded pane and writes the job mailbox activity record (2 file(s) changed)
- **hact-2** — The run verb reads the pane's own activity.json ahead of the screen classifier at all three wait points, fenced by round and a 120s freshness bound (2 file(s) changed)

## Verify

Each cell below was capped only against a recorded passing verify result — bee refuses a cap without one.

- **hact-1** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee hooks::`
- **hact-2** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee herding::`

## Deviations

- **hact-1** — A herded pane sets no waiting_on mark — that sink is bee-session state, and the pane holds no bee session

## Provenance

Proposed by `bee knowledge promote --work herding-activity-hook` from 2 capped cell trace(s) in `.bee/cells/` and the anchor `docs/history/herding-activity-hook/CONTEXT.md`; reviewed and accepted at the 2026-08-23 compounding pass. Signature, test-call-site and sync-ack rows were trimmed. The hook-runtime area was added: the feature changed that area's R7 (every-hook-silent) rule and the activity record's sink, both synced there.
