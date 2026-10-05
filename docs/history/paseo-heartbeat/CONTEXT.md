# Paseo heartbeat for the Pi leader — Context

**Feature slug:** paseo-heartbeat
**Date:** 2026-10-06
**Shaping session:** complete
**Scope:** Standard
**Domain types:** RUN

## Feature Boundary

A Pi leader that runs inside Paseo makes one Paseo heartbeat for itself at
session start. Each heartbeat runs the bee code tick and becomes a short
model turn: a news prompt, or one "ok" turn. The feature ends at the leader:
workers, worker answers (slice 3, paseo-pi D5) and the herdr path do not
change.

## Locked Decisions

This feature is slice 2 of paseo-pi. It implements decisions locked there;
cite them as `paseo-pi Dn`.

| ID | Decision | Source |
|----|----------|--------|
| D1 | The Pi leader wakes from a Paseo heartbeat. On each heartbeat the bee extension runs the bee code tick. With news, the heartbeat becomes a prompt that carries the news. With no news, it becomes one very short model turn, so Paseo sees the turn end. The extension never swallows a heartbeat. | paseo-pi D6 (store `83f5caad`) |
| D2 | The heartbeat fires every 5 minutes by default; a config key changes it. | paseo-pi D7 (store `8ae5134d`) |
| D3 | The leader turns its heartbeat on by itself at session start inside Paseo (`PASEO_AGENT_ID` set, not a herded worker). It never makes a second heartbeat for the same agent. | paseo-pi D8 (store `2aa8417a`) |

## Terms

| Term | Meaning in this feature |
|------|-------------------------|
| Leader | A Pi session with `PASEO_AGENT_ID` set and no `BEE_HERDING_WORKER`. |
| Code tick | `bee herding broker tick --json`: it routes worker questions with no model call of the leader. |
| News | A tick result with `claimed` or `notices_sent` above 0. |

## Existing Code Context

- `.pi/extensions/bee-guard/result-inbox.ts:381-434` — the result drain
  already polls every 2 seconds and starts a model turn when a worker result
  lands (`sendUserMessage`). Spike 2 (paseo-pi) proved such a turn works in
  Paseo. So worker results do not wait for the heartbeat; the tick covers
  the broker, which has no timer in the leader.
- `.pi/extensions/bee-guard/events.ts:64` (`session_start`), `:213`
  (`input`), `:358` (`agent_settled`, the continuation nudge at `:421-440`).
- `packages/bee-rs/crates/bee/src/herding/broker.rs:693-699` — the tick's
  JSON output `{heartbeat, claimed, notices_sent}`.
- `packages/bee-rs/crates/bee/tests/pi_plugin_contracts.rs` — the node
  harness that tests the extension against a stub.

## Canonical References

- `docs/history/paseo-pi/CONTEXT.md` — D6, D7, D8 and the spike facts.
- `docs/history/paseo-pi/reports/hat-wave.md` — "Next slices": the slice 2
  risks (leader-only guard, drain lock, nudge exemption, quiet turn).
- `docs/knowledge/areas/bee-herding/the-paseo-channel.md`.

## Outstanding Questions

None for the user. The plan settles the rest.
