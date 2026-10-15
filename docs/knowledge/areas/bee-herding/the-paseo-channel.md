---
type: bee.area
title: "Bee Herding — the Paseo channel, worker execution, and lifecycle"
description: "The herding transport that executes bee workers as Paseo agents, polls the file mailbox, tracks daemon liveness states, and archives completed agents."
timestamp: 2026-10-15
bee:
  id: bee-herding-the-paseo-channel
  lifecycle: active
  areas: [bee-herding]
  required_context: [areas/bee-herding/overview.md]
  decisions: ["claude-read-only D1-D5 (read-only Paseo claude workers: env marker, Claude PreToolUse worker-guard group, mode A/B, fail-closed pre-spawn check, extended matcher)", "paseo-pi D1 03c795c7 (Paseo is a new herding channel beside herdr, chosen per team config; herdr panes keep working unchanged)", "paseo-pi D4 9d884211 (each worker on the Paseo channel runs the provider and model that bee team config binds to its role; Paseo only carries the worker)", "paseo-pi D5 b6dd8b33 (on the Paseo channel, an answer goes into the running worker by steering only when the worker's provider can steer; otherwise stop and next round; never interrupt-and-replace)", "paseo-pi D6 83f5caad (the Pi leader on Paseo wakes from a Paseo heartbeat; extension runs the code tick and turns heartbeat into news prompt or short turn; never swallows it)", "paseo-pi D8 2aa8417a (the leader turns its heartbeat on by itself at session start inside Paseo; never a second heartbeat for the same agent)", "paseo-answers D1, D2", "paseo-observe D1, D2", "paseo-pi-hardening D1 9a502af1", "paseo-pi-hardening D2 a8d28361", "paseo-pi-hardening D3 c2a5af66", "paseo-pi-hardening D4 4cb9c644", "paseo-pi-hardening D6 976ef5a5", "paseo-pi-hardening D7 fb38df99", "paseo-pi-hardening D8 f26723a1", "paseo-pi-hardening D10 0a0ef8e7", "herding-leader-toil D1 cb3dc85d", "herding-leader-toil D4 2e46ebbe", "herding-leader-toil D5 8fa9692a", "herding-leader-toil D6 9e018527", "herding-leader-toil D7 e1dcf730", "paseo-leverage D1 a7071d2b", "paseo-leverage D2 906e0b1f (revised by claude-read-only D1; supersedes 168b8037)", "paseo-leverage D3 a7067517", "paseo-leverage D4 33df9d0d", "paseo-leverage D5 89d24cf0", "paseo-leverage D6 1742a721", "paseo-leverage D7 12211a7d", "paseo-leverage D8 28b0b5c2", "paseo-cli-autodetect 03ff4564 (bee resolves the paseo CLI without config: herding.paseo.command, else PASEO_CLI when it names an existing file, else the running daemon's CLI from paseo.pid and that pid's environ or exe directory, else plain paseo)", "paseo-leverage hat-wave refinements 6fc537f6", "paseo-worktree-workspace D1-D6 57793b86 (worktree new registers a Paseo workspace inside Paseo; teardown archives it; herding runs pass --workspace; paseo children drop PASEO_AGENT_ID and PASEO_AGENT_CWD; JSON read past log lines)", "paseo-worker-subagent c09228a8 (the paseo run that starts a worker keeps PASEO_AGENT_ID and PASEO_AGENT_CWD so the worker is the lead's subagent; every other paseo process runs without them; --workspace still rides the run; narrows paseo-worktree-workspace D5)"]
  sources: [docs/history/claude-read-only/CONTEXT.md, docs/history/paseo-pi/CONTEXT.md, docs/history/paseo-pi/plan.md, docs/history/paseo-pi-hardening/CONTEXT.md, docs/history/paseo-pi-hardening/plan.md, docs/history/herding-leader-toil/CONTEXT.md, docs/history/herding-leader-toil/plan.md, docs/history/paseo-leverage/CONTEXT.md, docs/history/paseo-leverage/plan.md, docs/history/paseo-worktree-workspace/CONTEXT.md]
  authoritative_for: "bee-herding: the Paseo channel, paseo agent block configuration, worker execution, and lifecycle management"
  owns.code: [packages/bee-rs/crates/bee/src/herding/paseo.rs, packages/bee-rs/crates/bee/src/herding/run.rs, packages/bee-rs/crates/bee/src/herding.rs, packages/bee-rs/crates/bee/src/doctor.rs, packages/bee-rs/crates/bee/src/herding/pi_agent_dir.rs, packages/bee-rs/crates/bee/src/herding/paseo_state.rs, packages/bee-rs/crates/bee/src/herding/fork.rs, packages/bee-rs/crates/bee/src/herding/notify.rs]
