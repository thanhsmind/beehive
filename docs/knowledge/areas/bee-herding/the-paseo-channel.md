---
type: bee.area
title: "Bee Herding — the Paseo channel, worker execution, and lifecycle"
description: "The herding transport that executes bee workers as Paseo agents, polls the file mailbox, tracks daemon liveness states, and archives completed agents."
timestamp: 2026-10-06
bee:
  id: bee-herding-the-paseo-channel
  lifecycle: active
  areas: [bee-herding]
  required_context: [areas/bee-herding/overview.md]
  decisions: ["paseo-pi D1 03c795c7 (Paseo is a new herding channel beside herdr, chosen per team config; herdr panes keep working unchanged)", "paseo-pi D4 9d884211 (each worker on the Paseo channel runs the provider and model that bee team config binds to its role; Paseo only carries the worker)", "paseo-pi D5 b6dd8b33 (on the Paseo channel, an answer goes into the running worker by steering only when the worker's provider can steer; otherwise stop and next round; never interrupt-and-replace)", "paseo-pi D6 83f5caad (the Pi leader on Paseo wakes from a Paseo heartbeat; extension runs the code tick and turns heartbeat into news prompt or short turn; never swallows it)", "paseo-pi D8 2aa8417a (the leader turns its heartbeat on by itself at session start inside Paseo; never a second heartbeat for the same agent)", "paseo-answers D1, D2", "paseo-observe D1, D2", "paseo-pi-hardening D1 9a502af1", "paseo-pi-hardening D2 a8d28361", "paseo-pi-hardening D3 c2a5af66", "paseo-pi-hardening D4 4cb9c644", "paseo-pi-hardening D6 976ef5a5", "paseo-pi-hardening D7 fb38df99", "paseo-pi-hardening D8 f26723a1", "paseo-pi-hardening D10 0a0ef8e7"]
  sources: [docs/history/paseo-pi/CONTEXT.md, docs/history/paseo-pi/plan.md, docs/history/paseo-pi-hardening/CONTEXT.md, docs/history/paseo-pi-hardening/plan.md]
  authoritative_for: "bee-herding: the Paseo channel, paseo agent block configuration, worker execution, and lifecycle management"
  owns.code: [packages/bee-rs/crates/bee/src/herding/paseo.rs, packages/bee-rs/crates/bee/src/herding/run.rs, packages/bee-rs/crates/bee/src/herding.rs, packages/bee-rs/crates/bee/src/doctor.rs]
---

# Bee Herding — the Paseo channel, worker execution, and lifecycle

## What the Paseo channel is

The Paseo channel is a herding transport (paseo-pi D1, store `03c795c7`). It operates beside the herdr and tmux transports. Herdr panes continue to work without changes.

The channel starts a worker as a Paseo agent instead of a terminal pane. Each worker uses the provider and model that team configuration assigns to its role (paseo-pi D4, store `9d884211`). Paseo only carries the worker. The dispatch door remains the single component that selects the model.

## Agent configuration and command

You configure a Paseo agent under the `herding.agents.<name>.paseo` object in team configuration.

The `paseo` block contains four fields:
- `provider`: A required string that names the model provider (for example, `pi`).
- `model`: An optional string that names the model.
- `thinking`: An optional string that sets the reasoning effort level.
- `mode`: An optional string for provider modes.

The key `herding.paseo.command` sets the CLI executable name. The default command is `paseo`.
The key `herding.paseo.broker_tick_secs` sets the broker timer interval in seconds (paseo-pi-hardening D8, store `f26723a1`). The default value is 30 seconds.
The key `herding.paseo.heartbeat_cron` sets the heartbeat cron schedule. The default schedule is `*/30 * * * *` (paseo-pi-hardening D8, store `f26723a1`, superseding paseo-pi D7, store `8ae5134d`).

Here is an example configuration:

```json
{
  "herding": {
    "paseo": {
      "command": "paseo"
    },
    "agents": {
      "pi-worker": {
        "paseo": {
          "provider": "pi",
          "model": "openrouter/~deepseek/deepseek-flash-latest",
          "thinking": "high",
          "mode": null
        }
      }
    }
  }
}
```

## How a run executes

The command `bee herding run` executes a worker through the Paseo CLI.

The runner executes `paseo run -d --json` with the following arguments:
- `--provider <provider>[/<model>]`
- `--cwd <directory>`
- `--title <job_id>`
- `--label bee_job=<job_id>`
- `--env` flags for child environment variables, including `BEE_HERDING_WORKER=1` and `BEE_HERDING_JOB_ID`
- The initial prompt string.

