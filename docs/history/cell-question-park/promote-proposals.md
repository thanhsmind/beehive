promote proposal for work item "cell-question-park" (docs/history/cell-question-park/CONTEXT.md + docs/history/cell-question-park/plan.md) — 4 capped cell(s): cqp-1, cqp-2, cqp-3, cqp-4
anchor: history — docs/history/cell-question-park/CONTEXT.md, docs/history/cell-question-park/plan.md
PROPOSAL ONLY — nothing was written. Applying any section below is a human or agent decision.

(a) DELIVERY DRAFT — save as docs/knowledge/work/cell-question-park/delivery.md

---
type: bee.delivery
title: cell-question-park — delivery
description: "Delivery record proposed by bee knowledge promote for work item cell-question-park: 4 capped cell(s), 21 recorded deviation(s)."
timestamp: 2026-11-06
bee:
  id: cell-question-park-delivery
  lifecycle: active
  areas: [workflow-state, worktree-parallelism]
  required_context: [docs/history/cell-question-park/CONTEXT.md, docs/history/cell-question-park/plan.md]
  sources: [docs/history/cell-question-park/CONTEXT.md, docs/history/cell-question-park/plan.md, .bee/cells/cqp-1.json, .bee/cells/cqp-2.json, .bee/cells/cqp-3.json, .bee/cells/cqp-4.json]
---

# cell-question-park — Delivery

## What shipped

- **cqp-1** — an open park question is read from trace.park: every refusal on such a cell names bee cells answer, a dissent verdict keeps it blocked, supervisor reads live plus archive (7 file(s) changed)
- **cqp-2** — bee close blocks on a parked-cell door, bee worktree merge refuses with WORKTREE_MERGE_PARKED_CELL, and the needs-you view lists each parked question (4 file(s) changed)
- **cqp-3** — Worker contract parks a question cell with [PARKED]; leader keeps working; answer via bee cells answer (7 file(s) changed)
- **cqp-4** — bee status text shows the parked count and answer remedy, default start-feature counts a prior parked cell, and brief answers collapse to one line (5 file(s) changed)

## Verify

Each cell below was capped only against a recorded passing verify result — bee refuses a cap without one.