---

# Bee Herding — the Paseo channel, worker execution, and lifecycle

## What the Paseo channel is

The Paseo channel is a herding transport (paseo-pi D1, store `03c795c7`). It operates beside the herdr and tmux transports. Herdr panes continue to work without changes.

The channel starts a worker as a Paseo agent instead of a terminal pane. Each worker uses the provider and model that team configuration assigns to its role (paseo-pi D4, store `9d884211`). Paseo only carries the worker. The dispatch door remains the single component that selects the model.

## Agent configuration and command

You configure a Paseo agent under the `herding.agents.<name>.paseo` object in team configuration.

The `paseo` block contains five fields:
- `provider`: A required string that names the model provider (for example, `pi`).
- `model`: An optional string that names the model.
- `thinking`: An optional string that sets the reasoning effort level.
- `mode`: An optional string for provider modes.
- `isolated_config`: An optional boolean for `pi` provider agents (herding-leader-toil D5, store `8fa9692a`, contract store `db780128`; default `false`).

### Isolated Pi configuration (`isolated_config`)

When `isolated_config` is `true` for a `pi` provider agent (herding-leader-toil D5, store `8fa9692a`, contract store `db780128`), bee manages an isolated directory at `.bee/runtime/pi-agent/<agent>/`:
- It links `auth.json` to `~/.pi/agent/auth.json`.
- It links `models.json`, `models-store.json`, and `npm` when those sources exist in `~/.pi/agent/`.
- It writes `settings.json` with `{"defaultProjectTrust": "always", "quietStartup": true}`.
- It provides empty `skills/` and `extensions/` subdirectories.
- It passes `PI_CODING_AGENT_DIR=<path>` to the worker process.

bee never replaces an existing real file or directory where a link belongs; it reports the item.
If `~/.pi/agent/auth.json` is missing, the spawn fails with a `FIX:` message.

The command resolution order (decision `03ff4564`):

1. The key `herding.paseo.command` (or the legacy top-level `paseo.command`) when it is set and not blank. This always wins.
2. The `PASEO_CLI` environment value, when it names an existing file.
3. The CLI of the running daemon: bee reads the `pid` key of `<paseo home>/paseo.pid`, then the `PASEO_CLI` entry of `/proc/<pid>/environ` when it names an existing file, else `<parent of /proc/<pid>/exe>/resources/bin/paseo` when that file exists. The Paseo home comes from `PASEO_HOME`, else `~/.paseo`.
4. The plain command `paseo`.

This order lets bee follow the live daemon with no per-machine config. The desktop AppImage mount path changes on every launch, so a fixed configured path goes stale. Keep `herding.paseo.command` for a deliberate override.
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
- `--workspace <id>` when the run has a Paseo workspace (see "Worktree workspaces")
- `--title <cell-id> <agent>` when the run has a cell id, else `<agent> <job-id>` (herding-leader-toil D7, store `e1dcf730`)
- `--label bee_job=<job_id>`
- `--label bee_agent=<agent>`
- `--label bee_cell=<cell_id>` when the run has a cell id
- `--env` flags for child environment variables, including `BEE_HERDING_WORKER=1`, `BEE_HERDING_JOB_ID`, and `PI_CODING_AGENT_DIR` when `isolated_config` is true
- The initial prompt string.