Every run applies the label `bee_job=<job_id>`. This label allows cleanup tools to identify orphan agents.

The runner reads `agentId` from the process output. It writes `paseo_agent_id` and `transport: "paseo"` into `job.json` immediately.

The worker communicates through the standard file mailbox. It reads `brief-N.txt` and writes `ack-N.json`, `report-N.md`, and `result-N.json`. The runner reads the final outcome only from `result-N.json`.

### Timeouts and inspect cadence

Every Paseo CLI call in `bee herding run` and `bee herding run --continue` carries a 15-second timeout (paseo-pi-hardening D1, store `9a502af1`).
The wait loop calls `paseo inspect` at most once every 3 seconds.
Mailbox file checks continue every 200 ms.
Ticks without an inspect call report no liveness to keep the debounce check accurate.

If the spawn command fails or times out, the runner queries `paseo ls` with the job label `bee_job=<job_id>`.
If an agent exists under that label, the runner names the agent ID in the `SpawnFailed` message.
The runner does not adopt or archive the agent.

### Silent-idle nudge and second-idle termination

Cheap models can finish a turn without writing `result-N.json`.
After a round observes a worker in the `Working` state, an `Idle` inspect with no result file triggers one nudge (paseo-pi-hardening D2, store `a8d28361`).
The runner sends a `paseo send` message naming the brief file and the owed result file.
The runner never sends a nudge while the worker state is `Working` or `Blocked`.
A failed send call counts as sent.

A second `Idle` inspection read at least 3 seconds after the nudge ends the round as an idle timeout.
The runner reports the silent-idle message and keeps the agent intact for inspection.

### Observed model verification

After agent spawn, the runner reads inspect `Model` and `Thinking` fields (paseo-pi-hardening D4, store `4cb9c644`).
The runner writes `paseo_model_observed` and `paseo_thinking_observed` into `job.json`.
The sentinel values `-` for model and `auto` for thinking record as unverified.
The runner compares the observed model with the configured `paseo` model.
On a confirmed mismatch, the runner prints a warning and records `paseo_model_mismatch: true` in `job.json`.
The runner never archives, stops, or retries a worker on a model mismatch.

## Liveness states

The runner checks agent health with the command `paseo inspect <agent-id> --json`.

The runner parses the `Status` field and the `PendingPermissions` array:
- `Status: "running"` maps to `Working`. If `PendingPermissions` is not empty, it maps to `Blocked`.
- `Status: "idle"` maps to `Idle`.
- `Status: "error"` and `Status: "closed"` map to `Dead`.
- Any other value or an invalid JSON response maps to `Unknown`.

`Unknown` never counts as alive. When inspection fails, the runner treats the state as unavailable and fails closed.

The inspect `UpdatedAt` timestamp is observe-only (paseo-pi-hardening D3, store `c2a5af66`).
The run loop keeps a `Working` agent fresh.
The command `bee herding status` shows status `stalled` for a `Working` Paseo agent whose `UpdatedAt` is older than 120 seconds.
When `UpdatedAt` changes, status transitions through `recovered` before returning to `working`.
A missing or unparseable `UpdatedAt` never marks an agent as `stalled`.

The `status` field is bee's freshness view. The `paseo_state` field is the daemon's raw state.
Therefore, status `stalled` with `paseo_state: "working"` is expected and normal when an agent runs a long tool call.

## Agent archival and the parent cascade

When a run completes, the runner evaluates whether to archive the Paseo agent.

The runner executes `paseo archive --force <agent-id>` only when `should_close_pane` returns true. This condition requires a valid result or an explicit close configuration.

If the worker fails, crashes, or times out, the runner does not archive the agent. It keeps the agent intact and reports the agent ID for diagnosis.

The runner enforces an own-ID guard. It never archives an agent whose ID matches `PASEO_AGENT_ID` from the environment. This guard prevents a leader agent from deleting itself.

When a leader spawns a worker, `paseo run` records the leader agent ID in `ParentAgentId` (paseo-pi-hardening D10, store `0a0ef8e7`).
Archiving the leader agent archives every active child worker agent automatically.

## Diagnostic FIX messages and setup verification

The Paseo channel supplies two diagnostic messages with `FIX:` guidance.

