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

## Provenance

Proposed by `bee knowledge promote --work lane-model-diversity` from 3 capped cell trace(s) in `.bee/cells/` and the anchor `docs/history/lane-model-diversity/CONTEXT.md`, `docs/history/lane-model-diversity/plan.md`. Every line above is copied from a trace or from the work item; nothing here is curated truth until a human or agent accepts it.
