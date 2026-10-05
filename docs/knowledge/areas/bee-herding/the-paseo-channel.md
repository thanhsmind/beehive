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
  decisions: ["paseo-pi D1 03c795c7 (Paseo is a new herding channel beside herdr, chosen per team config; herdr panes keep working unchanged)", "paseo-pi D4 9d884211 (each worker on the Paseo channel runs the provider and model that bee team config binds to its role; Paseo only carries the worker)", "paseo-pi D5 b6dd8b33 (on the Paseo channel, an answer goes into the running worker by steering only when the worker's provider can steer; otherwise stop and next round; never interrupt-and-replace)", "paseo-pi D6 83f5caad (the Pi leader on Paseo wakes from a Paseo heartbeat; extension runs the code tick and turns heartbeat into news prompt or short turn; never swallows it)"]
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

## Planned work

Two planned slices extend the Paseo channel:

Slice 2 implements the Paseo heartbeat tick (paseo-pi D6, store `83f5caad`). The Pi leader wakes from a Paseo heartbeat and runs the bee code tick.

Slice 3 implements worker answers through Paseo messaging (paseo-pi D5, store `b6dd8b33`). The runner steers answers into running workers that support steering, and uses stop-and-next-round for workers that cannot steer.

## Pointers

- Paseo configuration and command builder: `packages/bee-rs/crates/bee/src/herding/paseo.rs`.
- Paseo worker execution and lifecycle: `packages/bee-rs/crates/bee/src/herding/run.rs`.
- Feature context and locked decisions: `docs/history/paseo-pi/CONTEXT.md`.
- Implementation plan and test matrix: `docs/history/paseo-pi/plan.md`.
- Mailbox broker concept: `docs/knowledge/areas/bee-herding/the-mailbox-broker.md`.
