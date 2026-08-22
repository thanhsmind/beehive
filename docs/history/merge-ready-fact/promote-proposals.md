promote proposal for work item "merge-ready-fact" (docs/history/merge-ready-fact/CONTEXT.md + docs/history/merge-ready-fact/plan.md) — 3 capped cell(s): mrf-1, mrf-2, mrf-3
anchor: history — docs/history/merge-ready-fact/CONTEXT.md, docs/history/merge-ready-fact/plan.md
PROPOSAL ONLY — nothing was written. Applying any section below is a human or agent decision.

(a) DELIVERY DRAFT — save as docs/knowledge/work/merge-ready-fact/delivery.md

---
type: bee.delivery
title: merge-ready-fact — delivery
description: "Delivery record proposed by bee knowledge promote for work item merge-ready-fact: 3 capped cell(s), 13 recorded deviation(s)."
timestamp: 2026-08-22
bee:
  id: merge-ready-fact-delivery
  lifecycle: active
  areas: [workflow-state]
  required_context: [docs/history/merge-ready-fact/CONTEXT.md, docs/history/merge-ready-fact/plan.md]
  sources: [docs/history/merge-ready-fact/CONTEXT.md, docs/history/merge-ready-fact/plan.md, .bee/cells/mrf-1.json, .bee/cells/mrf-2.json, .bee/cells/mrf-3.json]
---

# merge-ready-fact — Delivery

## What shipped

- **mrf-1** — The last cap of a feature with a worktree grant now stores merge_ready on the feature record; every reopen door clears it (5 file(s) changed)
- **mrf-2** — merge_ready is written by close, gate uat, worktree merge and unregister, and surfaced verbatim on status lane rows (9 file(s) changed)
- **mrf-3** — Documented merge-readiness as a workflow-state concept: meaning, the five fields, the one set moment and four change moments, a lifecycle diagram, edge cases, the never-read rule, readers, and implementation pointers; workflow-state index regenerated. (4 file(s) changed)

## Verify

Each cell below was capped only against a recorded passing verify result — bee refuses a cap without one.

- **mrf-1** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml merge_ready`
- **mrf-2** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml merge_ready`
- **mrf-3** — `.bee/bin/bee knowledge check`

## Deviations

- **mrf-1** — set_after_cap resolves the DEFAULT record with no_lane=true rather than resolve_mutation_lock_scope(root, None, false): the cell text named the lane call, which refuses when no lane file exists, and the session-binding path could route the write onto a foreign lane
- **mrf-1** — added an inline #[allow(dead_code)] on set_uat/set_blocked_by, whose callers land in mrf-2 — the note names that cell as the one to drop it
- **mrf-1** — capped with --inline-reason: w-mrf-1 is a real dispatched worker but was never registered in state.workers[], and bee state worker add refuses from inside a granted worktree
- **mrf-1** — capped with --sync-ack: the cell declares affects_skills: [] and the workflow-state knowledge sync is this feature s own parallel cell mrf-3
- **mrf-1** — two tests in crates/bee/tests/opencode_plugin_contracts.rs are red on this base, before and after this cell (proved by stashing the diff and re-running) — unrelated to the touched files
- **mrf-1** — sync-ack: mrf-1 declares affects_skills: [] — the fact is additive and no skill instructs anyone to read or write it (D3); the workflow-state knowledge sync is this feature's own cell mrf-3 (docs/knowledge/areas/workflow-state/merge-readiness.md), running in parallel under w-mrf-3
- **mrf-2** — close.rs has THREE full-doors vectors, not the two the cell named: the proof-debt refusal arm assembles its own complete vector and returns before the green path. Wired all three, because the rule the cell states (record before any early-return refusal arm, so a blocked close still records why) is what asks for it; a close stopped at the tests door would otherwise record nothing. Pinned by a_close_stopped_at_the_tests_door_still_records_that_door.
- **mrf-2** — set_uat is called after run_gate_body drops its mutation locks, not immediately after the approved_gates write: the helper goes through the same ledger mutation seam and takes those very locks, so an earlier call would find them busy and fail-open into silence. Still strictly after the write.
- **mrf-2** — merge_ready.rs was not in the cell files list but had to be edited to drop the two allow(dead_code) attributes the cell asked for; reserved under w-mrf-2 before writing.
- **mrf-2** — worktree unregister takes the clear before the worktree-admin lock rather than inside it, so the record mutation locks are never nested under it. Still strictly before teardown. The test drives the extracted clear_merge_ready_for_worktree helper directly, since run_unregister itself needs a cwd-bound prelude fixture.
- **mrf-2** — sync-ack: Additive projection only (D3): merge_ready is written and surfaced, never read by any gate, door, or workflow step, so no workflow-state skill guidance changes; the cell itself predicts affects_skills [] and the concept doc docs/knowledge/areas/workflow-state/merge-readiness.md already describes this exact wiring (landed by mrf-1).
- **mrf-3** — Quoted the title and left two plain-safe sources unquoted to match the canonical frontmatter emitter — a first pass added a new not_canonical warning; the warning count is back at the 51 baseline.
- **mrf-3** — Registered w-mrf-3 in state.workers[] from the main checkout: the dispatch had not registered it, and the cap refused with no registered execution worker.

