promote proposal for work item "review-p1-batch-fixes" (.bee/logs/scribing-runs.jsonl) — 2 capped cell(s): rpb-1, rpb-2
anchor: ledger — .bee/logs/scribing-runs.jsonl
PROPOSAL ONLY — nothing was written. Applying any section below is a human or agent decision.

(a) DELIVERY DRAFT — save as docs/knowledge/work/review-p1-batch-fixes/delivery.md

---
type: bee.delivery
title: review-p1-batch-fixes — delivery
description: "Delivery record proposed by bee knowledge promote for work item review-p1-batch-fixes: 2 capped cell(s), 0 recorded deviation(s)."
timestamp: 2026-08-10
bee:
  id: review-p1-batch-fixes-delivery
  lifecycle: active
  areas: [worktree-parallelism, workflow-state]
  required_context: [.bee/logs/scribing-runs.jsonl]
  sources: [.bee/logs/scribing-runs.jsonl, .bee/cells/rpb-1.json, .bee/cells/rpb-2.json]
---

# review-p1-batch-fixes — Delivery

## What shipped

- **rpb-1** — sync_worktree_cells checks symlink_metadata on island .bee, .bee/cells, source cells, archive dirs before any prune/fill; symlink = whole sync skipped, named in report (CellsSync::Skipped); red-first proven against pre-fix code (2 file(s) changed)
- **rpb-2** — gpg-hang defense proven by failing-signer stubs on BOTH paths (red when flag removed); run_git stdin null; merge commit --no-gpg-sign; masking fixture line retired (4 file(s) changed)

## Verify

Each cell below was capped only against a recorded passing verify result — bee refuses a cap without one.

- **rpb-1** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml`
- **rpb-2** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml`

## Deviations

None recorded in the capped cell traces.

## Provenance

Proposed by `bee knowledge promote --work review-p1-batch-fixes` from 2 capped cell trace(s) in `.bee/cells/` and the anchor `.bee/logs/scribing-runs.jsonl`. Every line above is copied from a trace or from the work item; nothing here is curated truth until a human or agent accepts it.

(b) AREA UPDATES — candidate spec-sync bullets, each citing its cell

areas: from the scribing stamp for "review-p1-batch-fixes" — .bee/logs/scribing-runs.jsonl's most recent entry (2026-08-10T20:48:00.469Z), the work item declares no bee.areas.

area worktree-parallelism:
  - [rpb-1] sync_worktree_cells checks symlink_metadata on island .bee, .bee/cells, source cells, archive dirs before any prune/fill; symlink = whole sync skipped, named in report (CellsSync::Skipped); red-first proven against pre-fix code — feature-wide sync per the scribing stamp, 2 file(s) changed (trace .bee/cells/rpb-1.json)
  - [rpb-2] gpg-hang defense proven by failing-signer stubs on BOTH paths (red when flag removed); run_git stdin null; merge commit --no-gpg-sign; masking fixture line retired — feature-wide sync per the scribing stamp, 4 file(s) changed (trace .bee/cells/rpb-2.json)

area workflow-state:
  - [rpb-1] sync_worktree_cells checks symlink_metadata on island .bee, .bee/cells, source cells, archive dirs before any prune/fill; symlink = whole sync skipped, named in report (CellsSync::Skipped); red-first proven against pre-fix code — feature-wide sync per the scribing stamp, 2 file(s) changed (trace .bee/cells/rpb-1.json)
  - [rpb-2] gpg-hang defense proven by failing-signer stubs on BOTH paths (red when flag removed); run_git stdin null; merge commit --no-gpg-sign; masking fixture line retired — feature-wide sync per the scribing stamp, 4 file(s) changed (trace .bee/cells/rpb-2.json)

(c) PATTERN CANDIDATES — candidate bee.pattern concepts, bee.polarity pitfall

None: no capped cell trace carries a deviation or a failure signature.

knowledge promote: 2 capped cell(s) mined, 1 delivery draft, 4 area bullet(s), 0 pattern candidate(s), 0 file(s) written.