The one `paseo run` that starts the worker keeps `PASEO_AGENT_ID` and `PASEO_AGENT_CWD`, so Paseo records the calling lead as the worker's `ParentAgentId` (decision `c09228a8`, which narrows paseo-worktree-workspace D5, store `57793b86`). Every other paseo process that bee starts runs without the two variables. The `--env` pairs for a worker do not carry them. Thus the worker is the lead's subagent and still runs in its worktree workspace. This supersedes the parent rule of paseo-pi-hardening D10 (store `0a0ef8e7`).
Every run applies the label `bee_job=<job_id>`. This label allows cleanup tools to identify orphan agents.

The runner reads `agentId` from the process output. It writes `paseo_agent_id` and `transport: "paseo"` into `job.json` immediately.

The worker communicates through the standard file mailbox. It reads `brief-N.txt` and writes `ack-N.json`, `report-N.md`, and `result-N.json`. The runner reads the final outcome only from `result-N.json`.

### Timeouts and the event wait loop

Every Paseo CLI call in `bee herding run` and `bee herding run --continue` carries a 15-second timeout (paseo-pi-hardening D1, store `9a502af1`).

The wait loop learns state changes using `paseo agent wait <id> --timeout <n> --json` through a `WaitSource` seam (herding-leader-toil D4, store `2e46ebbe`, contract store `a0c36961`).
The real source spawns `paseo agent wait` as a child process with piped standard output.
The loop polls the child with a non-blocking check on the 200 ms tick.
It terminates the child process when the round ends or when a new wait arms.
The loop uses no threads.

Status mapping from wait output:
- `idle` maps to `Idle`.
- `permission` maps to `Blocked`.
- `error` maps to `Dead`.
- `timeout` maps to `Working`.

A new wait arms only after a timeout or after idle handling.
A new wait never arms sooner than 3 seconds after the previous wait returned.
While in `Blocked` state, and after an `Error` until a state is confirmed, the loop falls back to one `inspect` call every 3 seconds.
This fallback preserves the three-read died debounce check.
One `inspect` call after agent spawn records the model and thinking settings (paseo-pi-hardening D4, store `4cb9c644`).
Mailbox file checks continue every 200 ms.
The silent-idle nudge (paseo-pi-hardening D2, store `a8d28361`), idle timeouts, and ceiling timeouts remain unchanged.

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

### Observed mode and parked workers

After agent spawn, the runner records the inspect `Mode` as `paseo_mode_observed` in `job.json` (paseo-leverage D3, store `a7067517`).
`bee herding status` marks a Paseo worker `parked` when it is `Blocked` on a pending permission for longer than the shared 120-second freshness constant.
The status line then shows `parked=true` and a `parked_reason` that names the mode and the pending tool.
bee reports a parked worker only. bee never runs `paseo agent mode` on its own.

### Usage in job.json

When a Paseo round ends, the runner writes `usage` into `job.json` (paseo-leverage D5, store `89d24cf0`).
The object has these keys: `input`, `output`, `cache_read`, `cache_write`, `total_tokens`, `cost_usd`, and `source`.

The runner reads usage from two sources, in this order:
1. The inspect `LastUsage` field, when it is not null (`source: "inspect"`).
2. For provider `pi`, the sum of the `usage` entries in the session file that the on-disk `persistence.nativeHandle` names.

When neither source has usage, `usage` is `null` and `usage_unavailable` names the reason. bee never writes zero for unknown usage.
`bee herding status` shows `tokens` and `cost_usd` for each job.

### Read-only runs

`bee herding run --read-only` starts a read-only worker (paseo-leverage D2 as revised by claude-read-only D1, store `906e0b1f`).
`bee dispatch prepare` passes the flag for kinds `gather` and `advisor` only. Reviewers run tests, so they do not get the flag.
`job.json` records `read_only: true`. No Paseo mode is set.

