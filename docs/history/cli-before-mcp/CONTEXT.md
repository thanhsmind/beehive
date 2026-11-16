# CLI before MCP for Skillstation and mcp-gateway — Context

**Feature slug:** cli-before-mcp
**Date:** 2026-11-16
**Scope:** Standard
**Domain types:** RUN

## What was asked

The owner said: Skillstation and mcp-gateway both have a CLI, so beehive
must prefer the CLI over the MCP tools.

## What was found

- mcp-gateway serves `tools`, `describe` and `call` from the terminal with
  the same code as its MCP tools `search_tools`, `get_tool_definition` and
  `call_tool`.
- skillstation 0.14.1 serves `skill show <id> --json`, which returns the
  SKILL.md text (`content`) and the manifest. It has no verb for feedback,
  and `resolve` is a developer command that needs a request file.
- The bee cross-project guard refuses a read inside the hub folder. The MCP
  tool `skill_get` copies a skill into the runtime cache, which the session
  may read. A skill that sends the agent to its own relative files needs
  `skill_get`.
- Three bee render sites tell an agent how to load a hub skill: the alias
  SKILL.md, the hub-mode principle header of `bee orient`, and orient's
  playbook pointer. All three name only `skill_get`.

## Locked Decisions

| ID | Decision | Store id |
|---|---|---|
| D1 | Rule `agents-cli-before-mcp` in `packages/bee/LEADER.md` is the one home of the CLI-first mapping for mcp-gateway and Skillstation. A tool that ships a CLI and MCP tools is reached through its CLI first; the MCP tool is the fallback. waggledance keeps its own rule `agents-other-project-to-its-leader`. | bf107c86 |
| D2 | mcp-gateway: `mcp-gateway tools <keywords>` (MCP `search_tools`), `mcp-gateway describe <upstream.tool>` (`get_tool_definition`), `mcp-gateway call [--text] <upstream.tool> '<json>'` (`call_tool`). Fallback when the binary is not on PATH, the subcommand is unknown (exit 2), or a guard or permission refuses the shell call. After `call` exits 1 with `outcome_unknown`, never repeat the call through `call_tool`; read the system state first. A tool that is not read-only still needs the user's yes. | 27015a53 |
| D3 | Skillstation: `skillstation skill show <id> --json` loads a hub skill; follow its `content` field, only when `manifest.status` is `active`. Call `skill_get` instead when the content sends the agent to a relative file, when the status is not active, when the hub is on another machine, or when the CLI is missing or fails. `skill_resolve` and `skill_feedback` stay on MCP. Named guess: `skill show` records no use count; accepted. | 17412392 |
| D4 | The alias SKILL.md and the hub-mode principle header name `skillstation skill show <id> --json` first and `skill_get` as the fallback. Orient's playbook pointer keeps `skill_get`, because a playbook is a relative file. Rendered Claude agent files keep `mcp__skillstation__skill_get`: the reader roles have no shell, and it is the fallback. | cede1997 |
| D5 | contract:cli-first-hub-texts — the alias render and the hub-mode principle header each contain `skillstation skill show` and `skill_get`, with `skillstation skill show` first; local mode stays byte-identical. | 069eed40 |
| D6 | The mcp-gateway agent skill text belongs to the mcp-gateway project; it went to that project's leader through waggledance (run `run-659b9e7c8d7897c1`). The alias files in `.claude/skills` are re-rendered from main after the merge. | 73715451 |

## Read set

### Knowledge

| Concept | Grade at read | Why it matters |
|---------|---------------|----------------|
| docs/knowledge/areas/onboarding/skillstation-hub-skills-and-aliases.md | suspect | "Write an alias" and the `skillstation.principles` rule describe the texts this feature changes |
| docs/knowledge/areas/doctrine-layer/router-triage-and-the-agents-md-duplication-boundary.md | suspect | The rule index takes one row for the new rule |

### Code

| Path | What it decides |
|------|-----------------|
| `packages/bee-rs/crates/bee/src/verbs/skillstation_group.rs` | `render_alias`, the alias SKILL.md text |
| `packages/bee-rs/crates/bee/src/principles.rs` | the hub-mode principle header line |
| `packages/bee-rs/crates/bee/src/verbs/status_full/orient.rs` | `playbook_ref`, kept on `skill_get` |
| `packages/bee/LEADER.md` | the rule home |

## Out of scope

- A change to the skillstation or mcp-gateway binaries.
- Opening the hub folder to reads through `guards.cross_project_allow`.
