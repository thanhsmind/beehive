# Herding Liveness Signals — Context

**Feature slug:** herding-liveness-signals
**Date:** 2026-08-20
**Shaping session:** complete (user-directed design discussion, standard lane)
**Decision store:** D1–D6, scope `herding-liveness-signals`, tags `orchestration`,
`workers`. The active record supersedes `6194c0f2` — the plan review disproved
D1's CPU branch, so D1 is narrowed and D6 records the refusal.
**Scope:** Quick

## The use case

`bee herding run`'s poll loop has exactly one way to stop a worker that has not
written a result: silence. `decide_poll`
(`packages/bee-rs/crates/bee/src/herding/run.rs:518`) reduces every non-result
outcome to "the heartbeat went stale" or "the ceiling passed". Two real failures
fall through that model:

1. **A hard agent death costs a full idle timeout to notice.** The agent process
   exits; the pane survives and drops back to a shell prompt. Nothing observes
   the exit, so the run waits out `DEFAULT_IDLE_TIMEOUT_SECS` (900s, `run.rs:55`)
   before reporting `timed_out_idle` — a 15-minute blind window on an outcome
   that is knowable in one tick.
2. **A hung agent that keeps writing is never noticed at all.** The heartbeat is
   `log.txt` mtime OR `agent_status == "working"` (`run.rs:915-926`). A spinner
   or a status line satisfies both forever. "Still emitting bytes" is not
   "still making progress".

A third, smaller finding sits alongside them: the tick calls `herdr pane read`
unconditionally every 200ms (`run.rs:927`), pulling full screen text, while
`pane_text` is consumed only inside `decide_poll`'s idle-timeout branch
(`run.rs:534-539`). At `POLL_INTERVAL = 200ms` that is roughly 4,500 discarded
subprocess spawns per worker per idle window.

Of the two facts silence cannot fake, only one survives review. **Process
identity** works and is already reachable: `herdr pane process-info` returns
`foreground_processes[]` with `name`, `argv`, `pid`, and `shell_pid` (verified
live, 2026-08-20). **CPU accounting** was the intended answer to failure 2 and it
does not work — see D6. Failure 2 is therefore recorded and parked, not solved
here; failure 1 and the wasted `pane read` are.

## Locked Decisions

| ID | Decision | Rationale |
|----|----------|-----------|
| D1 | The poll decides on a **signal ladder**, not one signal. **Tier 0 — truth:** mailbox `result-N.json` with `round >= min_round`; a well-formed result outranks every other tier and short-circuits the ladder. **Tier 1 — liveness:** `herdr pane process-info`'s `foreground_processes`. **Agent-present** = at least one entry whose `pid != shell_pid` — NEVER a name match against the agent kind. No result AND no agent process is a new typed `died` outcome. The `died` rung sits **between the ceiling check and the stale-heartbeat check**, so `timed_out_ceiling` keeps winning every tie. **Tier 2 — progress:** keeps its CURRENT membership, `log.txt` mtime OR `agent_status == "working"` (see D6 — the third source originally named here does not work). **Tier 3 — classification:** runs only when tier 2 goes stale; `pane read` is pulled once at that moment to match `LIMIT_PATTERNS` for `paused_limit` vs `timed_out_idle` | Pane liveness is not agent liveness — an exited agent leaves a live pane at a shell prompt, so `pane list` cannot see the death. A name match would read an agent's own `cargo`/`git` child as a death and kill a working agent in seconds |
| D2 | The tier-1 liveness read **fails OPEN**: an unreachable or erroring herdr reports `unknown`, never `died`. This is the OPPOSITE direction from `pane_alive` (`run.rs:454`), which fails CLOSED by design | The two reads answer different questions. `pane_alive` is a refusal gate — refusing to dispatch on bad information is safe. A liveness check is a kill decision — a herdr hiccup must never end a healthy multi-hour job |
| D3 | `died` requires **N consecutive absent-process observations**: N = 3, read every 10th tick (~2s, since each read spawns a subprocess), so a real death is reported in ~6s instead of 900s. An `Unknown` read **RESETS the counter to zero** — it neither increments nor holds it | Debounce is the difference between "the agent exited" and "one read was flaky". Without the reset rule an `Absent → Unknown → Absent` interleave fires `died` off non-consecutive reads, which is the exact fail-open violation D2 forbids |
| D4 | `pane read` moves from every-tick to **tier-3 on-demand only** | `pane_text` is already consumed solely inside the idle-timeout branch; the per-tick pull is discarded work |
| D5 | herdr's `pane.agent_status_changed` **socket event stream is refused** as a liveness source | It is edge-triggered, so a missed edge stays wrong forever; it goes silent indistinguishably from "working" when the daemon dies; it carries the same heuristic `agent_status` underneath; and it says nothing about `result-N.json`, which is the actual truth. Poll is level-triggered and self-correcting, and `POLL_INTERVAL` is already 200ms — the latency it would buy back is under one tick |

