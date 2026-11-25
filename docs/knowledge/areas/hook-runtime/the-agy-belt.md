---
type: bee.area
title: "Hook Runtime — the agy belt, its event translation, and the tool map"
description: "How Antigravity CLI (agy 1.5.17+) integrates with bee through .agents/hooks.json and bee hook agy-belt: the five events, payload translation, tool map, ask/deny/continue verdicts, invoke_subagent denial, ANTIGRAVITY_CONVERSATION_ID, and onboarding merge."
timestamp: 2026-11-25
bee:
  id: hook-runtime-the-agy-belt
  lifecycle: active
  areas: [hook-runtime]
  required_context: [areas/hook-runtime/overview.md, areas/hook-runtime/catalog-projections-and-activation.md]
  decisions: [agy-runtime D1-D8 (fa33ba78-d185-4266-8202-1bc155607ec6)]
  sources: [docs/history/agy-runtime/CONTEXT.md, docs/history/agy-runtime/discovery.md]
  authoritative_for: "hook-runtime: the agy belt translation, tool map, and onboarding merge"
  anchors: [packages/bee-rs/crates/bee/src/devtools/hook_manifests.rs, packages/bee-rs/crates/bee/src/hooks/adapter.rs, packages/bee-rs/crates/bee/src/hooks/mod.rs, packages/bee-rs/crates/bee/src/session_identity.rs, packages/bee-rs/crates/bee/src/onboard/templates.rs, packages/bee-rs/crates/bee/src/onboard/apply.rs, packages/bee-rs/crates/bee/src/hooks/agy_belt.rs, .agents/hooks.json, packages/bee-rs/crates/bee/src/onboard/agy_belt.rs]
  verified_at: 2026-11-25
---

# Hook Runtime — The agy Belt, Its Event Translation, and the Tool Map

Read when: You need to understand how `agy` (Antigravity CLI 1.5.17+) is wired into bee, how `bee hook agy-belt` translates payloads and tool calls, or how onboarding merges `.agents/hooks.json`.

`agy` is the fifth bee runtime (floor: Antigravity CLI `1.5.17`). It takes the same named exclusion from the hook catalog that OpenCode and Pi take: no `Runtime::Agy` enum variant and no generated projection file. Its belt is `.agents/hooks.json` plus `bee hook agy-belt`.

## What agy Reads

`agy` reads three workspace surfaces natively:

- **Rules:** `AGENTS.md` (and `GEMINI.md`), walked from `cwd` up to the repository root.
- **Skills:** `.agents/skills/<name>/SKILL.md` (workspace) and `~/.agents/skills`. `agy` shares the `.agents/skills/` root owned by the Codex render, which keeps `<!-- bee:only agy -->` blocks alongside `<!-- bee:only pi -->` blocks. No separate skill tree exists for `agy`.
- **Hooks:** `.agents/hooks.json`. Top-level keys are hook names, and each hook name maps to lifecycle events. Hooks with different names on the same event are merged and run in order.

## The `bee` Key and Its Five Events

In `.agents/hooks.json`, bee owns one top-level hook key, `"bee"`, with five events:

- `SessionStart` (flat handler list)
- `PreInvocation` (flat handler list)
- `PreToolUse` (grouped with `matcher: ""`)
- `PostToolUse` (grouped with `matcher: ""`)
- `Stop` (flat handler list)

Every handler runs `bee hook agy-belt` through `sh -c` with `cwd` set to the `.agents/` folder that holds `hooks.json`. The command resolves `../.bee/bin/bee` first, then falls back to `$g/../.bee/bin/bee` where `$g` is the absolute git common directory, so it finds the main checkout's binary from a worktree. If no `bee` binary is executable, the fallback prints the neutral answer for that event (`{"decision":"ask"}` on `PreToolUse`, `{}` on all other events) so a missing binary never blocks every tool.

## Event Translation (`bee hook agy-belt`)

`bee hook agy-belt` reads `agy`'s camelCase JSON payload on stdin, translates it into the payload shape the existing Claude Code hooks expect (`hook_event_name`, `session_id` = `conversationId`, `cwd`, `transcript_path`, `bee_runtime: "agy"`), runs the matching checks from the hook catalog, and writes one JSON object to stdout with exit code `0`.

| agy Event | Translated Event & Extra Fields | Checks Run | agy Answer |
|---|---|---|---|
| `SessionStart` | `SessionStart` | `session-init` | `{"injectSteps":[{"systemMessage":{"systemMessage": text}}]}` or `{}` when empty |
| `PreInvocation` (`invocationNum == 0`) | `UserPromptSubmit` (`prompt` = `lastUserInput`) | `prompt-context` (and activity) | `{"injectSteps":[{"systemMessage":{"systemMessage": text}}]}` or `{}` when empty |
| `PreInvocation` (`invocationNum > 0`) | — | None | `{}` |
| `PreToolUse` | `PreToolUse` (D4 `tool_name`, `tool_input`) | Catalog `PreToolUse` checks | `{"decision":"deny","reason": r}`, `{"decision":"ask","overwrite": {...}}`, or `{"decision":"ask"}` |
| `PostToolUse` | `PostToolUse` (D4 `tool_name`, `tool_input`) | Catalog `PostToolUse` checks | `{}` |
| `Stop` | `Stop` (`stop_hook_active` = `continuationActive`) | `session-close` (and catalog `Stop` checks) | `{"decision":"continue","reason": r}` on block, else `{}` |

