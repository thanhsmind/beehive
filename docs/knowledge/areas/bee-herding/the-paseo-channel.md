---
type: bee.area
title: "Bee Herding — the Paseo channel, worker execution, and lifecycle"
description: "The herding transport that executes bee workers as Paseo agents, polls the file mailbox, tracks daemon liveness states, and archives completed agents."
timestamp: 2026-10-05
bee:
  id: bee-herding-the-paseo-channel
  lifecycle: active
  areas: [bee-herding]
  required_context: [areas/bee-herding/overview.md]
  decisions: ["paseo-pi D1 03c795c7 (Paseo is a new herding channel beside herdr, chosen per team config; herdr panes keep working unchanged)", "paseo-pi D4 9d884211 (each worker on the Paseo channel runs the provider and model that bee team config binds to its role; Paseo only carries the worker)", "paseo-pi D5 b6dd8b33 (on the Paseo channel, an answer goes into the running worker by steering only when the worker's provider can steer; otherwise stop and next round; never interrupt-and-replace)", "paseo-pi D6 83f5caad (the Pi leader on Paseo wakes from a Paseo heartbeat; extension runs the code tick and turns heartbeat into news prompt or short turn; never swallows it)", "paseo-pi D7 8ae5134d (the heartbeat fires every 5 minutes by default; a config key changes it)", "paseo-pi D8 2aa8417a (the leader turns its heartbeat on by itself at session start inside Paseo; never a second heartbeat for the same agent)", "paseo-answers D1, D2", "paseo-observe D1, D2"]
  sources: [docs/history/paseo-pi/CONTEXT.md, docs/history/paseo-pi/plan.md]
  authoritative_for: "bee-herding: the Paseo channel, paseo agent block configuration, worker execution, and lifecycle management"
  owns.code: [packages/bee-rs/crates/bee/src/herding/paseo.rs, packages/bee-rs/crates/bee/src/herding/run.rs]
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

## Liveness states

The runner checks agent health with the command `paseo inspect <agent-id> --json`.

The runner parses the `Status` field and the `PendingPermissions` array:
- `Status: "running"` maps to `Working`. If `PendingPermissions` is not empty, it maps to `Blocked`.
- `Status: "idle"` maps to `Idle`.
- `Status: "error"` and `Status: "closed"` map to `Dead`.
- Any other value or an invalid JSON response maps to `Unknown`.

`Unknown` never counts as alive. When inspection fails, the runner treats the state as unavailable and fails closed.

## Agent archival and the own-ID guard

When a run completes, the runner evaluates whether to archive the Paseo agent.

The runner executes `paseo archive --force <agent-id>` only when `should_close_pane` returns true. This condition requires a valid result or an explicit close configuration.

If the worker fails, crashes, or times out, the runner does not archive the agent. It keeps the agent intact and reports the agent ID for diagnosis.

The runner enforces an own-ID guard. It never archives an agent whose ID matches `PASEO_AGENT_ID` from the environment. This guard prevents a leader agent from deleting itself.

## Diagnostic FIX messages

The Paseo channel supplies two diagnostic messages with `FIX:` guidance.

When the installed version of Paseo is older than version 0.10.3, the runner refuses to spawn the agent:
```text
FIX: upgrade paseo to 0.10.3 or newer
```

When `paseo run` fails because the daemon is down, the runner refuses with:
```text
FIX: start the daemon with paseo daemon start (npm @getpaseo/cli 0.10.3+)
```

## Environment facts

Three environment facts apply from CONTEXT.md:

First, Paseo 0.10.3 is the minimum required version. It supports steering for the Pi provider. Older versions do not support steering.

Second, the npm package `@getpaseo/cli` 0.10.3 can start the daemon. The AppImage CLI cannot start the daemon.

Third, the CLI command `paseo send` has no steer flag. Steering requires the daemon WebSocket interface. Extensions in `.pi/extensions/` load inside a Paseo Pi agent and receive `PASEO_AGENT_ID`.

## The leader heartbeat

An agent is a leader when `PASEO_AGENT_ID` is set and `BEE_HERDING_WORKER` is unset.

At session start, the extension creates one `bee-leader` heartbeat for the agent (paseo-pi D8, store `2aa8417a`). It creates the heartbeat only once per agent.

The extension records the heartbeat in a marker file at `.bee/runtime/paseo-heartbeat/<agent id>.json`.

Two configuration keys control the heartbeat:
- `herding.paseo.heartbeat_cron`: Sets the cron schedule (paseo-pi D7, store `8ae5134d`). The default schedule is `*/5 * * * *`.
- `herding.paseo.command`: Sets the CLI executable name. The default command is `paseo`.

On each heartbeat firing, the extension executes the broker code tick. When the tick routes work, the extension transforms the heartbeat prompt into news text. When the tick routes no work, the extension transforms the heartbeat prompt into the one-word text `ok`.

The extension never swallows a heartbeat prompt (paseo-pi D6, store `83f5caad`). A swallowed prompt stalls Paseo because Paseo keeps the agent status as running.

Worker results do not wait for the heartbeat. The 2-second result drain processes worker results independently.

A turn without news skips the continuation nudge at settle to prevent extra model turns.

To stop the heartbeat, read the schedule ID from the marker file. Run the command `paseo heartbeat delete <id>` with that schedule ID. Then delete the marker file.

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

The command `bee herding status` displays information for Paseo workers. The status output includes five specific fields:
- `transport`: Shows the value `paseo`.
- `paseo_agent_id`: Shows the agent identifier from Paseo.
- `paseo_state`: Shows the current state: `working`, `idle`, `blocked`, `dead`, `unknown`, or `finished`. A job with a result for its round or a mark is `finished`, and status does not ask Paseo about it. Status asks Paseo only when the repo has a Paseo agent or job and the daemon port answers; each call has a 5-second limit, and after the first failure status stops asking.
- `permissions`: Lists pending permission requests and tool names when a worker is blocked.
- `untracked_paseo_agents`: Lists labeled agents that no active job tracks.

An `unknown` state means that agent inspection failed. The `unknown` state is neither alive nor dead. The system does not mark an unknown worker as interrupted.

The command `bee herding interrupt` stops an active turn with `paseo stop`. The command keeps the Paseo agent intact for inspection.

The command `bee herding cancel` stops the active turn with `paseo stop`. It then archives the agent with `paseo archive`. The command enforces an own-agent guard. It compares the target agent identifier with `PASEO_AGENT_ID`. The command never stops or archives the caller's own agent.

The command `bee herding permit` resolves a pending tool permission. When a Paseo worker requests a tool permission, its state becomes blocked. The command `bee herding run` keeps waiting while the worker is blocked. The mailbox broker files a permission decision in the human-decision queue. A person answers the decision with `bee herding permit`. The worker receives the decision, and the same turn continues.

Occupancy counting adds labeled, active Paseo agents to the live worker count.

The command `bee herding pane read` shows the recent log tail of a Paseo worker with `paseo logs`.

The field `untracked_paseo_agents` reports labeled Paseo agents that have no active mailbox or job. The system reports these untracked agents only. The system never archives untracked agents.

## Pointers

- Paseo configuration and command builder: `packages/bee-rs/crates/bee/src/herding/paseo.rs`.
- Paseo worker execution and lifecycle: `packages/bee-rs/crates/bee/src/herding/run.rs`.
- Feature context and locked decisions: `docs/history/paseo-pi/CONTEXT.md`.
- Implementation plan and test matrix: `docs/history/paseo-pi/plan.md`.
- Mailbox broker concept: `docs/knowledge/areas/bee-herding/the-mailbox-broker.md`.