- **cqp-1** — `cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee -- cells registry` — 520 passed 0 failed at c0aa8925e in the leader run and 523 passed in the worker run at db3a131ec; the leader then ran the full declared suite at db3a131ec: 5173 passed 0 failed 29 ignored; round-2 an…
- **cqp-2** — `cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee -- close worktree supervisor` — 720 passed, 0 failed, 5 new parked tests ran; run in a clean checkout of the cell commit because sibling WIP in the shared worktree (prepare.rs) did not compile
- **cqp-3** — `.bee/bin/bee dev release-manifest --check` — 600 files match after regen of the doctrine text; also cargo test --release -p bee -- native_cell_prompt prompts gave 9 passed in an isolated checkout of HEAD plus this diff (cqp-1 left the shared tr…
- **cqp-4** — `cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee -- workflow_store status orient policy state_sync prepare` — 389 passed 0 failed in the bin filter, rerun by the judge with the same result; 3 new rework tests seen red first; full suite not run for this round

## Deviations

- **cqp-1** — round 3 (db3a131ec): refuse_parked refuses when status is parked OR the last trace.park entry has no answer; the guard moved into apply_block_mutation so block and blocker dissent share it
- **cqp-1** — dissent.rs was not in the cell files; one line changed so a dissent verdict never releases a cell with an open park question; reserved before writing
- **cqp-1** — answer also stamps a dropped or blocked cell that still has an unanswered park entry and keeps its status; logged as a decision touching 16031715
- **cqp-1** — round 2 (c0aa8925e): claim, unclaim, update and reroute refusals name the remedy; answer probes the cells-archive lock before logging; supervisor reads parked_cells (live plus archive)
- **cqp-1** — (d) residual: answer logs inside mutate_cell before write_cell; an archive transaction between the lock probe and write_cell, or a disk write failure, leaves one uncited decision row
- **cqp-1** — judge advisory not in scope: capCell has no parked gate, so park then cap could cap a cell with an open question; filed to backlog
- **cqp-1** — sync-ack: the swarming skill text for park and the PARKED token landed in sibling cell cqp-3; this cell is code only
- **cqp-2** — found a better route: the parked-cell door is pushed only when a parked cell exists, so a feature with no parked cell keeps its dry-run door list byte-identical and the pinned door-list tests in drivers/tests.rs (outside this cell) stay green
- **cqp-2** — the supervisor rows read features from list_cells(root, None, Some(parked)) then call feature_parked_cells per feature, because open_question in park.rs is private and park.rs is not in this cell
- **cqp-2** — tests were written after the code, not red-first; the green run shows each new test ran
- **cqp-2** — verify ran with CARGO_TARGET_DIR=.bee/tmp/cqp1-target because the shared cargo target held a stale build-script output from .bee/tmp/cqp3-check
- **cqp-3** — scope: packages/bee-rs/crates/bee/src/verbs/drivers/tests.rs GOLDEN_NATIVE_CELL_PROMPT updated (one line) because the worker-cell.md edit reds native_cell_prompt_render_is_byte_identical_to_golden; reserved before the write
- **cqp-3** — scope: regen rewrote rendered copies, committed with the cell: .agents, .claude, .claude-plugin, .codex-plugin and .opencode skill trees (bee-swarming/SKILL.md, bee-hive/references/gates-and-delegation.md, .bee-render.json each), .claude/agents and .opencode/agent bee-worker.md and bee-comb.md, .bee/bin/prompts/worker-cell.md, .bee/onboarding.json
- **cqp-3** — found a fourth token list: the gates-and-delegation.md execution-worker line also lists the tokens; [PARKED] added there
- **cqp-3** — not changed: packages/bee-rs/crates/bee/src/hooks/chain_nudge.rs:147 still lists [DONE]/[BLOCKED]/[HANDOFF]/[NOOP]; outside cell files, a follow-up for the orchestrator
- **cqp-3** — path-rigor-floor prf-3 also edits gates-and-delegation.md and the manifest; whichever merges second re-runs regen
- **cqp-4** — render.rs reuses orient parked_cells_blocker_line so status and orient share one remedy line
- **cqp-4** — Cells line always prints parked=N
- **cqp-4** — start_lane and start_default share one unfinished_cells and parked_answer_clause helper
- **cqp-4** — verify ran with a private CARGO_TARGET_DIR because the shared target held a stale build-script path
- **cqp-4** — sync-ack: cqp-3 owns the skill text for parked cells; this cell only counts parked cells in readers and the brief, and its files list names no skill

## Provenance

Proposed by `bee knowledge promote --work cell-question-park` from 4 capped cell trace(s) in `.bee/cells/` and the anchor `docs/history/cell-question-park/CONTEXT.md`, `docs/history/cell-question-park/plan.md`. Every line above is copied from a trace or from the work item; nothing here is curated truth until a human or agent accepts it.

(b) AREA UPDATES — candidate spec-sync bullets, each citing its cell

areas: from the scribing stamp for "cell-question-park" — .bee/logs/scribing-runs.jsonl's most recent entry (2026-11-06T05:14:05.445Z), the work item declares no bee.areas.

area workflow-state:
  - [cqp-1] an open park question is read from trace.park: every refusal on such a cell names bee cells answer, a dissent verdict keeps it blocked, supervisor reads live plus archive — feature-wide sync per the scribing stamp, 7 file(s) changed (trace .bee/cells/cqp-1.json)
  - [cqp-2] bee close blocks on a parked-cell door, bee worktree merge refuses with WORKTREE_MERGE_PARKED_CELL, and the needs-you view lists each parked question — feature-wide sync per the scribing stamp, 4 file(s) changed (trace .bee/cells/cqp-2.json)
  - [cqp-4] bee status text shows the parked count and answer remedy, default start-feature counts a prior parked cell, and brief answers collapse to one line — feature-wide sync per the scribing stamp, 5 file(s) changed (trace .bee/cells/cqp-4.json)

area worktree-parallelism:
  - [cqp-1] an open park question is read from trace.park: every refusal on such a cell names bee cells answer, a dissent verdict keeps it blocked, supervisor reads live plus archive — feature-wide sync per the scribing stamp, 7 file(s) changed (trace .bee/cells/cqp-1.json)
  - [cqp-2] bee close blocks on a parked-cell door, bee worktree merge refuses with WORKTREE_MERGE_PARKED_CELL, and the needs-you view lists each parked question — feature-wide sync per the scribing stamp, 4 file(s) changed (trace .bee/cells/cqp-2.json)
  - [cqp-4] bee status text shows the parked count and answer remedy, default start-feature counts a prior parked cell, and brief answers collapse to one line — feature-wide sync per the scribing stamp, 5 file(s) changed (trace .bee/cells/cqp-4.json)

(c) PATTERN CANDIDATES — candidate bee.pattern concepts, bee.polarity pitfall

from cell cqp-1 — save as docs/knowledge/patterns/cell-question-park-cqp-1-pitfall.md

---
type: bee.pattern
title: cell-question-park cell cqp-1 — pitfall candidate
description: "Pitfall candidate mined from cell cqp-1's capped trace: round 3 (db3a131ec): refuse_parked refuses when status is parked OR the last trace.park entry has no answer; the guard moved into apply_block_mutation so block…"
timestamp: 2026-11-06
bee:
  id: cell-question-park-cqp-1-pitfall
  lifecycle: draft
  areas: [workflow-state, worktree-parallelism]
  sources: [.bee/cells/cqp-1.json]
  polarity: pitfall
---

# cell-question-park cell cqp-1 — pitfall candidate

## What the cell did

an open park question is read from trace.park: every refusal on such a cell names bee cells answer, a dissent verdict keeps it blocked, supervisor reads live plus archive

## Recorded evidence (verbatim from .bee/cells/cqp-1.json)

- **deviation** — round 3 (db3a131ec): refuse_parked refuses when status is parked OR the last trace.park entry has no answer; the guard moved into apply_block_mutation so block and blocker dissent share it
- **deviation** — dissent.rs was not in the cell files; one line changed so a dissent verdict never releases a cell with an open park question; reserved before writing
- **deviation** — answer also stamps a dropped or blocked cell that still has an unanswered park entry and keeps its status; logged as a decision touching 16031715
- **deviation** — round 2 (c0aa8925e): claim, unclaim, update and reroute refusals name the remedy; answer probes the cells-archive lock before logging; supervisor reads parked_cells (live plus archive)
- **deviation** — (d) residual: answer logs inside mutate_cell before write_cell; an archive transaction between the lock probe and write_cell, or a disk write failure, leaves one uncited decision row
- **deviation** — judge advisory not in scope: capCell has no parked gate, so park then cap could cap a cell with an open question; filed to backlog
- **deviation** — sync-ack: the swarming skill text for park and the PARKED token landed in sibling cell cqp-3; this cell is code only
- **failure_signature** — refusals from unclaim and claim on a parked cell do not name bee cells answer; block has no parked guard
- **failure_signature** — t6: blocker dissent (and drop) move a parked cell off parked, so reopen opens it with its question unanswered

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

from cell cqp-2 — save as docs/knowledge/patterns/cell-question-park-cqp-2-pitfall.md

---
type: bee.pattern
title: cell-question-park cell cqp-2 — pitfall candidate
description: "Pitfall candidate mined from cell cqp-2's capped trace: found a better route: the parked-cell door is pushed only when a parked cell exists, so a feature with no parked cell keeps its dry-run door list byte-identica…"
timestamp: 2026-11-06
bee:
  id: cell-question-park-cqp-2-pitfall
  lifecycle: draft
  areas: [workflow-state, worktree-parallelism]
  sources: [.bee/cells/cqp-2.json]
  polarity: pitfall
---

# cell-question-park cell cqp-2 — pitfall candidate

## What the cell did

bee close blocks on a parked-cell door, bee worktree merge refuses with WORKTREE_MERGE_PARKED_CELL, and the needs-you view lists each parked question

## Recorded evidence (verbatim from .bee/cells/cqp-2.json)

- **deviation** — found a better route: the parked-cell door is pushed only when a parked cell exists, so a feature with no parked cell keeps its dry-run door list byte-identical and the pinned door-list tests in drivers/tests.rs (outside this cell) stay green
- **deviation** — the supervisor rows read features from list_cells(root, None, Some(parked)) then call feature_parked_cells per feature, because open_question in park.rs is private and park.rs is not in this cell
- **deviation** — tests were written after the code, not red-first; the green run shows each new test ran
- **deviation** — verify ran with CARGO_TARGET_DIR=.bee/tmp/cqp1-target because the shared cargo target held a stale build-script output from .bee/tmp/cqp3-check

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

from cell cqp-3 — save as docs/knowledge/patterns/cell-question-park-cqp-3-pitfall.md

---
type: bee.pattern
title: cell-question-park cell cqp-3 — pitfall candidate
description: "Pitfall candidate mined from cell cqp-3's capped trace: scope: packages/bee-rs/crates/bee/src/verbs/drivers/tests.rs GOLDEN_NATIVE_CELL_PROMPT updated (one line) because the worker-cell.md edit reds native_cell_prom…"
timestamp: 2026-11-06
bee:
  id: cell-question-park-cqp-3-pitfall
  lifecycle: draft
  areas: [workflow-state, worktree-parallelism]
  sources: [.bee/cells/cqp-3.json]
  polarity: pitfall
---

# cell-question-park cell cqp-3 — pitfall candidate

## What the cell did

Worker contract parks a question cell with [PARKED]; leader keeps working; answer via bee cells answer

## Recorded evidence (verbatim from .bee/cells/cqp-3.json)

- **deviation** — scope: packages/bee-rs/crates/bee/src/verbs/drivers/tests.rs GOLDEN_NATIVE_CELL_PROMPT updated (one line) because the worker-cell.md edit reds native_cell_prompt_render_is_byte_identical_to_golden; reserved before the write
- **deviation** — scope: regen rewrote rendered copies, committed with the cell: .agents, .claude, .claude-plugin, .codex-plugin and .opencode skill trees (bee-swarming/SKILL.md, bee-hive/references/gates-and-delegation.md, .bee-render.json each), .claude/agents and .opencode/agent bee-worker.md and bee-comb.md, .bee/bin/prompts/worker-cell.md, .bee/onboarding.json
- **deviation** — found a fourth token list: the gates-and-delegation.md execution-worker line also lists the tokens; [PARKED] added there
- **deviation** — not changed: packages/bee-rs/crates/bee/src/hooks/chain_nudge.rs:147 still lists [DONE]/[BLOCKED]/[HANDOFF]/[NOOP]; outside cell files, a follow-up for the orchestrator
- **deviation** — path-rigor-floor prf-3 also edits gates-and-delegation.md and the manifest; whichever merges second re-runs regen

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

from cell cqp-4 — save as docs/knowledge/patterns/cell-question-park-cqp-4-pitfall.md

---
type: bee.pattern
title: cell-question-park cell cqp-4 — pitfall candidate
description: "Pitfall candidate mined from cell cqp-4's capped trace: render.rs reuses orient parked_cells_blocker_line so status and orient share one remedy line"
timestamp: 2026-11-06
bee:
  id: cell-question-park-cqp-4-pitfall
  lifecycle: draft
  areas: [workflow-state, worktree-parallelism]
  sources: [.bee/cells/cqp-4.json]
  polarity: pitfall
---

# cell-question-park cell cqp-4 — pitfall candidate

## What the cell did

bee status text shows the parked count and answer remedy, default start-feature counts a prior parked cell, and brief answers collapse to one line

## Recorded evidence (verbatim from .bee/cells/cqp-4.json)

- **deviation** — render.rs reuses orient parked_cells_blocker_line so status and orient share one remedy line
- **deviation** — Cells line always prints parked=N
- **deviation** — start_lane and start_default share one unfinished_cells and parked_answer_clause helper
- **deviation** — verify ran with a private CARGO_TARGET_DIR because the shared target held a stale build-script path
- **deviation** — sync-ack: cqp-3 owns the skill text for parked cells; this cell only counts parked cells in readers and the brief, and its files list names no skill
- **failure_signature** — bee status text omits parked; default start-feature ignores parked

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

knowledge promote: 4 capped cell(s) mined, 1 delivery draft, 6 area bullet(s), 4 pattern candidate(s), 0 file(s) written.