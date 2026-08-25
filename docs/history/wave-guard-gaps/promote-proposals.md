promote proposal for work item "wave-guard-gaps" (docs/history/wave-guard-gaps/CONTEXT.md) — 2 capped cell(s): wgg-1, wgg-2
anchor: history — docs/history/wave-guard-gaps/CONTEXT.md
PROPOSAL ONLY — nothing was written. Applying any section below is a human or agent decision.

(a) DELIVERY DRAFT — save as docs/knowledge/work/wave-guard-gaps/delivery.md

---
type: bee.delivery
title: wave-guard-gaps — delivery
description: "Delivery record proposed by bee knowledge promote for work item wave-guard-gaps: 2 capped cell(s), 10 recorded deviation(s)."
timestamp: 2026-08-25
bee:
  id: wave-guard-gaps-delivery
  lifecycle: active
  areas: [worktree-parallelism, workflow-state]
  required_context: [docs/history/wave-guard-gaps/CONTEXT.md]
  sources: [docs/history/wave-guard-gaps/CONTEXT.md, .bee/cells/wgg-1.json, .bee/cells/wgg-2.json]
---

# wave-guard-gaps — Delivery

## What shipped

- **wgg-1** — affects_skills format is refused at cells add/update with the exact skills/<name>/SKILL.md replacement; the sync door names a bare-name prediction as a format error (5 file(s) changed)
- **wgg-2** — The concurrent-worker git guard now counts siblings inside a granted worktree via the control root's mirrored-holds ledger (2 file(s) changed)

## Verify

Each cell below was capped only against a recorded passing verify result — bee refuses a cap without one.

- **wgg-1** — `cargo test --release --manifest-path packages/bee-rs/Cargo.toml cells::`
- **wgg-2** — `cargo test --release --manifest-path packages/bee-rs/Cargo.toml write_guard`

## Deviations

- **wgg-1** — Added handlers_write.rs (the cells update door) and generated/registry_payload.json (the cells add help line) — both required by the cell action, neither listed in files; reserved before writing
- **wgg-1** — Committed with git commit --only over my 5 files, per dispatch instruction, because a sibling worker shares this worktree
- **wgg-1** — Capped with --inline-reason: worker wk-wgg-1 was never registered in state.json workers[] and cannot self-register from inside the worktree
- **wgg-1** — Capped with --sync-ack: skills/bee-planning/references/planning-reference.md:158 still says only flat arrays — skill sync owed at the wave regen barrier
- **wgg-1** — sync-ack: Cell scope is the CLI validation plus its help line; the plan declared affects_skills [] and named no skills/ file. The skill-side wording (skills/bee-planning/references/planning-reference.md:158 still says only 'flat arrays') is a docs sync this wave owes at its regen barrier (regen_obligation_ack wave-barrier) — flagged to the orchestrator, not silently dropped.
- **wgg-2** — The cell files were not pre-reserved on dispatch; reserved both under wk-wgg-2 before the first write.
- **wgg-2** — The dispatch never registered wk-wgg-2 in state.workers[]; registered it via bee state worker add per the cap refusal FIX rather than recording a false inline run.
- **wgg-2** — Reused the write-guard own mirrored-holds reader (find_foreign_holds, hooks/write_guard/store.rs) with an all-holds sentinel instead of verbs/reservations/leases.rs, which is a private module whose export would need verbs/reservations/mod.rs — outside this cell files. Same sentinel trick bee reservations list already uses, so no second reader was written.
- **wgg-2** — Capped with --sync-ack: the workflow-state area owns four skills, none of which change here; the cell declares affects_skills [] and names only the two guard files.
- **wgg-2** — sync-ack: No skill text changes: the whole-tree denial and its temp-index remedy are already taught in skills/bee-swarming and AGENTS.md. This cell only makes the existing guard see the checkout it was blind to — no new agent-facing rule. The cell declares affects_skills [] and scopes its files to the two write_guard files, so a skill edit is out of scope for this worker.

## Provenance

Proposed by `bee knowledge promote --work wave-guard-gaps` from 2 capped cell trace(s) in `.bee/cells/` and the anchor `docs/history/wave-guard-gaps/CONTEXT.md`. Every line above is copied from a trace or from the work item; nothing here is curated truth until a human or agent accepts it.

(b) AREA UPDATES — candidate spec-sync bullets, each citing its cell

areas: from the scribing stamp for "wave-guard-gaps" — .bee/logs/scribing-runs.jsonl's most recent entry (2026-08-25T08:31:14.113Z), the work item declares no bee.areas.

area worktree-parallelism:
  - [wgg-1] affects_skills format is refused at cells add/update with the exact skills/<name>/SKILL.md replacement; the sync door names a bare-name prediction as a format error — feature-wide sync per the scribing stamp, 5 file(s) changed (trace .bee/cells/wgg-1.json)
  - [wgg-2] The concurrent-worker git guard now counts siblings inside a granted worktree via the control root's mirrored-holds ledger — feature-wide sync per the scribing stamp, 2 file(s) changed (trace .bee/cells/wgg-2.json)

