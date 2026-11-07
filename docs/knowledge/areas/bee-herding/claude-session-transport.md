---
type: bee.area
title: "Bee Herding — the claude-session transport, and how a leader steers a Claude worker"
description: "The herding transport that runs a native Claude role as a visible Claude Code background session, the claude --bg argv and bgIsolation none, the Paseo route that shows the session in Paseo, the poll decisions, the mailbox result channel, the steer_to name a leader sends SendMessage to, the dispatch.claude_sessions switch, the PATH fallback and the fallback reasons."
timestamp: 2026-11-07
bee:
  id: bee-herding-claude-session-transport
  lifecycle: active
  areas: [bee-herding]
  required_context: [areas/bee-herding/overview.md]
  decisions: ["claude-sub-session-dispatch D1 (a herding pane keeps herding; a Claude worker may be messaged and steered with SendMessage; a native Claude role starts as a Claude Code background sub-session; Codex, Pi and Grok stay herding-only)", "claude-sub-session-dispatch D2 (every dispatch kind on a native Claude role opens a sub-session and returns its result through the bee mailbox that bee herding run reads)", "claude-sub-session-dispatch D3 (dispatch.claude_sessions false keeps the Agent payload; no claude on PATH falls back to the Agent payload with fallback_reason claude_cli_missing; a session-model escalation keeps the Agent payload)", "paseo-visible-sessions D1 (inside Paseo a --session-model run starts through paseo run; outside Paseo claude --bg does not change)", "paseo-visible-sessions D2 (the Paseo model check accepts a Claude family alias against claude-<alias>-<version>)", "paseo-visible-sessions D3 (every claude-agent fallback of a native Claude model records native_fallback_reason)"]
  sources: [docs/history/claude-sub-session-dispatch/CONTEXT.md, docs/history/claude-sub-session-dispatch/plan.md, docs/history/report-usage-herding/plan.md, docs/history/paseo-visible-sessions/CONTEXT.md]
  authoritative_for: "bee-herding: the claude-session transport, its spawn argv and poll decisions, steer_to, and the dispatch door switch that selects it"
  owns.code: [packages/bee-rs/crates/bee/src/herding/claude_session.rs]
---

# Bee Herding — the claude-session transport, and how a leader steers a Claude worker

## What it is

The claude-session transport is a third herding transport, beside Paseo and the pane transports (claude-sub-session-dispatch D1, D2).
`bee herding run --session-model <model> [--session-effort <effort>]` starts one Claude Code background session. The session shows in `claude agents` and in the leader's `ListAgents`, so the leader can reach it.
`--session-effort` takes `low`, `medium`, `high`, `xhigh` or `max`.

The worker is bee-ignorant, like every herded worker. It reads `brief-1.txt` and writes `ack-1.json`, `report-1.md` and `result-1.json` in its job mailbox. `bee herding run` reads the outcome only from `result-1.json`. The transport never reads `claude logs`, because that output is raw terminal bytes.

The dispatch door picks this transport by itself. On runtime `claude`, a native-model role for kind `cell`, `gather`, `reviewer` or `advisor` returns a `Bash` payload that runs `bee herding run --task-file - --json … --session-model <model>`, with channel `claude-session`. Before this change the door returned an `Agent` payload.

## The spawn

bee spawns `claude` itself, from inside the `bee` process, with the argv in this order:

```text
claude --bg --name <name> --model <model> [--effort <effort>] --permission-mode bypassPermissions --settings <json> <prompt>
```

- The working directory is the `--cwd` of the run.
- The command is `claude` unless `herding.claude_command` names another binary.
- The name is `bee-<cell id, or seat, or run>-<last 12 characters of the job id>`.
- The prompt is the pointer prompt for `brief-1.txt`.
- `--settings` carries `worktree.bgIsolation: "none"` and an `env` object with `BEE_HERDING_WORKER`, `BEE_HERDING_JOB_ID` and, for a read-only run, `BEE_HERDING_READ_ONLY`. The Claude supervisor starts the session, not bee, so the child environment of the spawn call does not reach the worker. The settings `env` object does.
- The id comes from the line `backgrounded · <id> · <name>` on stdout. No such line is a `spawn_failed` outcome that names the output.

Why `bgIsolation: "none"`: without it the session moves itself into a new `.claude/worktrees/` checkout, away from the bee feature worktree.

Why bee spawns it: a `claude` call from a shell in a worktree is refused by the worker-outward guard, because it starts another agent. The guard never judges `bee` itself, so the spawn runs inside bee.

A `--read-only` run sets `BEE_HERDING_READ_ONLY=1` and refuses before spawn when the Claude worker guard is missing, as a Paseo `claude` run does (claude-read-only D1-D5).

## Inside Paseo

(paseo-visible-sessions D1)

A `claude --bg` session shows in `claude agents` only. It does not show in the Paseo app. Inside Paseo, bee starts the sub-session through Paseo, so the owner sees it there.

Inside Paseo means `PASEO_AGENT_ID` or `PASEO_CLI` is set and not empty. In that case a `--session-model` run starts through `paseo run` with these values:

- provider `claude/<model>`
- `--thinking <effort>`, from `--session-effort`
- `--mode bypassPermissions`
- title `bee-<cell id, or seat>-<last 12 characters of the job id>`
- the workspace that the paseo-worktree-workspace D4 resolution picks

