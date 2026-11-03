# Observing and controlling Paseo workers — Context

**Feature slug:** paseo-observe
**Date:** 2026-10-06
**Shaping session:** complete
**Scope:** Standard
**Domain types:** SEE | CALL | RUN

## Feature Boundary

bee's observation and control verbs handle a Paseo job as well as a herdr
pane: status shows the agent's real state, interrupt and cancel act on the
agent, a worker waiting on a permission is shown, filed for a person and
answered with a bee verb, occupancy counts Paseo workers, and a timeout
carries the agent's log tail. Herdr and tmux behaviour does not change.

## Locked Decisions

| ID | Decision | Rationale |
|----|----------|-----------|
| D1 | A Paseo worker that waits on a permission keeps its running turn: `bee herding run` keeps polling while it is blocked (up to the idle timeout), `bee herding status` shows it as blocked with the requested tools, it is filed as a human decision, and a person answers with `bee herding permit <job> allow` or `deny`, after which the same turn goes on. (store `0ab509ff`) | User choice 2026-10-06. |
| D2 | `bee herding cancel` on a Paseo job stops the agent and archives it (logs kept), never on the caller's own `PASEO_AGENT_ID`; `bee herding interrupt` stops the running turn with `paseo stop` and keeps the agent; a Paseo agent labelled `bee_job` that bee no longer tracks is only reported, never archived by bee. (store `635daa35`) | User choice 2026-10-06. |

These extend paseo-pi D5 (never interrupt-and-replace a running turn):
interrupt and cancel are the user's explicit stop, so they may stop a turn.

## Existing Code Context

From the advisor inventory of 2026-10-06 (digest in `reports/advisor.md`):

- `execute_paseo` writes `paseo_agent_id` but no `pane_id` and no
  wave-ledger row; every surface below keys on `pane_id`.
- `packages/bee-rs/crates/bee/src/herding.rs:915` — status reads `pane_id` only.
- `packages/bee-rs/crates/bee/src/herding/mailbox.rs:934` — the orphan sweep skips a job with no `pane_id`.
- `packages/bee-rs/crates/bee/src/herding/job_verbs.rs` — interrupt and cancel refuse a Paseo job (`pane_missing`).
- `packages/bee-rs/crates/bee/src/herding/pane_verbs.rs:162` — pane verbs know herdr and tmux only.
- `packages/bee-rs/crates/bee/src/herding/wave.rs` — the live set comes from herdr or tmux panes only.
- `packages/bee-rs/crates/bee/src/herding/run.rs:1269` — a blocked observation ends the run.
- `packages/bee-rs/crates/bee/src/herding/control_loop.rs:324` — the supervisor's tool allowlist.
- `packages/bee-rs/crates/bee/src/verbs/supervisor.rs:283` — the human-decision kinds.
- Paseo CLI 0.10.3: `paseo stop <id>`, `paseo archive --force <id>`,
  `paseo logs <id> --tail N [--filter …]`, `paseo ls --label k=v --json`,
  `paseo permit allow|deny <agent> [req] [--all]`, `paseo permit ls`.

## Outstanding Questions

None for the user.
