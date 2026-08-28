promote proposal for work item "merge-door-precision" (.bee/logs/scribing-runs.jsonl + .bee/lanes/merge-door-precision.json) — 1 capped cell(s): mdp-1
anchor: ledger — .bee/logs/scribing-runs.jsonl, .bee/lanes/merge-door-precision.json
PROPOSAL ONLY — nothing was written. Applying any section below is a human or agent decision.

(a) DELIVERY DRAFT — save as docs/knowledge/work/merge-door-precision/delivery.md

---
type: bee.delivery
title: merge-door-precision — delivery
description: "Delivery record proposed by bee knowledge promote for work item merge-door-precision: 1 capped cell(s), 3 recorded deviation(s)."
timestamp: 2026-08-28
bee:
  id: merge-door-precision-delivery
  lifecycle: active
  areas: [worktree-parallelism]
  required_context: [.bee/logs/scribing-runs.jsonl, .bee/lanes/merge-door-precision.json]
  sources: [.bee/logs/scribing-runs.jsonl, .bee/lanes/merge-door-precision.json, .bee/cells/mdp-1.json]
---

# merge-door-precision — Delivery

## What shipped

- **mdp-1** — The merge door now refuses only for dirt it can collide with: tracked dirt unchanged, untracked dirt blocks only inside the branch's merge-base changed-file set (3 file(s) changed)

## Verify

Each cell below was capped only against a recorded passing verify result — bee refuses a cap without one.

- **mdp-1** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee --bins worktree && PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee --bins merge`

## Deviations

- **mdp-1** — Deleted git_status_porcelain_excluding_untracked_all instead of keeping it beside the new -z splitter — it had exactly four callers, all in the code this cell rewrites, and leaving it would have been dead code in a non-test build; its `?? docs/history/<theirs>/plan.md` rationale moved verbatim onto the new reader — found a better route
- **mdp-1** — Wrote the two new refusal tests against plain git commands rather than the new helpers, so the red run failed on behaviour instead of on a missing symbol — found a better route
- **mdp-1** — docs/knowledge/areas/worktree-parallelism/returning-and-the-merge-gate.md lines 197-201 still state the old rule ("any dirty path outside those two roots still refuses exactly as before") and I left it untouched — it is not in the cell's files, so the sync is the orchestrator's scope call — hit an unforeseen obstacle

## Provenance

Proposed by `bee knowledge promote --work merge-door-precision` from 1 capped cell trace(s) in `.bee/cells/` and the anchor `.bee/logs/scribing-runs.jsonl`, `.bee/lanes/merge-door-precision.json`. Every line above is copied from a trace or from the work item; nothing here is curated truth until a human or agent accepts it.

(b) AREA UPDATES — candidate spec-sync bullets, each citing its cell

areas: from the scribing stamp for "merge-door-precision" — .bee/logs/scribing-runs.jsonl's most recent entry (2026-08-28T06:00:47.573Z), the work item declares no bee.areas.

area worktree-parallelism:
  - [mdp-1] The merge door now refuses only for dirt it can collide with: tracked dirt unchanged, untracked dirt blocks only inside the branch's merge-base changed-file set — feature-wide sync per the scribing stamp, 3 file(s) changed (trace .bee/cells/mdp-1.json)

(c) PATTERN CANDIDATES — candidate bee.pattern concepts, bee.polarity pitfall

from cell mdp-1 — save as docs/knowledge/patterns/merge-door-precision-mdp-1-pitfall.md

---
type: bee.pattern
title: merge-door-precision cell mdp-1 — pitfall candidate
description: "Pitfall candidate mined from cell mdp-1's capped trace: Deleted git_status_porcelain_excluding_untracked_all instead of keeping it beside the new -z splitter — it had exactly four callers, all in the code this cell …"
timestamp: 2026-08-28
bee:
  id: merge-door-precision-mdp-1-pitfall
  lifecycle: draft
  areas: [worktree-parallelism]
  sources: [.bee/cells/mdp-1.json]
  polarity: pitfall
---

# merge-door-precision cell mdp-1 — pitfall candidate

## What the cell did

The merge door now refuses only for dirt it can collide with: tracked dirt unchanged, untracked dirt blocks only inside the branch's merge-base changed-file set

## Recorded evidence (verbatim from .bee/cells/mdp-1.json)

- **deviation** — Deleted git_status_porcelain_excluding_untracked_all instead of keeping it beside the new -z splitter — it had exactly four callers, all in the code this cell rewrites, and leaving it would have been dead code in a non-test build; its `?? docs/history/<theirs>/plan.md` rationale moved verbatim onto the new reader — found a better route
- **deviation** — Wrote the two new refusal tests against plain git commands rather than the new helpers, so the red run failed on behaviour instead of on a missing symbol — found a better route
- **deviation** — docs/knowledge/areas/worktree-parallelism/returning-and-the-merge-gate.md lines 197-201 still state the old rule ("any dirty path outside those two roots still refuses exactly as before") and I left it untouched — it is not in the cell's files, so the sync is the orchestrator's scope call — hit an unforeseen obstacle

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

knowledge promote: 1 capped cell(s) mined, 1 delivery draft, 1 area bullet(s), 1 pattern candidate(s), 0 file(s) written.