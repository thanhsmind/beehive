# Herd Registry — Context

**Feature slug:** herd-registry
**Date:** 2026-08-20
**Shaping session:** complete (relayed ask + user interview, three answers)
**Scope:** Standard

## Feature Boundary

`herding.agents` names pane-agent commands once; tier slots, `bee herding run --agent`, and `herding.agent_command` reference them by name. The cli tier kind is untouched. Ends before any per-repo agent auto-selection logic.

## Locked Decisions

| ID | Decision | Rationale |
|----|----------|-----------|
| D1 | `herding.agents` = map name → argv token array, same shape and validation as `herding.agent_command` (token 0 = herdr kind, no newlines, fail-open per entry). A herd name ALWAYS means the pane transport; cli stays cli | User: "herd luôn đi theo pane, cli là cli, tên herding.agents" |
| D2 | Three reference spellings, one resolver: `{kind:"herding", agent:"<name>"}` on a tier slot; `bee herding run --agent <name>`; `herding.agent_command` as a plain string naming a herd. Unknown name = typed refusal LISTING the registry keys; absent = global agent_command default | One tier kind (extends kind:herding, no parallel kind:herd); resolution lives in resolve_agent_command alone |

## Existing Code Context

- `packages/bee-rs/crates/bee/src/herding/wave.rs` — `agent_command_tokens`/`resolve_agent_command` (the one resolver to extend)
- `packages/bee-rs/crates/bee/src/herding/run.rs` — flags + spawn (gains `--agent`)
- `packages/bee-rs/crates/bee/src/verbs/drivers/models.rs` — kind:herding normalize (gains optional `agent`)
- `packages/bee-rs/crates/bee/src/verbs/drivers/prepare.rs:748-770` — herding-exec arm (appends `--agent`)
- Samples + docs/config-reference.md + operational-invariants.md — the documented homes

## Outstanding Questions

- (none)
