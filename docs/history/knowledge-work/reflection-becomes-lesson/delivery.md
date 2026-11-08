---
type: bee.delivery
title: reflection-becomes-lesson — delivery
description: "Delivery record proposed by bee knowledge promote for work item reflection-becomes-lesson: 2 capped cell(s), 8 recorded deviation(s)."
timestamp: 2026-08-31
bee:
  id: reflection-becomes-lesson-delivery
  lifecycle: active
  required_context: [docs/history/reflection-becomes-lesson/CONTEXT.md, docs/history/reflection-becomes-lesson/plan.md]
  sources: [docs/history/reflection-becomes-lesson/CONTEXT.md, docs/history/reflection-becomes-lesson/plan.md, .bee/cells/archive/reflection-becomes-lesson/rbl-1.json, .bee/cells/archive/reflection-becomes-lesson/rbl-2.json]
---

# reflection-becomes-lesson — Delivery

## What shipped

- **rbl-1** — Made the mistakes answer survive a run: a clean-run entry kind that renders nowhere, a cap that fills both sinks from one reading, a close door that refuses cells that never answered, and the instruction in all three doctrine homes. (15 file(s) changed)
- **rbl-2** — Reflections join the miner's trouble sources by their what alone, the clean-run answer is excluded by shape, and the feedback digest reports the clean-run-to-reflection ratio (3 file(s) changed)

## Verify

Each cell below was capped only against a recorded passing verify result — bee refuses a cap without one.

- **rbl-1** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee mailbox && PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee handlers_close && PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee drivers::close && PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml --test registry_contracts && .bee/bin/bee dev release-manifest --check`
- **rbl-2** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee mailbox_digest && PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee feedback`

## Deviations

- **rbl-1** — Gave the close door a second way to pass — a mistakes answer recorded in the closing run clears it — because the refusal text the cell dictates offers bee mailbox reflect as its first remedy, and that verb writes to the run, never to an already-capped cell trace; without this the printed remedy is a dead end. The debt COUNT still reads the feature capped cells only, so the refuse-every-close hazard cannot occur. — a remedy that does not clear the door it is printed under is a trap — the plan was wrong about a fact
- **rbl-1** — Spelled the clean-run flag as bee mailbox reflect --no-mistakes rather than a flag on bee close: a close flag needed a new parameter through close_handler and its ~50 call sites, and it would have had to write before the doors ran, breaking the must-have that a refused close writes nothing. — found a better route
- **rbl-1** — Edited four files the cell did not list: finish_support.rs (parse_report_flag is where the optional mistakes key had to be accepted), cells/tests.rs, drivers/tests.rs, knowledge/tests.rs (every CapFlags literal is exhaustive, so two new fields break them all). All four reserved under this cell before writing. — the new field could not compile without them — something else had to be fixed first
- **rbl-1** — Left docs/knowledge/areas/human-mailbox/overview.md untouched though the cell names it in affects_specs: it is not in the cell files list, and that sync belongs to the scribe step or to rbl-2, which owns the miner half of the same area. — the cell file list is the scope — something else had to be fixed first
- **rbl-1** — sync-ack: D4 (c556c959) names exactly three doctrine homes for this instruction — AGENTS.md, packages/bee/AGENTS.block.md and the worker prompt — and the cell declares affects_skills [] with no skills path in its files. The skills copies of the cap and close contract are a separate sweep, not this cell.
- **rbl-2** — Restored CRLF line endings in feedback.rs after my scripted edit flattened them to LF and rewrote all 2695 lines — the file ships CRLF and a whole-file rewrite would have buried the 176-line change — hit an unforeseen obstacle
- **rbl-2** — Added docs/knowledge/areas/human-mailbox/overview.md beside the two code files — the cell named it in affects_specs and it still said lesson mining does not read reflections (LR4), which this cell supersedes — something else had to be fixed first
- **rbl-2** — sync-ack: The feedback-digest change is one additive counted key plus a printed suffix; it emits no candidate, drops nothing and refuses nothing, so no behaviour bee-evolving drives has changed. The skill line that teaches bee-evolving to READ the new clean-run ratio is real work and is filed as its own backlog row rather than smuggled into this cap.

## Provenance

Proposed by `bee knowledge promote --work reflection-becomes-lesson` from 2 capped cell trace(s) in `.bee/cells/` and the anchor `docs/history/reflection-becomes-lesson/CONTEXT.md`, `docs/history/reflection-becomes-lesson/plan.md`. Every line above is copied from a trace or from the work item; nothing here is curated truth until a human or agent accepts it.
