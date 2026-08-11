promote proposal for work item "state-swap-hygiene" (.bee/logs/scribing-runs.jsonl) — 2 capped cell(s): ssh-1, ssh-2
anchor: ledger — .bee/logs/scribing-runs.jsonl
PROPOSAL ONLY — nothing was written. Applying any section below is a human or agent decision.

(a) DELIVERY DRAFT — save as docs/knowledge/work/state-swap-hygiene/delivery.md

---
type: bee.delivery
title: state-swap-hygiene — delivery
description: "Delivery record proposed by bee knowledge promote for work item state-swap-hygiene: 2 capped cell(s), 1 recorded deviation(s)."
timestamp: 2026-08-11
bee:
  id: state-swap-hygiene-delivery
  lifecycle: active
  areas: [workflow-state, okf-profile]
  required_context: [.bee/logs/scribing-runs.jsonl]
  sources: [.bee/logs/scribing-runs.jsonl, .bee/cells/ssh-1.json, .bee/cells/ssh-2.json]
---

# state-swap-hygiene — Delivery

## What shipped

- **ssh-1** — state set feature swap reaps: outgoing workflow record closed, incoming ensured, via the same feature.rs functions start-feature uses; C1 direct write byte-identical; real seam run_set_body (1 file(s) changed)
- **ssh-2** — bundle paths pass one_line (cap 200) before joining the worker prompt; newline-bearing filename renders one collapsed line; normal paths byte-identical (2 file(s) changed)

## Verify

Each cell below was capped only against a recorded passing verify result — bee refuses a cap without one.

- **ssh-1** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml`
- **ssh-2** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml`

## Deviations

- **ssh-2** — sweep of the sibling joins at prepare.rs ~391-405 found only static string literals - no raw-path interpolation there, no change needed

## Provenance

Proposed by `bee knowledge promote --work state-swap-hygiene` from 2 capped cell trace(s) in `.bee/cells/` and the anchor `.bee/logs/scribing-runs.jsonl`. Every line above is copied from a trace or from the work item; nothing here is curated truth until a human or agent accepts it.

(b) AREA UPDATES — candidate spec-sync bullets, each citing its cell

areas: from the scribing stamp for "state-swap-hygiene" — .bee/logs/scribing-runs.jsonl's most recent entry (2026-08-11T03:03:04.183Z), the work item declares no bee.areas.

area workflow-state:
  - [ssh-1] state set feature swap reaps: outgoing workflow record closed, incoming ensured, via the same feature.rs functions start-feature uses; C1 direct write byte-identical; real seam run_set_body — feature-wide sync per the scribing stamp, 1 file(s) changed (trace .bee/cells/ssh-1.json)
  - [ssh-2] bundle paths pass one_line (cap 200) before joining the worker prompt; newline-bearing filename renders one collapsed line; normal paths byte-identical — feature-wide sync per the scribing stamp, 2 file(s) changed (trace .bee/cells/ssh-2.json)

area okf-profile:
  - [ssh-1] state set feature swap reaps: outgoing workflow record closed, incoming ensured, via the same feature.rs functions start-feature uses; C1 direct write byte-identical; real seam run_set_body — feature-wide sync per the scribing stamp, 1 file(s) changed (trace .bee/cells/ssh-1.json)
  - [ssh-2] bundle paths pass one_line (cap 200) before joining the worker prompt; newline-bearing filename renders one collapsed line; normal paths byte-identical — feature-wide sync per the scribing stamp, 2 file(s) changed (trace .bee/cells/ssh-2.json)

(c) PATTERN CANDIDATES — candidate bee.pattern concepts, bee.polarity pitfall

from cell ssh-2 — save as docs/knowledge/patterns/state-swap-hygiene-ssh-2-pitfall.md

---
type: bee.pattern
title: state-swap-hygiene cell ssh-2 — pitfall candidate
description: "Pitfall candidate mined from cell ssh-2's capped trace: sweep of the sibling joins at prepare.rs ~391-405 found only static string literals - no raw-path interpolation there, no change needed"
timestamp: 2026-08-11
bee:
  id: state-swap-hygiene-ssh-2-pitfall
  lifecycle: draft
  areas: [workflow-state, okf-profile]
  sources: [.bee/cells/ssh-2.json]
  polarity: pitfall
---

# state-swap-hygiene cell ssh-2 — pitfall candidate

## What the cell did

bundle paths pass one_line (cap 200) before joining the worker prompt; newline-bearing filename renders one collapsed line; normal paths byte-identical

## Recorded evidence (verbatim from .bee/cells/ssh-2.json)

- **deviation** — sweep of the sibling joins at prepare.rs ~391-405 found only static string literals - no raw-path interpolation there, no change needed

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

knowledge promote: 2 capped cell(s) mined, 1 delivery draft, 4 area bullet(s), 1 pattern candidate(s), 0 file(s) written.