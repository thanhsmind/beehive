# Lint setup guidance — Context

**Feature slug:** lint-setup-guidance
**Date:** 2026-11-16
**Scope:** Standard
**Domain types:** RUN

## What was asked

The owner reported that the lint check bee finds is only a reminder. In a
host project, an agent told "set it up" answered that it had no permission.
The owner wants bee to give the agent the steps to set it up when the user
asks.

## What was found

The notices tell the agent to "write them to .bee/config.json `commands`"
and "Never write an unconfirmed value", while AGENTS.md tells every agent to
change bee state only through the CLI, never by editing `.bee/*.json`. No
notice names `bee config set`, which writes `commands.test` with exit 0 and
no guard refusal. The bee-hive onboarding reference loads only when
onboarding itself is in question, so a "set up lint" ask after onboarding finds no
procedure. The agent saw a contradiction and stopped.

## Locked Decisions

| ID | Decision | Store id |
|---|---|---|
| D1 | Every notice that asks for standard commands or static checks names the write verb `bee config set --key commands.test --value '<JSON list>'` (and `commands.setup`, `commands.start`). The `commands` key is the agent's to write once the user confirms; only the governed keys (team, herding.agents, herding.agent_command, hooks) are the user's. | d509d105 |
| D2 | A user request to set up lint, type check or test commands is the confirmation for the checks the agent proposes for it: run each check once, report green or the finding count, write the list with `bee config set`, log the answer as a decision. Never reply that permission is missing. | 1bc79e0e |
| D3 | `skills/bee-hive/references/onboarding.md` gains a "Set up the project's checks" procedure, loaded from SKILL.md on a set-up ask. With no linter in the project, the agent proposes the stack's usual linter, adds it as a dev dependency after the user agrees, then records it in `commands.test`. | 72a0f753 |
| D4 | contract:commands-notice-names-config-set — every `commands_notices` notice and the status_full "No standard commands recorded" warning contains the text bee config set --key commands., and none tells the agent to write `.bee/config.json` by hand. | af69659c |

## Read set

### Knowledge

| Concept | Grade at read | Why it matters |
|---------|---------------|----------------|
| docs/knowledge/areas/onboarding/host-project-artifacts.md | suspect | "Static checks are offered as members of `commands.test`" holds the notice behavior this feature changes |

### Code

| Path | What it decides |
|------|-----------------|
| `packages/bee-rs/crates/bee/src/onboard/templates.rs` | the `STATIC_CHECK_*` notice texts |
| `packages/bee-rs/crates/bee/src/onboard/notices.rs` | `commands_notices` and its two "No standard commands recorded" lines |
| `packages/bee-rs/crates/bee/src/verbs/status_full/build.rs` | the status staleness warning for empty `commands` |

## Out of scope

The propose-only rule for values bee detects on its own stays (trust-layers
D1): bee writes nothing without the user.
