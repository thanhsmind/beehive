promote proposal for work item "herding-prompt-verify" (docs/history/herding-prompt-verify/CONTEXT.md) — 1 capped cell(s): hpv-1
anchor: history — docs/history/herding-prompt-verify/CONTEXT.md
PROPOSAL ONLY — nothing was written. Applying any section below is a human or agent decision.

(a) DELIVERY DRAFT — save as docs/knowledge/work/herding-prompt-verify/delivery.md

---
type: bee.delivery
title: herding-prompt-verify — delivery
description: "Delivery record proposed by bee knowledge promote for work item herding-prompt-verify: 1 capped cell(s), 0 recorded deviation(s)."
timestamp: 2026-08-20
bee:
  id: herding-prompt-verify-delivery
  lifecycle: active
  areas: [bee-herding]
  required_context: [docs/history/herding-prompt-verify/CONTEXT.md]
  sources: [docs/history/herding-prompt-verify/CONTEXT.md, .bee/cells/hpv-1.json]
---

# herding-prompt-verify — Delivery

## What shipped

- **hpv-1** — Pointer delivery verified against pane text with bounded resends; silent drop impossible (1 file(s) changed)

## Verify

Each cell below was capped only against a recorded passing verify result — bee refuses a cap without one.

- **hpv-1** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml herding::run`

## Deviations

None recorded in the capped cell traces.

## Provenance

Proposed by `bee knowledge promote --work herding-prompt-verify` from 1 capped cell trace(s) in `.bee/cells/` and the anchor `docs/history/herding-prompt-verify/CONTEXT.md`. Every line above is copied from a trace or from the work item; nothing here is curated truth until a human or agent accepts it.

(b) AREA UPDATES — candidate spec-sync bullets, each citing its cell

areas: from the scribing stamp for "herding-prompt-verify" — .bee/logs/scribing-runs.jsonl's most recent entry (2026-08-20T03:19:43.414Z), the work item declares no bee.areas.

area bee-herding:
  - [hpv-1] Pointer delivery verified against pane text with bounded resends; silent drop impossible — feature-wide sync per the scribing stamp, 1 file(s) changed (trace .bee/cells/hpv-1.json)

(c) PATTERN CANDIDATES — candidate bee.pattern concepts, bee.polarity pitfall

None: no capped cell trace carries a deviation or a failure signature.

knowledge promote: 1 capped cell(s) mined, 1 delivery draft, 1 area bullet(s), 0 pattern candidate(s), 0 file(s) written.