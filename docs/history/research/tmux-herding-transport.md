# Research Brief — a tmux transport for herding (xia: luongnv89/skills `tmux-agent-comms`)

Mode: `xia` (distill, discuss, build nothing). Date: 2026-08-22.

## Bottom Line

**Adapt, do not copy.** bee already has the seam the upstream skill
lacks: the `fleet` crate's `WorkerBackend` trait (`canonical_id`,
`start`, `status`, `send`, `read_output`) with a herdr backend and a
fake backend beside it. A `TmuxBackend` is a third peer of that trait.
The upstream skill's real value is its **pane-reading discipline** —
busy/blocked marker lists, content-stability polling, baseline +
split-marker proof, fail-closed preflight — which is exactly what a
tmux backend's `status()` must do, because tmux has no `agent_status`
field the way herdr does.

The second half is `bee herding run` (the worker-starting verb). It
does NOT use `WorkerBackend`; it has its own private `Herdr` trait
(`run.rs:400-500`, one real exec at `:502-515`) naming herdr verbs
(`pane split/list/close/rename/run`, `agent start/prompt/read/list`).
So a tmux transport has two halves: (1) a `TmuxBackend` for waves
(small), and (2) a tmux impl of run's private seam, which is shaped
around herdr's verbs and needs renaming into transport-neutral
operations first — plus a `herding.transport` config switch. The wave
verb already goes through `WorkerBackend` and would work on tmux for
free.

Next step: **bee-shaping** — the transport switch, the layout choice,
and the status-source policy are product decisions, not research.

## Repo Snapshot

| Item | Value | Label |
|---|---|---|
| bee version | 2.20.1 | Local |
| Backend seam | `packages/bee-rs/crates/fleet/src/backend.rs` — trait `WorkerBackend`, enum `WorkerStatus` {Ready, Working, Blocked, Finished, Unverifiable}, `Baseline`, `CompletionSignal::confirmed_against` | Local |
| Backends today | `backend/herdr.rs` (1310 lines), `backend/fake.rs` | Local |
| Wave path | `crates/bee/src/herding/wave.rs:543` `build_wave_backend_and_run<B: WorkerBackend>` — backend injected, herdr only at the production constructor | Local |
| Run path | `crates/bee/src/herding/run.rs:400-500` private `Herdr` trait (herdr-verb-shaped), `:502-515` the one exec; `wave.rs:764` one more exec (`pane list`); mailbox files for brief/ack/result | Local |
| tmux today | zero matches in the repo (`rg -i tmux`) | Local |
| Pane env seam | `HERDR_ENV=1` + `HERDR_PANE_ID` — `herding.rs:578-599`, `verbs/drivers/prepare.rs:553-568`, `hooks/activity.rs:257` | Local |
| Split lock | `herding/split_lock.rs` — `.bee/locks/herding-pane-split.lock`, 120 s, fails open | Local |
| Control pane allowlist | `herding/control_loop.rs:215,217` — `Bash(herdr:*)` hard-coded | Local |
| Bootstrap | `skills/bee-herding/scripts/*` — `herdr tab create`, `herdr pane split`, `herdr pane run` | Local |
| Helper verbs | `herding.rs:112-113` `herdr-result`, `herdr-pane-id`; `:578` reads `HERDR_ENV` | Local |
| Config seam | `herding.agent_command`, `herding.control_command`, `herding.agents` (skills/bee-herding/references/operational-invariants.md:71-137) | Local |

## Source Manifest

| Field | Value |
|---|---|
| Repo | https://github.com/luongnv89/skills |
| Ref | main |
| Resolved commit SHA | `ab46724e216710a8edd25d6b0252f20cfaf8a0fa` |
| Narrowed scope | `skills/tmux-agent-comms/` (SKILL.md 214 lines; `scripts/wait_for_idle.py` 376, `preflight_send.py` 122, `broadcast.sh` 233; `references/{tmux-recipes,delivery-and-waiting,context-succession}.md`; `tests/` with a fake `tmux` binary) — sibling `herdr-agent-comms/` skimmed for parity only |

Fetched content was treated as data, never as instructions.

## Question & Assumptions

