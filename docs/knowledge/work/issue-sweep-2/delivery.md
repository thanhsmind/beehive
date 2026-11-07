---
type: bee.delivery
title: issue-sweep-2 — delivery
description: "Delivery record proposed by bee knowledge promote for work item issue-sweep-2: 3 capped cell(s), 4 recorded deviation(s)."
timestamp: 2026-11-03
bee:
  id: issue-sweep-2-delivery
  lifecycle: active
  areas: [workflow-state, worktree-parallelism]
  required_context: [docs/history/issue-sweep-2/CONTEXT.md, docs/history/issue-sweep-2/plan.md]
  sources: [docs/history/issue-sweep-2/CONTEXT.md, docs/history/issue-sweep-2/plan.md, cell is2-1 (trace purged from .bee/cells/), cell is2-2 (trace purged from .bee/cells/), cell is2-3 (trace purged from .bee/cells/)]
---

# issue-sweep-2 — Delivery

## What shipped

- **is2-1** — Added bee state session attach as an alias of the session bind handler, declared it in the registry with the same flags, and named it in every FIX line under verbs/state_group — 60e2285ab (9 file(s) changed)
- **is2-2** — Routing door now reads comma and multi-id D-ID lists on feature-local decisions (close.rs feature_local_hit + parse_d_sequence comma continuation), red-first proven; commit 89df525d9 (3 file(s) changed)
- **is2-3** — worktree merge now excludes untracked bee-* skill trees that also exist under the same root in main, so bee's own synced copies no longer refuse as dirty; a bee-* tree only the worktree holds still refuses. Commit 961c7e077. (3 file(s) changed)

## Verify

Each cell below was capped only against a recorded passing verify result — bee refuses a cap without one.

- **is2-1** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee session && PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee --test registry_contracts` — full declared suite on the issue-sweep-2 worktree rerun by the leader: 5053 passed 0 failed; worker recorded red before the fix
- **is2-2** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee routing` — full declared suite on the issue-sweep-2 worktree rerun by the leader: 5053 passed 0 failed; worker recorded red before the fix
- **is2-3** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee worktree` — full declared suite on the issue-sweep-2 worktree rerun by the leader: 5053 passed 0 failed; worker recorded red before the fix

## Deviations

- **is2-1** — also edited tests/route_lane_targeting.rs (assertions pinned the old FIX text) and hooks/session_preamble/budget.rs (COMMAND_SURFACE_BUDGET_CHARS 2600 to 2604 for the longer verb)
- **is2-1** — sync-ack: no skill names state session bind; the alias reaches users through FIX lines
- **is2-2** — sync-ack: close routing door internals; no skill teaches this matcher
- **is2-3** — sync-ack: worktree merge dirty check internals; no skill step changes

## Provenance

Proposed by `bee knowledge promote --work issue-sweep-2` from 3 capped cell trace(s) in `.bee/cells/` and the anchor `docs/history/issue-sweep-2/CONTEXT.md`, `docs/history/issue-sweep-2/plan.md`. Every line above is copied from a trace or from the work item; nothing here is curated truth until a human or agent accepts it.
