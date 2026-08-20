# Herding Tier — Context

**Feature slug:** herding-tier
**Date:** 2026-08-20
**Shaping session:** complete (scope B of herding-executor, ordered by the user; trigger scope-a-...__de911edd resolved by owner supersession)
**Scope:** Standard

## Feature Boundary

`{"kind": "herding"}` becomes a valid `models.<runtime>.generation` tier value: a cell dispatch resolves to a `bee herding run` Bash payload automatically — no per-cell user request — while gathers on the same slot keep their default model. Ends at the config seam; the run verb's own behavior is untouched.

## Locked Decisions

| ID | Decision | Rationale |
|----|----------|-----------|
| D1 | `normalize_tier_value` accepts `{kind:"herding"}` (no other fields required); `resolve_tier` returns a new `Resolved::Herding` only for cell purpose | The tier value is a router, nothing else |
| D2 | The agent command stays the single global `herding.agent_command` — the tier value carries NO per-slot agent override | One authority for which agent runs; herding-executor D2's pass-through already covers any kind |
| D3 | A gather/review/advisor purpose against a herding slot falls back to the runtime's default model for that slot, never a pane (cites herding-executor D7 — the cli mirror) | A gather in a pane is waste; the slot must still serve gathers |
| D4 | `bee dispatch prepare` emits for `Resolved::Herding` a Bash payload `channel:"herding-exec"`: command = `bee herding run --task-file - --json` (+ `--cwd` when a worktree is granted), prompt via the payload's stdin field — `run` gains stdin support via the `-` sentinel on `--task-file` | Mirrors the cli-exec arm byte-for-byte in shape (prepare.rs:739 precedent); argv cannot carry a long brief |
| D5 | The model guard denies an Agent/Task dispatch against a herding-resolved tier with a fix naming the herding-exec Bash path (mirror of cli-tier-denied) | Same guard shape users already know |
| D6 | The orchestrator still owes ALL bee bookkeeping after reading the herding result (cites herding-executor D4) — prepare's payload carries the brief only, never bee verbs for the worker | The pane worker stays bee-ignorant |

### Agent's Discretion

Display strings in status_full/onboard for the new kind; exact test names; whether `--task-file -` or a new `--task-stdin` flag (pick the smaller diff).

## Existing Code Context

- `packages/bee-rs/crates/bee/src/verbs/drivers/models.rs:76,278` — cli kind normalize + for_gather branch to invert
- `packages/bee-rs/crates/bee/src/verbs/drivers/prepare.rs:739-747` — the cli-exec payload arm to mirror
- `packages/bee-rs/crates/bee/src/hooks/model_guard.rs:273,651-665` — cli-tier-denied to mirror
- `packages/bee-rs/crates/bee/src/herding/run.rs` — the verb; needs stdin task input
- `packages/bee-rs/crates/bee/src/verbs/status_full/store.rs:368,413` + `render.rs:30`, `onboard/agents.rs:66` — kind displays
- docs: docs/config-reference.md models section; both `.bee/config-sample*.json`; `skills/bee-herding/references/operational-invariants.md`

## Outstanding Questions

- (none)
