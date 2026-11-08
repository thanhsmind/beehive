---
type: bee.delivery
title: awaiting-human — delivery
description: "Delivery record proposed by bee knowledge promote for work item awaiting-human: 4 capped cell(s), 1 recorded deviation(s)."
timestamp: 2026-08-14
bee:
  id: awaiting-human-delivery
  lifecycle: active
  areas: [workflow-state]
  required_context: [docs/history/awaiting-human/CONTEXT.md, docs/history/awaiting-human/plan.md]
  sources: [docs/history/awaiting-human/CONTEXT.md, docs/history/awaiting-human/plan.md, .bee/cells/archive/awaiting-human/ah-1.json, .bee/cells/archive/awaiting-human/ah-2.json, .bee/cells/archive/awaiting-human/ah-3.json, .bee/cells/archive/awaiting-human/ah-4.json]
---

# awaiting-human — Delivery

## What shipped

- **ah-1** — A waiting mark exists, can be set, and makes the run read awaiting-approval (5 file(s) changed)
- **ah-2** — Three live ways for the waiting mark to end, with the hook path now covered by tests that drive the real hook entry point, including a failure injection proving the hook survives a failing clear (3 file(s) changed)
- **ah-3** — Six reporting surfaces enumerated and handled: five name a live wait, status --brief deliberately excluded per status-diet D1/D2; enumeration written to docs/history/awaiting-human/reports/ah-3-rework.md (7 file(s) changed)
- **ah-4** — Wired bee state waiting-on set/clear onto ah-1/ah-2's existing store functions, with D3 target resolution and projection sync (9 file(s) changed)

## Verify

Each cell below was capped only against a recorded passing verify result — bee refuses a cap without one.

- **ah-1** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml`
- **ah-2** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml`
- **ah-3** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml`
- **ah-4** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml`

## Deviations

- **ah-4** — File list corrected: added waiting_on.rs (the new module), catalog.rs (PINNED_FLAG_COUNT bump for --subject), and tests/workflow_verbs.rs (true through-the-binary CLI proof) beyond the cell's guessed set.

## Provenance

Proposed by `bee knowledge promote --work awaiting-human` from 4 capped cell trace(s) in `.bee/cells/` and the anchor `docs/history/awaiting-human/CONTEXT.md`, `docs/history/awaiting-human/plan.md`. Every line above is copied from a trace or from the work item; Applied 2026-08-16 from docs/history/awaiting-human/promote-proposals.md; area bullets declined (feature-wide scribing sync already stamped), no pattern candidates survived review.