When the installed version of Paseo is older than version 0.10.3, the runner refuses to spawn the agent:
```text
FIX: upgrade paseo to 0.10.3 or newer
```

When `paseo run` fails because the daemon is down, the runner refuses with:
```text
FIX: start the daemon with paseo daemon start (npm @getpaseo/cli 0.10.3+)
```

On the `pi` runtime, `bee doctor` adds a report-only `paseo_ready` row when any `team.pi` slot configures a Paseo agent (paseo-pi-hardening D7, store `fb38df99`, contract store `28ffec0f`).
The check verifies five conditions:
1. The Paseo daemon answers probes.
2. The CLI version is at least 0.10.3.
3. The file `~/.pi/agent/auth.json` exists when a configured provider is `pi`.
4. Every heartbeat marker in `.bee/runtime/paseo-heartbeat/` names an agent present in `paseo ls` and not archived.
5. No marker file contains a `delete_failed` record.

The doctor row reports findings only and changes no state.

## Environment facts

Three environment facts apply from CONTEXT.md:

First, Paseo 0.10.3 is the minimum required version. It supports steering for the Pi provider. Older versions do not support steering.

Second, the npm package `@getpaseo/cli` 0.10.3 can start the daemon. The AppImage CLI cannot start the daemon, and its `paseo run --json` prints no JSON object, so `bee herding run` fails with `spawn_failed` ("could not parse agent id") while `bee doctor` still reports `paseo_ready` ok (live run 2026-10-06; backlog finding filed). Set `herding.paseo.command` to the npm CLI path when a desktop AppImage wrapper comes first on `PATH`.

Third, the CLI command `paseo send` has no steer flag. Steering requires the daemon WebSocket interface. Extensions in `.pi/extensions/` load inside a Paseo Pi agent and receive `PASEO_AGENT_ID`.

## The leader heartbeat and broker timer

An agent is a leader when `PASEO_AGENT_ID` is set and `BEE_HERDING_WORKER` is unset.

The Pi leader extension runs `bee herding broker tick --json` on its own timer (paseo-pi-hardening D8, store `f26723a1`, contract store `a7326910`).
The timer runs every `herding.paseo.broker_tick_secs` seconds (default 30).
The extension sends a message to the model through `pi.sendUserMessage` only when the tick reports news.
If the leader is busy, the message delivers as a steer.
The timer shares one in-flight execution flag with the heartbeat tick.
The timer and the result drain send through one shared helper, so the busy and steer rule lives in one place.
While an idle send is still opening its turn (the turn-start latch), the timer holds the news and sends it on the first tick after the latch clears; a newer news tick replaces the held text. The shared helper never sends a plain message while that latch is set, so a failed send cannot clear a latch the drain set. Held news is dropped when the timer stops at shutdown.

The Paseo heartbeat remains as a backstop.
The default schedule is `*/30 * * * *` (superseding paseo-pi D7, store `8ae5134d`).
At session start, the extension creates one `bee-leader` heartbeat for the agent (paseo-pi D8, store `2aa8417a`).
The extension records the heartbeat in a marker file at `.bee/runtime/paseo-heartbeat/<agent id>.json`.
If the marker file contains a cron schedule that differs from configuration, `ensureHeartbeat` deletes the old heartbeat and creates a new one. When that delete fails, the marker keeps the old schedule and records `delete_failed`, and no new heartbeat is created.

The extension never swallows a heartbeat prompt (paseo-pi D6, store `83f5caad`). A swallowed prompt stalls Paseo because Paseo keeps the agent status as running.

Worker results do not wait for the heartbeat. The 2-second result drain processes worker results independently.

On `session_shutdown` (for all reasons except `reload`), the extension stops the broker timer (paseo-pi-hardening D6, store `976ef5a5`).
The extension awaits `paseo heartbeat delete <id>`.
On success, the extension removes the marker file.
On failure, the extension keeps the marker file and writes `delete_failed: <reason>` into it.

## Answers and steering

A Paseo worker that asks a question keeps its Paseo agent intact (paseo-answers D1, store `240f3db5`). The runner does not archive the agent on a question outcome.

When the answer arrives, the broker sends the answer to that same agent. The answer runs as the next round of the same job. The broker sends the answer only when Paseo inspection shows the agent in the `Idle` state. If the agent is missing or is not `Idle`, the broker starts a fresh child job instead. The system never sends an answer into an active turn.

