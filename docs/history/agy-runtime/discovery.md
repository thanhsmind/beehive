# agy-runtime — discovery

Live probe, 2026-11-25, Antigravity CLI `agy` 1.5.17, headless
(`agy -p … --dangerously-skip-permissions --model gemini-3.8-flash-medium`)
in a scratch git repo with a workspace `.agents/hooks.json` that logs every
hook stdin line to a file.

## Surfaces agy reads (bundled skill `agy-customizations`, docs/hooks.md)

- Rules: `AGENTS.md` and `GEMINI.md`, walked from the cwd up to the repo root.
- Skills: `.agents/skills/<name>/SKILL.md` (workspace) and `~/.agents/skills`.
- Hooks: one `hooks.json` per customization root, workspace file
  `.agents/hooks.json`. Top-level keys are hook names; each hook name maps to
  events. Hooks with different names on one event are merged and run in order.
- Events: `SessionStart`, `PreInvocation`, `PostInvocation`, `Stop` (flat
  handler lists) and `PreToolUse`, `PostToolUse` (grouped, `matcher` regex on
  the tool name, `""` or `"*"` matches all).
- Handler: `{"type":"command","command":"…","timeout":30}`. The command runs
  through `sh -c` with its working directory set to the folder that holds
  `hooks.json`.
- Input: camelCase JSON on stdin. Common fields: `conversationId`,
  `workspacePaths`, `transcriptPath`, `modelName`, `lastUserInput`,
  `hookEventName`, `cwd`.

## Observed payloads

| Event | Fields seen |
|-------|-------------|
| SessionStart | common fields only |
| PreInvocation | `invocationNum` (0 for the first model call of a turn), `initialNumSteps` |
| PostInvocation | same as PreInvocation |
| PreToolUse / PostToolUse | `toolCall {id, name, args}`, `stepIdx`; PostToolUse adds `error` on failure |
| Stop | `executionNum`, `terminationReason`, `error`, `fullyIdle`, `continuationActive` |

Tool names and argument keys seen:

| agy tool | args |
|----------|------|
| `read_file` | `Path` |
| `run_command` | `CommandLine`, `WaitMsBeforeAsync`, `toolSummary` |
| `write_file` | `Path`, `Content` |
| `edit_file` | `Path`, `Target`, `Replacement` |
| `invoke_subagent` | `Prompt`, `Role`, `TypeName` (`self`, `research`, …), `Model` (`inherit`) |
| `send_message` | `Message`, `Recipient` |
| `search_web` | `query` |
| `call_mcp_tool` | `ServerName`, `ToolName`, `Arguments` |
| (plan tool) | `steps[] {status, title}` |

A subagent runs the workspace hooks too: it fires its own `SessionStart`,
`PreToolUse` and `Stop` with its own `conversationId`.

## Verdict behavior

- A PreToolUse hook that prints `{}` DENIES every tool ("tool call denied by
  pre-tool hook"). `decision` is required.
- `{"decision":"ask"}` keeps agy's normal approval path: in the headless
  skip-permissions run the command ran and wrote its file.
- `{"decision":"allow"}` auto-approves; `"deny"` hard-blocks; `overwrite`
  merges top-level argument keys before the call runs.
- Stop: `{"decision":"continue","reason":"…"}` re-enters the loop;
  `continuationActive` is true while a Stop hook already kept the turn going.
- SessionStart / PreInvocation inject context with
  `{"injectSteps":[{"systemMessage":{"systemMessage":"…"}}]}`.

## Environment inside agy commands

`ANTIGRAVITY_CONVERSATION_ID`, `ANTIGRAVITY_AGENT`, `ANTIGRAVITY_PROJECT_ID`,
`ANTIGRAVITY_TRAJECTORY_ID` are set. An agy started from a Claude Code session
also inherits `CLAUDE_CODE_SESSION_ID`, so the agy variable must win.
