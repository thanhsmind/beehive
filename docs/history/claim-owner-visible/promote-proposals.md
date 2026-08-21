promote proposal for work item "claim-owner-visible" (docs/history/claim-owner-visible/CONTEXT.md) — 1 capped cell(s): cov-1
anchor: history — docs/history/claim-owner-visible/CONTEXT.md
PROPOSAL ONLY — nothing was written. Applying any section below is a human or agent decision.

(a) DELIVERY DRAFT — save as docs/knowledge/work/claim-owner-visible/delivery.md

---
type: bee.delivery
title: claim-owner-visible — delivery
description: "Delivery record proposed by bee knowledge promote for work item claim-owner-visible: 1 capped cell(s), 1 recorded deviation(s)."
timestamp: 2026-08-21
bee:
  id: claim-owner-visible-delivery
  lifecycle: active
  required_context: [docs/history/claim-owner-visible/CONTEXT.md]
  sources: [docs/history/claim-owner-visible/CONTEXT.md, .bee/cells/archive/claim-owner-visible/cov-1.json]
---

# claim-owner-visible — Delivery

## What shipped

- **cov-1** — cells list and cells show now name the session holding a claimed cell, with a held/sweepable verdict read from the same two gates the claim sweep uses (2 file(s) changed)

## Verify

Each cell below was capped only against a recorded passing verify result — bee refuses a cap without one.

- **cov-1** — `PATH="$HOME/.cargo/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml cells`

## Deviations

- **cov-1** — Rewrote one comment the worker had copied verbatim from the cell action text (it read as an instruction, not a statement) and amended it into the cell's own commit.

## Provenance

Proposed by `bee knowledge promote --work claim-owner-visible` from 1 capped cell trace(s) in `.bee/cells/` and the anchor `docs/history/claim-owner-visible/CONTEXT.md`. Every line above is copied from a trace or from the work item; nothing here is curated truth until a human or agent accepts it.

(b) AREA UPDATES — candidate spec-sync bullets, each citing its cell

None: the work item declares no bee.areas, so there is no area to sync (D19).

(c) PATTERN CANDIDATES — candidate bee.pattern concepts, bee.polarity pitfall

from cell cov-1 — save as docs/knowledge/patterns/claim-owner-visible-cov-1-pitfall.md

---
type: bee.pattern
title: claim-owner-visible cell cov-1 — pitfall candidate
description: "Pitfall candidate mined from cell cov-1's capped trace: Rewrote one comment the worker had copied verbatim from the cell action text (it read as an instruction, not a statement) and amended it into the cell's own co…"
timestamp: 2026-08-21
bee:
  id: claim-owner-visible-cov-1-pitfall
  lifecycle: draft
  sources: [.bee/cells/archive/claim-owner-visible/cov-1.json]
  polarity: pitfall
---

# claim-owner-visible cell cov-1 — pitfall candidate

## What the cell did

cells list and cells show now name the session holding a claimed cell, with a held/sweepable verdict read from the same two gates the claim sweep uses

## Recorded evidence (verbatim from .bee/cells/archive/claim-owner-visible/cov-1.json)

- **deviation** — Rewrote one comment the worker had copied verbatim from the cell action text (it read as an instruction, not a statement) and amended it into the cell's own commit.

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

knowledge promote: 1 capped cell(s) mined, 1 delivery draft, 0 area bullet(s), 1 pattern candidate(s), 0 file(s) written.