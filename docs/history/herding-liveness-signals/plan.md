---
artifact_contract: bee-plan/v1
mode: standard
approved_gate2: 2026-08-20
---

# Plan: Herding Liveness Signals

Mode: `standard` — 2 risk flags: external-systems (a new `herdr` subcommand plus
`/proc`), cross-platform (`/proc` is Linux-only).
Why this is the least workflow that protects the work: the change rewrites the
stop conditions of a loop that decides when to kill running workers, so the shape
needs a written approach and a test matrix — but it touches one module, adds no
trust boundary, and removes no existing proof.

## Requirements (from CONTEXT.md)

- **D1** — the poll decides on a four-tier ladder: result (truth) → agent-process
  liveness → progress (mtime OR pane `revision` OR `/proc` CPU) → classification.
  A new typed `died` outcome fires when there is no result and no agent process.
- **D2** — the tier-1 liveness read fails OPEN: unreachable herdr reports
  `unknown`, never `died`. Opposite direction from `pane_alive`'s fail-closed gate.
- **D3** — `died` requires N consecutive absent-process observations.
- **D4** — `pane read` moves from every-tick to tier-3 on-demand only.
- **D5** — the herdr socket event stream is refused as a liveness source.

Every existing outcome keeps its current meaning: `timed_out_ceiling` still caps
regardless of activity, `paused_limit` keeps its herding-limit-pause D1 trigger,
and a well-formed result still outranks everything.

**One requirement does not survive review as written — see "Open decision (D6)".**
D1's `/proc` CPU branch cannot do the job D1 assigns it, and slice 2 is blocked on
the user's answer. Slices 1 and 3 are unaffected and stand on their own.

## Definitions pinned before slice 1

These three were left implicit in the decisions and are the difference between a
correct implementation and one that kills healthy jobs:

- **"Agent process present"** = `foreground_processes[]` contains at least one
  entry whose `pid != shell_pid`. NOT a name match against the agent kind: an
  agent that shells out to `cargo`, `git`, or a build puts a different `name` in
  the foreground group, and a name-matching check would read that as a death and
  kill a working agent within seconds.
- **Ladder position** = result → **ceiling** → died → idle/limit → continue. The
  `died` rung goes BETWEEN the ceiling check (`run.rs:530-531`) and the stale
  heartbeat check (`run.rs:533`), so `timed_out_ceiling` keeps winning every tie,
  exactly as CONTEXT.md's outcome table states.
- **Debounce (D3)** = the liveness read runs every 10th tick (~2s, not every
  200ms — it spawns a subprocess), N = 3 consecutive `Absent` reads, so a real
  death is reported in ~6s instead of 900s. `Unknown` **RESETS the counter to
  zero** — it does not increment it and does not hold it. D3 says *consecutive*,
  and a counter that treats `Unknown` as "not Alive" would fire `died` off an
  Absent → Unknown → Absent interleave, which is precisely the fail-open
  violation D2 forbids.

## Discovery

- `decide_poll` (`run.rs:518`) takes result → ceiling → stale-heartbeat →
  continue, and has no process-liveness input at all. `pane_alive` exists
  (`run.rs:454`) but is called only at pane resolution (`run.rs:1207`,
  `run.rs:1320`), never inside the tick.
- The production tick (`run.rs:915-928`) computes the heartbeat as `log.txt`
  mtime OR `agent_status == "working"`, and calls `pane_read` unconditionally —
  though `pane_text` is consumed only inside the idle-timeout branch
  (`run.rs:534-539`).
- `herdr pane process-info` returns `foreground_processes[]` with `name`, `argv`,
  `pid`, `shell_pid`; `herdr pane list` rows carry a monotonic `revision`.
  Both verified live, 2026-08-20.
- `/proc/<pid>/stat` fields 14/15 (`utime`/`stime`) advance on a working agent:
  two samples two seconds apart on pid 2898247 read `utime=12313 stime=4012`
  then `utime=12315 stime=4013`. The process *state* letter stayed `S` through
  both — **state is not a progress signal, the CPU delta is.**
- **The poll outcome never reaches dispatch.jsonl or the wave ledger.**
  `record_dispatch` (`run.rs:810-841`) fires once right after `agent start`
  succeeds (`run.rs:1088`, `run.rs:1352`) — before the poll — and writes the
  worker row with `outcome: None` (`run.rs:830`). The single string authority for
  a run outcome is `outcome_label` (`run.rs:1406-1420`). Ledger resolution is a
  read-time fold under the same `wave_id` (`wave.rs:817-819`), using the separate
  `bee herding wave` vocabulary (`wave.rs:407`), not these strings.
