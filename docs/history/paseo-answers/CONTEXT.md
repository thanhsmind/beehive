# Answers and steering for Paseo workers — Context

**Feature slug:** paseo-answers
**Date:** 2026-10-06
**Shaping session:** complete
**Scope:** Standard
**Domain types:** CALL | RUN

## Feature Boundary

A worker on the Paseo channel that asks a question gets the answer in the
same Paseo agent, as the next round of the same job. `bee herding steer`
reaches a running Claude, Codex or OpenCode worker on Paseo. The feature
ends there: herdr and tmux jobs, the leader heartbeat and the broker's
routing rules do not change.

## Locked Decisions

| ID | Decision | Rationale |
|----|----------|-----------|
| D1 | A Paseo worker that ends its round with a question keeps its Paseo agent. The answer goes to that same agent as the next round of the same job, sent only when Paseo shows the agent idle, never into a running turn. When that agent is gone or not idle, the broker starts a fresh child job as today. (store `240f3db5`) | User choice 2026-10-06: the agent keeps its context; a question ends the round, so the agent is idle when the answer arrives (paseo-pi D5). |
| D2 | `bee herding steer` on a Paseo job whose provider is claude, codex or opencode steers the running turn through the Paseo daemon (`activeTurnBehavior: "steer"`) with a small Node helper. A Pi worker keeps the steer-file drain. A steer that cannot be delivered refuses with a clear error and never falls back to an interrupting send. (store `e13feabc`) | User choice 2026-10-06: otherwise a steer to a Claude or Codex worker on Paseo is lost in silence. |

Both decisions implement paseo-pi D5 (store `b6dd8b33`): never
interrupt-and-replace a running worker turn.

## Existing Code Context

- `packages/bee-rs/crates/bee/src/herding/broker.rs:307` — every answer path
  (tick at `:538`, `:661`; `answer_with` at `:763`) goes through
  `start_child_job_for_answered_question`.
- `packages/bee-rs/crates/bee/src/herding/broker.rs:175-216` — `JobSpawner`
  and `RealJobSpawner`, which build the `bee herding run` command.
- `packages/bee-rs/crates/bee/src/herding/run.rs:3602` — `execute_continue`
  writes the next round's brief from `--task` and prompts the recorded pane.
- `packages/bee-rs/crates/bee/src/herding/run.rs:3262` — `execute_paseo`
  (paseo-pi slice 1); it archives the agent on any valid result today,
  a question included.
- `packages/bee-rs/crates/bee/src/herding/job_verbs.rs:500` — `steer_job`
  writes `steer-N.json`; only a Pi worker drains it.
- `packages/bee-rs/crates/bee/src/herding/paseo.rs` — the Paseo CLI helpers.

## Environment facts

- The daemon WebSocket steer worked in paseo-pi spike 2 with
  `connectToDaemon` and `selectDaemonTarget` from the npm `@getpaseo/cli`
  0.10.3 (`dist/utils/client.js`, `dist/utils/daemon-target.js`) and
  `sendAgentMessage(id, text, {activeTurnBehavior: "steer"})`.
- `paseo send` with no steer option interrupts a running turn, so it may go
  only to an idle agent.

## Outstanding Questions

None for the user.
