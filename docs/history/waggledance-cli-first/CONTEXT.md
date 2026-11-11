# Waggledance CLI first, MCP tools as fallback — Context

**Feature slug:** waggledance-cli-first
**Date:** 2026-11-12
**Shaping session:** complete (Qualify path; the owner asked for the full build)
**Scope:** Standard
**Domain types:** CALL

## Feature Boundary

Every place where bee tells an agent how to reach waggledance names the
`waggledance` CLI verb first and the MCP tool as the fallback. The change ends
at doctrine, skills, handbook, config reference, knowledge and the
cross-project guard refusal text. It adds no new bee verb and no new waggledance
call from bee code.

## Locked Decisions

| ID | Decision | Rationale (only if it changes implementation) |
|----|----------|-----------------------------------------------|
| D1 | Every bee home that tells an agent how to reach waggledance names the CLI verb first and the MCP tool second. Project state: `waggledance state <project>`, fallback `waggledance_ask_state`. Leader dispatch: `waggledance dispatch --project <P> --leader --task-file <file>` (add `--follow-up-of <run>`; add `--handoff` for a handoff), fallback `waggledance_dispatch` with `leader: true` (`follow_up_of`, `handoff: true`). | Owner ask (spec `docs/discovery/spec-drops/sup-20261112-1646.md`). The CLI calls the same handler as MCP and sends far less text. |
| D2 | The agent takes the MCP fallback when the `waggledance` binary is not on PATH, when the binary does not know the verb, or when the verb exits non-zero. It then calls the matching MCP tool with the same arguments, once. | The installed binary refuses `projects` and `schema` on 2026-11-12, so an older install is a real case now. |
| D3 | bee never calls or names a `wg` command. | WireGuard owns `wg`; it is only an optional user symlink. |
| D4 | The cross-project guard refusal names the CLI first (`waggledance state <project>`, then `waggledance dispatch --project <project> --leader`) and the MCP tools `waggledance_ask_state` / `waggledance_dispatch` as the fallback. All other parts of the refusal stay. | The refusal is the remedy agents follow; it must match D1. |
| D5 | The guard registry read keeps `waggledance list --json`. Named guess: the spec lists no removal of `list`, and the installed binary still serves it. | Do not change a working read to an unverified verb. |
| D6 | No bee code or doc reads the `waggledance_runs` `task` field or the project-less `ask_state` keys `sessions` / `recent_decisions`. No code change follows; the feature proves the absence with a search. | Verified by `rg` over `skills/`, `packages/` and `docs/`. |
| D7 | The full CLI/MCP mapping and the fallback rule have one home: rule `agents-other-project-to-its-leader` in `packages/bee/LEADER.md`. The handoff homes name their CLI call and cite that rule for the fallback. | One fact, one home. |

## Terms

| Term | Meaning in this feature |
|------|-------------------------|
| CLI first | The agent runs the `waggledance <verb>` command before it calls an MCP tool. |
| MCP fallback | The agent calls the matching `waggledance_*` MCP tool after the CLI path fails per D2. |

## Read set

### Knowledge

| Concept | Grade at read | Why it matters |
|---------|---------------|----------------|
| docs/knowledge/areas/doctrine-layer/router-triage-and-the-agents-md-duplication-boundary.md | suspect | Rule index entries for `agents-other-project-to-its-leader` and `agents-context-handoff-65` quote the MCP tools. |
| docs/knowledge/areas/hook-runtime/write-guard-request-shapes.md | suspect | Cross-project section quotes the refusal remedy and the `waggledance list --json` registry read. |

### Code

| Path | What it decides |
|------|-----------------|
| `packages/bee-rs/crates/bee/src/hooks/write_guard/cross_project.rs` | The refusal text (D4) and the registry read (D5). |
| `packages/bee-rs/crates/bee/src/hooks/write_guard/tests.rs` | Assertions on the refusal text. |
| `packages/bee/LEADER.md` | The rule home (D1, D2, D7). |
| `packages/bee/playbooks/handoff.md` | Handoff leader door. |
| `skills/bee-swarming/SKILL.md`, `skills/bee-swarming/references/swarming-reference.md`, `skills/bee-hive/references/go-mode.md` | Handoff leader door in skills. |
| `docs/handbook/stages/swarming.md`, `docs/config-reference.md` | Handbook and config reference text. |

## Outstanding Questions

### Deferred To Planning

- [ ] Which rendered copies `bee dev regen` rewrites — answered by running it.

## Deferred Ideas

- Switch the guard registry read to a new CLI verb — only if waggledance removes `list` (D5).

## Handoff Note

CONTEXT.md is the source of truth. Decision IDs are stable.
