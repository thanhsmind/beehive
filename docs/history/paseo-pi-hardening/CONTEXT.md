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
| D2 | A Paseo worker that is `Idle` with no `result-N.json` for its round, after the round has seen it `Working`, gets ONE nudge: a `paseo send` that names its brief and the result file it owes. The send is safe because the agent is idle (paseo-pi D5 holds: never into a running turn). A second idle-with-no-result ends the round as the existing idle-timeout outcome, with a message that names the silent idle; the agent is kept. No new outcome word. | Cheap models end a turn without writing the verdict; today that costs the full 900 s idle timeout. One nudge is not a retry, so herding-cockpit-completeness 9615be76 (no automatic behavior) stands. |
| D3 | For a `Working` Paseo agent, the inspect `UpdatedAt` field is the liveness heartbeat: the tick counts as fresh only when `UpdatedAt` moved since the last read. When `UpdatedAt` is absent or unparseable, `Working` counts as fresh, as today. `bee herding status` shows the existing `stalled` word for a Paseo worker whose `UpdatedAt` is older than the shared 120 s freshness constant. Observe only: nothing is stopped or archived. | Today `Working` always sets the heartbeat fresh, so a stuck turn is caught only by the 6 h ceiling. |
| D4 | After a Paseo spawn, bee reads the inspect `Model` and `Thinking` fields once the agent answers, writes them into `job.json` as `paseo_model_observed` and `paseo_thinking_observed`, and compares them with the configured `paseo` block. A confirmed mismatch prints one warning line and `bee herding status` shows `model_mismatch: true`. A missing field is recorded as unverified and warns nothing. bee never archives, stops or retries on a mismatch. | Catches a silent model fallback on cheap routes without adding automatic behavior. |
| D5 | A herded worker gets one narrow guard. Under `BEE_HERDING_WORKER`, the hook `worker-guard` is the second hook that passes the marker short-circuit (after `activity`). It judges a shell command only and refuses: a git push, a GitHub write, an agent launch (the worker-outward forms, judged regardless of the working directory), a `paseo` command head, and the bee verbs `dispatch`, `herding run`, `worktree merge` and `gate`. Everything else is allowed, and every other hook still exits 0 under the marker. The Pi belt sends a herded worker's shell calls to this hook and blocks on a deny, a crash or a missing binary (fail closed). | Pi workers on Paseo run with no guard today (`hooks/mod.rs` `marker_short_circuits`). This keeps the worker standalone (herding-worker-standalone D3) while closing the outward and self-dispatch holes. |
| D6 | The Pi leader's extension deletes its own `bee-leader` heartbeat (`paseo heartbeat delete <id>`) and its marker file on `session_shutdown` for every reason except `reload`. | Nothing deletes a heartbeat today; an orphan cron can fire at an archived leader and wake it again. |
| D7 | On the pi runtime, `bee doctor` adds one `paseo_ready` row when any `team.pi` slot names a herding agent with a `paseo` block. It checks: the daemon answers, the CLI version is at least 0.10.3, `~/.pi/agent/auth.json` exists when a configured provider is `pi`, and no heartbeat marker names an agent that `paseo ls` no longer lists. Report only; it fixes nothing. | A setup fault shows before a worker fails. |
| D8 | The Pi leader's extension runs `bee herding broker tick --json` on its own timer, every `herding.paseo.broker_tick_secs` seconds (default 30), and starts a model turn with `sendUserMessage` only when the tick reports news (steer when the leader is busy, as the result drain does). The Paseo heartbeat stays as a backstop, and its default cron becomes `*/30 * * * *`. paseo-pi D6 stands unchanged: a heartbeat prompt is never swallowed. Supersedes paseo-pi D7 (store `8ae5134d`, default every 5 minutes). | News arrives in seconds, and the empty paid turns drop from about 288 to about 48 a day. Spike 2 (store `da10d6ad`) showed an extension timer that calls `sendUserMessage` starts a turn Paseo shows and settles. |
| D9 | `herding.supervisor_runtime` picks the supervisor's team table: `claude` (absent means this, unchanged) or `pi`. With `pi`, the model comes from `team.pi.supervisor` (a herding agent's `paseo` model and thinking, or a plain model slot), and the default argv is `pi --print <prompt> --model <model> [--thinking <level>] --no-session --tools read,grep,find,ls,bash`, run with `BEE_SUPERVISOR_ALLOWED` set to the supervisor's allowlist. `worker-guard` refuses any shell command outside that allowlist while that variable is set. An illegal value refuses before any spawn. | `team.pi.supervisor` is dead config today (`control_loop.rs` reads `team.claude` only). Pi `--tools` cannot limit shell commands, so the allowlist rides the one guard D5 adds. |
| D10 | The Paseo channel concept states that `paseo run` from inside the leader records the leader as the parent, and that archiving the leader archives every in-flight worker with it. | A fact the own-ID guard does not cover; docs only. |

### Environment facts

- Paseo inspect output carries `Status`, `PendingPermissions`, `Model`, `Thinking`,
  `UpdatedAt` and `ParentAgentId` (slp research, `refs/paseo` `agent/inspect.ts`).
- Pi 0.87.1 CLI: `--print`, `--model <pattern>`, `--thinking <level>`,
  `--no-session`, `--tools <list>` (`pi/docs/cli.md`).
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
