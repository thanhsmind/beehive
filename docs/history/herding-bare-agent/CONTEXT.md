# Herding Bare-Run Agent Resolution — Context

**Feature slug:** herding-bare-agent
**Date:** 2026-08-20
**Shaping session:** complete (user-directed, cross-session hand-off from beehive-eb)
**Scope:** Quick

## The problem (live, 2026-08-20)

Two independent places name the agent a herded worker runs as, and they disagree:

1. The tier slot — `.bee/config.json` `models.claude.generation = {"kind":"herding","agent":"agy-flash"}`
   — reaches only the dispatch/model-guard path (`verbs/drivers/models.rs` `resolve_tier`,
   `hooks/model_guard.rs`). It says: cell-execution work runs as `agy-flash`.
2. `herding.agent_command = "claude-sonnet"` — the default of `bee herding run` when no
   `--agent` is passed (`herding/wave.rs:305` `resolve_agent_command`, caller
   `herding/run.rs:896`).

Result: the user configured the herd to open `agy`, and a bare `bee herding run` opened
`claude-sonnet`. The tier slot — the place the user actually configures the role-to-agent
mapping — is invisible to the verb that opens the pane.

The user's intent: "if herding is configured, just run it — you may pass no agent, because
herding is already configured with the right roles on a specific agent and model." A bare
`bee herding run` executes a cell, so it must resolve its agent through the CELL-EXECUTION
ROLE (the `generation` slot), not through a second, parallel default.

## Locked Decisions

| ID | Decision | Rationale |
|----|----------|-----------|
| D1 | `resolve_agent_command` gains one step between the `--agent` lookup and the `herding.agent_command` fallback: the cell-execution role. Full precedence — `--agent <name>` > `models.<runtime>.generation` when it is a herd slot naming an agent > `herding.agent_command` > built-in default array | One role, one answer: the slot the user configures for cell execution is what the pane-opening verb obeys. The explicit flag still wins; the old fallbacks stay for configs that never set a herd slot |
| D2 | Only a slot of the exact shape `{"kind":"herding", "agent":"<non-empty string>"}` participates. `{"kind":"herding"}` with no `agent`, a plain model-name slot (`"sonnet"`), `{"kind":"cli",...}`, `null`, or any other shape is SKIPPED — resolution falls through to `herding.agent_command` unchanged | A model name names no herd entry; a kind-herding slot without an agent expresses "use the herd", not "use THIS herd agent". Skipping is the only reading that cannot invent a name |
| D3 | The runtime block read for D1 is `BEE_RUNTIME` when it names a known runtime (`claude`, `codex`, `opencode`), else `claude` — the same fallback `resolve_tier` already applies to an unknown runtime name. No new flag on `bee herding run` | `bee herding run` carries no runtime today; the env-or-claude default matches existing behavior and adds no surface. A flag can follow later if a real codex-host need appears — registered as trigger `a-herding-run-is-needed-from-a-non-claud__7abf296f` |
| D4 | A slot agent absent from the `herding.agents` registry fails closed — `AgentCommandError::UnknownAgent` listing every known key, exactly as `--agent <typo>` does. It NEVER silently falls back to `herding.agent_command` | The whole defect being fixed is a silent wrong agent. Loud beats silently-wrong; the message names the fix |
| D5 | `bee herding wave` (`wave.rs:457`, `resolve_agent_command(cfg, None)`) inherits the same precedence — it dispatches cell workers on the same role | One resolver, one role rule; a second spelling would re-create this bug |

## Non-goals

- `herding.control_command` (the control-loop transport) is untouched — it is not a cell-execution role.
- The `extraction` / `review` / `advisor` slots are untouched: `bee herding run` opens cell-execution panes only.
- No config migration and no deprecation of `herding.agent_command`; it stays the fallback.

## Sequencing

`herding/wave.rs` and `herding/run.rs` are contested. Queue agreed with beehive-eb
(2026-08-20): **herding-worker-standalone** (awaiting uat/merge) → **herding-limit-pause**
(cells hlp-1/hlp-2 open) → **this feature**. Execution starts only after both land on main;
watch the store / `git log main`. Cells here are shaped and left open until then.

## Evidence

- `.bee/config.json` (live): `models.claude.generation = {"kind":"herding","agent":"agy-flash","fallback":"default"}`; `herding.agent_command = "claude-sonnet"`.
- `packages/bee-rs/crates/bee/src/herding/wave.rs:305` — `resolve_agent_command(cfg, agent)`: `--agent`, then a string `agent_command`, then the array split, then the default. `cfg` is the WHOLE config document, so `models` is already in hand; D1 needs no new plumbing.
- `packages/bee-rs/crates/bee/src/herding/run.rs:896` — the single bare-run call site.
- `packages/bee-rs/crates/bee/src/verbs/drivers/models.rs:321,327` — `resolve_tier`'s unknown-runtime fallback to `claude`, the precedent D3 follows.
