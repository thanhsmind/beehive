promote proposal for work item "windows-build-red" (.bee/lanes/windows-build-red.json) — 1 capped cell(s): wbr-1
anchor: ledger — .bee/lanes/windows-build-red.json
PROPOSAL ONLY — nothing was written. Applying any section below is a human or agent decision.

(a) DELIVERY DRAFT — save as docs/knowledge/work/windows-build-red/delivery.md

---
type: bee.delivery
title: windows-build-red — delivery
description: "Delivery record proposed by bee knowledge promote for work item windows-build-red: 1 capped cell(s), 0 recorded deviation(s)."
timestamp: 2026-08-21
bee:
  id: windows-build-red-delivery
  lifecycle: active
  required_context: [.bee/lanes/windows-build-red.json]
  sources: [.bee/lanes/windows-build-red.json, .bee/cells/wbr-1.json]
---

# windows-build-red — Delivery

## What shipped

- **wbr-1** — the unix-only trust-file test is cfg-gated so the Windows crate compiles again (1 file(s) changed)

## Verify

Each cell below was capped only against a recorded passing verify result — bee refuses a cap without one.

- **wbr-1** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee preflight_warns`

## Deviations

None recorded in the capped cell traces.

## Provenance

Proposed by `bee knowledge promote --work windows-build-red` from 1 capped cell trace(s) in `.bee/cells/` and the anchor `.bee/lanes/windows-build-red.json`. Every line above is copied from a trace or from the work item; nothing here is curated truth until a human or agent accepts it.

(b) AREA UPDATES — candidate spec-sync bullets, each citing its cell

None: the work item declares no bee.areas, so there is no area to sync (D19).

(c) PATTERN CANDIDATES — candidate bee.pattern concepts, bee.polarity pitfall

None: no capped cell trace carries a deviation or a failure signature.

knowledge promote: 1 capped cell(s) mined, 1 delivery draft, 0 area bullet(s), 0 pattern candidate(s), 0 file(s) written.