# Herding Limit Pause — Context

**Feature slug:** herding-limit-pause
**Date:** 2026-08-20
**Shaping session:** complete (user-directed use case, standard lane)
**Scope:** Quick

## The use case (live, 2026-08-20, job hws-1-r1)

A herded worker (claude-sonnet pane) finished its edits, then its SESSION hit a usage limit ("You've hit your session limit · resets 6:20pm"). The pane sat idle; `bee herding run` classified it as `timed_out_idle` after the idle window and exited 1. Nothing recorded that this was a LIMIT pause, nothing knew the reset time, and nothing resumed the session — even though the pane held a fully paid, resumable context. The orchestrator (and earlier the human) had to notice by reading the pane. A limit-stop is a pause, not a death; the flow must treat it as one.

## Locked Decisions

| ID | Decision | Rationale |
|----|----------|-----------|
| D1 | `wait_for_round` distinguishes a LIMIT pause from idle silence: when the heartbeat is stale AND the pane's recent text matches a known limit pattern (const list, initially Claude Code's "hit your session limit" / a generic "usage limit" family, case-insensitive, extensible per agent kind), the poll ends as a new typed outcome `paused_limit` — never `timed_out_idle` | The two need different operator responses: idle-timeout is forensics, limit-pause is "wait and resume" |
| D2 | `paused_limit` NEVER closes the pane (even under `--close-always`), exits with its own outcome string in the JSON result, and stamps `job.json` with `paused_limit_at` plus `limit_reset_hint` (the raw matched line — best-effort, no time parsing in v1) | The pane's paid context is the asset; the stamp is what a resume path or control-loop reads |
| D3 | `bee herding run --continue <job-id>` grows a same-round resume branch: when the job is stamped `paused_limit` and the recorded pane is alive, it re-delivers a resume nudge to the SAME round ("your session was paused by a usage limit; continue the task and write the round-N result file") through the standard state-receipt delivery, clears the pause stamp, and re-enters the round wait. The existing next-round continue contract is untouched (a prior result still routes to round N+1) | One verb, two branches keyed off recorded state — the operator's gesture stays `--continue <job-id>` either way |
| D4 | The dispatch/merge control-loop treats a `paused_limit` job as OCCUPYING its slot (the worker is coming back) and never re-dispatches its work while the stamp is live; documentation of the use case lands in the bee-herding knowledge area at capture | Re-dispatching a paused worker's cell duplicates work — the exact failure herding-worker-standalone D4 exists to prevent |

## Sequencing

`run.rs` is also touched by herding-worker-standalone (awaiting uat/merge) and by beehive-2c's bare-run agent-resolution shaping. Execution of this feature starts only after herding-worker-standalone lands on main; coordinate with beehive-2c through the store.

## Evidence

- Job hws-1-r1 pane text (w4:p12, since closed — a mistake recorded in session memory): worker done, blocked on result write, then "You've hit your session limit · resets 6:20pm (Asia/Bangkok)"; `bee herding run` output: `{"outcome":"timed_out_idle","pane_id":"w4:p12","closed_pane":false}`.
- `packages/bee-rs/crates/bee/src/herding/run.rs` — `wait_for_round`/`decide_poll` know only result/heartbeat/ceiling; no pane-text classification of the stall.
