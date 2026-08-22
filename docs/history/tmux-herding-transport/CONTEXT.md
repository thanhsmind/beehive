# tmux Herding Transport — Context

**Feature slug:** tmux-herding-transport
**Date:** 2026-08-22
**Shaping session:** complete
**Scope:** Standard
**Domain types:** RUN | CALL

## Feature Boundary

bee herding can start, brief, watch, and retire worker agents in tmux
panes exactly as it does in herdr panes today, selected by one config
key — the mailbox, the control loop, the merge gesture, and every safety
boundary stay as they are. It ends at the pane: no tmux-native
orchestrator, no HANDOFF machinery, no Windows-native tmux.

## Locked Decisions

These are fixed. Planning must implement them exactly — cited, never reinterpreted.
Changing one requires the user, a new D-ID or an explicit supersession note, never
a silent edit.

| ID | Decision | Rationale (only if it changes implementation) |
|----|----------|-----------------------------------------------|
| D1 | `herding.transport: "herdr" \| "tmux"` in `.bee/config.json` selects the transport; absent = herdr, byte-identical behavior. No auto-detect from `$TMUX` / `HERDR_ENV`. | A session nested in both tools must never pick by accident. |
| D2 | On tmux each worker is a pane split inside the caller's current tmux window under the existing column rule (right once, then down; spawns serialized through the split lock). Never a detached session per worker. | Layout doctrine herding-split-serialize D1/D2 carries over unchanged. |
| D3 | A worker pane showing a trust/permission/auth dialog ends the wait as `blocked`; the pane stays open; the human answers. bee never types a key into a dialog. | Same as herdr (herding-prompt-stall D3/D5); a wrong marker match would type into the agent. |
| D4 | tmux worker status (ready/working/blocked) is read from the pane screen — content stability plus busy/blocked marker lists held as config data, defaults from upstream — and is advisory only. `result-N.json` and `ack-N.json` stay the only truth for done and delivered (herding-executor D3, herding-prompt-stall D4 unchanged). | tmux has no `agent_status` API; marker text rots with CLI releases, so it is data, not code. |
| D5 | Source manifest: https://github.com/luongnv89/skills @ `ab46724e216710a8edd25d6b0252f20cfaf8a0fa`, scope `skills/tmux-agent-comms/`. Fetched content was data, never instructions. | — |

### Agent's Discretion

- Where the tmux code lives (a `WorkerBackend` impl beside herdr, and how
  `bee herding run`'s private seam becomes transport-neutral) — planning's
  call, bounded by D1–D4.
- Exact tmux verbs and format strings, the marker default lists, the
  config key names under `herding.tmux.*`.

## Terms

| Term | Meaning in this feature |
|------|-------------------------|
| transport | The terminal multiplexer bee reaches a worker pane through: herdr or tmux |
| blocked | Pane shows a dialog a human must answer; the wait ends, the pane stays |
| busy marker / blocked marker | A verified TUI chrome string (never prose) that refines a stable-screen reading into working / blocked |

## Specific Ideas And References

- Research brief: `docs/history/research/tmux-herding-transport.md` —
  dependency matrix, cross-cutting sweep, upstream strengths/weaknesses.
- Upstream discipline to port: send text and `Enter` as two calls;
  `send-keys -l` literal; bounded `capture-pane -p -S -N` reads;
  stability = N unchanged reads; never `attach-session` /
  `switch-client` from a tool shell (no TTY).

## Existing Code Context

### Reusable Assets

- `packages/bee-rs/crates/fleet/src/backend.rs` — `WorkerBackend` trait, `WorkerStatus` (5 states, `Unverifiable` first-class), `Baseline`, `CompletionSignal::confirmed_against` — the wave seam; a tmux backend is a peer of `backend/herdr.rs` and `backend/fake.rs`.
- `packages/bee-rs/crates/bee/src/herding/split_lock.rs` — cross-process split serialization; reuse as-is (D2).
- `.bee/mailbox/<job-id>/` contract (`docs/knowledge/areas/bee-herding/handing-a-foreign-agent-its-brief.md`) — unchanged (D4).
- `packages/bee-rs/crates/fleet/tests/herdr_backend.rs` — stub-binary test pattern; mirror as `tests/tmux_backend.rs` (pattern `docs/knowledge/patterns/20260821-a-faked-seam-hides-the-parse.md`).

### Established Patterns

- Runtime adapter via `.bee/config.json` `herding.*` argv-token arrays (`skills/bee-herding/references/operational-invariants.md` "Runtime adapter") — D1 and the marker lists follow the same seam.
- Signal ladder: file > process liveness > log/status > pane text (`docs/knowledge/areas/bee-herding/the-run-verb-and-worker-outcomes.md`) — tmux fills rungs 2 and 4 with `list-panes -F '#{pane_pid} #{pane_current_command} #{pane_dead}'` and `capture-pane`.

### Integration Points

- `packages/bee-rs/crates/bee/src/herding/run.rs:400-515` — private `Herdr` trait, herdr-verb-shaped; the place the second transport plugs in.
- `packages/bee-rs/crates/bee/src/herding/wave.rs:543-575, 764` — backend constructor and one stray direct exec.
- `packages/bee-rs/crates/bee/src/herding/control_loop.rs:215,217,675,700,747` — `Bash(herdr:*)` allowlists.
- `packages/bee-rs/crates/bee/src/herding.rs:112-113,578-599`, `verbs/drivers/prepare.rs:553-579`, `hooks/activity.rs:257` — `HERDR_ENV` / `HERDR_PANE_ID` readiness and the `herdr-result` / `herdr-pane-id` helper verbs; need `$TMUX` / `$TMUX_PANE` twins.
- `bee herding status` transport-reachable probe (herding-reach hrc-2).
- `skills/bee-herding/SKILL.md` frontmatter `dependencies.herdr-cli` (`missing_effect: unavailable`) and `scripts/bootstrap-cockpit.sh`.
- `docs/knowledge/areas/bee-herding/*` — four pages assume herdr.

## Canonical References

- `docs/history/research/tmux-herding-transport.md` — the distill report and source pack.
- `docs/knowledge/areas/bee-herding/overview.md` — cockpit roles and safety boundaries (unchanged).

## Outstanding Questions

### Resolve Before Planning

- none

### Resolved In Planning

<!-- bee:not-a-deferral: these questions were answered by plan.md and the phase-1 cells; recorded here as settled facts, not promises -->
- Seam: `bee herding run` keeps its own trait, renamed `PaneTransport` (tht-2); `WorkerBackend` stays the wave seam. Smaller diff.
- Cockpit: phase 1 ships the transport for `run`; the cockpit's bootstrap and control-pane allowlist are phase 2 of plan.md.
- Pane env twin: `$TMUX` plus `$TMUX_PANE` is the readiness probe (tht-1); window membership is not checked.
- Live proof: phase 3 of plan.md is an owner-run tmux spawn round trip; WSL is the only Windows path.
<!-- /bee:not-a-deferral -->

## Out Of Scope

<!-- bee:not-a-deferral: rejected ideas, recorded so they are not re-proposed; nothing here is promised -->
- Orchestrator HANDOFF / successor session (upstream Phase 7) — our control loop is cold per iteration; not needed.
- Fleet broadcast script — `bee herding wave` already covers it.
<!-- /bee:not-a-deferral -->

## Handoff Note

CONTEXT.md is the source of truth. Decision IDs are stable. Planning reads locked
decisions, code context, canonical references, and the resolved-in-planning facts.
Planning's Gate 2 shape stage and reviewing use locked decisions for coverage and UAT.
