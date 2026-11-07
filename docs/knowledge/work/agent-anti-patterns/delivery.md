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
  sources: [docs/history/agent-anti-patterns/CONTEXT.md, cell aap-1 (trace purged from .bee/cells/), cell aap-2 (trace purged from .bee/cells/)]
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
