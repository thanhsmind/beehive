# Paseo 0.17 compatibility — Context

**Feature slug:** paseo-017-compat
**Date:** 2026-11-23
**Shaping session:** complete (gate bypass full; the owner asked what beehive needs for Paseo 0.17, then said go)
**Scope:** Small
**Domain types:** CALL

## Feature Boundary

Paseo 0.17 changed the `paseo send` default so it depends on the sender:
`queue` for an agent sender (`PASEO_AGENT_ID` set), `interrupt` for a
person. bee sends the round brief pointer and the idle nudge as a person,
because it scrubs `PASEO_AGENT_ID` from every paseo call except the worker
`run`. bee must stop relying on that default.

Evidence (Paseo CLI and daemon 0.17.3 on this machine, 2026-11-23):
`paseo send --help` documents the sender-dependent default; a person
`paseo send <id> --behavior queue` fails with `activeTurnBehavior "queue"
needs an agent sender (sourceAgentId)`; `paseo send <id> --behavior
interrupt --no-wait -- "-x probe"` reaches the agent lookup. The live
contract test `paseo_live_contract` passes 3 of 3 on 0.17.3.

## Locked Decisions

| ID | Decision | Rationale (only if it changes implementation) |
|----|----------|-----------------------------------------------|
| D1 | Every plain `paseo send` bee makes (round brief pointer, idle nudge in both wait branches) passes `--behavior interrupt --no-wait -- <text>`. | Keeps the current behavior and stops a Paseo default change from moving it; `queue` is refused for a person sender. |
| D2 | bee adopts no other Paseo 0.17 surface: it does not read `workspaceId` or `labels` from `paseo ls --json` and does not parse the send receipt guidance. | No bee code reads an agent's workspace; the guidance exists only for a queued agent send, which bee never makes. |
| D3 | Contract `paseo-plain-send-behavior`: a `run.rs` test over the recorded fake-CLI argv fails when any plain send lacks `--behavior interrupt` or carries `--behavior queue`. | Pins D1 against a refactor of `send_argv` or the round path. |

## Read set

| Kind | Path |
|------|------|
| Knowledge | docs/knowledge/areas/bee-herding/the-paseo-channel.md |
| Code | packages/bee-rs/crates/bee/src/herding/paseo.rs |
| Code | packages/bee-rs/crates/bee/src/herding/run.rs |
