promote proposal for work item "windows-suite-green" (.bee/lanes/windows-suite-green.json) — 2 capped cell(s): wsg-1, wsg-2
anchor: ledger — .bee/lanes/windows-suite-green.json
PROPOSAL ONLY — nothing was written. Applying any section below is a human or agent decision.

(a) DELIVERY DRAFT — save as docs/knowledge/work/windows-suite-green/delivery.md

---
type: bee.delivery
title: windows-suite-green — delivery
description: "Delivery record proposed by bee knowledge promote for work item windows-suite-green: 2 capped cell(s), 0 recorded deviation(s)."
timestamp: 2026-08-21
bee:
  id: windows-suite-green-delivery
  lifecycle: active
  required_context: [.bee/lanes/windows-suite-green.json]
  sources: [.bee/lanes/windows-suite-green.json, .bee/cells/wsg-1.json, .bee/cells/wsg-2.json]
---

# windows-suite-green — Delivery

## What shipped

- **wsg-1** — mailbox assertions derive their paths; expand_tilde falls back to USERPROFILE (2 file(s) changed)
- **wsg-2** — seven session_close fixtures bind the canonical root; the self-exec test child no longer prints into the parent suite (2 file(s) changed)

## Verify

Each cell below was capped only against a recorded passing verify result — bee refuses a cap without one.

- **wsg-1** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee herding::mailbox herding::wave`
- **wsg-2** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee -- hooks::session_close herding::control_loop`

## Deviations

None recorded in the capped cell traces.

## Provenance

Proposed by `bee knowledge promote --work windows-suite-green` from 2 capped cell trace(s) in `.bee/cells/` and the anchor `.bee/lanes/windows-suite-green.json`. Every line above is copied from a trace or from the work item; nothing here is curated truth until a human or agent accepts it.

(b) AREA UPDATES — candidate spec-sync bullets, each citing its cell

None: the work item declares no bee.areas, so there is no area to sync (D19).

(c) PATTERN CANDIDATES — candidate bee.pattern concepts, bee.polarity pitfall

None: no capped cell trace carries a deviation or a failure signature.

knowledge promote: 2 capped cell(s) mined, 1 delivery draft, 0 area bullet(s), 0 pattern candidate(s), 0 file(s) written.