promote proposal for work item "agent-anti-patterns" (docs/history/agent-anti-patterns/CONTEXT.md) — 2 capped cell(s): aap-1, aap-2
anchor: history — docs/history/agent-anti-patterns/CONTEXT.md
PROPOSAL ONLY — nothing was written. Applying any section below is a human or agent decision.

(a) DELIVERY DRAFT — save as docs/knowledge/work/agent-anti-patterns/delivery.md

---
type: bee.delivery
title: agent-anti-patterns — delivery
description: "Delivery record proposed by bee knowledge promote for work item agent-anti-patterns: 2 capped cell(s), 2 recorded deviation(s)."
timestamp: 2026-11-04
bee:
  id: agent-anti-patterns-delivery
  lifecycle: active
  areas: [workflow-state]
  required_context: [docs/history/agent-anti-patterns/CONTEXT.md]
  sources: [docs/history/agent-anti-patterns/CONTEXT.md, .bee/cells/aap-1.json, .bee/cells/aap-2.json]
---

# agent-anti-patterns — Delivery

## What shipped

- **aap-1** — Planning, worker and goal-check doctrine now carry agent-anti-patterns D1 and D2; regen outputs committed (6 file(s) changed)
- **aap-2** — bee report outcomes shows plan departures by kind (7a1dff3f8) (1 file(s) changed)

## Verify

Each cell below was capped only against a recorded passing verify result — bee refuses a cap without one.

- **aap-1** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee -- drivers prompts && PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee --test pointer_integrity --test instruction_laws --test skill_triggers && .bee/bin/bee dev release-manifest --check` — cell verify command: 465 drivers/prompts lib tests incl. native_cell_prompt_render_is_byte_identical_to_golden (confirmed by an --exact run), pointer_integrity 9, instruction_laws 10, skill_triggers …
- **aap-2** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee -- report_outcomes` — 38 report_outcomes tests incl. departure fixtures; leader also ran the built binary from the granted worktree: 731 cells with departures over the main ledger, feature filter shows aap-1 followed the …

## Deviations

- **aap-1** — followed the plan
- **aap-2** — followed the plan

## Provenance

Proposed by `bee knowledge promote --work agent-anti-patterns` from 2 capped cell trace(s) in `.bee/cells/` and the anchor `docs/history/agent-anti-patterns/CONTEXT.md`. Every line above is copied from a trace or from the work item; nothing here is curated truth until a human or agent accepts it.

(b) AREA UPDATES — candidate spec-sync bullets, each citing its cell

areas: from the scribing stamp for "agent-anti-patterns" — .bee/logs/scribing-runs.jsonl's most recent entry (2026-11-04T03:46:38.382Z), the work item declares no bee.areas.

area workflow-state:
  - [aap-1] Planning, worker and goal-check doctrine now carry agent-anti-patterns D1 and D2; regen outputs committed — feature-wide sync per the scribing stamp, 6 file(s) changed (trace .bee/cells/aap-1.json)
  - [aap-2] bee report outcomes shows plan departures by kind (7a1dff3f8) — feature-wide sync per the scribing stamp, 1 file(s) changed (trace .bee/cells/aap-2.json)

(c) PATTERN CANDIDATES — candidate bee.pattern concepts, bee.polarity pitfall

from cell aap-1 — save as docs/knowledge/patterns/agent-anti-patterns-aap-1-pitfall.md

---
type: bee.pattern
title: agent-anti-patterns cell aap-1 — pitfall candidate
description: "Pitfall candidate mined from cell aap-1's capped trace: followed the plan"
timestamp: 2026-11-04
bee:
  id: agent-anti-patterns-aap-1-pitfall
  lifecycle: draft
  areas: [workflow-state]
  sources: [.bee/cells/aap-1.json]
  polarity: pitfall
---

# agent-anti-patterns cell aap-1 — pitfall candidate

## What the cell did

Planning, worker and goal-check doctrine now carry agent-anti-patterns D1 and D2; regen outputs committed

## Recorded evidence (verbatim from .bee/cells/aap-1.json)

- **deviation** — followed the plan

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

from cell aap-2 — save as docs/knowledge/patterns/agent-anti-patterns-aap-2-pitfall.md

---
type: bee.pattern
title: agent-anti-patterns cell aap-2 — pitfall candidate
description: "Pitfall candidate mined from cell aap-2's capped trace: followed the plan"
timestamp: 2026-11-04
bee:
  id: agent-anti-patterns-aap-2-pitfall
  lifecycle: draft
  areas: [workflow-state]
  sources: [.bee/cells/aap-2.json]
  polarity: pitfall
---

# agent-anti-patterns cell aap-2 — pitfall candidate

## What the cell did

bee report outcomes shows plan departures by kind (7a1dff3f8)

## Recorded evidence (verbatim from .bee/cells/aap-2.json)

- **deviation** — followed the plan

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

knowledge promote: 2 capped cell(s) mined, 1 delivery draft, 2 area bullet(s), 2 pattern candidate(s), 0 file(s) written.