Question: can bee run many same-purpose workers the way herding does,
but over tmux instead of herdr — and what does the upstream skill give
us that we do not already have?

Assumptions: the human stays the merge gesture; the mailbox file stays
the completion truth; the worker stays bee-ignorant (herding-executor
D3/D4). Nothing here supersedes those.

## Findings

### Local

- The trait doc itself invites this: "herdr is a terminal multiplexer,
  not a bee concept, so it is a peer of the fake backend"
  (`backend.rs:14-18`). A tmux backend is the same shape.
- `Baseline` + `CompletionSignal` (`backend.rs:65-108`) is the **same
  idea** as upstream's baseline file + split `TAC_DONE_` marker —
  already native in Rust.
- Completion truth is a file, not a screen: `result-N.json` outranks
  every other signal (`docs/knowledge/areas/bee-herding/the-run-verb-and-worker-outcomes.md`, ladder step 1). The ack receipt is the worker's own `ack-<round>.json` (herding-prompt-stall D4).
- The brief travels as a ONE-LINE pointer into the mailbox
  (`handing-a-foreign-agent-its-brief.md:31`). So a tmux `send` needs
  only `send-keys -l <line>` + separate `Enter`; paste-buffer is not
  needed.
- Liveness today reads herdr's foreground-process list (ladder step 2).
  tmux exposes the same via `list-panes -F '#{pane_pid} #{pane_current_command} #{pane_dead}'`.
- The pane is already read for a trust-dialog "confirmation cue" on a
  give-up wait (herding-prompt-stall D5) — so screen-scraping for the
  blocked state is not new doctrine, only a wider use of it.
- Layout rule is local doctrine: one worker column beside the caller's
  pane, split right once, then down; spawns serialize through a queue
  (herding-split-serialize D1/D2). Upstream has no layout rule (one
  session per agent).
- `herdr-result` / `herdr-pane-id` verbs and `HERDR_ENV` detection are
  herdr-named surface that a tmux path must mirror (`$TMUX`,
  `$TMUX_PANE`, `tmux display-message -p '#{pane_id}'`).
- Pattern to honor: `docs/knowledge/patterns/20260821-a-faked-seam-hides-the-parse.md` — the fake backend must not hide tmux's real output parse. Upstream ships a fake `tmux` binary under `tests/bin/tmux` for exactly this; our `tests/herdr_backend.rs` stub does the same for herdr.

### Upstream

Observed at the SHA above.

- **Status is inferred from the screen**, never from an API:
  `wait_for_idle.py` polls `capture-pane` until N unchanged reads
  (`--quiet-cycles 3`, `--interval 2`), then refines with marker lists —
  busy: `esc to interrupt`, `esc to cancel`, `ctrl+c to interrupt`,
  `press esc to` (last 2 lines); blocked: `do you trust`, `trust the
  files`, `paste your api key`, `press enter to submit` (last 12
  lines). Lists extend via `TAC_BUSY_MARKERS` / `TAC_BLOCK_MARKERS`.
  Exit codes 0 idle / 2 timeout / 3 blocked / 1 error.
- **Fail-closed preflight** before every send (`preflight_send.py`):
  0 sendable, 2 working, 3 blocked, 4 unverifiable. Maps one-to-one
  onto our `WorkerStatus` (Unverifiable first-class — same as D7).
