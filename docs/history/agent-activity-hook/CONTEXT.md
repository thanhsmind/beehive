# Context: agent-activity-hook

## Problem

A dashboard (waggledance) reads `.bee/` from every registered project and wants to show,
per agent session, whether the agent is **working**, **waiting for the human** (a question),
**blocked** (a permission prompt), **idle**, or **gone**. Today bee's session record carries
only a heartbeat, and the agent sets `waiting_on` by hand. herdr derives status from a screen
regex and cannot split "waiting for input" from "blocked"; agent-orchestrator derives it from
harness hooks with sticky needs-input states and a 90 s `no_signal` rule.

Research brief (outside this repo):
`/home/thanhsmind/projects/goglbe/waggledance/docs/history/research/agent-status-herdr-vs-agent-orchestrator.md`.

The user's direction (2026-08-22, relayed by the waggledance session): bee installs the
Claude Code hook set and records agent activity in the bee store; waggledance renders from it.

## Locked decisions

Logged in `.bee/decisions.jsonl`; ids are the authority.

- **D1** (`b17bfa89`) — one Rust verb, `bee hook activity`, stdin payload, switch on
  `hook_event_name` for UserPromptSubmit, PreToolUse, PostToolUse, PostToolUseFailure,
  PermissionRequest, Stop, Notification, SessionEnd. Never SubagentStop. Always exit 0;
  failures to a capped `.bee/logs/hook-activity.log`. Declared in the hook-manifest source
  so `bee dev render-hook-manifests` writes it into `.claude/settings.json`; foreign hooks in
  `settings.local.json` untouched.
- **D2** (`2f782f51`) — `.bee/sessions/<session_id>.json` gains
  `activity: {state, event, tool_name?, tool_use_id?, at, pane?, cwd}`;
  `.bee/sessions/<session_id>.activity.jsonl` holds the last 50 transitions (atomic trim).
  A missing session file is created minimally by the hook.
- **D3** (`40c707ba`) — event→state mapping and the sticky rule: `waiting_input`/`blocked`
  never age; `blocked` clears only on the PostToolUse(/Failure) with the same `tool_use_id`
  (or same `tool_name` when no id) or a turn boundary; `waiting_input` clears on a turn
  boundary only.
- **D4** (`2d4e3900`) — `signal: live|no_signal` is computed at read time (90 s) in
  `bee state session list --json` and `bee status --json`; never stored.
- **D5** (`b4f21f29`) — `waiting_input`/`blocked` set the record's `waiting_on` mark
  (question/gate); a turn boundary clears a hook-set mark; an agent-set mark is never
  overwritten by the hook.

## Out of scope

- waggledance's reader side (rendering, Approve button, notifier).
- Codex / opencode hook equivalents — Claude Code only in this slice.
- SubagentStop handling of any kind.
