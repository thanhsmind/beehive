promote proposal for work item "role-edge-hardening" (docs/history/role-edge-hardening/CONTEXT.md) — 2 capped cell(s): reh-1, reh-2
anchor: history — docs/history/role-edge-hardening/CONTEXT.md
PROPOSAL ONLY — nothing was written. Applying any section below is a human or agent decision.

(a) DELIVERY DRAFT — save as docs/knowledge/work/role-edge-hardening/delivery.md

---
type: bee.delivery
title: role-edge-hardening — delivery
description: "Delivery record proposed by bee knowledge promote for work item role-edge-hardening: 2 capped cell(s), 6 recorded deviation(s)."
timestamp: 2026-08-25
bee:
  id: role-edge-hardening-delivery
  lifecycle: active
  areas: [doctrine-layer]
  required_context: [docs/history/role-edge-hardening/CONTEXT.md]
  sources: [docs/history/role-edge-hardening/CONTEXT.md, .bee/cells/reh-1.json, .bee/cells/reh-2.json]
---

# role-edge-hardening — Delivery

## What shipped

- **reh-1** — the forbidden ceiling key is named with a teach line, the advisor identity folds case at both doors, and a dead chain key warns instead of dying silently (6 file(s) changed)
- **reh-2** — the opt-in window's null boundary is pinned by a mutation-proven test, and the backfill's count provenance is stated exactly with the movable direction folded from fresh reads (3 file(s) changed)

## Verify

Each cell below was capped only against a recorded passing verify result — bee refuses a cap without one.

- **reh-1** — `cargo test --release --no-fail-fast --manifest-path packages/bee-rs/Cargo.toml`
- **reh-2** — `cargo test --release --no-fail-fast --manifest-path packages/bee-rs/Cargo.toml`

## Deviations

- **reh-1** — Executed inline; reason on trace.inline_reason.
- **reh-1** — sync-ack: All touches are role-surface code and its own tests; no owned skill describes the ceiling config key, the advisor case rule, or chain-key liveness — the fixes make code match what the docs already teach.
- **reh-2** — Executed inline; reason on trace.inline_reason.
- **reh-2** — Counts-under-lock must-have narrowed with recorded reason: full recount would undo the P1-B short-hold lock design.
- **reh-2** — The null-window test file rode the sibling commit faef734d; the diff carries it.
- **reh-2** — sync-ack: Test additions plus a count-provenance fix inside the migration verb; no owned skill teaches count provenance, and the null-window semantics pinned are the ones the code already shipped.

## Provenance

Proposed by `bee knowledge promote --work role-edge-hardening` from 2 capped cell trace(s) in `.bee/cells/` and the anchor `docs/history/role-edge-hardening/CONTEXT.md`. Every line above is copied from a trace or from the work item; nothing here is curated truth until a human or agent accepts it.

(b) AREA UPDATES — candidate spec-sync bullets, each citing its cell

areas: from the scribing stamp for "role-edge-hardening" — .bee/logs/scribing-runs.jsonl's most recent entry (2026-08-25T21:34:09.529Z), the work item declares no bee.areas.

area doctrine-layer:
  - [reh-1] the forbidden ceiling key is named with a teach line, the advisor identity folds case at both doors, and a dead chain key warns instead of dying silently — feature-wide sync per the scribing stamp, 6 file(s) changed (trace .bee/cells/reh-1.json)
  - [reh-2] the opt-in window's null boundary is pinned by a mutation-proven test, and the backfill's count provenance is stated exactly with the movable direction folded from fresh reads — feature-wide sync per the scribing stamp, 3 file(s) changed (trace .bee/cells/reh-2.json)

(c) PATTERN CANDIDATES — candidate bee.pattern concepts, bee.polarity pitfall

from cell reh-1 — save as docs/knowledge/patterns/role-edge-hardening-reh-1-pitfall.md

---
type: bee.pattern
title: role-edge-hardening cell reh-1 — pitfall candidate
description: "Pitfall candidate mined from cell reh-1's capped trace: Executed inline; reason on trace.inline_reason."
timestamp: 2026-08-25
bee:
  id: role-edge-hardening-reh-1-pitfall
  lifecycle: draft
  areas: [doctrine-layer]
  sources: [.bee/cells/reh-1.json]
  polarity: pitfall
---

# role-edge-hardening cell reh-1 — pitfall candidate

## What the cell did

the forbidden ceiling key is named with a teach line, the advisor identity folds case at both doors, and a dead chain key warns instead of dying silently

## Recorded evidence (verbatim from .bee/cells/reh-1.json)

- **deviation** — Executed inline; reason on trace.inline_reason.
- **deviation** — sync-ack: All touches are role-surface code and its own tests; no owned skill describes the ceiling config key, the advisor case rule, or chain-key liveness — the fixes make code match what the docs already teach.

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

from cell reh-2 — save as docs/knowledge/patterns/role-edge-hardening-reh-2-pitfall.md

---
type: bee.pattern
title: role-edge-hardening cell reh-2 — pitfall candidate
description: "Pitfall candidate mined from cell reh-2's capped trace: Executed inline; reason on trace.inline_reason."
timestamp: 2026-08-25
bee:
  id: role-edge-hardening-reh-2-pitfall
  lifecycle: draft
  areas: [doctrine-layer]
  sources: [.bee/cells/reh-2.json]
  polarity: pitfall
---

# role-edge-hardening cell reh-2 — pitfall candidate

## What the cell did

the opt-in window's null boundary is pinned by a mutation-proven test, and the backfill's count provenance is stated exactly with the movable direction folded from fresh reads

## Recorded evidence (verbatim from .bee/cells/reh-2.json)

- **deviation** — Executed inline; reason on trace.inline_reason.
- **deviation** — Counts-under-lock must-have narrowed with recorded reason: full recount would undo the P1-B short-hold lock design.
- **deviation** — The null-window test file rode the sibling commit faef734d; the diff carries it.
- **deviation** — sync-ack: Test additions plus a count-provenance fix inside the migration verb; no owned skill teaches count provenance, and the null-window semantics pinned are the ones the code already shipped.

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

knowledge promote: 2 capped cell(s) mined, 1 delivery draft, 2 area bullet(s), 2 pattern candidate(s), 0 file(s) written.