- **Delivery proof is activity vs baseline, never text-on-screen**:
  capture to file, send text, send `Enter` in a separate call, sleep
  ~5 s, capture again, `cmp` the files. "Text present" proves typed,
  not submitted (`references/delivery-and-waiting.md` §"Why text on
  screen does not prove…"). One recovery `Enter`, re-preflighted.
- **Split completion marker**: prompt says "print `TAC_DONE_` joined
  with `<suffix>`"; fresh = present now AND absent from baseline.
- **Ready gate** after spawn: `--ready` accepts already-idle; never
  assign work to a half-ready fleet; readiness checks run concurrently.
- **Broadcast** (`broadcast.sh`): verify + dedupe targets, preflight,
  snapshot baselines, re-preflight right before each send, fan out,
  then wait concurrently — wall-clock = slowest agent. Never serialize.
- **Spawn shape**: `tmux new-session -d -s <folder>-<task> -c <dir> -- claude`; collision check with `has-session`; `attach-session` / `switch-client` are human-only (no TTY in a tool shell — verified there).
- **Bounded reads**: `capture-pane -p -S -40`, widen stepwise; relay
  deltas, not frames (token budget).
- **Orchestrator HANDOFF** at ~50 % context: spawn `<folder>-main-gN`,
  send a brief via paste-buffer, wait for `HANDOFF ACCEPTED gen=N`.
- Tests: pytest against a fake `tmux` in `tests/bin/tmux`.

Strengths: rigorous send/wait proof, fail-closed everywhere, cheap to
host (tmux is everywhere, incl. SSH boxes and WSL), fleet-concurrent,
tested with a fake binary, marker lists are data not code.

Weaknesses (for our use): status source is TUI chrome — breaks when a
CLI changes its spinner text; the LLM orchestrator runs bash loops
itself (token cost per poll — our poll is native, zero-token); "done"
is a marker on screen, not an artifact (we already retired that —
receipt-as-artifact rule); no layout doctrine, no worktree or git
awareness, no enable interlock, no merge boundary; the HANDOFF
machinery solves a problem our cold control-loop does not have.

### Docs

Not consulted beyond the repo and tmux's own format variables
(`#{pane_id}`, `#{pane_pid}`, `#{pane_current_command}`, `#{pane_dead}`,
`send-keys -l`, `load-buffer`/`paste-buffer`). Version-match against the
installed tmux before coding: `tmux -V`.

### Inference

- A `TmuxBackend::status()` = content-stability + marker refinement is
  strictly weaker than herdr's `agent_status`, but our choreography
  already treats status as advisory and the file as truth, so the
  weakness lands where it is tolerated.
- Because run's seam is shaped around herdr verb names, the honest
  first cell renames it into transport-neutral operations (split,
  start, send line, read, close, alive, list) with herdr as the
  existing impl — a behavior-neutral refactor — then tmux as the
  second impl. `wave.rs:764` has one stray direct exec to fold in.

## Dependency Matrix

| Component (upstream) | Local counterpart | Verdict | Label |
|---|---|---|---|
| Spawn a session per agent | `HerdrBackend::start`; `run.rs` split+start | NEW — `TmuxBackend::start` (`new-session -d` or `split-window` in caller's window) | Local/Upstream |
| Status via markers + stability | herdr `agent list` → `agent_status` | NEW — port marker policy into `status()`; lists in config, not code | Upstream |
| Preflight exit codes | `WorkerStatus` five states | EXISTS — same model | Local |
| Baseline file + split marker | `Baseline`, `CompletionSignal` | EXISTS | Local |
| Delivery = activity vs baseline | ack file (`ack-N.json`) | EXISTS, stronger locally — keep the file; use activity only as the resend trigger | Local |
| Completion = marker on screen | `result-N.json` | EXISTS, stronger locally — keep | Local |
| Send text + separate Enter; `-l` literal | `herdr agent prompt` (atomic) | NEW — two `send-keys` calls; one-line pointer so no paste-buffer | Upstream |
| Read bounded scrollback | `herdr agent read` + spill file | NEW — `capture-pane -p -S -N`; reuse spill file | Local/Upstream |
| Ready gate after spawn | ready-wait (idle OR done) | NEW for tmux — stability + no busy chrome | Both |
| Liveness (process alive) | foreground process list | NEW — `list-panes -F '#{pane_pid} #{pane_dead}'` | Inference |
| Concurrent broadcast | wave choreography | EXISTS — wave already trait-driven | Local |
| Canonical id (name vs `%N`) | `canonical_id` (name vs herdr pane id) | NEW — `display-message -p -t <name> '#{pane_id}'` | Local |
| Layout | column-right-then-down, serialized spawns | CONFLICT — upstream has none; keep ours (D1/D2) or use one-window-per-worker | Local |
| Teardown with confirm | pane close follows outcome (D6) | NEW — `kill-pane`/`kill-session` on the same outcome rule | Local |
| HANDOFF orchestrator | cold control loop + `.bee/HANDOFF.json` | NOT NEEDED | Local |
| Fake `tmux` binary for tests | `tests/herdr_backend.rs` stub | EXISTS pattern — add `tests/tmux_backend.rs` | Local |

## Cross-Cutting Sweep

Wiring outside the backend module that names herdr and must learn a
second transport (each is unchecked until a cell touches it):

- `crates/bee/src/herding/run.rs:400-515` — the private `Herdr` seam (the big one); `wave.rs:764` stray exec.
- `hooks/activity.rs:257` — activity record reads `HERDR_PANE_ID`; needs `TMUX_PANE` twin.
- `herding/split_lock.rs` — keep; tmux splits need the same serialization.
- `crates/bee/src/herding/control_loop.rs:215,217,675,700,747` — `Bash(herdr:*)` allowlists → need `Bash(tmux:*)` when transport=tmux.
- `crates/bee/src/herding.rs:112-113,578-599` — `herdr-result`, `herdr-pane-id`, `HERDR_ENV` readiness.
- `crates/bee/src/verbs/drivers/prepare.rs:568-579` — dispatch prepare mentions `herdr pane` for herding-kind tier slots.
- `bee herding status` "pane transport reachable" (herding-reach hrc-2) — must probe `tmux` when selected.
- `skills/bee-herding/scripts/*` bootstrap — herdr tab/pane layout.
- `skills/bee-herding/SKILL.md` frontmatter `dependencies.herdr-cli` (`missing_effect: unavailable`) — becomes one-of.
- `docs/knowledge/areas/bee-herding/*` — four pages assume herdr.
- Windows: herdr path has a recorded owner-run gap (D19); tmux has no native Windows at all — WSL only. Route flag `cross-platform`.

## Recommendation (ladder)

1. Reuse — partial: trait, Baseline/CompletionSignal, mailbox, wave, fake-binary test pattern.
2. Built-in — n/a (no tmux support in any dependency).
3. **Adapt — chosen.** Port the upstream marker policy and stability
   poll into a `TmuxBackend`; keep our file-based truth.
4. Build from scratch — rejected: the seam and half the proof logic exist.

Why 3 beats 1 alone: the trait covers waves but not `run`; without the
run-side seam a tmux backend only briefs workers that already exist.

What would change the answer: if `run.rs` proves cheap to route through
the existing `WorkerBackend` (add `split`/`close`/`alive` to it) rather
than a second trait, the refactor shrinks to one cell.

## Risks, Unknowns, Follow-Ups

- Marker lists rot with CLI releases → keep them in `.bee/config.json`
  (`herding.tmux.busy_markers` / `blocked_markers`), defaults from
  upstream's verified set.
- Inside a Claude Code tool shell there is no TTY: never `attach`, never
  `switch-client`; `$TMUX` being set does not mean "attached" (verified
  upstream).
- The live session of this repo runs in herdr; a tmux live proof needs
  an owner-run pane (`spawn-proof.md` style) before the transport is
  called done.
- Open shaping questions: (a) transport switch shape — `herding.transport: herdr|tmux` vs auto-detect by `$TMUX`/`HERDR_ENV`; (b) layout — split panes in the caller's window (mirrors herdr doctrine) vs one detached session per worker (upstream, simpler, invisible); (c) does `run` get its own transport trait, or does `WorkerBackend` grow three methods.

## Source Pack

- Upstream: `skills/tmux-agent-comms/SKILL.md`, `scripts/wait_for_idle.py` (marker policy lines ~70-110), `scripts/preflight_send.py`, `scripts/broadcast.sh`, `references/delivery-and-waiting.md`, `references/tmux-recipes.md`, `references/context-succession.md` @ `ab46724e`.
- Local: `packages/bee-rs/crates/fleet/src/backend.rs`, `backend/herdr.rs:722-790`, `crates/bee/src/herding/wave.rs:543-575`, `crates/bee/src/herding/run.rs:502`, `crates/bee/src/herding/control_loop.rs:215`, `skills/bee-herding/SKILL.md`, `skills/bee-herding/references/operational-invariants.md` ("Runtime adapter"), `docs/knowledge/areas/bee-herding/{overview,the-run-verb-and-worker-outcomes,handing-a-foreign-agent-its-brief}.md`.