The job record has transport `paseo`, with `session_model` and `session_effort` next to it. The Paseo loop applies: bee archives the agent when the run finishes, and `steer_to` is recorded as for any Paseo Claude run.

Outside Paseo, the `claude --bg` path does not change.

The Paseo model check accepts a Claude family alias (`opus`, `sonnet`, `haiku` or `fable`, with an optional `[..]` suffix) against the id Paseo reports, `claude-<alias>-<version>` (paseo-visible-sessions D2).

## The poll decisions

The loop is the Paseo loop with one change in its liveness source: `claude agents --json --all` replaces `paseo inspect`. The loop checks the mailbox on every tick and asks Claude for the listing on a slower cadence.

- A `result-1.json` file ends the wait with the result.
- A row with state `failed` or `stopped`, or no row for the session id, is `Gone`. The run ends as `died`. bee does not stop a session that is already gone.
- A listing that cannot be read is `Unknown`. It never counts toward `died`.
- Two idle listings in a row with no result end the run as a silent idle timeout. A heartbeat is fresh while the listing says `working` or `log.txt` or `ack-1.json` change. A stale heartbeat past `--idle-timeout` ends the run the same way.
- The ceiling ends the run as `timed_out_ceiling`.
- A control mark ends the run as `interrupted` or `cancelled`.

After every outcome except `died`, bee runs `claude stop <id>`, so a finished, idle-timed-out or ceiling-hit session is never left running.

## Steering: `steer_to`

The job record and the run JSON carry `session_id` and `steer_to` (claude-sub-session-dispatch D1). `bee herding status --job <id>` prints `steer_to=<name>`. A leader sends `SendMessage` to that name to steer the worker.

Steering is not a transcript read. The rule that a leader never reads a worker's transcript, pane log or activity stream stays in force.

`steer_to` also covers a Claude worker that runs in a Paseo pane. After a Paseo `claude` spawn, bee lists the Claude sessions and records `steer_to` only when exactly one session in the run's `--cwd` started after the spawn. With none or several, bee omits `steer_to`.

## Worker tokens in the completion report

A claude-session worker writes its own transcript, outside the leader's transcript and its `subagents/` directory. `bee report usage` reads it through the job mailbox: every `.bee/mailbox/<job>/` under the control root (a worktree reads the main checkout's mailbox) whose `job.json` `leader_session` equals the leader transcript's file stem, and whose `activity.json` `transcript_path` exists, adds that transcript and its `subagents/*.jsonl` to the same per-model rows, inside the same window.

- Each transcript path counts once. A job of another leader, a job with no `leader_session` and a job with a missing transcript are skipped without error.
- `bee report usage` prints the number of herded sessions it counted (`Herded Sessions` in text, `herding_sessions` in `--json`).
- This is a usage read of the worker's own transcript. The rule that a leader never reads a worker's transcript to steer it stays in force.

## The dispatch door switch and fallbacks

(claude-sub-session-dispatch D3)

- `dispatch.claude_sessions: false` in the main checkout's `.bee/config.json` keeps the old `Agent` payload, byte for byte. Absent or any other value means sessions are on.
- When `claude` (or the binary `herding.claude_command` names) does not resolve on `PATH`, the door keeps the `Agent` payload and adds `fallback_reason: "claude_cli_missing"`.
- A session-model escalation keeps the `Agent` payload. The escalated worker runs on the session's own model.
- Every `claude-agent` fallback of a native Claude model records `native_fallback_reason` in `.bee/logs/dispatch.jsonl`: `escalated`, `claude_sessions_off` or `claude_cli_missing`. An escalation lands on channel `session-model` (paseo-visible-sessions D3).
- A role configured as a herding pane stays herding. Codex and Pi payloads do not change.

## Known limits

- Only local sessions: `claude --bg`, or `paseo run` inside Paseo. Cloud and cross-machine sessions are out of scope.
- The result channel is the mailbox only. A worker that never writes `result-1.json` ends as a silent idle timeout.

## Pointers

- Transport helpers, argv builders, id and listing parsers, poll watch: `packages/bee-rs/crates/bee/src/herding/claude_session.rs`.
- The run path and its poll loop (`execute_claude_session`, `wait_for_round_claude`): `packages/bee-rs/crates/bee/src/herding/run.rs`.
- The status line that prints `steer_to`: `packages/bee-rs/crates/bee/src/herding.rs`.
- The herded-session read in the completion report (`herded_transcripts`): `packages/bee-rs/crates/bee/src/verbs/report_usage.rs`.
- The dispatch door arm, the switch, the PATH fallback and the command builder (`claude_session_command`): `packages/bee-rs/crates/bee/src/verbs/drivers/prepare.rs`.
- Paseo loop this transport copies: `docs/knowledge/areas/bee-herding/the-paseo-channel.md`.
- Feature context and plan: `docs/history/claude-sub-session-dispatch/CONTEXT.md`, `docs/history/claude-sub-session-dispatch/plan.md`.
- The Paseo route: `docs/history/paseo-visible-sessions/CONTEXT.md`; code in `packages/bee-rs/crates/bee/src/herding/run.rs` (`execute_paseo_steered`).
