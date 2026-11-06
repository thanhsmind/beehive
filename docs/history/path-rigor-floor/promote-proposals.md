promote proposal for work item "path-rigor-floor" (docs/history/path-rigor-floor/CONTEXT.md + docs/history/path-rigor-floor/plan.md) — 2 capped cell(s): rfl-1, rfl-2
anchor: history — docs/history/path-rigor-floor/CONTEXT.md, docs/history/path-rigor-floor/plan.md
PROPOSAL ONLY — nothing was written. Applying any section below is a human or agent decision.

(a) DELIVERY DRAFT — save as docs/knowledge/work/path-rigor-floor/delivery.md

---
type: bee.delivery
title: path-rigor-floor — delivery
description: "Delivery record proposed by bee knowledge promote for work item path-rigor-floor: 2 capped cell(s), 8 recorded deviation(s)."
timestamp: 2026-11-06
bee:
  id: path-rigor-floor-delivery
  lifecycle: active
  areas: [workflow-state]
  required_context: [docs/history/path-rigor-floor/CONTEXT.md, docs/history/path-rigor-floor/plan.md]
  sources: [docs/history/path-rigor-floor/CONTEXT.md, docs/history/path-rigor-floor/plan.md, .bee/cells/rfl-1.json, .bee/cells/rfl-2.json]
---

# path-rigor-floor — Delivery

## What shipped

- **rfl-1** — cells add/update refuse RIGOR_FLOOR below a path floor and GATE_OWNER_FIRST after an auto gate; bee gate refuses auto approval over owner-first files before and after the lock (6 file(s) changed)
- **rfl-2** — Route names the path rigor floor; Gate bypass mode names GATE_OWNER_FIRST; regen carried both (3 file(s) changed)

## Verify

Each cell below was capped only against a recorded passing verify result — bee refuses a cap without one.

- **rfl-1** — `cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee -- cells set_gate gate` — the cell verify filter over the touched cells and gate code (676 bin tests plus integration filters, 0 failed); full suite not run
- **rfl-2** — `.bee/bin/bee dev release-manifest --check` — 600 files match; doc-only cell, manifest parity is the proof; no other suite run

## Deviations

- **rfl-1** — the verify command ran with CARGO_TARGET_DIR=~/.cache/cargo-target-rfl1 because the shared ~/.cache/cargo-target held a stale build-script output from the cell-question-park worktree and did not compile
- **rfl-1** — cells add/update tests were written after the wiring, so only the 2 gate refusal tests were seen red before set_gate.rs changed
- **rfl-1** — the update test spawns its own child with --nocapture so the refusal text is readable; the shared cells_update_behavior_run helper captures it
- **rfl-1** — the after-lock GATE_OWNER_FIRST re-check has no race test; it runs the same function as the pre-lock check
- **rfl-1** — an unrelated .bee/onboarding.json change in the worktree was left unstaged
- **rfl-1** — sync-ack: sibling cell rfl-2 carries the skill text for this behavior (skills/bee-planning/SKILL.md Route and gates-and-delegation.md)
- **rfl-2** — bee dev regen rewrote the 15 rendered copies (.agents, .claude-plugin, .claude, .codex-plugin, .opencode skills plus their .bee-render.json); committed with the cell as a named scope deviation
- **rfl-2** — bee dev regen also bumped .bee/onboarding.json updated_at; left uncommitted (timestamp only, CLI-owned, guard refuses restore)

## Provenance

Proposed by `bee knowledge promote --work path-rigor-floor` from 2 capped cell trace(s) in `.bee/cells/` and the anchor `docs/history/path-rigor-floor/CONTEXT.md`, `docs/history/path-rigor-floor/plan.md`. Every line above is copied from a trace or from the work item; nothing here is curated truth until a human or agent accepts it.

(b) AREA UPDATES — candidate spec-sync bullets, each citing its cell

