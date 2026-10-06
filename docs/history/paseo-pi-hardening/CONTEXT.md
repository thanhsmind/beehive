# Paseo-on-Pi hardening — Context

**Feature slug:** paseo-pi-hardening
**Date:** 2026-10-06
**Shaping session:** complete (gate bypass full; the user asked for all eight items: "làm cả 8")
**Scope:** High-risk
**Domain types:** CALL | RUN

## Feature Boundary

bee already runs a Pi leader and its workers on Paseo (paseo-pi, paseo-answers,
paseo-observe, paseo-heartbeat). A comparison with Seatworks (`2e11099f`) and
slp (`paseo-pi-team`) found eight gaps: a wait loop that can hang and polls too
hard, a worker that goes quiet costing the full idle timeout, a hung turn
invisible for six hours, no check of the model that actually runs, Pi workers
with no guard at all, heartbeats nobody deletes, no doctor row for Paseo, a paid
model turn on every empty heartbeat, and a supervisor that only runs on Claude.
This feature closes those eight. Herdr and tmux transports, native Agent workers,
and the bee workflow do not change.

## Locked Decisions

These are fixed. Planning must implement them exactly — cited, never reinterpreted.

| ID | Decision | Rationale (only if it changes implementation) |
|----|----------|-----------------------------------------------|
| D1 | Every Paseo CLI call that `bee herding run` and `bee herding run --continue` make carries a 15-second timeout. The Paseo wait loop calls `paseo inspect` at most once every 3 seconds; the mailbox file checks keep the existing 200 ms poll. | Today the run path builds the CLI with no timeout (`run.rs` `RealPaseoCli::new(cmd)`), so a hung daemon hangs the run forever, and it spawns one Node process per worker every 200 ms. |
| D2 | A Paseo worker that is `Idle` with no `result-N.json` for its round, after the round has seen it `Working`, gets ONE nudge: a `paseo send` that names its brief and the result file it owes (safe because the agent is idle; paseo-pi D5 holds). A second idle-with-no-result, read at least one inspect interval (3 s) after the nudge, ends the round as the existing idle-timeout outcome with a message that names the silent idle; a failed nudge send counts as sent. The end reaches the decision through a `PollTick` flag that herdr and tmux ticks leave false. The agent is kept. No new outcome word. Revised after the hat wave (store `a8d28361`). | Cheap models end a turn without writing the verdict; today that costs the full 900 s idle timeout. One nudge is not a retry, so herding-cockpit-completeness 9615be76 stands. |
| D3 | The inspect `UpdatedAt` field is observe-only. The run loop keeps a `Working` agent fresh as today. `bee herding status` (and the run poll tick status line) shows the existing `stalled` word for a `Working` Paseo agent whose `UpdatedAt` is older than the shared 120 s freshness constant; a missing or unparseable `UpdatedAt` never shows `stalled`. Revised after the hat wave (store `c2a5af66`). | Paseo moves `UpdatedAt` only on stream events, so ending a round on it would kill a healthy worker inside one long silent tool call such as `cargo test --release`. |
| D4 | After a Paseo spawn, the run loop records the inspect `Model` and `Thinking` fields into `job.json` as `paseo_model_observed` and `paseo_thinking_observed`, and compares them with the configured `paseo` block. Paseo's sentinels `-` (model) and `auto` (thinking) and a missing field count as unverified. A confirmed mismatch prints one warning line and writes `paseo_model_mismatch: true`; `bee herding status` shows `model_mismatch`. bee never archives, stops or retries on a mismatch. Revised after the hat wave (store `4cb9c644`). | Catches a silent model fallback on cheap routes without adding automatic behavior. |
| D5 | A Paseo-carried worker (both `BEE_HERDING_WORKER` and `PASEO_AGENT_ID` set) gets one narrow guard. The hook `worker-guard` is the second hook that passes the marker short-circuit (after `activity`). It judges a shell command only and refuses: a git push, a GitHub write, an agent launch (the worker-outward forms, judged regardless of the working directory), a `paseo` command head, and the bee verbs `dispatch`, `herding run`, `worktree merge` and `gate`. Everything else is allowed. The Pi belt routes such a worker's shell calls to this hook and blocks on a deny, a crash, a missing binary, or a binary whose `bee hook --help` does not list `worker-guard`; for such a worker the extension does not register `bee_dispatch`, `bee_advisor` and `bee_steer`. Herdr and tmux pane workers keep today's posture (psd-12 nested stacking). Revised after the hat wave (store `be1699f8`). | Pi workers on Paseo run with no guard today (`hooks/mod.rs` `marker_short_circuits`). |
| D6 | The Pi leader's extension deletes its own `bee-leader` heartbeat (`paseo heartbeat delete <id>`) on `session_shutdown` for every reason except `reload`, and awaits it before returning. A successful delete removes the marker; a failed delete keeps it and writes `delete_failed` into it. `ensureHeartbeat` re-creates the heartbeat when the marker's cron differs from the configured cron. Revised after the hat wave (store `976ef5a5`). | Nothing deletes a heartbeat today; an orphan cron can fire at an archived leader. A kept marker lets doctor find a failed delete, and leaders created under `*/5` move to the new default. |
| D7 | On the pi runtime, `bee doctor` adds one `paseo_ready` row when any `team.pi` slot names a herding agent with a `paseo` block. It checks: the daemon answers, the CLI version is at least 0.10.3, `~/.pi/agent/auth.json` exists when a configured provider is `pi`, and no heartbeat marker names an agent that `paseo ls` no longer lists. Report only; it fixes nothing. | A setup fault shows before a worker fails. |
| D8 | The Pi leader's extension runs `bee herding broker tick --json` on its own timer, every `herding.paseo.broker_tick_secs` seconds (default 30), and starts a model turn with `sendUserMessage` only when the tick reports news (steer when the leader is busy, as the result drain does). The Paseo heartbeat stays as a backstop, and its default cron becomes `*/30 * * * *`. paseo-pi D6 stands unchanged: a heartbeat prompt is never swallowed. Supersedes paseo-pi D7 (store `8ae5134d`, default every 5 minutes). | News arrives in seconds, and the empty paid turns drop from about 288 to about 48 a day. Spike 2 (store `da10d6ad`) showed an extension timer that calls `sendUserMessage` starts a turn Paseo shows and settles. |
| D9 | `herding.supervisor_runtime` picks the supervisor's team table: `claude` (absent means this, unchanged) or `pi`. With `pi`, `team.pi.supervisor` must be a herding agent whose `paseo` provider is `pi` (its model and thinking) or a plain model slot; any other provider refuses with a FIX line. The spawn runs in the main root with argv `pi --print <prompt> --model <model> [--thinking <level>] --no-session --no-extensions -e <main root>/.pi/extensions/bee-guard --tools read,grep,find,ls,bash` and `BEE_SUPERVISOR_ALLOWED` set to the supervisor's allowlist through an env parameter on the spawn seam; `worker-guard` refuses any shell command outside that allowlist while the variable is set. The tick refuses when the extension directory is absent, and refuses an illegal runtime value, before any spawn. Revised after the hat wave (store `e2af1dab`). | `team.pi.supervisor` is dead config today. Pi `--tools` cannot limit shell commands, and extension discovery depends on the cwd, so the guard is loaded explicitly. |
| D10 | The Paseo channel concept states that `paseo run` from inside the leader records the leader as the parent, and that archiving the leader archives every in-flight worker with it. | A fact the own-ID guard does not cover; docs only. |

