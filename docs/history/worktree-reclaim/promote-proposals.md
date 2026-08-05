promote proposal for work item "worktree-reclaim" (.bee/logs/scribing-runs.jsonl + .bee/lanes/worktree-reclaim.json) — 6 capped cell(s): wr-0, wr-1, wr-2, wr-3, wr-4, wr-5
anchor: ledger — .bee/logs/scribing-runs.jsonl, .bee/lanes/worktree-reclaim.json
PROPOSAL ONLY — nothing was written. Applying any section below is a human or agent decision.

(a) DELIVERY DRAFT — save as docs/knowledge/work/worktree-reclaim/delivery.md

---
type: bee.delivery
title: worktree-reclaim — delivery
description: "Delivery record proposed by bee knowledge promote for work item worktree-reclaim: 6 capped cell(s), 0 recorded deviation(s)."
timestamp: 2026-08-05
bee:
  id: worktree-reclaim-delivery
  lifecycle: active
  areas: [worktree-parallelism]
  required_context: [.bee/logs/scribing-runs.jsonl, .bee/lanes/worktree-reclaim.json]
  sources: [.bee/logs/scribing-runs.jsonl, .bee/lanes/worktree-reclaim.json, .bee/cells/wr-0.json, .bee/cells/wr-1.json, .bee/cells/wr-2.json, .bee/cells/wr-3.json, .bee/cells/wr-4.json, .bee/cells/wr-5.json]
---

# worktree-reclaim — Delivery

## What shipped

- **wr-0** — Pinned the dirty, remove-failed, and branch-delete-failed perform_cleanup key-order shapes with three new tests (1 file(s) changed)
- **wr-1** — Lifted perform_cleanup's five teardown steps into one teardown_worktree helper with a directory-removal guard; run_unregister now wires to its registry half and no longer orphans workspace records (3 file(s) changed)
- **wr-2** — Added the fail-closed dead-worktree classifier (classify_worktree, resolve_prune_base) with a keep-on-doubt test per condition; no subcommand wired yet (3 file(s) changed)
- **wr-3** — Added bee worktree prune over classify_worktree, with dry-run, older-than-days, union enumeration of grants and workspace records, and a hand-edited registry payload entry (5 file(s) changed)
- **wr-4** — Cleanup runs by default on a real merge; --no-cleanup and worktree_cleanup_on_merge:false opt out, non-boolean values refuse, ALREADY_UP_TO_DATE removes nothing (9 file(s) changed)
- **wr-5** — bee orient and the session preamble now surface a reclaimable-worktree count (grants file + one metadata() call per candidate, no git, no size walk) above a count of one, pointing at bee worktree prune --dry-run (5 file(s) changed)

## Verify

Each cell below was capped only against a recorded passing verify result — bee refuses a cap without one.

- **wr-0** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml`
- **wr-1** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml`
- **wr-2** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml`
- **wr-3** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml`
- **wr-4** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml`
- **wr-5** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml`

## Deviations

None recorded in the capped cell traces.

## Provenance

Proposed by `bee knowledge promote --work worktree-reclaim` from 6 capped cell trace(s) in `.bee/cells/` and the anchor `.bee/logs/scribing-runs.jsonl`, `.bee/lanes/worktree-reclaim.json`. Every line above is copied from a trace or from the work item; nothing here is curated truth until a human or agent accepts it.

(b) AREA UPDATES — candidate spec-sync bullets, each citing its cell

areas: from the scribing stamp for "worktree-reclaim" — .bee/logs/scribing-runs.jsonl's most recent entry (2026-08-05T11:42:02.584Z), the work item declares no bee.areas.

area worktree-parallelism:
  - [wr-1] Lifted perform_cleanup's five teardown steps into one teardown_worktree helper with a directory-removal guard; run_unregister now wires to its registry half and no longer orphans workspace records — feature-wide sync per the scribing stamp, 3 file(s) changed (trace .bee/cells/wr-1.json)
  - [wr-2] Added the fail-closed dead-worktree classifier (classify_worktree, resolve_prune_base) with a keep-on-doubt test per condition; no subcommand wired yet — feature-wide sync per the scribing stamp, 3 file(s) changed (trace .bee/cells/wr-2.json)
  - [wr-3] Added bee worktree prune over classify_worktree, with dry-run, older-than-days, union enumeration of grants and workspace records, and a hand-edited registry payload entry — feature-wide sync per the scribing stamp, 5 file(s) changed (trace .bee/cells/wr-3.json)
  - [wr-4] Cleanup runs by default on a real merge; --no-cleanup and worktree_cleanup_on_merge:false opt out, non-boolean values refuse, ALREADY_UP_TO_DATE removes nothing — feature-wide sync per the scribing stamp, 9 file(s) changed (trace .bee/cells/wr-4.json)
  - [wr-5] bee orient and the session preamble now surface a reclaimable-worktree count (grants file + one metadata() call per candidate, no git, no size walk) above a count of one, pointing at bee worktree prune --dry-run — feature-wide sync per the scribing stamp, 5 file(s) changed (trace .bee/cells/wr-5.json)

(c) PATTERN CANDIDATES — candidate bee.pattern concepts, bee.polarity pitfall

None: no capped cell trace carries a deviation or a failure signature.

knowledge promote: 6 capped cell(s) mined, 1 delivery draft, 5 area bullet(s), 0 pattern candidate(s), 0 file(s) written.