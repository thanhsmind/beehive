promote proposal for work item "debt-door-archive" (docs/history/debt-door-archive/plan.md) — 2 capped cell(s): dda-1, dda-2
anchor: history — docs/history/debt-door-archive/plan.md
PROPOSAL ONLY — nothing was written. Applying any section below is a human or agent decision.

(a) DELIVERY DRAFT — save as docs/knowledge/work/debt-door-archive/delivery.md

---
type: bee.delivery
title: debt-door-archive — delivery
description: "Delivery record proposed by bee knowledge promote for work item debt-door-archive: 2 capped cell(s), 0 recorded deviation(s)."
timestamp: 2026-08-06
bee:
  id: debt-door-archive-delivery
  lifecycle: active
  areas: [workflow-state]
  required_context: [docs/history/debt-door-archive/plan.md]
  sources: [docs/history/debt-door-archive/plan.md, .bee/cells/dda-1.json, .bee/cells/dda-2.json]
---

# debt-door-archive — Delivery

## What shipped

- **dda-1** — scribing_debt now walks .bee/cells/archive/<feature>/ with live-copy-wins dedup, so an archived behavior_change cell still counts against the door (3 file(s) changed)
- **dda-2** — Made status_full/chain_nudge/session_preamble scribing-debt counters (and both global orphan sweeps) archive-aware like dda-1's door; added a parity test over hot+archived+duplicate cells proving all four counters and both global sweeps agree; live repo shows 0 newly-surfaced debt from archived features (21-cell orphan count is pre-existing hot-store debt, unrelated). (4 file(s) changed)

## Verify

Each cell below was capped only against a recorded passing verify result — bee refuses a cap without one.

- **dda-1** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml`
- **dda-2** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml`

## Deviations

None recorded in the capped cell traces.

## Provenance

Proposed by `bee knowledge promote --work debt-door-archive` from 2 capped cell trace(s) in `.bee/cells/` and the anchor `docs/history/debt-door-archive/plan.md`. Every line above is copied from a trace or from the work item; nothing here is curated truth until a human or agent accepts it.

(b) AREA UPDATES — candidate spec-sync bullets, each citing its cell

areas: from the scribing stamp for "debt-door-archive" — .bee/logs/scribing-runs.jsonl's most recent entry (2026-08-06T03:03:53.193Z), the work item declares no bee.areas.

area workflow-state:
  - [dda-1] scribing_debt now walks .bee/cells/archive/<feature>/ with live-copy-wins dedup, so an archived behavior_change cell still counts against the door — feature-wide sync per the scribing stamp, 3 file(s) changed (trace .bee/cells/dda-1.json)
  - [dda-2] Made status_full/chain_nudge/session_preamble scribing-debt counters (and both global orphan sweeps) archive-aware like dda-1's door; added a parity test over hot+archived+duplicate cells proving all four counters and both global sweeps agree; live repo shows 0 newly-surfaced debt from archived features (21-cell orphan count is pre-existing hot-store debt, unrelated). — feature-wide sync per the scribing stamp, 4 file(s) changed (trace .bee/cells/dda-2.json)

(c) PATTERN CANDIDATES — candidate bee.pattern concepts, bee.polarity pitfall

None: no capped cell trace carries a deviation or a failure signature.

knowledge promote: 2 capped cell(s) mined, 1 delivery draft, 2 area bullet(s), 0 pattern candidate(s), 0 file(s) written.