A Paseo `pi` worker gets `BEE_HERDING_READ_ONLY=1`. The pi belt then sends every tool call to `worker-guard`.
The write zone is the worker's own job mailbox directory. The guard refuses a write to any other path.
Shell commands must match a read-only allowlist. The allowlist fails closed on truncation, command substitution, a redirect outside the mailbox other than the literal `/dev/null`, and per-tool exec or output flags. `cd` is on the allowlist.

A Paseo `claude` worker also gets `BEE_HERDING_READ_ONLY=1` and keeps its configured `paseo.mode` (claude-read-only D1). A Claude-only PreToolUse group runs `bee hook worker-guard` on `Edit|Write|MultiEdit|NotebookEdit|Bash|Monitor|EnterWorktree|ExitWorktree|mcp__.*`, with no status message, in both hook renderers (claude-read-only D2, D5). Outside a herded worker the hook returns at once. In read-only mode the guard judges `Monitor` like Bash and denies `EnterWorktree`, `ExitWorktree` and every `mcp__` tool (claude-read-only D5). Mode A and mode B apply to a Paseo claude worker the same way (claude-read-only D3).
A read-only claude run fails closed before spawn unless `<main root>/.bee/bin/bee hook --help` lists `worker-guard` and `<main root>/.claude/settings.json` has the PreToolUse entry (claude-read-only D4).

Any other Paseo provider refuses a read-only run before spawn with a `FIX:` line that names `pi` and `claude`. Herdr and tmux runs record `read_only` in `job.json`, but nothing enforces it.

### Observed model verification

After agent spawn, the runner reads inspect `Model` and `Thinking` fields (paseo-pi-hardening D4, store `4cb9c644`).
The runner writes `paseo_model_observed` and `paseo_thinking_observed` into `job.json`.
The sentinel values `-` for model and `auto` for thinking record as unverified.
The runner compares the observed model with the configured `paseo` model.
On a confirmed mismatch, the runner prints a warning and records `paseo_model_mismatch: true` in `job.json`, with the configured values as `paseo_model_configured` and `paseo_thinking_configured`.
A confirmed mismatch needs both sides known: a sentinel or an unset configured value never counts.

The config key `herding.paseo.on_model_mismatch` sets what a mismatch does (slp-role-gaps D9, store `0ccaa387`):

- `warn` — the default, also when the key is absent or holds any other value. The runner only warns. The run continues.
- `refuse` — when the runner sees the mismatch while it still polls for the result, it stops the agent with `paseo stop` (the command `bee herding interrupt` uses) and ends the run with outcome `model_mismatch`. The run exits non-zero. Its `error` names the configured and observed model and thinking and two fixes: point the agent's `paseo` block at the model that runs, or set `herding.paseo.on_model_mismatch` to `warn`. The runner never archives a refused agent, also with `--close-always`.

`refuse` never fires after the worker finished: a mismatch first seen with the result already on disk keeps the result and only warns, because the work is paid for and done.
In both modes the run envelope carries `model_mismatch: {configured: {model, thinking}, observed: {model, thinking}}`, read from `job.json`.
The runner never retries a worker on a model mismatch.

Today's flash route reports `thinking off` against a configured `thinking medium`, so every flash worker would be refused under `refuse`. That is why `warn` stays the default.

## Liveness states

The runner checks agent health with the command `paseo inspect <agent-id> --json`.

The runner parses the `Status` field and the `PendingPermissions` array:
- `Status: "running"` maps to `Working`. If `PendingPermissions` is not empty, it maps to `Blocked`.
- `Status: "idle"` maps to `Idle`.
- `Status: "error"` and `Status: "closed"` map to `Dead`.
- Any other value or an invalid JSON response maps to `Unknown`.

`Unknown` never counts as alive. When inspection fails, the runner treats the state as unavailable and fails closed.
A failed or unparseable inspect is `Unknown`: it resets the died debounce and never counts toward `died` (paseo-leverage D4, store `33df9d0d`).

The inspect `UpdatedAt` timestamp is observe-only (paseo-pi-hardening D3, store `c2a5af66`).
The run loop keeps a `Working` agent fresh.
The command `bee herding status` shows status `stalled` for a `Working` Paseo agent whose `UpdatedAt` is older than 120 seconds.
When `UpdatedAt` changes, status transitions through `recovered` before returning to `working`.
A missing or unparseable `UpdatedAt` never marks an agent as `stalled`.