### Environment facts

- Paseo inspect output carries `Status`, `PendingPermissions`, `Model`, `Thinking`,
  `UpdatedAt` and `ParentAgentId` (slp research, `refs/paseo` `agent/inspect.ts`).
- Pi 0.87.1 CLI: `--print`, `--model <pattern>`, `--thinking <level>`,
  `--no-session`, `--tools <list>` (`pi/docs/cli.md`).
- Pi extensions load in print mode (`pi/docs/extensions.md:189`), and `-e` paths still load under `--no-extensions` (`pi/docs/cli.md:153`).
- The bee-guard extension is embedded into the binary by `build.rs`; its TS
  is tested from Rust through `node` (`tests/pi_paseo_heartbeat_contracts.rs`).

## Terms

| Term | Meaning in this feature |
|------|-------------------------|
| Silent idle | A Paseo worker that is `Idle` with no `result-N.json` for its round. |
| Broker tick | One run of `bee herding broker tick --json`: code only, no model call. |
| worker-guard | The new hook that judges a herded worker's or a Pi supervisor's shell commands. |

## Specific Ideas And References

- Research digests (2026-10-06): Seatworks practices (outbox nudge
  `turns.ts:203-233`, event-driven wake, heartbeat cleanup, doctor quirks) and slp
  practices (watchdog timeouts `watchdog.mjs:6-26`, model verification
  `paseo-team-lead/SKILL.md:116-152`, worker denylist
  `paseo-team-policy.ts:348-351`).
- Rejected: Seatworks' MCP-plus-spool tools, its over-gate landing, and slp's
  acting supervisor and in-turn `paseo send` messaging.
