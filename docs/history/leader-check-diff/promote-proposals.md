promote proposal for work item "leader-check-diff" (.bee/lanes/leader-check-diff.json) — 1 capped cell(s): lcdiff-1
anchor: ledger — .bee/lanes/leader-check-diff.json
PROPOSAL ONLY — nothing was written. Applying any section below is a human or agent decision.

(a) DELIVERY DRAFT — save as docs/knowledge/work/leader-check-diff/delivery.md

---
type: bee.delivery
title: leader-check-diff — delivery
description: "Delivery record proposed by bee knowledge promote for work item leader-check-diff: 1 capped cell(s), 2 recorded deviation(s)."
timestamp: 2026-09-21
bee:
  id: leader-check-diff-delivery
  lifecycle: active
  required_context: [.bee/lanes/leader-check-diff.json]
  sources: [.bee/lanes/leader-check-diff.json, .bee/cells/lcdiff-1.json]
---

# leader-check-diff — Delivery

## What shipped

- **lcdiff-1** — leader-check verifies files_changed against the cell's trailer-commit diff and records commit_diff on each entry (1 file(s) changed)

## Verify

Each cell below was capped only against a recorded passing verify result — bee refuses a cap without one.

- **lcdiff-1** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee --bin bee leader_check` — filter covers leader_check.rs tests plus close and worktree-merge leader-check door tests (24 passed)

## Deviations

- **lcdiff-1** — capped with --sync-ack instead of editing a bee-swarming skill — the SYNC_DOOR asked for a skill touch but the cell names only leader_check.rs and affects_skills is empty — hit an unforeseen obstacle
- **lcdiff-1** — sync-ack: cell planned affects_skills []; skills/bee-swarming only names the leader-check verb, not its checks; any skill note is an orchestrator scope call outside this cell's files

## Provenance

Proposed by `bee knowledge promote --work leader-check-diff` from 1 capped cell trace(s) in `.bee/cells/` and the anchor `.bee/lanes/leader-check-diff.json`. Every line above is copied from a trace or from the work item; nothing here is curated truth until a human or agent accepts it.

(b) AREA UPDATES — candidate spec-sync bullets, each citing its cell

None: the work item declares no bee.areas, so there is no area to sync (D19).

(c) PATTERN CANDIDATES — candidate bee.pattern concepts, bee.polarity pitfall

from cell lcdiff-1 — save as docs/knowledge/patterns/leader-check-diff-lcdiff-1-pitfall.md

---
type: bee.pattern
title: leader-check-diff cell lcdiff-1 — pitfall candidate
description: "Pitfall candidate mined from cell lcdiff-1's capped trace: capped with --sync-ack instead of editing a bee-swarming skill — the SYNC_DOOR asked for a skill touch but the cell names only leader_check.rs and affects_skil…"
timestamp: 2026-09-21
bee:
  id: leader-check-diff-lcdiff-1-pitfall
  lifecycle: draft
  sources: [.bee/cells/lcdiff-1.json]
  polarity: pitfall
---

# leader-check-diff cell lcdiff-1 — pitfall candidate

## What the cell did

leader-check verifies files_changed against the cell's trailer-commit diff and records commit_diff on each entry

## Recorded evidence (verbatim from .bee/cells/lcdiff-1.json)

- **deviation** — capped with --sync-ack instead of editing a bee-swarming skill — the SYNC_DOOR asked for a skill touch but the cell names only leader_check.rs and affects_skills is empty — hit an unforeseen obstacle
- **deviation** — sync-ack: cell planned affects_skills []; skills/bee-swarming only names the leader-check verb, not its checks; any skill note is an orchestrator scope call outside this cell's files

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

knowledge promote: 1 capped cell(s) mined, 1 delivery draft, 0 area bullet(s), 1 pattern candidate(s), 0 file(s) written.