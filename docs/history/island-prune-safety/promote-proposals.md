promote proposal for work item "island-prune-safety" (.bee/logs/scribing-runs.jsonl + docs/history/island-prune-safety/promote-proposals.md) — 1 capped cell(s): ips-1
anchor: ledger — .bee/logs/scribing-runs.jsonl, docs/history/island-prune-safety/promote-proposals.md
PROPOSAL ONLY — nothing was written. Applying any section below is a human or agent decision.

(a) DELIVERY DRAFT — save as docs/knowledge/work/island-prune-safety/delivery.md

---
type: bee.delivery
title: island-prune-safety — delivery
description: "Delivery record proposed by bee knowledge promote for work item island-prune-safety: 1 capped cell(s), 0 recorded deviation(s)."
timestamp: 2026-08-10
bee:
  id: island-prune-safety-delivery
  lifecycle: active
  areas: [worktree-parallelism]
  required_context: [.bee/logs/scribing-runs.jsonl, docs/history/island-prune-safety/promote-proposals.md]
  sources: [.bee/logs/scribing-runs.jsonl, docs/history/island-prune-safety/promote-proposals.md, .bee/cells/archive/island-prune-safety/ips-1.json]
---

# island-prune-safety — Delivery

## What shipped

- **ips-1** — sync_worktree_cells computes tracked set once (git ls-files); both prune passes skip tracked entries; git-unavailable = prune nothing; P1 regression pin: island git-status clean after bootstrap from real worktree checkout (2 file(s) changed)

## Verify

Each cell below was capped only against a recorded passing verify result — bee refuses a cap without one.

- **ips-1** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml`

## Deviations

None recorded in the capped cell traces.

## Provenance

Proposed by `bee knowledge promote --work island-prune-safety` from 1 capped cell trace(s) in `.bee/cells/` and the anchor `.bee/logs/scribing-runs.jsonl`, `docs/history/island-prune-safety/promote-proposals.md`. Every line above is copied from a trace or from the work item; nothing here is curated truth until a human or agent accepts it.

(b) AREA UPDATES — candidate spec-sync bullets, each citing its cell

areas: from the scribing stamp for "island-prune-safety" — .bee/logs/scribing-runs.jsonl's most recent entry (2026-08-10T16:22:35.017Z), the work item declares no bee.areas.

area worktree-parallelism:
  - [ips-1] sync_worktree_cells computes tracked set once (git ls-files); both prune passes skip tracked entries; git-unavailable = prune nothing; P1 regression pin: island git-status clean after bootstrap from real worktree checkout — feature-wide sync per the scribing stamp, 2 file(s) changed (trace .bee/cells/archive/island-prune-safety/ips-1.json)

(c) PATTERN CANDIDATES — candidate bee.pattern concepts, bee.polarity pitfall

None: no capped cell trace carries a deviation or a failure signature.

knowledge promote: 1 capped cell(s) mined, 1 delivery draft, 1 area bullet(s), 0 pattern candidate(s), 0 file(s) written.