promote proposal for work item "lane-model-diversity" (docs/history/lane-model-diversity/CONTEXT.md + docs/history/lane-model-diversity/plan.md) — 3 capped cell(s): lmd-1, lmd-2, lmd-3
anchor: history — docs/history/lane-model-diversity/CONTEXT.md, docs/history/lane-model-diversity/plan.md
PROPOSAL ONLY — nothing was written. Applying any section below is a human or agent decision.

(a) DELIVERY DRAFT — save as docs/knowledge/work/lane-model-diversity/delivery.md

---
type: bee.delivery
title: lane-model-diversity — delivery
description: "Delivery record proposed by bee knowledge promote for work item lane-model-diversity: 3 capped cell(s), 7 recorded deviation(s)."
timestamp: 2026-08-29
bee:
  id: lane-model-diversity-delivery
  lifecycle: active
  required_context: [docs/history/lane-model-diversity/CONTEXT.md, docs/history/lane-model-diversity/plan.md]
  sources: [docs/history/lane-model-diversity/CONTEXT.md, docs/history/lane-model-diversity/plan.md, .bee/cells/lmd-1.json, .bee/cells/lmd-2.json, .bee/cells/lmd-3.json]
---

# lane-model-diversity — Delivery

## What shipped

- **lmd-1** — Seat roles resolve per-seat models; unconfigured seats fall through to the advisor at the advisor door only; doctor advises on undescribed hat slots (4 file(s) changed)
- **lmd-2** — Seat-role parity pinned across the dispatch door and the marker guard, with both refusal and admit arms falsified by reverted mutations (1 file(s) changed)
- **lmd-3** — Both procedure homes name the eight seat roles, SEAT_ROLES as the constant of record, and the fall-through rule; config sample carries the seat rows with hat descriptions (2 file(s) changed)

## Verify

Each cell below was capped only against a recorded passing verify result — bee refuses a cap without one.

- **lmd-1** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --no-fail-fast --manifest-path packages/bee-rs/Cargo.toml verbs::drivers`
- **lmd-2** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --no-fail-fast --manifest-path packages/bee-rs/Cargo.toml model_guard`
- **lmd-3** — `rg -n "lane-1" skills/bee-hive/references/gates-and-delegation.md && rg -n "hat-risks" skills/bee-hive/references/gates-and-delegation.md .bee/config-sample.json`

## Deviations

- **lmd-1** — requested_role rides economics rather than the tool payload — economics is inserted into BOTH the returned envelope and the appended dispatch-log row, while the payload is the literal Agent/Bash/spawn_agent argument map and an unknown key there would be handed to the tool — found a better route
- **lmd-1** — Added packages/bee-rs/crates/bee/src/doctor/tests.rs (reserved before writing) — doctor.rs declares mod tests; against that separate file, so the advisory has nowhere else to be tested — hit an unforeseen obstacle
- **lmd-1** — The helper is named role_slot_resolves rather than a seat-only name — it asks a question about any role slot and nothing in it is seat-specific, so a seat-shaped name would have been a lie about its scope — found a better route
- **lmd-1** — Doctor names undescribed hats in the CONFIG's own order, not sorted; my first test expectation assumed alphabetical and went red — the plan was wrong about a fact
- **lmd-2** — Added a fifth row (cell_role_list over every seat) inside model_guard.rs rather than beside cell_role_list — the cell scopes writes to this one file, and the row is the same parity claim from the other side — hit an unforeseen obstacle
- **lmd-2** — Ran two temporary mutations of the guard's known_role_named wrapper and reverted both, because a contract test that has never failed proves only that the guard agrees with itself — found a better route
- **lmd-3** — followed the plan

## Provenance

Proposed by `bee knowledge promote --work lane-model-diversity` from 3 capped cell trace(s) in `.bee/cells/` and the anchor `docs/history/lane-model-diversity/CONTEXT.md`, `docs/history/lane-model-diversity/plan.md`. Every line above is copied from a trace or from the work item; nothing here is curated truth until a human or agent accepts it.

(b) AREA UPDATES — candidate spec-sync bullets, each citing its cell

None: the work item declares no bee.areas, so there is no area to sync (D19).

