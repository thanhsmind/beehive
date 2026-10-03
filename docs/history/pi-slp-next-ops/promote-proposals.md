promote proposal for work item "pi-slp-next-ops" (docs/history/pi-slp-next-ops/CONTEXT.md + docs/history/pi-slp-next-ops/plan.md) — 3 capped cell(s): nop-1, nop-2, nop-3
anchor: history — docs/history/pi-slp-next-ops/CONTEXT.md, docs/history/pi-slp-next-ops/plan.md
PROPOSAL ONLY — nothing was written. Applying any section below is a human or agent decision.

(a) DELIVERY DRAFT — save as docs/knowledge/work/pi-slp-next-ops/delivery.md

---
type: bee.delivery
title: pi-slp-next-ops — delivery
description: "Delivery record proposed by bee knowledge promote for work item pi-slp-next-ops: 3 capped cell(s), 5 recorded deviation(s)."
timestamp: 2026-10-03
bee:
  id: pi-slp-next-ops-delivery
  lifecycle: active
  areas: [rust-runtime]
  required_context: [docs/history/pi-slp-next-ops/CONTEXT.md, docs/history/pi-slp-next-ops/plan.md]
  sources: [docs/history/pi-slp-next-ops/CONTEXT.md, docs/history/pi-slp-next-ops/plan.md, .bee/cells/nop-1.json, .bee/cells/nop-2.json, .bee/cells/nop-3.json]
---

# pi-slp-next-ops — Delivery

## What shipped

- **nop-1** — orient reads the session's lane record, names one runnable next command with run_from, and prints a run line (3 file(s) changed)
- **nop-2** — The per-turn hint prints run: <command> (from <run_from>) from next_operation, with a five-line cap and the command in the dedup hash (1 file(s) changed)
- **nop-3** — Worktree-first denies and route notices name bee worktree enter --id; the generic containment deny says outside-project paths belong to the user (4 file(s) changed)

## Verify

Each cell below was capped only against a recorded passing verify result — bee refuses a cap without one.

- **nop-1** — `cd packages/bee-rs && cargo test --release -p bee --bin bee verbs::status_full` — 116 passed (111 at base); lane-bound, broken-binding, each next_operation branch, priority order and guidance tests added
- **nop-2** — `cd packages/bee-rs && cargo test --release -p bee --bin bee prompt_context` — 38 passed (35 at base); run line present, absent with old hash, and five-line cap with gate pending and triggers due
- **nop-3** — `cd packages/bee-rs && cargo test --release -p bee --bin bee hooks::write_guard && cargo test --release -p bee --bin bee verbs::state_group` — write_guard 279, state_group 243; asserts added to existing deny and notice tests

## Deviations

- **nop-1** — round 1 reordered the next-command priority (worktree before wayfinding resume); round 2 restored the old order with a test — hit an unforeseen obstacle
- **nop-2** — followed the plan
- **nop-2** — sync-ack: no owned skill documents the per-turn hint lines or the deny texts (rg over skills/ found none); knowledge docs carry the rule
- **nop-3** — followed the plan
- **nop-3** — sync-ack: only deny and notice wording changed; no workflow-state skill quotes these texts

## Provenance

Proposed by `bee knowledge promote --work pi-slp-next-ops` from 3 capped cell trace(s) in `.bee/cells/` and the anchor `docs/history/pi-slp-next-ops/CONTEXT.md`, `docs/history/pi-slp-next-ops/plan.md`. Every line above is copied from a trace or from the work item; nothing here is curated truth until a human or agent accepts it.

(b) AREA UPDATES — candidate spec-sync bullets, each citing its cell

areas: from the scribing stamp for "pi-slp-next-ops" — .bee/logs/scribing-runs.jsonl's most recent entry (2026-10-03T06:27:13.494Z), the work item declares no bee.areas.

area rust-runtime:
  - [nop-1] orient reads the session's lane record, names one runnable next command with run_from, and prints a run line — feature-wide sync per the scribing stamp, 3 file(s) changed (trace .bee/cells/nop-1.json)
  - [nop-2] The per-turn hint prints run: <command> (from <run_from>) from next_operation, with a five-line cap and the command in the dedup hash — feature-wide sync per the scribing stamp, 1 file(s) changed (trace .bee/cells/nop-2.json)

(c) PATTERN CANDIDATES — candidate bee.pattern concepts, bee.polarity pitfall

from cell nop-1 — save as docs/knowledge/patterns/pi-slp-next-ops-nop-1-pitfall.md

---
type: bee.pattern
title: pi-slp-next-ops cell nop-1 — pitfall candidate
description: "Pitfall candidate mined from cell nop-1's capped trace: round 1 reordered the next-command priority (worktree before wayfinding resume); round 2 restored the old order with a test — hit an unforeseen obstacle"
timestamp: 2026-10-03
bee:
  id: pi-slp-next-ops-nop-1-pitfall
  lifecycle: draft
  areas: [rust-runtime]
  sources: [.bee/cells/nop-1.json]
  polarity: pitfall
---

# pi-slp-next-ops cell nop-1 — pitfall candidate

## What the cell did

orient reads the session's lane record, names one runnable next command with run_from, and prints a run line

## Recorded evidence (verbatim from .bee/cells/nop-1.json)

- **deviation** — round 1 reordered the next-command priority (worktree before wayfinding resume); round 2 restored the old order with a test — hit an unforeseen obstacle

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

from cell nop-2 — save as docs/knowledge/patterns/pi-slp-next-ops-nop-2-pitfall.md

---
type: bee.pattern
title: pi-slp-next-ops cell nop-2 — pitfall candidate
description: "Pitfall candidate mined from cell nop-2's capped trace: followed the plan"
timestamp: 2026-10-03
bee:
  id: pi-slp-next-ops-nop-2-pitfall
  lifecycle: draft
  areas: [rust-runtime]
  sources: [.bee/cells/nop-2.json]
  polarity: pitfall
---

# pi-slp-next-ops cell nop-2 — pitfall candidate

## What the cell did

The per-turn hint prints run: <command> (from <run_from>) from next_operation, with a five-line cap and the command in the dedup hash

## Recorded evidence (verbatim from .bee/cells/nop-2.json)

- **deviation** — followed the plan
- **deviation** — sync-ack: no owned skill documents the per-turn hint lines or the deny texts (rg over skills/ found none); knowledge docs carry the rule

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

from cell nop-3 — save as docs/knowledge/patterns/pi-slp-next-ops-nop-3-pitfall.md

---
type: bee.pattern
title: pi-slp-next-ops cell nop-3 — pitfall candidate
description: "Pitfall candidate mined from cell nop-3's capped trace: followed the plan"
timestamp: 2026-10-03
bee:
  id: pi-slp-next-ops-nop-3-pitfall
  lifecycle: draft
  areas: [rust-runtime]
  sources: [.bee/cells/nop-3.json]
  polarity: pitfall
---

# pi-slp-next-ops cell nop-3 — pitfall candidate

## What the cell did

Worktree-first denies and route notices name bee worktree enter --id; the generic containment deny says outside-project paths belong to the user

## Recorded evidence (verbatim from .bee/cells/nop-3.json)

- **deviation** — followed the plan
- **deviation** — sync-ack: only deny and notice wording changed; no workflow-state skill quotes these texts

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

knowledge promote: 3 capped cell(s) mined, 1 delivery draft, 2 area bullet(s), 3 pattern candidate(s), 0 file(s) written.