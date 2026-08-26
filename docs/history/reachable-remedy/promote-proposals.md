promote proposal for work item "reachable-remedy" (.bee/logs/scribing-runs.jsonl + .bee/lanes/reachable-remedy.json) — 1 capped cell(s): rr-1
anchor: ledger — .bee/logs/scribing-runs.jsonl, .bee/lanes/reachable-remedy.json
PROPOSAL ONLY — nothing was written. Applying any section below is a human or agent decision.

(a) DELIVERY DRAFT — save as docs/knowledge/work/reachable-remedy/delivery.md

---
type: bee.delivery
title: reachable-remedy — delivery
description: "Delivery record proposed by bee knowledge promote for work item reachable-remedy: 1 capped cell(s), 3 recorded deviation(s)."
timestamp: 2026-08-26
bee:
  id: reachable-remedy-delivery
  lifecycle: active
  areas: [workflow-state]
  required_context: [.bee/logs/scribing-runs.jsonl, .bee/lanes/reachable-remedy.json]
  sources: [.bee/logs/scribing-runs.jsonl, .bee/lanes/reachable-remedy.json, .bee/cells/rr-1.json]
---

# reachable-remedy — Delivery

## What shipped

- **rr-1** — The unregistered-worker refusal now names dispatch prepare --claim, the door registration actually rides on, instead of a control-plane verb that refuses from the worktree reading the message (1 file(s) changed)

## Verify

Each cell below was capped only against a recorded passing verify result — bee refuses a cap without one.

- **rr-1** — `cargo test --release --manifest-path packages/bee-rs/Cargo.toml verbs::cells`

## Deviations

- **rr-1** — Ran inline rather than dispatched: tiny lane, one message string, and the bee-build agent type is no longer available in this session. Kind: hit an unforeseen obstacle.
- **rr-1** — Found while correcting my own misfiled P2 — I had filed that dispatch never registers a worker; reading prepare.rs:1814 showed it does, but only on the --claim path, so the real defect was the unreachable remedy rather than a missing write. Kind: the plan was wrong about a fact.
- **rr-1** — sync-ack: A refusal message's own wording, not a rule any skill states. The workflow-state skills teach that cells from small up run through dispatched workers — unchanged here; only the FIX line now points at a door the reader can open. No skill documents the old wording.

## Provenance

Proposed by `bee knowledge promote --work reachable-remedy` from 1 capped cell trace(s) in `.bee/cells/` and the anchor `.bee/logs/scribing-runs.jsonl`, `.bee/lanes/reachable-remedy.json`. Every line above is copied from a trace or from the work item; nothing here is curated truth until a human or agent accepts it.

(b) AREA UPDATES — candidate spec-sync bullets, each citing its cell

areas: from the scribing stamp for "reachable-remedy" — .bee/logs/scribing-runs.jsonl's most recent entry (2026-08-26T05:15:00.948Z), the work item declares no bee.areas.

area workflow-state:
  - [rr-1] The unregistered-worker refusal now names dispatch prepare --claim, the door registration actually rides on, instead of a control-plane verb that refuses from the worktree reading the message — feature-wide sync per the scribing stamp, 1 file(s) changed (trace .bee/cells/rr-1.json)

(c) PATTERN CANDIDATES — candidate bee.pattern concepts, bee.polarity pitfall

from cell rr-1 — save as docs/knowledge/patterns/reachable-remedy-rr-1-pitfall.md

---
type: bee.pattern
title: reachable-remedy cell rr-1 — pitfall candidate
description: "Pitfall candidate mined from cell rr-1's capped trace: Ran inline rather than dispatched: tiny lane, one message string, and the bee-build agent type is no longer available in this session. Kind: hit an unforeseen …"
timestamp: 2026-08-26
bee:
  id: reachable-remedy-rr-1-pitfall
  lifecycle: draft
  areas: [workflow-state]
  sources: [.bee/cells/rr-1.json]
  polarity: pitfall
---

# reachable-remedy cell rr-1 — pitfall candidate

## What the cell did

The unregistered-worker refusal now names dispatch prepare --claim, the door registration actually rides on, instead of a control-plane verb that refuses from the worktree reading the message

## Recorded evidence (verbatim from .bee/cells/rr-1.json)

- **deviation** — Ran inline rather than dispatched: tiny lane, one message string, and the bee-build agent type is no longer available in this session. Kind: hit an unforeseen obstacle.
- **deviation** — Found while correcting my own misfiled P2 — I had filed that dispatch never registers a worker; reading prepare.rs:1814 showed it does, but only on the --claim path, so the real defect was the unreachable remedy rather than a missing write. Kind: the plan was wrong about a fact.
- **deviation** — sync-ack: A refusal message's own wording, not a rule any skill states. The workflow-state skills teach that cells from small up run through dispatched workers — unchanged here; only the FIX line now points at a door the reader can open. No skill documents the old wording.

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

knowledge promote: 1 capped cell(s) mined, 1 delivery draft, 1 area bullet(s), 1 pattern candidate(s), 0 file(s) written.