The `status` field is bee's freshness view. The `paseo_state` field is the daemon's raw state.
Therefore, status `stalled` with `paseo_state: "working"` is expected and normal when an agent runs a long tool call.

## Agent archival and parent links

When a run completes, the runner evaluates whether to archive the Paseo agent.

The runner executes `paseo archive --force <agent-id>` only when `should_close_pane` returns true. This condition requires a valid result or an explicit close configuration.

If the worker fails, crashes, or times out, the runner does not archive the agent. It keeps the agent intact and reports the agent ID for diagnosis.

The runner enforces an own-ID guard. It never archives an agent whose ID matches `PASEO_AGENT_ID` from the environment. This guard prevents a leader agent from deleting itself.

A worker's `ParentAgentId` is the lead that started it, because the worker-starting `paseo run` keeps `PASEO_AGENT_ID` (decision `c09228a8`). Thus archiving the leader agent also archives its workers.

### Completing herded runs without capping

When a worker finishes a cell without capping it, `bee herding run` prints an uncapped-success line:

```text
worker reported success for cell <cell> without capping it — settle with bee cells finish --id <cell> --from-job <job-id> --proof-result <green:unit|green:static|green:live>
```

(herding-leader-toil D1, store `cb3dc85d`, contract store `d248942f`).
The leader checks the completed work.
The leader then caps the cell with that command and supplies the required mistakes answer (`--no-mistakes` or `--mistake <text> --fix-at <layer>`).
The command builds the cap report from the latest `result-N.json` and git changes in the job working directory.

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
The check verifies these conditions:
1. The Paseo daemon answers probes.
2. The CLI version output is valid: the first non-blank stdout line of `paseo --version` must be a bare version number (such as `0.10.3`). The command must not resolve to an AppImage wrapper script. If either check fails, doctor fails with:
   `FIX: set herding.paseo.command to the npm @getpaseo/cli paseo`
   (herding-leader-toil D6, store `9e018527`, contract store `a79e3edc`).
3. For each Pi agent with `isolated_config`:
   - When `.bee/runtime/pi-agent/<agent>/` exists, `settings.json` must specify `defaultProjectTrust: "always"`. The file `auth.json` must be a working symbolic link.
   - When the folder does not exist yet, doctor verifies that `~/.pi/agent/auth.json` exists.
4. The file `~/.pi/agent/auth.json` exists when a configured provider is `pi`.
5. Every heartbeat marker in `.bee/runtime/paseo-heartbeat/` names an agent present in `paseo ls` and not archived.
6. No marker file contains a `delete_failed` record.

The doctor row reports findings only and changes no state.

## Environment facts

Three environment facts apply from CONTEXT.md:

First, Paseo 0.10.3 is the minimum required version. It supports steering for the Pi provider. Older versions do not support steering.

Second, the npm package `@getpaseo/cli` 0.10.3 can start the daemon. The AppImage CLI cannot start the daemon, and its `paseo run --json` prints no JSON object, so `bee herding run` fails with `spawn_failed` ("could not parse agent id") while `bee doctor` originally reported `paseo_ready` ok (live run 2026-10-06). Decision 9e018527 (herding-leader-toil D6) adds doctor AppImage checks to detect this condition. When a desktop AppImage wrapper comes first on `PATH`, decision 03ff4564 makes bee skip it and use the CLI of the running daemon. `herding.paseo.command` stays as a deliberate override.

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

## Mid-flight messages

A running herded worker can send its leader a typed note with `bee herding notify` (paseo-leverage D1, store `a7071d2b`):

```text
bee herding notify --kind <question|blocked|dependency|progress|reopen> --text <text> [--correlation-id <id>]
```

The verb works only inside a herded worker (`BEE_HERDING_JOB_ID` set). It refuses an unknown kind.
It appends one JSON line to `.bee/mailbox/<job>/messages.jsonl`. The line carries the job id, cell id, kind, text, correlation id, and time.
The worker brief names `notify` as the one bee command a worker may run.