## Provenance

Proposed by `bee knowledge promote --work merge-ready-fact` from 3 capped cell trace(s) in `.bee/cells/` and the anchor `docs/history/merge-ready-fact/CONTEXT.md`, `docs/history/merge-ready-fact/plan.md`. Every line above is copied from a trace or from the work item; nothing here is curated truth until a human or agent accepts it.

(b) AREA UPDATES — candidate spec-sync bullets, each citing its cell

areas: from the scribing stamp for "merge-ready-fact" — .bee/logs/scribing-runs.jsonl's most recent entry (2026-08-22T16:56:09.997Z), the work item declares no bee.areas.

area workflow-state:
  - [mrf-1] The last cap of a feature with a worktree grant now stores merge_ready on the feature record; every reopen door clears it — feature-wide sync per the scribing stamp, 5 file(s) changed (trace .bee/cells/mrf-1.json)
  - [mrf-2] merge_ready is written by close, gate uat, worktree merge and unregister, and surfaced verbatim on status lane rows — feature-wide sync per the scribing stamp, 9 file(s) changed (trace .bee/cells/mrf-2.json)

(c) PATTERN CANDIDATES — candidate bee.pattern concepts, bee.polarity pitfall

from cell mrf-1 — save as docs/knowledge/patterns/merge-ready-fact-mrf-1-pitfall.md

---
type: bee.pattern
title: merge-ready-fact cell mrf-1 — pitfall candidate
description: "Pitfall candidate mined from cell mrf-1's capped trace: set_after_cap resolves the DEFAULT record with no_lane=true rather than resolve_mutation_lock_scope(root, None, false): the cell text named the lane call, whic…"
timestamp: 2026-08-22
bee:
  id: merge-ready-fact-mrf-1-pitfall
  lifecycle: draft
  areas: [workflow-state]
  sources: [.bee/cells/mrf-1.json]
  polarity: pitfall
---

# merge-ready-fact cell mrf-1 — pitfall candidate

## What the cell did

The last cap of a feature with a worktree grant now stores merge_ready on the feature record; every reopen door clears it

## Recorded evidence (verbatim from .bee/cells/mrf-1.json)

- **deviation** — set_after_cap resolves the DEFAULT record with no_lane=true rather than resolve_mutation_lock_scope(root, None, false): the cell text named the lane call, which refuses when no lane file exists, and the session-binding path could route the write onto a foreign lane
- **deviation** — added an inline #[allow(dead_code)] on set_uat/set_blocked_by, whose callers land in mrf-2 — the note names that cell as the one to drop it
- **deviation** — capped with --inline-reason: w-mrf-1 is a real dispatched worker but was never registered in state.workers[], and bee state worker add refuses from inside a granted worktree
- **deviation** — capped with --sync-ack: the cell declares affects_skills: [] and the workflow-state knowledge sync is this feature s own parallel cell mrf-3
- **deviation** — two tests in crates/bee/tests/opencode_plugin_contracts.rs are red on this base, before and after this cell (proved by stashing the diff and re-running) — unrelated to the touched files
- **deviation** — sync-ack: mrf-1 declares affects_skills: [] — the fact is additive and no skill instructs anyone to read or write it (D3); the workflow-state knowledge sync is this feature's own cell mrf-3 (docs/knowledge/areas/workflow-state/merge-readiness.md), running in parallel under w-mrf-3

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

