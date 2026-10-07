# Practices borrowed from piggery — Context

**Feature slug:** piggery-borrow
**Date:** 2026-10-07
**Shaping session:** complete (gate bypass full; the user asked to do all seven items: "làm hết 7 cái đi")
**Scope:** Standard
**Domain types:** CALL | RUN

## Feature Boundary

A study of the piggery reference repo (`/home/thanhsmind/Projects/refs/piggery`,
v0.6.0, two advisor digests on 2026-10-06) found seven practices bee should
adopt or adapt. They were filed as backlog items p-596857ac, p-7fd7ef0e,
p-a0609bac, p-91e36c63, p-215ba2a9, p-5771b088 and p-cf8f7437. This feature
delivers all seven. The skipped ideas (a shared tools.json, peer mail routing
and rate limits, pi MCP for the belt, a TUI, YAML team templates) stay out.

## Locked Decisions

| ID | Decision | Rationale (only if it changes implementation) |
|----|----------|-----------------------------------------------|
| D1 | The pi belt records the outcome of each leader turn from `agent_before_settle`. At `agent_settled`, it deletes the in-flight result-inbox claims only when that outcome is `completed`. For any other outcome (aborted, error, or none seen), it requeues each claim with the existing `requeueClaim` and logs one advisory line. The 2-second drain also skips its tick while a UI prompt is open for the session (`promptDepths` above 0). (p-596857ac) | piggery acks mail only on a turn that ended ok (`internal/core/harness.go:269-313`). Today an Esc deletes a result the model may never have read (`events.ts:528`). |
| D2 | `bee herding cancel` stops the worker's whole process tree on unix. Before it closes the pane, it reads the tree below the captured foreground pid with one `ps -A -o pid=,ppid=,pgid=` call, sends SIGTERM to the tree, waits up to 2 seconds, re-reads, and sends SIGKILL to every member still alive. It then confirms exit as today. The 5-second fail-closed rule and `cancel_termination_failed` stay. Windows keeps today's behavior. (p-7fd7ef0e) | piggery reads the tree first because orphans reparent to init and cannot be found later (`internal/driver/local/proctree.go:10-14`). Detached children survive a pane close today (`control_loop.rs:768`). |
| D3 | `.bee/config.json` may set `herding.limits.concurrency` (live herding workers) and `herding.limits.depth` (nested `bee herding run` levels). Absent means no limit. `bee herding run` refuses before spawn with `{ok:false, reason:"limits.concurrency", live, limit, fix}` when live occupancy is at the limit, with `reason:"limits.unverifiable"` when occupancy is only a fallback count and a concurrency limit is set, and with `reason:"limits.depth"` when its own depth reaches the limit. Depth travels in `BEE_HERDING_DEPTH`, which each run exports to its child as its own depth plus one. `bee dispatch prepare` refuses the same way when the resolved role is herding-shaped. `bee dispatch prepare --explain` prints each check (role, claim, limits) with pass or refuse and writes nothing. A config with no `limits` key produces byte-identical payloads. Per-role `can_dispatch` is not built: paseo workers already cannot dispatch (`hooks/worker_guard.rs`), so it is filed as a backlog row. (p-a0609bac) | The four-slot cap lives only in a markdown role today; piggery refuses at the spawn door with a rule id (`internal/core/agent.go:93-110`). |
| D4 | The pi and opencode belts declare `BELT_CONTRACT_VERSION` and pass it on their session-init call as `--belt <pi\|opencode> --contract <N>`. The binary holds its own contract number. On a mismatch, session-init emits one advisory line naming the side to update: belt older means `bee onboard --apply`, binary older means the binary-freshness remedy. `bee doctor` gains a `belt_contract` row per belt found in the repo, read from the belt source. Bump rule: any change to a `bee hook` stdin shape or a belt-visible verdict shape bumps both. An old belt that passes no flag is accepted and reported as contract unknown. (p-91e36c63) | piggery's PROTOCOL_VERSION handshake tells the user which side to update (`extensions/pi/index.ts:134-142`). bee sniffs mismatches one feature at a time today (`commands.ts:51`). |
| D5 | The pi belt keeps its session state on one `globalThis[Symbol.for("bee.pi.state")]` slot, created with `??=`, so a `/reload` with a fresh module scope keeps it and session-init runs once. The leader decision (`isPaseoLeader`) is taken once per process. The belt then keeps `PASEO_AGENT_ID` in that state slot and removes it from `process.env`, so a child `pi` or `claude` started from the leader's shell never reads as a leader. Calls the belt makes to the bee CLI and to `paseo` still receive the id. `BEE_HERDING_WORKER` and `BEE_HERDING_JOB_ID` are never stripped. (p-215ba2a9) | piggery keeps per-process state on globalThis and strips its identity from children (`extensions/pi/index.ts:34-63`). |
| D6 | `bee herding status` adds `ctx_tokens` and `turns` per job to its JSON and a `ctx=12.3k turns=4` suffix to the plain line when known. bee reads them from the worker's own session transcript: the pi session file for a Paseo pi worker, or the pane log for herdr and tmux workers. When the source is missing or unreadable, both fields are `null` and the suffix is omitted. The pi belt's `bee-workers` widget shows the same suffix. Read-only; nothing gates on it. (p-5771b088) | `piggery top` shows a worker's context and turns, read from its transcript (`view/text.go:44-52`, `top.go:504-540`). |
| D7 | `bee doctor` adds one `runtime:<name>` row for each runtime bee supports (claude, codex, opencode, pi, paseo). Each row names the detected tool version (or not installed), whether the repo's bee wiring for it is installed and current, and on every not-ok row a `fix:` with the exact bee command. A runtime with no tool and no wiring is a plain info row, never a failure. (p-cf8f7437) | `piggery setup` prints one line per harness with its fix command (`internal/cli/setup.go:178-218`). |

## Environment facts

- `.pi/extensions/bee-guard/result-inbox.ts:159` already exports `requeueClaim`.
- `.pi/extensions/bee-guard/events.ts:450` already listens to `agent_before_settle`.
- `packages/bee-rs/crates/bee/src/herding/job_verbs.rs:317` holds `cancel_with_backends_and_timeout`.
- `packages/bee-rs/crates/bee/src/herding/wave.rs:1093` holds the occupancy verb, with live and fallback answers.

## Specific Ideas And References

- The two advisor digests from 2026-10-06 (study a: enforcement, contract version, identity; study b: ack, setup, top).
- Rejected: stripping the herding worker markers (inheritance is the mechanism); a shared tools.json (one consumer); MCP for the belt (it needs `ctx`); a TUI (the cockpit lives in waggledance).