A message never ends the worker's turn. It never replaces the `question` outcome: an answer still reaches the worker as the next round.

Delivery never uses `paseo send`, because a send to a busy leader replaces its turn. Delivery has two paths, with these limits (hat-wave refinement, store `6fc537f6`):
- Unread means the lines past the per-job `messages.cursor` file.
- `bee herding status` shows `messages`, `unread_messages`, and `last_message` for every leader.
- The Pi leader's result-inbox drain injects new lines only for jobs run with `--inbox-session`. It fences worker text as data inside a `bee-message` fence. It steers a busy leader and opens a turn for an idle one.
- The drain advances the cursor only when the send reports steer or turn. A failed send leaves the lines unread.

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

## Forking a Paseo agent

`bee herding fork --job <id> | --agent <paseo-id> [--force]` makes a successor Paseo agent that keeps the source agent's history (paseo-leverage D6, store `1742a721`; hat-wave refinement, store `6fc537f6`).

For provider `pi`, the verb does these steps:
1. It reads the session file that the on-disk `persistence.nativeHandle` names.
2. It writes a copy whose first line is a new session header: a new UUIDv7 id, a new timestamp, and `parentSession` set to the source path. It writes a temp file and renames it.
3. It runs `paseo import <new id> --provider pi --cwd <source cwd> --label bee_fork_of=<source id>`. When the import fails, it removes the copy.

The verb refuses these sources with a `FIX:` line:
- A provider other than `pi`.
- A missing session file, or a session file outside the default pi sessions root `~/.pi/agent/sessions/`.
- A `Working` source, unless you pass `--force`.

The verb prints the new agent id, the source, the undo command, and that the model is the provider default.
The undo command is `paseo archive <new id>; rm <session file>`.

The verb never stops, archives, or sends to the source. Only a human or the leader runs it.
When the supervisor's `dead-lead` observation names a Paseo lead, its resume line names the fork command.

## The on-disk agent reader

bee reads Paseo's on-disk agent state at `<paseo home>/agents/<project>/<id>.json` (paseo-leverage D7, store `12211a7d`; hat-wave refinement, store `6fc537f6`).
The Paseo home comes from `--home` in `herding.paseo.command`, else `PASEO_HOME`, else `~/.paseo`.

The reader serves views that need many agents or data that the CLI does not give: the untracked-agent report in `bee herding status`, round usage, and fork.
A missing or unreadable home or file falls back to the CLI: the untracked-agent report then runs `paseo ls`.
The run loop keeps the CLI (`inspect`, `wait`) as the liveness authority.

## Worktree workspaces

A Paseo workspace is a Paseo `wks_…` record that groups agents under one directory. bee shows each bee worktree in Paseo as a workspace (decision `57793b86`, paseo-worktree-workspace D1-D6).

### When bee is inside Paseo

bee is inside Paseo when `PASEO_AGENT_ID` or `PASEO_CLI` is set and not empty (D1). Outside Paseo, `bee worktree new` and `bee herding run` make no new Paseo call. Unit tests see "outside Paseo" unless a test sets a seam.

A human `bee worktree new` in a Paseo terminal also counts as inside Paseo, because the terminal sets `PASEO_CLI`. That command also registers a workspace.

### Create: `bee worktree new`

Inside Paseo, `bee worktree new` makes sure that a workspace exists for the new worktree (D2):
1. bee reads `paseo workspace ls --json`. It reuses a workspace whose `cwd` is the worktree directory. bee compares the paths after canonicalize.
2. Else bee reads `paseo project ls --json`. It takes the `projectId` whose `path` is the main checkout.
3. bee runs `paseo workspace create --path <worktree> --isolation local --project <projectId> --json`. When no project matches, bee omits `--project`.

Do not use the `project` field of `workspace ls` as the id. It is a name, and `--project <name>` fails.

