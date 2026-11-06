# Claude sub-session dispatch — Context

**Feature slug:** claude-sub-session-dispatch
**Date:** 2026-11-06
**Scope:** Standard (dispatch door, herding transport, doctrine)
**Domain types:** CALL, RUN

## What was asked

The owner read a post about how the Claude Code app runs sub-sessions
(visible background sessions that a main session starts, steers and hears
back from) and asked what bee can learn. Two gaps came out: bee has no way
to start a Claude Code sub-session, and a leader cannot steer a running
Claude worker. The owner then set the rule below and chose option Y.

## Locked Decisions

| ID | Decision | Store id |
|---|---|---|
| D1 | A role configured as a herding pane keeps herding to start the worker; when that worker is a Claude session the leader may message and steer it with SendMessage (A1). A role configured as a native Claude model starts as a Claude Code background sub-session (A2). Codex, Pi and Grok stay herding-only. (owner) | 7fe67d57 |
| D2 | Every dispatch kind — cell, gather, reviewer, advisor — on a native Claude role opens a sub-session, and it returns its result through the same channel herding panes use today (the bee mailbox read by `bee herding run`), so no result is lost. (owner, option Y) | 16fb3feb |

## Facts the plan stands on

- `claude --bg --name <n> --model <m> "<prompt>"` starts a background
  session in the current directory and prints `backgrounded · <id> · <name>`.
- `claude agents --json --all` lists it with `id`, `name`, `cwd`, `state`,
  `status`; `claude stop <id>` stops it.
- Without `--settings '{"worktree":{"bgIsolation":"none"}}'` the session
  moves itself into a new `.claude/worktrees/` checkout, away from the bee
  feature worktree.
- The session shows in `ListAgents` as `<name> · bg` and receives
  `SendMessage`.
- `claude logs` output is raw terminal bytes, so results cannot be read from
  it; the mailbox is the result channel.

## Deviations

- The dispatch helper for code reading returned an undeclared herding agent
  (issue https://github.com/thanhsmind/beehive/issues/164), so the leader
  read the code itself.