| D6 | `/proc` CPU delta is **refused** as the hang-detecting progress signal, and hang detection is **parked without a mechanism** against its registered trigger. The feature ships the death verdict (D1 tier 1) and the `pane read` economy (D4) without it | As an OR-branch it never goes stale: a TUI agent's event loop burns CPU while blocked (measured — `utime` 12313→12315 on a process in state `S`) and a hung agent's spinner redraw burns it identically. As an override it kills an agent legitimately blocked for minutes on an LLM API call with near-flat CPU. The pane `revision` counter fails the same way a spinner advances mtime. Picking a real threshold needs calibration traces the repo does not have |

## Outcome table (D1, resolved)

| result | agent process | progress | outcome |
|---|---|---|---|
| present | — | — | `done` / `blocked`, per the result document |
| absent | absent (×N, D3) | — | `died` — **new** |
| absent | present | fresh | continue |
| absent | present | stale + pane text matches `LIMIT_PATTERNS` | `paused_limit` (herding-limit-pause D1) |
| absent | present | stale | `timed_out_idle` |
| absent | unknown (D2) | — | treated as present — never `died` |
| — | — | — | `timed_out_ceiling` still caps regardless (`run.rs:530-531`) |

## Cross-platform constraint (confirmed)

The repo supports Linux/WSL2 **and** Windows
(`docs/knowledge/areas/workflow-state/claims-and-ownership.md:47`), and
herding-orchestration D19 (`d891fc43`) settles the shape: the *mechanism* must
stay proven on Windows — the whole suite runs unexcluded on a `windows-latest`
CI lane, pinned by platform-portable tests — while a *live* run is accepted as
Linux-only, owner-run. macOS is unaddressed by any recorded decision.

With D6 parking the `/proc` read, nothing in this feature is Linux-only: the work
adds one more `herdr` subprocess call, which is as portable as the nine already
there. The constraint survives as a rule for the parked work — when hang
detection returns, its platform-specific source must degrade to absent on
Windows without excluding a single test from the `windows-latest` lane.

## Evidence

- `packages/bee-rs/crates/bee/src/herding/run.rs:518` — `decide_poll`: result →
  ceiling → stale-heartbeat → continue. No process-liveness input exists.
- `packages/bee-rs/crates/bee/src/herding/run.rs:915-928` — the production tick:
  heartbeat is mtime OR `agent_status == "working"`; `pane_read` is
  unconditional.
- `packages/bee-rs/crates/bee/src/herding/run.rs:454` — `pane_alive`, fail-closed,
  called only at pane resolution (`run.rs:1207`, `run.rs:1320`), never in the tick.
- Live `herdr pane process-info --current` (2026-08-20):
  `{"foreground_processes":[{"argv":["claude"],"cmdline":"claude","name":"claude","pid":2898247}],"shell_pid":5952}`.
- Live `herdr pane list` (2026-08-20): each pane row carries a monotonic
  `revision` counter (observed 278 and 17988 on two active panes).