from cell mrf-2 — save as docs/knowledge/patterns/merge-ready-fact-mrf-2-pitfall.md

---
type: bee.pattern
title: merge-ready-fact cell mrf-2 — pitfall candidate
description: "Pitfall candidate mined from cell mrf-2's capped trace: close.rs has THREE full-doors vectors, not the two the cell named: the proof-debt refusal arm assembles its own complete vector and returns before the green pa…"
timestamp: 2026-08-22
bee:
  id: merge-ready-fact-mrf-2-pitfall
  lifecycle: draft
  areas: [workflow-state]
  sources: [.bee/cells/mrf-2.json]
  polarity: pitfall
---

# merge-ready-fact cell mrf-2 — pitfall candidate

## What the cell did

merge_ready is written by close, gate uat, worktree merge and unregister, and surfaced verbatim on status lane rows

## Recorded evidence (verbatim from .bee/cells/mrf-2.json)

- **deviation** — close.rs has THREE full-doors vectors, not the two the cell named: the proof-debt refusal arm assembles its own complete vector and returns before the green path. Wired all three, because the rule the cell states (record before any early-return refusal arm, so a blocked close still records why) is what asks for it; a close stopped at the tests door would otherwise record nothing. Pinned by a_close_stopped_at_the_tests_door_still_records_that_door.
- **deviation** — set_uat is called after run_gate_body drops its mutation locks, not immediately after the approved_gates write: the helper goes through the same ledger mutation seam and takes those very locks, so an earlier call would find them busy and fail-open into silence. Still strictly after the write.
- **deviation** — merge_ready.rs was not in the cell files list but had to be edited to drop the two allow(dead_code) attributes the cell asked for; reserved under w-mrf-2 before writing.
- **deviation** — worktree unregister takes the clear before the worktree-admin lock rather than inside it, so the record mutation locks are never nested under it. Still strictly before teardown. The test drives the extracted clear_merge_ready_for_worktree helper directly, since run_unregister itself needs a cwd-bound prelude fixture.
- **deviation** — sync-ack: Additive projection only (D3): merge_ready is written and surfaced, never read by any gate, door, or workflow step, so no workflow-state skill guidance changes; the cell itself predicts affects_skills [] and the concept doc docs/knowledge/areas/workflow-state/merge-readiness.md already describes this exact wiring (landed by mrf-1).

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

from cell mrf-3 — save as docs/knowledge/patterns/merge-ready-fact-mrf-3-pitfall.md

---
type: bee.pattern
title: merge-ready-fact cell mrf-3 — pitfall candidate
description: "Pitfall candidate mined from cell mrf-3's capped trace: Quoted the title and left two plain-safe sources unquoted to match the canonical frontmatter emitter — a first pass added a new not_canonical warning; the warn…"
timestamp: 2026-08-22
bee:
  id: merge-ready-fact-mrf-3-pitfall
  lifecycle: draft
  areas: [workflow-state]
  sources: [.bee/cells/mrf-3.json]
  polarity: pitfall
---

# merge-ready-fact cell mrf-3 — pitfall candidate

## What the cell did

Documented merge-readiness as a workflow-state concept: meaning, the five fields, the one set moment and four change moments, a lifecycle diagram, edge cases, the never-read rule, readers, and implementation pointers; workflow-state index regenerated.

## Recorded evidence (verbatim from .bee/cells/mrf-3.json)

- **deviation** — Quoted the title and left two plain-safe sources unquoted to match the canonical frontmatter emitter — a first pass added a new not_canonical warning; the warning count is back at the 51 baseline.
- **deviation** — Registered w-mrf-3 in state.workers[] from the main checkout: the dispatch had not registered it, and the cap refused with no registered execution worker.

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

knowledge promote: 3 capped cell(s) mined, 1 delivery draft, 2 area bullet(s), 3 pattern candidate(s), 0 file(s) written.