promote proposal for work item "leader-check-worktree-root" (docs/history/leader-check-worktree-root/plan.md) — 1 capped cell(s): lcwr-1
anchor: history — docs/history/leader-check-worktree-root/plan.md
PROPOSAL ONLY — nothing was written. Applying any section below is a human or agent decision.

(a) DELIVERY DRAFT — save as docs/knowledge/work/leader-check-worktree-root/delivery.md

---
type: bee.delivery
title: leader-check-worktree-root — delivery
description: "Delivery record proposed by bee knowledge promote for work item leader-check-worktree-root: 1 capped cell(s), 1 recorded deviation(s)."
timestamp: 2026-10-06
bee:
  id: leader-check-worktree-root-delivery
  lifecycle: active
  areas: [worktree-parallelism]
  required_context: [docs/history/leader-check-worktree-root/plan.md]
  sources: [docs/history/leader-check-worktree-root/plan.md, .bee/cells/archive/leader-check-worktree-root/lcwr-1.json]
---

# leader-check-worktree-root — Delivery

## What shipped

- **lcwr-1** — leader-check resolves artifacts against the feature worktree root (2 file(s) changed)

## Verify

Each cell below was capped only against a recorded passing verify result — bee refuses a cap without one.

- **lcwr-1** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml --bin bee leader_check` — 27 leader_check tests incl. 3 new worktree cases, red first for the reported reasons; full --bin bee 4130 passed, integration tests not run

## Deviations

- **lcwr-1** — sync-ack: no owned skill describes where leader-check resolves artifact paths; routing-and-contracts.md:181 (deferral is the named escape) stays true

## Provenance

Proposed by `bee knowledge promote --work leader-check-worktree-root` from 1 capped cell trace(s) in `.bee/cells/` and the anchor `docs/history/leader-check-worktree-root/plan.md`. Every line above is copied from a trace or from the work item; nothing here is curated truth until a human or agent accepts it.

(b) AREA UPDATES — candidate spec-sync bullets, each citing its cell

areas: from the scribing stamp for "leader-check-worktree-root" — .bee/logs/scribing-runs.jsonl's most recent entry (2026-10-06T15:40:04.421Z), the work item declares no bee.areas.

area worktree-parallelism:
  - [lcwr-1] leader-check resolves artifacts against the feature worktree root — feature-wide sync per the scribing stamp, 2 file(s) changed (trace .bee/cells/archive/leader-check-worktree-root/lcwr-1.json)

(c) PATTERN CANDIDATES — candidate bee.pattern concepts, bee.polarity pitfall

from cell lcwr-1 — save as docs/knowledge/patterns/leader-check-worktree-root-lcwr-1-pitfall.md

---
type: bee.pattern
title: leader-check-worktree-root cell lcwr-1 — pitfall candidate
description: "Pitfall candidate mined from cell lcwr-1's capped trace: sync-ack: no owned skill describes where leader-check resolves artifact paths; routing-and-contracts.md:181 (deferral is the named escape) stays true"
timestamp: 2026-10-06
bee:
  id: leader-check-worktree-root-lcwr-1-pitfall
  lifecycle: draft
  areas: [worktree-parallelism]
  sources: [.bee/cells/archive/leader-check-worktree-root/lcwr-1.json]
  polarity: pitfall
---

# leader-check-worktree-root cell lcwr-1 — pitfall candidate

## What the cell did

leader-check resolves artifacts against the feature worktree root

## Recorded evidence (verbatim from .bee/cells/archive/leader-check-worktree-root/lcwr-1.json)

- **deviation** — sync-ack: no owned skill describes where leader-check resolves artifact paths; routing-and-contracts.md:181 (deferral is the named escape) stays true

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

knowledge promote: 1 capped cell(s) mined, 1 delivery draft, 1 area bullet(s), 1 pattern candidate(s), 0 file(s) written.