(c) PATTERN CANDIDATES — candidate bee.pattern concepts, bee.polarity pitfall

from cell lmd-1 — save as docs/knowledge/patterns/lane-model-diversity-lmd-1-pitfall.md

---
type: bee.pattern
title: lane-model-diversity cell lmd-1 — pitfall candidate
description: "Pitfall candidate mined from cell lmd-1's capped trace: requested_role rides economics rather than the tool payload — economics is inserted into BOTH the returned envelope and the appended dispatch-log row, while th…"
timestamp: 2026-08-29
bee:
  id: lane-model-diversity-lmd-1-pitfall
  lifecycle: draft
  sources: [.bee/cells/lmd-1.json]
  polarity: pitfall
---

# lane-model-diversity cell lmd-1 — pitfall candidate

## What the cell did

Seat roles resolve per-seat models; unconfigured seats fall through to the advisor at the advisor door only; doctor advises on undescribed hat slots

## Recorded evidence (verbatim from .bee/cells/lmd-1.json)

- **deviation** — requested_role rides economics rather than the tool payload — economics is inserted into BOTH the returned envelope and the appended dispatch-log row, while the payload is the literal Agent/Bash/spawn_agent argument map and an unknown key there would be handed to the tool — found a better route
- **deviation** — Added packages/bee-rs/crates/bee/src/doctor/tests.rs (reserved before writing) — doctor.rs declares mod tests; against that separate file, so the advisory has nowhere else to be tested — hit an unforeseen obstacle
- **deviation** — The helper is named role_slot_resolves rather than a seat-only name — it asks a question about any role slot and nothing in it is seat-specific, so a seat-shaped name would have been a lie about its scope — found a better route
- **deviation** — Doctor names undescribed hats in the CONFIG's own order, not sorted; my first test expectation assumed alphabetical and went red — the plan was wrong about a fact

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

from cell lmd-2 — save as docs/knowledge/patterns/lane-model-diversity-lmd-2-pitfall.md

---
type: bee.pattern
title: lane-model-diversity cell lmd-2 — pitfall candidate
description: "Pitfall candidate mined from cell lmd-2's capped trace: Added a fifth row (cell_role_list over every seat) inside model_guard.rs rather than beside cell_role_list — the cell scopes writes to this one file, and the r…"
timestamp: 2026-08-29
bee:
  id: lane-model-diversity-lmd-2-pitfall
  lifecycle: draft
  sources: [.bee/cells/lmd-2.json]
  polarity: pitfall
---

# lane-model-diversity cell lmd-2 — pitfall candidate

## What the cell did

Seat-role parity pinned across the dispatch door and the marker guard, with both refusal and admit arms falsified by reverted mutations

## Recorded evidence (verbatim from .bee/cells/lmd-2.json)

- **deviation** — Added a fifth row (cell_role_list over every seat) inside model_guard.rs rather than beside cell_role_list — the cell scopes writes to this one file, and the row is the same parity claim from the other side — hit an unforeseen obstacle
- **deviation** — Ran two temporary mutations of the guard's known_role_named wrapper and reverted both, because a contract test that has never failed proves only that the guard agrees with itself — found a better route

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

from cell lmd-3 — save as docs/knowledge/patterns/lane-model-diversity-lmd-3-pitfall.md

---
type: bee.pattern
title: lane-model-diversity cell lmd-3 — pitfall candidate
description: "Pitfall candidate mined from cell lmd-3's capped trace: followed the plan"
timestamp: 2026-08-29
bee:
  id: lane-model-diversity-lmd-3-pitfall
  lifecycle: draft
  sources: [.bee/cells/lmd-3.json]
  polarity: pitfall
---

# lane-model-diversity cell lmd-3 — pitfall candidate

## What the cell did

Both procedure homes name the eight seat roles, SEAT_ROLES as the constant of record, and the fall-through rule; config sample carries the seat rows with hat descriptions

## Recorded evidence (verbatim from .bee/cells/lmd-3.json)

- **deviation** — followed the plan

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

knowledge promote: 3 capped cell(s) mined, 1 delivery draft, 0 area bullet(s), 3 pattern candidate(s), 0 file(s) written.