## Verdict Answers and Why `{}` Is Never Printed on `PreToolUse`

On `agy` 1.5.17, a `PreToolUse` hook that prints `{}` causes `agy` to **deny every tool call** (`tool call denied by pre-tool hook`). The `decision` field is required on `PreToolUse`:

- **Allow / normal approval (`ask`):** `{"decision":"ask"}` keeps `agy`'s own approval path. When a check repairs tool arguments (`updatedInput`), `agy-belt` maps the repaired fields back to `agy` argument keys (`command` -> `CommandLine`, `file_path` -> `Path`, `content` -> `Content`, `old_string` -> `Target`, `new_string` -> `Replacement`) and emits `{"decision":"ask","overwrite":{...}}`.
- **Deny (`deny`):** `{"decision":"deny","reason": r}` blocks the tool call on the first denial. A crash, spawn error, or unreadable payload on `PreToolUse` fails closed and prints `{"decision":"deny","reason": ...}`.
- **Turn continuation (`continue`):** On `Stop`, a blocking `session-close` result emits `{"decision":"continue","reason": r}`; otherwise `{}`.
- **Advisory failure policy:** `SessionStart`, `PreInvocation`, `PostToolUse`, and `Stop` fail open on a crash or spawn error of a child hook, printing the neutral `{}` answer and exiting `0`. An unreadable payload fails closed on every event with `{"decision":"deny","reason":"agy-belt: unreadable hook payload: ..."}` because `handle_payload` parses stdin before the event name is known.

## The D4 Tool Map

On `PreToolUse` and `PostToolUse`, `agy-belt` maps `toolCall.name` and `toolCall.args` as follows:

| agy Tool | agy Args | Mapped Hook Tool & Input | Behavior |
|---|---|---|---|
| `run_command` | `CommandLine` | `Bash` `{command}` | Checked by `PreToolUse` / `PostToolUse` |
| `write_file` | `Path`, `Content` | `Write` `{file_path, content}` | Checked by `PreToolUse` / `PostToolUse` |
| `edit_file` | `Path`, `Target`, `Replacement` | `Edit` `{file_path, old_string, new_string}` | Checked by `PreToolUse` / `PostToolUse` |
| `read_file` | `Path` | `Read` `{file_path}` | Checked by `PreToolUse` / `PostToolUse` |
| `invoke_subagent` | `Prompt`, `Role`, `TypeName`, `Model` | — | Denied on `PreToolUse` (D5) |
| Named no-write tools (`search_web`, `send_message`, `call_mcp_tool`, `read_url_content`, `list_dir`, `view_file`, `list_directory`, plan tool, `browser_*`, `capture_browser*`, `read_browser*`) | — | — | Answers `{"decision":"ask"}` with no check |
| Any other tool carrying `Path`, `AbsolutePath`, `TargetFile`, or `File` | path field | `Write` `{file_path, content: ""}` | Checked as a `Write` on that path |
| Any other unknown tool | — | — | Answers `{"decision":"ask"}` and logs a coverage-gap row |

## `invoke_subagent` Denial and Herding-Only Dispatch

Dispatch on `agy` is herding-only, matching the Pi rule. `bee dispatch prepare --runtime agy` reads `team.agy`, and any native, model, cli, or budget slot refuses with the herding remedy. Because a subagent spawned via `agy`'s `invoke_subagent` would bypass the herding door, `agy-belt` denies `invoke_subagent` on `PreToolUse` with a reason naming `bee dispatch prepare --runtime agy`.

## Session Identity (`ANTIGRAVITY_CONVERSATION_ID`)

Inside `agy` commands, `ANTIGRAVITY_CONVERSATION_ID` identifies the session. `session_identity` checks `ANTIGRAVITY_CONVERSATION_ID` before Pi, Codex, and Claude environment variables because an `agy` process started from a Claude Code session inherits `CLAUDE_CODE_SESSION_ID`.

## Onboarding Merge Rules and Workspace Trust

`bee onboard --apply` writes or merges `.agents/hooks.json`:

- If `.agents/hooks.json` does not exist, onboarding writes `{"bee": ...}`.
- If `.agents/hooks.json` is a JSON object, onboarding replaces only the `"bee"` key and keeps every other key's value equal as JSON and its key order (the file is parsed and re-printed, so formatting may change).
- Before the first rewrite of a pre-existing file, onboarding backs it up to `.agents/hooks.json.bee-backup`.
- If the file is not a JSON object or cannot be parsed, onboarding refuses with a typed error and writes nothing.
- A second `bee onboard --apply` run changes nothing.
- The onboarding template gives `team.agy` the same herding roles as `team.pi`, and `bee doctor` includes an `agy` wiring row that names `bee onboard --apply` as the fix.
- Granting workspace trust in `agy` remains the owner's one-time manual step.