areas: from the scribing stamp for "path-rigor-floor" — .bee/logs/scribing-runs.jsonl's most recent entry (2026-11-06T04:48:41.308Z), the work item declares no bee.areas.

area workflow-state:
  - [rfl-1] cells add/update refuse RIGOR_FLOOR below a path floor and GATE_OWNER_FIRST after an auto gate; bee gate refuses auto approval over owner-first files before and after the lock — feature-wide sync per the scribing stamp, 6 file(s) changed (trace .bee/cells/rfl-1.json)

(c) PATTERN CANDIDATES — candidate bee.pattern concepts, bee.polarity pitfall

from cell rfl-1 — save as docs/knowledge/patterns/path-rigor-floor-rfl-1-pitfall.md

---
type: bee.pattern
title: path-rigor-floor cell rfl-1 — pitfall candidate
description: "Pitfall candidate mined from cell rfl-1's capped trace: the verify command ran with CARGO_TARGET_DIR=~/.cache/cargo-target-rfl1 because the shared ~/.cache/cargo-target held a stale build-script output from the cell…"
timestamp: 2026-11-06
bee:
  id: path-rigor-floor-rfl-1-pitfall
  lifecycle: draft
  areas: [workflow-state]
  sources: [.bee/cells/rfl-1.json]
  polarity: pitfall
---

# path-rigor-floor cell rfl-1 — pitfall candidate

## What the cell did

cells add/update refuse RIGOR_FLOOR below a path floor and GATE_OWNER_FIRST after an auto gate; bee gate refuses auto approval over owner-first files before and after the lock

## Recorded evidence (verbatim from .bee/cells/rfl-1.json)

- **deviation** — the verify command ran with CARGO_TARGET_DIR=~/.cache/cargo-target-rfl1 because the shared ~/.cache/cargo-target held a stale build-script output from the cell-question-park worktree and did not compile
- **deviation** — cells add/update tests were written after the wiring, so only the 2 gate refusal tests were seen red before set_gate.rs changed
- **deviation** — the update test spawns its own child with --nocapture so the refusal text is readable; the shared cells_update_behavior_run helper captures it
- **deviation** — the after-lock GATE_OWNER_FIRST re-check has no race test; it runs the same function as the pre-lock check
- **deviation** — an unrelated .bee/onboarding.json change in the worktree was left unstaged
- **deviation** — sync-ack: sibling cell rfl-2 carries the skill text for this behavior (skills/bee-planning/SKILL.md Route and gates-and-delegation.md)

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

from cell rfl-2 — save as docs/knowledge/patterns/path-rigor-floor-rfl-2-pitfall.md

---
type: bee.pattern
title: path-rigor-floor cell rfl-2 — pitfall candidate
description: "Pitfall candidate mined from cell rfl-2's capped trace: bee dev regen rewrote the 15 rendered copies (.agents, .claude-plugin, .claude, .codex-plugin, .opencode skills plus their .bee-render.json); committed with th…"
timestamp: 2026-11-06
bee:
  id: path-rigor-floor-rfl-2-pitfall
  lifecycle: draft
  areas: [workflow-state]
  sources: [.bee/cells/rfl-2.json]
  polarity: pitfall
---

# path-rigor-floor cell rfl-2 — pitfall candidate

## What the cell did

Route names the path rigor floor; Gate bypass mode names GATE_OWNER_FIRST; regen carried both

## Recorded evidence (verbatim from .bee/cells/rfl-2.json)

- **deviation** — bee dev regen rewrote the 15 rendered copies (.agents, .claude-plugin, .claude, .codex-plugin, .opencode skills plus their .bee-render.json); committed with the cell as a named scope deviation
- **deviation** — bee dev regen also bumped .bee/onboarding.json updated_at; left uncommitted (timestamp only, CLI-owned, guard refuses restore)

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

knowledge promote: 2 capped cell(s) mined, 1 delivery draft, 1 area bullet(s), 2 pattern candidate(s), 0 file(s) written.