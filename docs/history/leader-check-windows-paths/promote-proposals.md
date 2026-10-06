promote proposal for work item "leader-check-windows-paths" (docs/history/leader-check-windows-paths/plan.md) — 1 capped cell(s): lcwp-1
anchor: history — docs/history/leader-check-windows-paths/plan.md
PROPOSAL ONLY — nothing was written. Applying any section below is a human or agent decision.

(a) DELIVERY DRAFT — save as docs/knowledge/work/leader-check-windows-paths/delivery.md

---
type: bee.delivery
title: leader-check-windows-paths — delivery
description: "Delivery record proposed by bee knowledge promote for work item leader-check-windows-paths: 1 capped cell(s), 1 recorded deviation(s)."
timestamp: 2026-10-06
bee:
  id: leader-check-windows-paths-delivery
  lifecycle: active
  areas: [worktree-parallelism]
  required_context: [docs/history/leader-check-windows-paths/plan.md]
  sources: [docs/history/leader-check-windows-paths/plan.md, .bee/cells/lcwp-1.json]
---

# leader-check-windows-paths — Delivery

## What shipped

- **lcwp-1** — leader-check canonicalizes on a failed strip so aliased worktree paths match (2 file(s) changed)

## Verify

Each cell below was capped only against a recorded passing verify result — bee refuses a cap without one.

- **lcwp-1** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml --bin bee leader_check` — 28 leader_check tests; new unix symlink test red first with the Windows failure shape; full --bin bee 4131 passed on Linux; Windows itself not run locally, left to Windows CI

## Deviations

- **lcwp-1** — sync-ack: no owned skill describes leader-check path matching

## Provenance

Proposed by `bee knowledge promote --work leader-check-windows-paths` from 1 capped cell trace(s) in `.bee/cells/` and the anchor `docs/history/leader-check-windows-paths/plan.md`. Every line above is copied from a trace or from the work item; nothing here is curated truth until a human or agent accepts it.

(b) AREA UPDATES — candidate spec-sync bullets, each citing its cell

areas: from the scribing stamp for "leader-check-windows-paths" — .bee/logs/scribing-runs.jsonl's most recent entry (2026-10-06T17:15:31.827Z), the work item declares no bee.areas.

area worktree-parallelism:
  - [lcwp-1] leader-check canonicalizes on a failed strip so aliased worktree paths match — feature-wide sync per the scribing stamp, 2 file(s) changed (trace .bee/cells/lcwp-1.json)

(c) PATTERN CANDIDATES — candidate bee.pattern concepts, bee.polarity pitfall

from cell lcwp-1 — save as docs/knowledge/patterns/leader-check-windows-paths-lcwp-1-pitfall.md

---
type: bee.pattern
title: leader-check-windows-paths cell lcwp-1 — pitfall candidate
description: "Pitfall candidate mined from cell lcwp-1's capped trace: sync-ack: no owned skill describes leader-check path matching"
timestamp: 2026-10-06
bee:
  id: leader-check-windows-paths-lcwp-1-pitfall
  lifecycle: draft
  areas: [worktree-parallelism]
  sources: [.bee/cells/lcwp-1.json]
  polarity: pitfall
---

# leader-check-windows-paths cell lcwp-1 — pitfall candidate

## What the cell did

leader-check canonicalizes on a failed strip so aliased worktree paths match

## Recorded evidence (verbatim from .bee/cells/lcwp-1.json)

- **deviation** — sync-ack: no owned skill describes leader-check path matching

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

knowledge promote: 1 capped cell(s) mined, 1 delivery draft, 1 area bullet(s), 1 pattern candidate(s), 0 file(s) written.