The command `bee herding run --continue <job_id>` continues an existing Paseo job. The runner inspects the Paseo agent before it sends the next round prompt. If the agent is `Idle`, the runner sends the new brief pointer to the agent. If the agent is busy or in any state other than `Idle`, the runner refuses the command:

```text
FIX: wait for it to finish, then run bee herding run --continue <job>
```

The runner makes no send call when the agent is busy.

The command `bee herding steer` steers a running turn on Paseo (paseo-answers D2, store `e13feabc`). The steer path depends on the provider of the Paseo agent. For `claude`, `codex`, and `opencode` providers, the command steers through the Paseo daemon helper. This daemon helper requires Node.js and the npm package `@getpaseo/cli`. The helper sends the steer message with `activeTurnBehavior: "steer"`. For a `pi` provider, the command writes a steer file to the mailbox as before. The runner refuses steer requests for other providers with an error and a `FIX:` message. The command never uses `paseo send` to steer because an ordinary send interrupts and replaces a running turn.

## Watching and controlling Paseo workers

The command `bee herding status` displays information for Paseo workers. The status output includes these specific fields:
- `transport`: Shows the value `paseo`.
- `paseo_agent_id`: Shows the agent identifier from Paseo.
- `status`: Shows bee's freshness view (`working`, `idle`, `stalled`, `recovered`).
- `paseo_state`: Shows the daemon's raw state: `working`, `idle`, `blocked`, `dead`, `unknown`, or `finished`.
- `permissions`: Lists pending permission requests and tool names when a worker is blocked.
- `model_mismatch`: Boolean flag set to true when the observed model differs from configuration (paseo-pi-hardening D4, store `4cb9c644`).
- `paseo_model_observed`: Shows the observed model string from inspect output.
- `untracked_paseo_agents`: Lists labeled agents that no active job tracks.

A job with a result for its round or a mark is `finished`, and status does not ask Paseo about it. Status asks Paseo only when the repo has a Paseo agent or job and the daemon port answers; each call has a 5-second limit, and after the first failure status stops asking.

The `status` field is bee's freshness view and `paseo_state` is the daemon's raw state. Therefore, `status=stalled` with `paseo_state=working` is expected and normal when an agent executes a long tool call without stream events.

The command `bee herding interrupt` stops an active turn with `paseo stop`. The command keeps the Paseo agent intact for inspection.

The command `bee herding cancel` stops the active turn with `paseo stop`. It then archives the agent with `paseo archive`. The command enforces an own-agent guard. It compares the target agent identifier with `PASEO_AGENT_ID`. The command never stops or archives the caller's own agent.

The command `bee herding permit` resolves a pending tool permission. When a Paseo worker requests a tool permission, its state becomes blocked. The command `bee herding run` keeps waiting while the worker is blocked. The mailbox broker files a permission decision in the human-decision queue. A person answers the decision with `bee herding permit`. The worker receives the decision, and the same turn continues.

Occupancy counting adds labeled, active Paseo agents to the live worker count.

The command `bee herding pane read` shows the recent log tail of a Paseo worker with `paseo logs`.

The field `untracked_paseo_agents` reports labeled Paseo agents that have no active mailbox or job. The system reports these untracked agents only. The system never archives untracked agents.

## Pointers

- Paseo configuration and command builder: `packages/bee-rs/crates/bee/src/herding/paseo.rs`.
- Paseo worker execution and lifecycle: `packages/bee-rs/crates/bee/src/herding/run.rs`.
- Paseo status view: `packages/bee-rs/crates/bee/src/herding.rs`.
- Setup check doctor row: `packages/bee-rs/crates/bee/src/doctor.rs`.
- Worker shell guard: `packages/bee-rs/crates/bee/src/hooks/worker_guard.rs`.
- Leader broker timer and heartbeat: `.pi/extensions/bee-guard/paseo-heartbeat.ts`.
- Leader result inbox and steer sender: `.pi/extensions/bee-guard/result-inbox.ts`.
- Extension lifecycle hooks: `.pi/extensions/bee-guard/events.ts`.
- Feature context and locked decisions: `docs/history/paseo-pi-hardening/CONTEXT.md`.
- Implementation plan and test matrix: `docs/history/paseo-pi-hardening/plan.md`.
- Base feature context: `docs/history/paseo-pi/CONTEXT.md`.
- Base implementation plan: `docs/history/paseo-pi/plan.md`.
- Mailbox broker concept: `docs/knowledge/areas/bee-herding/the-mailbox-broker.md`.
