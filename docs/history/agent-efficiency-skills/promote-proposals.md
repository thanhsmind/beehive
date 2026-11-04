promote proposal for work item "agent-efficiency-skills" (docs/history/agent-efficiency-skills/CONTEXT.md) — 2 capped cell(s): asf-1, asf-2
anchor: history — docs/history/agent-efficiency-skills/CONTEXT.md
PROPOSAL ONLY — nothing was written. Applying any section below is a human or agent decision.

(a) DELIVERY DRAFT — save as docs/knowledge/work/agent-efficiency-skills/delivery.md

---
type: bee.delivery
title: agent-efficiency-skills — delivery
description: "Delivery record proposed by bee knowledge promote for work item agent-efficiency-skills: 2 capped cell(s), 3 recorded deviation(s)."
timestamp: 2026-11-04
bee:
  id: agent-efficiency-skills-delivery
  lifecycle: active
  areas: [workflow-state, bee-herding]
  required_context: [docs/history/agent-efficiency-skills/CONTEXT.md]
  sources: [docs/history/agent-efficiency-skills/CONTEXT.md, .bee/cells/asf-1.json, .bee/cells/asf-2.json]
---

# agent-efficiency-skills — Delivery

## What shipped

- **asf-1** — Skills name bee feature start and finish as the one-verb open and land path, herding and swarming skills say herding run caps a done cell with a trailer commit and a green proof and when it does not, and leader-check help says a gap verdict reopens the cell (8 file(s) changed)
- **asf-2** — Leave lane mode as null on feature start matching state start-feature --as-lane so small routes pass gate without plan.md 5add191a9 (1 file(s) changed)

## Verify

Each cell below was capped only against a recorded passing verify result — bee refuses a cap without one.

- **asf-1** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee --test pointer_integrity --test instruction_laws --test skill_triggers --test registry_contracts && .bee/bin/bee dev release-manifest --check` — the cell verify over the skill text, registry payload and regen outputs this cell touched; built bee cells leader-check --help shows the new gap sentence; full suite not run
- **asf-2** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee -- feature_group` — packages/bee-rs/crates/bee/src/verbs/feature_group.rs unit tests

## Deviations

- **asf-1** — routing-and-contracts.md said a gap verdict records without reopening the cell; corrected to match leader_check.rs
- **asf-1** — left .bee/onboarding.json (timestamp-only regen churn) and the sibling cell uncommitted feature_group.rs out of the commit
- **asf-1** — sync-ack: herding run text lives in bee-herding/references/operational-invariants.md, not SKILL.md; the swarming herding loop, the hive gap-verdict rule and the worktree-first feature-start line live in their references, so those were edited in place

## Provenance

Proposed by `bee knowledge promote --work agent-efficiency-skills` from 2 capped cell trace(s) in `.bee/cells/` and the anchor `docs/history/agent-efficiency-skills/CONTEXT.md`. Every line above is copied from a trace or from the work item; nothing here is curated truth until a human or agent accepts it.

(b) AREA UPDATES — candidate spec-sync bullets, each citing its cell

areas: from the scribing stamp for "agent-efficiency-skills" — .bee/logs/scribing-runs.jsonl's most recent entry (2026-11-04T05:27:42.646Z), the work item declares no bee.areas.

area workflow-state:
  - [asf-1] Skills name bee feature start and finish as the one-verb open and land path, herding and swarming skills say herding run caps a done cell with a trailer commit and a green proof and when it does not, and leader-check help says a gap verdict reopens the cell — feature-wide sync per the scribing stamp, 8 file(s) changed (trace .bee/cells/asf-1.json)
  - [asf-2] Leave lane mode as null on feature start matching state start-feature --as-lane so small routes pass gate without plan.md 5add191a9 — feature-wide sync per the scribing stamp, 1 file(s) changed (trace .bee/cells/asf-2.json)

area bee-herding:
  - [asf-1] Skills name bee feature start and finish as the one-verb open and land path, herding and swarming skills say herding run caps a done cell with a trailer commit and a green proof and when it does not, and leader-check help says a gap verdict reopens the cell — feature-wide sync per the scribing stamp, 8 file(s) changed (trace .bee/cells/asf-1.json)
  - [asf-2] Leave lane mode as null on feature start matching state start-feature --as-lane so small routes pass gate without plan.md 5add191a9 — feature-wide sync per the scribing stamp, 1 file(s) changed (trace .bee/cells/asf-2.json)

(c) PATTERN CANDIDATES — candidate bee.pattern concepts, bee.polarity pitfall

from cell asf-1 — save as docs/knowledge/patterns/agent-efficiency-skills-asf-1-pitfall.md

---
type: bee.pattern
title: agent-efficiency-skills cell asf-1 — pitfall candidate
description: "Pitfall candidate mined from cell asf-1's capped trace: routing-and-contracts.md said a gap verdict records without reopening the cell; corrected to match leader_check.rs"
timestamp: 2026-11-04
bee:
  id: agent-efficiency-skills-asf-1-pitfall
  lifecycle: draft
  areas: [workflow-state, bee-herding]
  sources: [.bee/cells/asf-1.json]
  polarity: pitfall
---

# agent-efficiency-skills cell asf-1 — pitfall candidate

## What the cell did

Skills name bee feature start and finish as the one-verb open and land path, herding and swarming skills say herding run caps a done cell with a trailer commit and a green proof and when it does not, and leader-check help says a gap verdict reopens the cell

## Recorded evidence (verbatim from .bee/cells/asf-1.json)

- **deviation** — routing-and-contracts.md said a gap verdict records without reopening the cell; corrected to match leader_check.rs
- **deviation** — left .bee/onboarding.json (timestamp-only regen churn) and the sibling cell uncommitted feature_group.rs out of the commit
- **deviation** — sync-ack: herding run text lives in bee-herding/references/operational-invariants.md, not SKILL.md; the swarming herding loop, the hive gap-verdict rule and the worktree-first feature-start line live in their references, so those were edited in place

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

knowledge promote: 2 capped cell(s) mined, 1 delivery draft, 4 area bullet(s), 1 pattern candidate(s), 0 file(s) written.