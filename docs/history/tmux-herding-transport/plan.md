---
artifact_contract: bee-plan/v1
mode: standard
# approved_gate2: <unset until approval; then a date stamp — the only permitted post-approval write>
---

# Plan: tmux Herding Transport

Mode: `standard` — 2 risk flags: cross-platform, external-systems (tmux binary)
Why this is the least workflow that protects the work: one new process
boundary (tmux) behind an existing trait, selected by one config key, with
the mailbox truth untouched — a phase plan keeps every step demoable.

Route note: `bee route --set` is refused inside a feature worktree (control
plane lives on main) and the main session is bound to no lane; the route
(class=feature, lane=standard, flags=2, files=7) is recorded here and in the
cells' `lane` field instead.

## Requirements (from CONTEXT.md)
- D1: `herding.transport: "herdr" | "tmux"`; absent = herdr, byte-identical; no env auto-detect.
- D2: tmux workers are split panes in the caller's window under the existing column rule and split lock; never a detached session.
- D3: a dialog ends the wait as `blocked`; the pane stays; bee never types into a dialog.
- D4: tmux status is screen-derived (stability + marker lists as config data, upstream defaults) and advisory; `result-N.json` / `ack-N.json` stay the truth.
- D5: source manifest — luongnv89/skills @ `ab46724e`, `skills/tmux-agent-comms/`.

## Discovery
- `bee herding run` isolates every herdr call behind a private 13-method
  trait `Herdr` (`packages/bee-rs/crates/bee/src/herding/run.rs:404-495`),
  with ONE production construction site: `execute(&opts, &RealHerdr)`
  (`run.rs:2538`). A second implementer plugs in there. Evidence:
  `rg -n "RealHerdr" packages/bee-rs/crates/bee/src/herding/run.rs`.
- The transport probe is two pure functions reading env:
  `herding.rs:577 transport_state_with`, `prepare.rs:553 herding_transport_probe`.
- Occupancy reads live pane ids through one exec: `wave.rs:764 live_pane_ids_via_herdr`.
- Wave briefing goes through `fleet::backend::WorkerBackend` (`wave.rs:543`) — a `TmuxBackend` is a later slice; `run` is the walking skeleton.
- Test pattern for a shelled-out binary: `crates/fleet/tests/herdr_backend.rs` — PATH-prepended stub script.
- Full matrix and sweep: `docs/history/research/tmux-herding-transport.md`.

## Approach
Recommended (D1–D4): keep run's private trait, rename it to the
transport-neutral `PaneTransport` (behavior-neutral refactor), add
`herding/tmux.rs` implementing it with tmux verbs, select the implementer
at `run.rs:2538` from `herding.transport`. The probe functions grow a tmux
arm (`$TMUX` + `$TMUX_PANE`) chosen by the same key. Occupancy lists panes
via `tmux list-panes -a` under tmux.

Rejected:
- Grow `WorkerBackend` to absorb run's seam — run needs 12 ops (layout,
  split, tab, env line, liveness); the wave trait needs 5. Merging widens
  the wave seam for no wave benefit.
- Auto-detect transport from env — rejected by D1.
- Detached session per worker — rejected by D2.

Risk map:
| Component | Risk | Reason | Proof needed |
|---|---|---|---|
| `tmux.rs` status classifier | MEDIUM | screen text is the only signal; marker rot | unit tests on captured pane bodies; markers are config data (D4) |
| `agent_start` via `send-keys` | MEDIUM | the argv is typed into a shell; quoting | shell-quote every token; stub-tmux test asserts the exact line |
| probe/config switch | LOW | pure functions | unit tests both arms |
| Windows | LOW | WSL-only; route flag carried | none beyond the existing herdr D19 gap |

## Shape

| Phase | What Changes | Why Now | Demo | Unlocks |
|---|---|---|---|---|
| 1 (current) | `herding.transport` key + tmux probe; `PaneTransport` rename; `RealTmux` module with in-module stub-tmux tests; the run-verb switch; activity-record pane twin; docs/skill/knowledge sync | the walking skeleton: `bee herding run` starts, briefs, watches and retires one worker in a tmux pane | `herding.transport=tmux` → `bee herding run --task "…"` from inside a tmux pane ends with a result file | phase 2 |
| 2 | occupancy via `tmux list-panes -a` (`wave.rs:763`, page `waves-and-occupancy.md`), `fleet::backend::tmux::TmuxBackend` (waves), control-loop `Bash(tmux:*)` allowlist, bootstrap cockpit tmux form | only after a worker round-trips | `bee herding wave` briefs two tmux panes | cockpit on tmux |
| 3 | owner-run live spawn proof (`spawn-proof.md` twin), WSL note | transport is "done" only with a live round trip | recorded proof doc | uat |

Current slice: phase 1, six cells (`tht-1` … `tht-6`). Smaller-path check: occupancy moved to phase 2 — it serves only the cockpit's dispatch role; `bee herding run` needs none of it. Review wave: one bee-review pass, 6 blockers / 5 warnings / 7 criticals applied to the cells before the gate (dead verify filters, binary-crate test placement, job→pane map, config seam, activity.rs owner, cell split).

## Test matrix
- Happy: `transport=tmux` + `$TMUX_PANE` set → probe `ready`; `RealTmux::pane_split` builds `split-window -h|-v -l <n> -c <cwd> -d -P -F '#{pane_id}'`; `agent_start` types one shell-quoted line + `Enter`; status classifier: stable screen with no marker → `idle`; `esc to interrupt` in tail → `working`; `do you trust` in last 12 lines → `blocked` (D3); `process_info` maps `pane_dead=1` → `Dead`, a pid → `Alive`.
- Edge: key absent → herdr path byte-identical (existing tests stay green); key `tmux` but `$TMUX` unset → probe `not ready` with reason; marker lists overridden in config replace defaults; stale marker present in baseline never counts (reuse `CompletionSignal` rule).
- Error: `tmux` missing on PATH → `pane_alive` false (fail closed), `pane_layout` None (fail open), `agent_status` None; `split-window` non-zero → typed Err naming the argv; unknown `herding.transport` value → refusal naming the two values.
- Existing coverage cited by each writer: `run.rs` `mod tests` (fake `Herdr` impls at `run.rs:3165,3287,3520`), `fleet/tests/herdr_backend.rs` stub pattern. `crates/bee` is a binary crate: new unit tests live in-module, never under `crates/bee/tests/`.

## Out of scope
- Orchestrator HANDOFF / successor session (upstream Phase 7).
- Broadcast script — `bee herding wave` covers it.
- Native Windows tmux.
- Auto-answering dialogs (rejected by D3).
