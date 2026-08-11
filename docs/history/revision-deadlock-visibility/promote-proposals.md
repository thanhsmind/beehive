promote proposal for work item "revision-deadlock-visibility" (.bee/logs/scribing-runs.jsonl) — 2 capped cell(s): rdv-1, rdv-2
anchor: ledger — .bee/logs/scribing-runs.jsonl
PROPOSAL ONLY — nothing was written. Applying any section below is a human or agent decision.

(a) DELIVERY DRAFT — save as docs/knowledge/work/revision-deadlock-visibility/delivery.md

---
type: bee.delivery
title: revision-deadlock-visibility — delivery
description: "Delivery record proposed by bee knowledge promote for work item revision-deadlock-visibility: 2 capped cell(s), 1 recorded deviation(s)."
timestamp: 2026-08-11
bee:
  id: revision-deadlock-visibility-delivery
  lifecycle: active
  areas: [workflow-state]
  required_context: [.bee/logs/scribing-runs.jsonl]
  sources: [.bee/logs/scribing-runs.jsonl, .bee/cells/rdv-1.json, .bee/cells/rdv-2.json]
---

# revision-deadlock-visibility — Delivery

## What shipped

- **rdv-1** — claim refusal over a revision-reopened dep names the dep, quotes NEEDS_REVISION, states both sanctioned roads; ordinary unmet-dep text byte-identical; new CLI-driven integration suite (2 file(s) changed)
- **rdv-2** — mid-phase session-close warning renders the third sanctioned exit (decision-0017 capture stub via bee capture add) beside finish-and-cap and handoff (2 file(s) changed)

## Verify

Each cell below was capped only against a recorded passing verify result — bee refuses a cap without one.

- **rdv-1** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml`
- **rdv-2** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml`

## Deviations

- **rdv-1** — real refusal site is handlers_write.rs claimCell closure, not read.rs (read.rs only filters ready/claim-next); tests live in a new tests/revision_deadlock_visibility.rs binary because the CLI handlers root off current_dir

## Provenance

Proposed by `bee knowledge promote --work revision-deadlock-visibility` from 2 capped cell trace(s) in `.bee/cells/` and the anchor `.bee/logs/scribing-runs.jsonl`. Every line above is copied from a trace or from the work item; nothing here is curated truth until a human or agent accepts it.

(b) AREA UPDATES — candidate spec-sync bullets, each citing its cell

areas: from the scribing stamp for "revision-deadlock-visibility" — .bee/logs/scribing-runs.jsonl's most recent entry (2026-08-11T04:17:11.868Z), the work item declares no bee.areas.

area workflow-state:
  - [rdv-1] claim refusal over a revision-reopened dep names the dep, quotes NEEDS_REVISION, states both sanctioned roads; ordinary unmet-dep text byte-identical; new CLI-driven integration suite — feature-wide sync per the scribing stamp, 2 file(s) changed (trace .bee/cells/rdv-1.json)
  - [rdv-2] mid-phase session-close warning renders the third sanctioned exit (decision-0017 capture stub via bee capture add) beside finish-and-cap and handoff — feature-wide sync per the scribing stamp, 2 file(s) changed (trace .bee/cells/rdv-2.json)

(c) PATTERN CANDIDATES — candidate bee.pattern concepts, bee.polarity pitfall

from cell rdv-1 — save as docs/knowledge/patterns/revision-deadlock-visibility-rdv-1-pitfall.md

---
type: bee.pattern
title: revision-deadlock-visibility cell rdv-1 — pitfall candidate
description: "Pitfall candidate mined from cell rdv-1's capped trace: real refusal site is handlers_write.rs claimCell closure, not read.rs (read.rs only filters ready/claim-next); tests live in a new tests/revision_deadlock_visi…"
timestamp: 2026-08-11
bee:
  id: revision-deadlock-visibility-rdv-1-pitfall
  lifecycle: draft
  areas: [workflow-state]
  sources: [.bee/cells/rdv-1.json]
  polarity: pitfall
---

# revision-deadlock-visibility cell rdv-1 — pitfall candidate

## What the cell did

claim refusal over a revision-reopened dep names the dep, quotes NEEDS_REVISION, states both sanctioned roads; ordinary unmet-dep text byte-identical; new CLI-driven integration suite

## Recorded evidence (verbatim from .bee/cells/rdv-1.json)

- **deviation** — real refusal site is handlers_write.rs claimCell closure, not read.rs (read.rs only filters ready/claim-next); tests live in a new tests/revision_deadlock_visibility.rs binary because the CLI handlers root off current_dir

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

knowledge promote: 2 capped cell(s) mined, 1 delivery draft, 2 area bullet(s), 1 pattern candidate(s), 0 file(s) written.