- Adding a `PollDecision`/`RunOutcome` variant has an exact precedent in this
  same enum: `paused_limit`, cell `hlp-1`
  (`.bee/cells/archive/herding-limit-pause/hlp-1.json:44`).

## Approach

Recommended path (cites `6194c0f2` D1–D5): grow the poll's observation struct
from two booleans into a tiered observation, add one `Herdr` trait method for
`pane process-info`, and keep `decide_poll` pure so the whole ladder stays unit
testable with a simulated clock — the property the current tests already rely on.
The tiers are added in dependency order so each slice is independently green.

Rejected alternatives:

- Socket subscription to `pane.agent_status_changed` — refused, D5.
- Shortening `DEFAULT_IDLE_TIMEOUT_SECS` — penalises legitimately quiet long work
  and still cannot see a hang.
- Calling the existing fail-closed `pane_alive` inside the tick — wrong fail
  direction for a kill decision, D2.

Risk map:

| Component | Risk | Reason | Proof needed |
|---|---|---|---|
| `decide_poll` ladder ordering | MEDIUM | Ceiling/idle/limit precedence is already load-bearing. Note the existing tests CANNOT catch a misplaced rung — none of them supplies a process input, so they pass with `died` on either side of the ceiling check | A NEW case: ceiling passed AND absent×N in the same tick must assert `TimedOutCeiling`. That case, not the old ones, is the ordering proof |
| Defining "absent" (see Definitions) | HIGH | A name-matching liveness check reads an agent's own `cargo`/`git` child as a death and kills it in seconds | A test where the foreground process is a non-agent pid != `shell_pid` and the verdict is "continue" |
| Fail-open liveness (D2) | HIGH | Getting the direction backwards kills healthy multi-hour jobs — the exact failure this feature must not introduce | A test where the process read ERRORS and the verdict is "continue", never `died` |
| Debounce (D3) | MEDIUM | An off-by-one makes a single flaky read fatal | A test with N-1 absent reads asserting "continue", the Nth asserting `died` |
| `/proc` absence off Linux | — | Slice-2 only, and slice 2 is blocked on D6. No `/proc` read enters slices 1 or 3 | Deferred with slice 2 |
| `pane read` moving to on-demand (D4) | LOW | `paused_limit` classification must still fire at the same moment it does today | Existing `paused_limit` test stays green |
| New `died` outcome string | LOW | Additive, and no schema or full-set test pins the vocabulary — the risk is a missed exhaustive match, not a broken reader | `outcome_label`, `emit_result`, `exit_code_for` and the should-close gating all extended; the string asserted in the run's JSON result |
| Windows CI stays green | LOW (was MEDIUM) | D19 (`d891fc43`) requires the suite to run unexcluded on `windows-latest`. With `/proc` deferred to slice 2, slices 1 and 3 add only a herdr subprocess call, which is already platform-portable | `cargo test --release` green with no `#[cfg]`-excluded test |

## Shape

Phase plan — three slices, each independently green and demonstrable.

