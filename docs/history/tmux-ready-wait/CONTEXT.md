# tmux Ready Wait — Context

**Feature slug:** tmux-ready-wait
**Date:** 2026-08-23
**Lane:** tiny (bugfix)

## Asked

Run the tmux transport live (phase 3 of `docs/history/tmux-herding-transport/plan.md`).

## Found

Live proof in a detached tmux session (`beeproof`, 2026-08-23): `bee herding status`, every `bee herding pane …` verb, `pane list --with-status`, `run --dry-run` and `bootstrap-cockpit.sh --dry-run` all correct on tmux. The real `bee herding run` split pane %3, started Claude Code, which booted to its idle `❯` prompt — but the run ended `spawn_failed: agent never reported ready within 60s`. Cause: `run.rs` polls `agent_wait(job, 200 ms)` (`POLL_INTERVAL`), and `RealTmux::agent_wait` (`packages/bee-rs/crates/bee/src/herding/tmux.rs`) restarts its stability window (3 unchanged reads × 2000 ms) on every call, so it can never return `idle` inside a 200 ms call.

## Locked

| ID | Decision |
|----|----------|
| D1 | `RealTmux::agent_wait` keeps its stability state per pane across calls (last screen, unchanged count, first-unchanged time); short per-call timeouts still reach `idle` once the screen has been unchanged for `quiet_cycles` reads spanning ≥ `quiet_cycles × interval_ms`. A long single call behaves as before. |

## Will do

One cell on `tmux.rs`: the cross-call stability state plus a test driving `agent_wait` with 200 ms calls against a stub that returns an unchanged screen. Then re-run the live proof and record it.