area workflow-state:
  - [wgg-1] affects_skills format is refused at cells add/update with the exact skills/<name>/SKILL.md replacement; the sync door names a bare-name prediction as a format error — feature-wide sync per the scribing stamp, 5 file(s) changed (trace .bee/cells/wgg-1.json)
  - [wgg-2] The concurrent-worker git guard now counts siblings inside a granted worktree via the control root's mirrored-holds ledger — feature-wide sync per the scribing stamp, 2 file(s) changed (trace .bee/cells/wgg-2.json)

(c) PATTERN CANDIDATES — candidate bee.pattern concepts, bee.polarity pitfall

from cell wgg-1 — save as docs/knowledge/patterns/wave-guard-gaps-wgg-1-pitfall.md

---
type: bee.pattern
title: wave-guard-gaps cell wgg-1 — pitfall candidate
description: "Pitfall candidate mined from cell wgg-1's capped trace: Added handlers_write.rs (the cells update door) and generated/registry_payload.json (the cells add help line) — both required by the cell action, neither liste…"
timestamp: 2026-08-25
bee:
  id: wave-guard-gaps-wgg-1-pitfall
  lifecycle: draft
  areas: [worktree-parallelism, workflow-state]
  sources: [.bee/cells/wgg-1.json]
  polarity: pitfall
---

# wave-guard-gaps cell wgg-1 — pitfall candidate

## What the cell did

affects_skills format is refused at cells add/update with the exact skills/<name>/SKILL.md replacement; the sync door names a bare-name prediction as a format error

## Recorded evidence (verbatim from .bee/cells/wgg-1.json)

- **deviation** — Added handlers_write.rs (the cells update door) and generated/registry_payload.json (the cells add help line) — both required by the cell action, neither listed in files; reserved before writing
- **deviation** — Committed with git commit --only over my 5 files, per dispatch instruction, because a sibling worker shares this worktree
- **deviation** — Capped with --inline-reason: worker wk-wgg-1 was never registered in state.json workers[] and cannot self-register from inside the worktree
- **deviation** — Capped with --sync-ack: skills/bee-planning/references/planning-reference.md:158 still says only flat arrays — skill sync owed at the wave regen barrier
- **deviation** — sync-ack: Cell scope is the CLI validation plus its help line; the plan declared affects_skills [] and named no skills/ file. The skill-side wording (skills/bee-planning/references/planning-reference.md:158 still says only 'flat arrays') is a docs sync this wave owes at its regen barrier (regen_obligation_ack wave-barrier) — flagged to the orchestrator, not silently dropped.

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

from cell wgg-2 — save as docs/knowledge/patterns/wave-guard-gaps-wgg-2-pitfall.md

---
type: bee.pattern
title: wave-guard-gaps cell wgg-2 — pitfall candidate
description: "Pitfall candidate mined from cell wgg-2's capped trace: The cell files were not pre-reserved on dispatch; reserved both under wk-wgg-2 before the first write."
timestamp: 2026-08-25
bee:
  id: wave-guard-gaps-wgg-2-pitfall
  lifecycle: draft
  areas: [worktree-parallelism, workflow-state]
  sources: [.bee/cells/wgg-2.json]
  polarity: pitfall
---

# wave-guard-gaps cell wgg-2 — pitfall candidate

## What the cell did

The concurrent-worker git guard now counts siblings inside a granted worktree via the control root's mirrored-holds ledger

## Recorded evidence (verbatim from .bee/cells/wgg-2.json)

- **deviation** — The cell files were not pre-reserved on dispatch; reserved both under wk-wgg-2 before the first write.
- **deviation** — The dispatch never registered wk-wgg-2 in state.workers[]; registered it via bee state worker add per the cap refusal FIX rather than recording a false inline run.
- **deviation** — Reused the write-guard own mirrored-holds reader (find_foreign_holds, hooks/write_guard/store.rs) with an all-holds sentinel instead of verbs/reservations/leases.rs, which is a private module whose export would need verbs/reservations/mod.rs — outside this cell files. Same sentinel trick bee reservations list already uses, so no second reader was written.
- **deviation** — Capped with --sync-ack: the workflow-state area owns four skills, none of which change here; the cell declares affects_skills [] and names only the two guard files.
- **deviation** — sync-ack: No skill text changes: the whole-tree denial and its temp-index remedy are already taught in skills/bee-swarming and AGENTS.md. This cell only makes the existing guard see the checkout it was blind to — no new agent-facing rule. The cell declares affects_skills [] and scopes its files to the two write_guard files, so a skill edit is out of scope for this worker.

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

knowledge promote: 2 capped cell(s) mined, 1 delivery draft, 4 area bullet(s), 2 pattern candidate(s), 0 file(s) written.