| Phase | What Changes | Why Now | Demo | Unlocks |
|---|---|---|---|---|
| 1 | `Herdr::process_info` (new trait method + `RealHerdr` + `FakeHerdr` + `PanicHerdr`), returning a tri-state `Alive(pid)` / `Absent` / `Unknown`; tier-1 rung wired into `decide_poll` behind the D3 debounce; new `died` variant. Exhaustive matches that MUST be extended: the three `PollDecision`→`RunOutcome` matches (`run.rs:1106`, `1255`, `1378`), `outcome_label` (`1406-1420`), and `emit_result`'s payload match (`1441-1466`, whose final arm names its variants with no catch-all — `died` carries the vanished pid as forensics). Sites that must be VERIFIED and left alone: `exit_code_for` (`1422-1428`) already lands `died` on `FAILURE` through its catch-all, and the close gating (`run.rs:1117-1118`) already keeps the pane open unless `--close-always`, which is the wanted forensics behaviour | Slice 1 is the walking skeleton — it is the whole point of the feature (a death seen in one tick instead of 900s) and it exercises every layer end to end | A killed agent process reports `died` on the next tick, not after the idle window | Everything else; the ladder's shape exists after this |
| 2 | **BLOCKED — see "Open decision (D6)".** Tier-2 progress widened with a hang-detecting signal | The hang case (D1's second failure) needs a progress signal a spinner cannot fake. Review established that `/proc` CPU delta is not that signal | — | The hung-worker verdict |
| 3 | `pane read` moves to tier-3 on-demand: pulled once when tier 2 goes stale, for `LIMIT_PATTERNS` classification only. Independent of slice 2 | Pure cleanup on top of slice 1's ladder | `paused_limit` still fires identically, with one `pane read` per stall instead of ~4,500 | — |

Tier 2 keeps its CURRENT membership through slices 1 and 3: `log.txt` mtime OR
`agent_status == "working"` (`run.rs:915-926`). `agent_status` is deliberately NOT
dropped — D1 listed tier-2 sources without addressing it, and removing it would
change today's verdict for an mtime-quiet agent that herdr reports as working.

Slice 1 is the current slice. Slice 3 stays a one-line headline until it lands.

## Test matrix

The triad, at its smallest demonstrating size. Each cell's writer judges existing
coverage first (`.bee/expertise/tests.md`) and authors only the gap.

**The regression proof, stated precisely.** `decide_poll` gains a parameter, so
its seven existing direct callers (`run.rs:1515`, `1520`, `1525`, `1530`, `1538`,
`1550`, `1558`) plus the production caller (`run.rs:581`) must all be edited. What
must stay **byte-identical is every existing assertion**: each test passes the new
parameter at its neutral value (`Unknown` liveness, zero absent-observations) and
still expects the verdict it expects today. This proves the neutral path is
untouched — it does NOT prove the rung sits at the right height, because no
existing test supplies a process input at all. Ordering is proved only by the new
ceiling-plus-absent case below. The three `run_poll_loop` tests
(`run.rs:1566`, `1589`, `1607`) and the five `FakeHerdr` end-to-end tests
(`run.rs:2159`, `2171`, `2184`, `2203`, `2217`) get the same treatment through
`PollTick` and the fake.

**Happy path**
- Result present + process absent → still `ResultReady` (tier 0 outranks tier 1).
- Process alive + progress fresh → continue across many ticks.

**Edge cases**
- N-1 consecutive absent process reads → continue; the Nth → `died` (D3).
- **Interleave: Absent×(N-1) → Unknown → Absent → continue**, counter reset, no
  `died`. This is the case that pins "consecutive" and it is the one an
  increment-on-`!Alive` implementation silently fails.
- **Ordering: ceiling passed AND absent×N in the same tick → `TimedOutCeiling`.**
  The only case that proves the rung sits below the ceiling check.
- Process absent but a result lands in the same tick → `ResultReady`, not `died`.
- Foreground process present with a non-agent `name` and `pid != shell_pid` (the
  agent shelled out to a build) → continue, never `died`.
- Only `shell_pid` in the foreground group → Absent (this is the real death shape:
  the agent exited and the pane fell back to its shell).
- A `died` verdict leaves the pane OPEN as forensics; `--close-always` closes it.

**Error paths**
- `process_info` returns an error / herdr unreachable → `Unknown` → continue,
  never `died` (D2 — the highest-risk case in the matrix).
- Malformed `process-info` JSON → `Unknown`, same as above.
- `pane read` fails at tier 3 → falls back to `timed_out_idle`, never a panic.

## Open decision (D6) — blocks slice 2 only

Review killed D1's CPU branch on its own evidence. `/proc` CPU delta cannot
separate a working agent from a hung one, in either possible spelling:

- **As an OR-branch (what D1 says):** a TUI agent's event loop burns CPU while it
  sits blocked — the plan's own measurement (`utime` 12313→12315 on a process in
  state `S`) is that fact. A hung agent redrawing a spinner burns CPU the same
  way, so any-delta-above-zero never goes stale and the hang is never caught.
- **As an override:** an agent legitimately blocked for minutes on an LLM API call
  has near-flat CPU. Overriding a fresh mtime with flat CPU kills it. That is the
  exact failure D2 and D3 exist to prevent.

Widening tier 2 with the pane `revision` counter does not rescue slice 2 either —
a spinner advances `revision` for the same reason it advances mtime.

So slice 2 has no working mechanism, and picking one needs calibration data this
repo does not have yet: real CPU and output traces from healthy-but-blocked
workers versus genuinely hung ones. The question for the user is what happens to
it — recorded here rather than resolved silently, because shrinking a locked
decision is not the agent's call (AGENTS.md, bee-planning "Scope integrity").

## Out of scope

- Any resume or restart behaviour on `died` — this feature reports the outcome; a
  control-loop response to it is separate work.
- Fixing `--continue` after a `died` job. Known consequence, named so the cell
  writer is not surprised: the pane survives the agent, so `pane_alive` passes
  (`run.rs:1319-1322`) and `deliver_pointer` fires 30 pointer prompts at a shell
  prompt before failing as a generic `SpawnFailed` (`run.rs:674-686`). Ugly, and
  no worse than today.
- Parsing a limit's reset time (still herding-limit-pause D2's best-effort hint).
- Windows/macOS process inspection beyond degrading the `/proc` branch to absent.
- Replacing the poll with an event stream — refused, D5.
