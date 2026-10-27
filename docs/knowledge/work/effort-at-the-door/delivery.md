---
type: bee.delivery
title: effort-at-the-door — delivery
description: "Delivery record proposed by bee knowledge promote for work item effort-at-the-door: 3 capped cell(s), 15 recorded deviation(s)."
timestamp: 2026-10-17
bee:
  id: effort-at-the-door-delivery
  lifecycle: active
  areas: [doctrine-layer]
  required_context: [docs/history/effort-at-the-door/CONTEXT.md, docs/history/effort-at-the-door/plan.md]
  sources: [docs/history/effort-at-the-door/CONTEXT.md, docs/history/effort-at-the-door/plan.md, .bee/cells/ead-1.json, .bee/cells/ead-2.json, .bee/cells/ead-3.json]
---

# effort-at-the-door — Delivery

## What shipped

- **ead-1** — Door moves each dispatch effort by kind and lane, sends it to claude and codex sinks, marks bee-kind and bee-lane, records four economics keys and an explain row (4 file(s) changed)
- **ead-2** — Both guard legs read the effort ladder; Agent effort repaired, codex denied against the laddered value (1 file(s) changed)
- **ead-3** — team show prints model/effort and the effort ladder line with idle suffix from EFFORT_LADDER, plus effort_ladder/effort_ladder_idle JSON; concept gains B24, Open Gap and Pointers (2 file(s) changed)

## Verify

Each cell below was capped only against a recorded passing verify result — bee refuses a cap without one.

- **ead-1** — ``PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee verbs::drivers` is green, AND the recorded red run of the new tests failed before the implementation; both outputs in the cap evidence. `.bee/bin/bee dev comment-baseline --check` is clean. `packages/bee-rs/target/release/bee dispatch prepare --runtime claude --kind advisor --role hat-value --stage plan-check-value --json` prints economics with keys effort, effort_base, effort_step, effort_reason and a prompt containing `[bee-kind: advisor]`.` — drivers tests 446 passed 0 failed, full -p bee crate green once with 4311 unit plus integration suites and 0 failed, red seen first as 5 new or updated prepare tests FAILED against a None stub after …
- **ead-2** — ``PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee hooks::model_guard` is green, AND the recorded red run of the new tests failed before the fix; both outputs in the cap evidence. `.bee/bin/bee dev comment-baseline --check` is clean.` — green run: 80 passed, 0 failed (scoped to the touched hook); red run before the fix: 74 passed, 6 failed (effort_wrong_value, effort_missing, effort_matching, effort_on_a_role_with_no_base, wrong_mod…
- **ead-3** — ``PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee verbs::models_group` is green, AND the recorded red run of the three new tests failed before the change. `packages/bee-rs/target/release/bee team show --runtime claude` (built with `cargo build --release --manifest-path packages/bee-rs/Cargo.toml`) prints a line starting `effort ladder:` ending with the idle suffix. `rg -c 'effort-at-the-door D2' docs/knowledge/areas/doctrine-layer/model-roles-and-escalation.md` prints at least 1. `.bee/bin/bee knowledge index --check` is clean. `.bee/bin/bee dev comment-baseline --check` is clean.` — touched models_group.rs team show and the model-roles concept; models_group filter 18 passed incl. 3 new tests seen red first (15 passed, 3 failed before the change); full crate run once 4322 passed …

## Deviations

- **ead-1** — verify ran prepare with --role hat-alternatives --stage plan-check-alternatives instead of hat-value and plan-check-value — the plan marks plan-check-value not-applicable so the door refuses it with stage_not_applicable — the plan was wrong about a fact
- **ead-1** — test c checks the codex gather effort model_reasoning_effort low and its economics but not [bee-kind: gather] in a prompt — a codex gather is a command-line payload whose stdin has no marker line, so D10 which covers native payloads does not reach it, and a codex native cell test checks [bee-kind: cell] [bee-lane: tiny] and reasoning_effort instead — the plan was wrong about a fact
- **ead-1** — the earlier partial test expected effort None for an unknown lane, changed to Some high because the action says no row means step 0 applied to the base — the plan was wrong about a fact
- **ead-1** — the effort explain row reads skip with role did not resolve when prepare refused, and the two explain tests in prepare.rs now expect four rows — hit an unforeseen obstacle
- **ead-1** — extra_markers became a plain String that always carries [bee-kind:], and codex_spawn_payload takes a str, so the dead None arms are gone — found a better route
- **ead-1** — about 15 existing tests in drivers tests.rs and guard.rs changed their expected marker bytes, economics bytes, or gather effort to low by design — something else had to be fixed first
- **ead-1** — the release binary lands in /home/thanhsmind/.cache/cargo-target/release/bee because CARGO_TARGET_DIR points there, not packages/bee-rs/target/release/bee — the plan was wrong about a fact
- **ead-2** — Composed the effort repair by wrapping the model verdict (evaluate_claude_dispatch now calls evaluate_claude_model, then adds the effort fix onto the same updated_input) instead of extending repair() and its three callers — one edit point covers every model, type and allow branch with no signature change; Verdict.effort carries the EffortChoice for the economics twin — found a better route
- **ead-2** — Imported effort_for_dispatch through crate::verbs::drivers instead of crate::verbs::drivers::models — the models module is private and re-exported by glob; it is the same one function — the plan was wrong about a fact
- **ead-2** — Codex fixture codex_spawn_mismatched_effort_override_refused needed no move: with no [bee-kind:] token its expected value stays the configured base — the plan was wrong about a fact
- **ead-2** — Tests (f) and (g) passed on the base too: the base never checks effort, so these rows pin allow-unchanged behavior; the other six new tests and the twin key-order assertions were red on base — something else had to be fixed first
- **ead-3** — test uses the drivers re-export EFFORT_LADDER instead of the path drivers::models::EFFORT_LADDER — the models module is private and the re-export is the same constant — the plan was wrong about a fact
- **ead-3** — committed with a path-scoped commit instead of staging into the shared index — the concurrent-worker guard refuses staging while w-ead-2 is live — hit an unforeseen obstacle
- **ead-3** — ran the team show verify with /home/thanhsmind/.cache/cargo-target/release/bee instead of packages/bee-rs/target/release/bee — CARGO_TARGET_DIR points there, per the leader note — the plan was wrong about a fact
- **ead-3** — proof command segment is the whole verify text verbatim — the cap refuses any proof command that is not an exact match of the prose verify field — hit an unforeseen obstacle

## Provenance

Proposed by `bee knowledge promote --work effort-at-the-door` from 3 capped cell trace(s) in `.bee/cells/` and the anchor `docs/history/effort-at-the-door/CONTEXT.md`, `docs/history/effort-at-the-door/plan.md`. Every line above is copied from a trace or from the work item; nothing here is curated truth until a human or agent accepts it.
