promote proposal for work item "tmux-ready-wait" (docs/history/tmux-ready-wait/CONTEXT.md) — 1 capped cell(s): trw-1
anchor: history — docs/history/tmux-ready-wait/CONTEXT.md
PROPOSAL ONLY — nothing was written. Applying any section below is a human or agent decision.

(a) DELIVERY DRAFT — save as docs/knowledge/work/tmux-ready-wait/delivery.md

---
type: bee.delivery
title: tmux-ready-wait — delivery
description: "Delivery record proposed by bee knowledge promote for work item tmux-ready-wait: 1 capped cell(s), 1 recorded deviation(s)."
timestamp: 2026-08-23
bee:
  id: tmux-ready-wait-delivery
  lifecycle: active
  areas: [bee-herding]
  required_context: [docs/history/tmux-ready-wait/CONTEXT.md]
  sources: [docs/history/tmux-ready-wait/CONTEXT.md, .bee/cells/trw-1.json]
---

# tmux-ready-wait — Delivery

## What shipped

- **trw-1** — agent_wait keeps its screen-stability window per pane across calls, so short polling calls reach idle (1 file(s) changed)

## Verify

Each cell below was capped only against a recorded passing verify result — bee refuses a cap without one.

- **trw-1** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee herding::tmux`

## Deviations

- **trw-1** — sync-ack: internal timing fix; no operator-facing change

## Provenance

Proposed by `bee knowledge promote --work tmux-ready-wait` from 1 capped cell trace(s) in `.bee/cells/` and the anchor `docs/history/tmux-ready-wait/CONTEXT.md`. Every line above is copied from a trace or from the work item; nothing here is curated truth until a human or agent accepts it.

(b) AREA UPDATES — candidate spec-sync bullets, each citing its cell

areas: from the scribing stamp for "tmux-ready-wait" — .bee/logs/scribing-runs.jsonl's most recent entry (2026-08-23T02:55:05.658Z), the work item declares no bee.areas.

area bee-herding:
  - [trw-1] agent_wait keeps its screen-stability window per pane across calls, so short polling calls reach idle — feature-wide sync per the scribing stamp, 1 file(s) changed (trace .bee/cells/trw-1.json)

(c) PATTERN CANDIDATES — candidate bee.pattern concepts, bee.polarity pitfall

from cell trw-1 — save as docs/knowledge/patterns/tmux-ready-wait-trw-1-pitfall.md

---
type: bee.pattern
title: tmux-ready-wait cell trw-1 — pitfall candidate
description: "Pitfall candidate mined from cell trw-1's capped trace: sync-ack: internal timing fix; no operator-facing change"
timestamp: 2026-08-23
bee:
  id: tmux-ready-wait-trw-1-pitfall
  lifecycle: draft
  areas: [bee-herding]
  sources: [.bee/cells/trw-1.json]
  polarity: pitfall
---

# tmux-ready-wait cell trw-1 — pitfall candidate

## What the cell did

agent_wait keeps its screen-stability window per pane across calls, so short polling calls reach idle

## Recorded evidence (verbatim from .bee/cells/trw-1.json)

- **deviation** — sync-ack: internal timing fix; no operator-facing change

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

knowledge promote: 1 capped cell(s) mined, 1 delivery draft, 1 area bullet(s), 1 pattern candidate(s), 0 file(s) written.