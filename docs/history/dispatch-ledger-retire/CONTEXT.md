# Dispatch ledger retires rows, wave re-print, transport doc, budget signal — Context

**Feature slug:** dispatch-ledger-retire
**Date:** 2026-11-19
**Backlog items:** p-7242d305, p-babca9e6, p-cbe95561, p-1126e3ec
**Shaping session:** complete under gate bypass full (owner said go on the 2026-11-19 backlog sweep)
**Scope:** Standard
**Domain types:** RUN

## Feature Boundary

The `state.json` `workers[]` dispatch-registration ledger retires its rows at
cap, and an empty wave is flagged. Each dispatch wave entry names its re-print
command. bee-swarming states the transport check. The dead budget-overrun
supervisor signal is removed.

## Locked Decisions

| ID | Decision | Rationale |
|----|----------|-----------|
| D1 | Red first: a failing test reproduces the growing ledger (a capped cell's row stays `running`) and the prune deadlock (a capped cell's row keeps its `.bee/workers/` files). Then the cap path calls `push_worker_record` to set the row status to `capped`, and the prune keep set skips rows whose status is `capped`. The status field stays. | p-7242d305 (a)(b)(c)(e). Chesterton's fence: the field and its tests exist; only the caller is missing. |
| D2 | A wave result where every worker returned nothing is flagged in the wave output (`empty_wave: true` plus a one-line reason), distinct from a wave that launched nothing. The herding path reads fleet `WaveResult`. | p-7242d305 (d). |
| D3 | Each `bee dispatch wave` entry carries the command that rebuilds its payload: `.bee/bin/bee dispatch prepare --runtime <rt> --kind cell --cell <id> --worker w-<id> --json`. The worktree refusal (herding-leader-toil D2) stays. | p-babca9e6. |
| D4 | The transport_ready code is already built (prepare.rs near 3046-3076). The remaining work is doc: the bee-swarming Delegation contract states that a dispatch reads `transport_ready` and `transport_reason`, and that the fallback model applies only when the payload says `transport_ready: false`. `bee herding status` without a built job says it does not answer availability and names `HERDR_ENV` and `HERDR_PANE_ID`. | p-cbe95561 remainder. |
| D5 | The `budget-overrun` signal leaves the supervisor's closed signal set, its reader of `estimate_minutes`, and the supervisor prompt. Nothing writes `estimate_minutes`, so the signal can never fire. | p-1126e3ec; subtract before you add. A later estimate input is a new PBI. |

## Read set

### Code

| Path | What it decides |
|------|-----------------|
| `packages/bee-rs/crates/bee/src/verbs/state_group/workers.rs` | D1. |
| `packages/bee-rs/crates/bee/src/verbs/drivers/prepare.rs` | D1 registration, D3. |
| `packages/bee-rs/crates/fleet/src/choreography.rs` | D2. |
| `packages/bee-rs/crates/bee/src/verbs/supervisor.rs` | D5. |
| `skills/bee-swarming/SKILL.md` | D4. |

## Not In Scope

- Adding an estimate input to cells.

## Handoff Note

CONTEXT.md is the source of truth. Decision IDs are stable.
