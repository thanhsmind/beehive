# Herding activity hook — Context

**Feature slug:** herding-activity-hook
**Date:** 2026-08-23
**Shaping session:** complete
**Scope:** Standard
**Domain types:** RUN | CALL

## Feature Boundary

A herded worker pane (`BEE_HERDING_WORKER=1`) keeps exactly one hook alive —
`activity` — which writes the agent's own state into the job mailbox, and
`bee herding run` reads that state ahead of the tmux screen classifier at
every wait point. Delivered and done stay file-proven (`ack-N.json`,
`result-N.json`); nothing else about the worker contract changes.

Source: `docs/history/research/agent-orchestrator-mailbox-distill.md` (xia).

## Locked Decisions

| ID | Decision | Rationale |
|----|----------|-----------|
| D1 | Under `BEE_HERDING_WORKER=1` exactly ONE hook keeps running — `activity`. Every other hook still exits 0 silently before reading stdin (herding-worker-standalone D3 holds). | `activity` never denies, never prints; the guard silence stays intact. |
| D2 | In a herded pane the activity hook writes `.bee/mailbox/<job-id>/activity.json`, stamped with job id and round, tmp-then-rename. It does not write `.bee/sessions/` for that pane. | One truth per job beside ack/result; a record with an older round is ignored (the round is the launch-id fence). |
| D3 | When a fresh `activity.json` exists for the job, the run verb reads it BEFORE the screen classifier at the ready gate, pointer delivery, and the round poll: `blocked`/`waiting_input` ends the wait as blocked; `working` satisfies the submit-observed check. Screen classifier stays the fallback for hookless agent kinds. `ack-N.json` / `result-N.json` remain the only truth for delivered and done. | The hook signal is exact where the screen read missed a trust dialog live (herding-prompt-stall). Upgrade, never a requirement. |
| D4 | Source manifest: agent-orchestrator `/home/thanhsmind/projects/AI/agent-orchestrator` @ `d4ae9b318e2a14748661c5b71ad589c2f1153521`, scope tmux adapter / delivery / sessionguard / hooks / outbox / human-in-loop. Only the hook return channel is adopted. | Provenance. |

Decision-log ids: D1 `09f16084`, D2 `8040996b`, D3 `2284af77`, D4 `848e0aa5`.

### Agent's Discretion

- How the hook learns the job id and round (env vars exported into the pane
  before `agent start` is the expected path — verify live).
- What "fresh" means for `activity.json` (an age bound; pick one and record it).
- Optional, if cheap: a short pause before the `Enter` keystroke on tmux
  (agent-orchestrator uses 300 ms). Not required by any decision.

## Terms

| Term | Meaning in this feature |
|------|-------------------------|
| activity record | `.bee/mailbox/<job-id>/activity.json`: `{state, event, tool_name?, tool_use_id?, at, job_id, round}` — same state vocabulary as `hooks/activity.rs`. |
| hookless agent kind | a herd entry whose tool installs no hooks; it never produces an activity record and keeps the screen path. |

## Existing Code Context

### Reusable Assets

- `packages/bee-rs/crates/bee/src/hooks/activity.rs` — the state machine
  (working / waiting_input / blocked / idle / exited, same-`tool_use_id`
  unblock rule). Reuse as-is; only the sink changes under the marker.
- `packages/bee-rs/crates/bee/src/herding/mailbox.rs` — path arithmetic and
  tmp-then-rename pattern; add `activity_path` beside `ack_path`.

### Integration Points

- `packages/bee-rs/crates/bee/src/hooks/mod.rs:116-122` — the early exit
  under the marker; `activity` must pass through it.
- `packages/bee-rs/crates/bee/src/herding/run.rs` — `PaneTransport::agent_wait`
  / `agent_status` / `agent_prompt` callers: the ready gate, `deliver_pointer`,
  the round poll.
- `packages/bee-rs/crates/bee/src/herding/run.rs:2002-2008` — pane env export
  (`BEE_HERDING_WORKER=1`); the job id and round ride the same export.
- `packages/bee-rs/crates/fleet/src/screen.rs` — the classifier that becomes
  the fallback.

## Canonical References

- `docs/knowledge/areas/bee-herding/handing-a-foreign-agent-its-brief.md` —
  receipt-as-artifact rule; D3 must not weaken it.
- `docs/history/herding-prompt-stall/CONTEXT.md` — the live trust-dialog miss.
- `docs/history/research/agent-orchestrator-mailbox-distill.md` — the xia.

## Outstanding Questions

### Resolve Before Planning

- None.

### Can Defer

- Should a dashboard (waggledance) also read `activity.json` from the
  mailbox? Out of scope here; it keeps reading `.bee/sessions/` for
  non-herded sessions.
