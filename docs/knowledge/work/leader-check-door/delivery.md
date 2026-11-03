---
type: bee.delivery
title: leader-check-door — delivery
description: "Delivery record proposed by bee knowledge promote for work item leader-check-door: 4 capped cell(s), 7 recorded deviation(s)."
timestamp: 2026-09-21
bee:
  id: leader-check-door-delivery
  lifecycle: active
  required_context: [docs/history/leader-check-door/CONTEXT.md, docs/history/leader-check-door/plan.md]
  sources: [docs/history/leader-check-door/CONTEXT.md, docs/history/leader-check-door/plan.md, .bee/cells/lcd-1.json, .bee/cells/lcd-2.json, .bee/cells/lcd-3.json, .bee/cells/lcd-4.json]
---

# leader-check-door — Delivery

## What shipped

- **lcd-1** — bee cells leader-check records a verified completeness check; the debt scan counts a capped cell unless its newest entry reads ok (4 file(s) changed)
- **lcd-2** — bee close grows a leader-check door at every lane (2 file(s) changed)
- **lcd-3** — bee worktree merge refuses a capped cell with no ok leader check, before any staging (2 file(s) changed)
- **lcd-4** — The doctrine names the recording verb and both doors; the rule keeps one home (5 file(s) changed)

## Verify

Each cell below was capped only against a recorded passing verify result — bee refuses a cap without one.

- **lcd-1** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee leader_check && PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml --test registry_contracts --test registry_dispatch` — 10 leader_check + 12 registry_contracts + 9 registry_dispatch passed, 0 failed; the leader re-ran both itself after the worker returned without capping
- **lcd-2** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee close` — 259 close tests passed, 0 failed, including all 8 must-have truths plus the archived-offender case; leader re-ran after the worker returned without capping
- **lcd-3** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee worktree` — 282 worktree tests passed, 0 failed, including the base refusal with its zero-mutation assertions; leader re-ran after the worker returned without capping
- **lcd-4** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee skill && .bee/bin/bee dev release-manifest --check` — 52 skill tests passed 0 failed; bee dev release-manifest --check reports 376 file(s) match stored manifest

## Deviations

- **lcd-1** — The worker returned a success summary without capping the cell; the leader capped it after running the completeness check. Named, not silent.
- **lcd-1** — sync-ack: The workflow-state skill sync is lcd-4, a planned slice-2 cell (docs/history/leader-check-door/plan.md, 'Slice 2 — the doctrine sync'). Slicing it away from lcd-1 is deliberate: the doctrine text must describe the verb AND both doors, and lcd-2/lcd-3 have not landed yet, so a sync written now would document a half-built door. The close-time scribing door still holds the feature to it.
- **lcd-2** — Touched packages/bee-rs/crates/bee/src/verbs/drivers/tests.rs, which the cell's files list did not name — two lines, both updating an existing door-list assertion that now has to carry the new 'door leader-check: clear' row. bee cells judge reports no hits. Widened scope to keep an existing assertion honest.
- **lcd-2** — sync-ack: The workflow-state skill sync is lcd-4, a planned slice-2 cell; the doctrine text must describe the verb AND both doors, and lcd-3 had not landed when this cell was written.
- **lcd-3** — Touched packages/bee-rs/crates/bee/src/verbs/worktree/tests.rs, which the cell's files list did not name — the new merge-door tests live there beside the existing merge-door tests, which is where the cell's action told the worker to put them. bee cells judge reports no hits. The files list should have named it; the action line did.
- **lcd-3** — sync-ack: The workflow-state skill sync is lcd-4, a planned slice-2 cell covering the verb and both doors together.
- **lcd-4** — The regen chain rewrote 26 generated mirror files the cell's files list did not name (.claude/skills/**, .agents/skills/**, .claude-plugin/**, .codex-plugin/**, .opencode/skills/**, .bee/onboarding.json). They are derived from the four edited skill sources, which is what the cell's REGEN instruction asked for. bee cells judge reports no hits.

## Provenance

Proposed by `bee knowledge promote --work leader-check-door` from 4 capped cell trace(s) in `.bee/cells/` and the anchor `docs/history/leader-check-door/CONTEXT.md`, `docs/history/leader-check-door/plan.md`. Every line above is copied from a trace or from the work item; nothing here is curated truth until a human or agent accepts it.
