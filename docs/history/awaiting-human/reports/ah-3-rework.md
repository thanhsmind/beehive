# ah-3 rework — the fifth (and sixth-checked) reporting surface

**Status:** [DONE] (pending re-judge)
**Worker:** dave3
**Reopened for:** NEEDS_REVISION — census of run_state/phase-and-gate
reporting surfaces was incomplete and never written down (`trace.outcome`
was null, this directory did not exist).

## Fresh, full enumeration

A repo-wide search for every surface that reports `run_state`, `waiting_on`,
or the phase-and-gate situation (`rg -n "run_state" crates/bee/src`, `rg -n
"waiting_on" crates/bee/src`, `rg -ln "Phase:|Gate pending|Gates:|gates_line|
first_open_gate|approved_gates" crates/bee/src`), read against every hit,
turned up exactly six candidates:

| # | Surface | Reads run_state/waiting_on already? | Reports phase/gate to a reader? | Handling |
|---|---|---|---|---|
| 1 | `bee status --json` (`verbs/status_full/build.rs:259`) | yes | yes | Wired in the first pass (`fe0b8885`) — confirmed sound by the judge, left alone. |
| 2 | `bee orient` (`verbs/status_full/orient.rs:327`, `:405`) | yes | yes | Wired in the first pass — confirmed sound, left alone. |
| 3 | The text status renderer (`verbs/status_full/render.rs:193`) | yes | yes | Wired in the first pass — confirmed sound, left alone. |
| 4 | The session preamble (`hooks/session_preamble/render.rs:105`) | yes | yes | Wired in the first pass — confirmed sound (inline-test deviation judged sound), left alone. |
| 5 | **The compact capsule** (`hooks/compaction.rs`, `build_compact_capsule`) | yes — reads `.bee/state.json` at `:187` via `read_state_failopen`, so `waiting_on` is already in the record | yes — `- Phase: … \| Mode: … \| Feature: … \| Lane: …` at `:1419-1425` and `- Gate pending: {gate}` at `:1477-1478` | **This rework**: wired. Reuses `session_preamble::waiting_on_note` (the same helper the preamble's Gates line already calls) and appends `- Waiting on human — {kind}: {subject}` right after the phase line. This is the text a session reads immediately after a context compaction — the moment it most needs to know it is stopped on a person. |
| 6 | `bee status --brief` (`verbs/status_brief.rs`) | no — `BriefState` (`state.rs:34-40`) reads only `phase, feature, mode, gates, route`; `waiting_on` is not in its struct | yes, minimally — `phase=… feature=… mode=… gates=t/t/t/f bypass=…` | **Deliberately left unwired** — see Deviation below. |

No seventh surface was found. Every other hit for `approved_gates` /
`Gates:` / `Phase:` across the tree (`chain_nudge.rs`, `state_sync.rs`,
`session_close/*`, `write_guard/checks.rs`, `state_group/*`) is internal
state-projection/merge machinery (gate approval bookkeeping, JS-parity
shape coercion) — never a place a human or agent reader consults to learn
"is this run stopped on a person right now." None of those render a
phase/gate line for a reader.

## Deviation — `bee status --brief` left unwired

Judgment call, recorded per the reopened cell's instruction ("if you
deliberately do not [wire it], record why as a deviation rather than
leaving it silent").

`status --brief` is not a general routing surface — it is a deliberately
minimal, frozen-shape, performance-optimized fast path (status-diet D1,
`docs/history/status-diet/CONTEXT.md`): "reads ONLY the state layer needed
for orientation — phase, feature, mode, gates, gate_bypass_level,
ship_visibility, route — no cells scan, no review scan, **no handoff
resolution**, no models/tier_mix." D2 in the same record narrows its job
further: it is worker-startup's cheap liveness ping ("cells show stays the
claim authority"), and the same record says full `status --json` — not
`--brief` — is the surface kept "for humans/orchestrator routing." A live
human wait is conceptually the same kind of fact as the handoff block D1
names as explicitly out of scope for `--brief`.

Wiring it would also mean adding a field to `BriefState`/`read_state_brief`
in `crates/bee/src/state.rs`, a file outside this rework's assigned scope
(`hooks/compaction.rs`, `verbs/status_brief.rs`, and their tests) and
outside the original cell's file list — a shape change to a documented
"frozen 7-key order" surface belongs in front of the human as a scoped
decision, not folded silently into a rework pass.

Net: the four confirmed surfaces plus the capsule (this rework) are where
a reader learns a run is stopped on a person; `--brief` is a machine
liveness ping for an already-dispatched, already-claimed worker and was
left as-is.

## Files touched

- `packages/bee-rs/crates/bee/src/hooks/compaction.rs` — `build_compact_capsule` now names a live wait right after the phase line; two new tests (`the_capsule_names_a_live_wait_right_after_the_phase_line`, `the_capsule_stays_silent_without_a_live_wait`).
- `docs/history/awaiting-human/reports/ah-3-rework.md` — this report.

## Verification

`cargo test --release --manifest-path packages/bee-rs/Cargo.toml` — 1649
passed (unit) + all integration suites green, 0 failed, 11 ignored.
`hooks::compaction::tests::*` — 11 passed, including the two new tests.
