promote proposal for work item "read-role-description" (.bee/lanes/read-role-description.json) — 1 capped cell(s): rrd-1
anchor: ledger — .bee/lanes/read-role-description.json
PROPOSAL ONLY — nothing was written. Applying any section below is a human or agent decision.

(a) DELIVERY DRAFT — save as docs/knowledge/work/read-role-description/delivery.md

---
type: bee.delivery
title: read-role-description — delivery
description: "Delivery record proposed by bee knowledge promote for work item read-role-description: 1 capped cell(s), 1 recorded deviation(s)."
timestamp: 2026-09-10
bee:
  id: read-role-description-delivery
  lifecycle: active
  required_context: [.bee/lanes/read-role-description.json]
  sources: [.bee/lanes/read-role-description.json, .bee/cells/rrd-1.json]
---

# read-role-description — Delivery

## What shipped

- **rrd-1** — read role's seeded description now names the narrow read job and points a broad scan at a role-less --kind gather; routing untouched (3 file(s) changed)

## Verify

Each cell below was capped only against a recorded passing verify result — bee refuses a cap without one.

- **rrd-1** — `cargo test --release --manifest-path packages/bee-rs/Cargo.toml onboard and drivers tests green; bee dispatch prepare --runtime claude --kind gather --role read still returns the bee-extract payload.`

## Deviations

- **rrd-1** — bee config set is not built into this binary (its own help says so); .bee/config.json was edited directly, which is the CLI-only rule taking its named fallback

## Provenance

Proposed by `bee knowledge promote --work read-role-description` from 1 capped cell trace(s) in `.bee/cells/` and the anchor `.bee/lanes/read-role-description.json`. Every line above is copied from a trace or from the work item; nothing here is curated truth until a human or agent accepts it.

(b) AREA UPDATES — candidate spec-sync bullets, each citing its cell

None: the work item declares no bee.areas, so there is no area to sync (D19).

(c) PATTERN CANDIDATES — candidate bee.pattern concepts, bee.polarity pitfall

from cell rrd-1 — save as docs/knowledge/patterns/read-role-description-rrd-1-pitfall.md

---
type: bee.pattern
title: read-role-description cell rrd-1 — pitfall candidate
description: "Pitfall candidate mined from cell rrd-1's capped trace: bee config set is not built into this binary (its own help says so); .bee/config.json was edited directly, which is the CLI-only rule taking its named fallback"
timestamp: 2026-09-10
bee:
  id: read-role-description-rrd-1-pitfall
  lifecycle: draft
  sources: [.bee/cells/rrd-1.json]
  polarity: pitfall
---

# read-role-description cell rrd-1 — pitfall candidate

## What the cell did

read role's seeded description now names the narrow read job and points a broad scan at a role-less --kind gather; routing untouched

## Recorded evidence (verbatim from .bee/cells/rrd-1.json)

- **deviation** — bee config set is not built into this binary (its own help says so); .bee/config.json was edited directly, which is the CLI-only rule taking its named fallback

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

knowledge promote: 1 capped cell(s) mined, 1 delivery draft, 0 area bullet(s), 1 pattern candidate(s), 0 file(s) written.