bee records the id as `paseo_workspace_id` on the bee workspace record `.bee/runtime/workspaces/<id>.json`. The text output prints `paseo:       workspace <id>`. The JSON output carries the key `paseoWorkspace`: `{"id": "<id>"}`, or `{"error": "<reason>"}` on a failure.

A Paseo failure prints one warning. It never fails the worktree and never rolls it back.

### Teardown: merge cleanup, prune, and unregister

`bee worktree merge` cleanup, `bee worktree prune`, and `bee worktree unregister` remove the bee workspace record. When the record carries `paseo_workspace_id`, bee first runs `paseo workspace archive <id> --json` (D3). This step does not check D1: the record decides.

A failed archive prints one warning on stderr with its FIX command:

```text
paseo: could not archive workspace <id>: <error> — FIX: paseo workspace archive <id>
```

A failed archive never blocks the teardown.

### The run workspace rule

`bee herding run` on a Paseo agent passes `--workspace <id>` to `paseo run` (D4). The id is the `paseo_workspace_id` of the bee workspace record whose root holds the run cwd.

When no record carries an id and bee is inside Paseo, the run makes sure of a workspace for the cwd as `bee worktree new` does. It records the id when a bee record matches. When that step fails, the run refuses before spawn:

```text
paseo workspace for <dir> could not be found or created: <error> — FIX: paseo workspace create --path <dir> --isolation local, then re-run
```

The run never falls back to the caller's workspace. Without `--workspace`, `paseo run` from inside an agent ignores `--cwd` and joins the caller's workspace. Outside Paseo with no recorded id, the argv does not change.

### Paseo children and JSON output

The one `paseo run` that starts a worker keeps `PASEO_AGENT_ID` and `PASEO_AGENT_CWD` in its environment, so Paseo records the caller as the worker's parent and the worker is the lead's subagent (decision `c09228a8`, which narrows paseo-worktree-workspace D5, store `57793b86`). `--workspace` still rides that run, so the worker runs in its worktree workspace. Every other paseo process that bee starts runs without the two variables: the `wait` source, the `ls` fallback, `inspect`, `archive`, `send`, the workspace calls, and the prepare probe. The `--env` pairs for a worker do not carry these two variables.

bee reads paseo JSON output as the first JSON document in stdout (D6). It skips the log lines before it, because the AppImage wrapper prints startup log lines first.

### Out of scope

`bee herding fork` still imports into the caller's workspace. Paseo-owned worktrees (`--isolation worktree`) and the herdr and tmux transports are also out of scope.

## The live contract test

A live contract test checks the installed Paseo against the keys that bee parses (paseo-leverage D8, store `28b0b5c2`).
It checks the `inspect --json` keys, the `ls --json` shape, and the on-disk agent file keys that bee reads.
The tests are `#[ignore]` by default. They skip with a message when Paseo is not installed or the daemon does not answer.

Run them with this command:

```text
cargo test -- --ignored paseo_live
```

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
- Isolated Pi configuration directory: `packages/bee-rs/crates/bee/src/herding/pi_agent_dir.rs`.
- Herding leader toil feature context: `docs/history/herding-leader-toil/CONTEXT.md`.
- Herding leader toil implementation plan: `docs/history/herding-leader-toil/plan.md`.
- On-disk Paseo agent reader: `packages/bee-rs/crates/bee/src/herding/paseo_state.rs`.
- Fork verb: `packages/bee-rs/crates/bee/src/herding/fork.rs`.
- Mid-flight message verb: `packages/bee-rs/crates/bee/src/herding/notify.rs`.
- Live contract test: `packages/bee-rs/crates/bee/tests/paseo_live_contract.rs`.
- Paseo leverage feature context: `docs/history/paseo-leverage/CONTEXT.md`.
- Paseo leverage implementation plan: `docs/history/paseo-leverage/plan.md`.
- Worktree workspace register and archive: `packages/bee-rs/crates/bee/src/verbs/worktree/paseo_ws.rs`.
- Paseo worktree workspace feature context: `docs/history/paseo-worktree-workspace/CONTEXT.md`.
