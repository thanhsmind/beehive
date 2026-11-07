# Paseo-visible sub-sessions — Context

**Feature slug:** paseo-visible-sessions
**Date:** 2026-11-07
**Scope:** Small (herding run routing, dispatch log field)
**Domain types:** RUN, CALL

## What was asked

Since v2.80.1 every native-model dispatch on runtime claude runs
`bee herding run --session-model <m>`, which starts `claude --bg`. A
`claude --bg` session shows only in `claude agents`, never in the Paseo
UI. The owner works in Paseo, sees no review or hat session, and thinks
the work is already finished. The owner chose to start the sub-session
through Paseo when bee runs inside Paseo, over a config-only reroute to
Paseo herding agents.

## Locked Decisions

| ID | Decision | Store id |
|---|---|---|
| D1 | Inside Paseo (`PASEO_AGENT_ID` or `PASEO_CLI` set and not empty, the paseo-worktree-workspace D1 rule), a `--session-model` run starts through the existing `paseo run` path with provider claude, the session model, the session effort as `--thinking`, mode `bypassPermissions`, and the Paseo title `bee-<cell or seat>-<job>`. The workspace comes from paseo-worktree-workspace D4. Outside Paseo the `claude --bg` path does not change. (owner) | 360fb6b0 |
| D2 | The Paseo model check accepts a Claude family alias (`opus`, `sonnet`, `haiku`, `fable`, with an optional `[..]` suffix) against an observed `claude-<alias>-<version>` id. (agent) | b55f955f |
| D3 | A claude-agent fallback of a native Claude model records `native_fallback_reason` in `.bee/logs/dispatch.jsonl` for every cause: `escalated`, `claude_sessions_off`, `claude_cli_missing`. The stale roster of paseo-worktree-workspace goes to the backlog, not to this feature. (agent) | dba6cc91 |

## Facts the plan stands on

- `paseo run --provider claude/haiku --thinking low --mode bypassPermissions`
  (paseo 0.13.2, 2026-11-07) ran model `claude-haiku-5-5`, thinking `low`:
  Paseo accepts the alias and reports the full id.
- The Claude thinking ids in Paseo are `low`, `medium`, `high`, `xhigh`,
  `max`, the same set as `--session-effort`.
- `execute_paseo_steered` already writes the brief, resolves the workspace,
  guards read-only Claude runs, records `steer_to`, honors the ceiling and
  archives the agent on finish.
- Four reviewer dispatches at 2026-11-07T01:25Z returned `claude-agent`
  with no `native_fallback_reason`; only `claude_cli_missing` was logged.
