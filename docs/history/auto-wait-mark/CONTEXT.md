# Auto Wait Mark — Context

**Feature slug:** auto-wait-mark
**Date:** 2026-08-18
**Shaping session:** complete
**Scope:** Standard
**Domain types:** RUN | READ

## Feature Boundary

The Stop hook writes a `waiting_on` mark on every turn end, so a run's
`run_state` tells a reader that control sits with the human even when the agent
never called `bee state waiting-on set` itself; the feature ends at that written
mark plus the one `kind` value the split needs — it adds no dashboard, no new
reporting surface, and no change to how the agent is instructed to behave.

This is the follow-on `awaiting-human` named and left out of its own scope:
"Detecting an unmarked wait automatically (the agent asks in prose and forgets to
mark) — would need the session-stop hook to infer intent"
(`docs/history/awaiting-human/CONTEXT.md:120-122`).

## Locked Decisions

These are fixed. Planning must implement them exactly — cited, never reinterpreted.
Changing one requires the user, a new D-ID or an explicit supersession note, never
a silent edit.

| ID | Decision | Rationale (only if it changes implementation) |
|----|----------|-----------------------------------------------|
| D1 | The Stop hook sets a `waiting_on` mark on **every** turn end, not only on turns whose last line ends in a question mark. No text heuristic, no phrase table, no language detection. | Stop means control returned to the human, so the run *is* waiting on a person at that instant. A question-mark rule provably misses this repo's most common wait shape — a turn closing on "say go and I will do it" — and a bilingual phrase table only trades the miss for a guess. Accepted cost: an idle session now always reads `awaiting-approval`, so the distinction moves into `kind` (D3), never into whether a mark exists. |
| D2 | A hook-set mark carries **no** provenance marker. The hook calls the same store setter the agent-facing verb calls and writes the same record shape; every reading surface treats both as one kind of fact. | Under D1 the hook infers nothing — it fires on the definitional fact that control returned — so there is no guess whose accuracy a reader would discount. Keeps the record shape unchanged, so the five surfaces `ah-3` already wired stay untouched. |
| D3 | The `kind` vocabulary grows from two closed values to three: `gate`, `question`, `turn-end`. `gate` and `question` keep their meaning — the run is blocked on a person. `turn-end` is what the Stop hook writes on an ordinary turn end: control is back with the human, but nothing is owed. A reader wanting only real blocks filters `kind != turn-end`. | D1 makes the mark constant, so the mark's *existence* stops carrying signal. Free text in `subject` cannot be filtered by the dashboard consumer that `traceable-runs` D8 named; a third enum value can. Cost: R125 documents `kind` as closed two-value and `build_waiting_on` refuses an unknown kind, so validation, its tests, and the workflow-state concept doc widen together. |
| D4 | Derived from D3. A surface that renders a live wait as a **blocker** must not list a `turn-end` mark — a turn-end is by definition not a blocker. Surfaces that merely **display** the wait keep showing all three kinds. | `bee orient` is the only such surface (`packages/bee-rs/crates/bee/src/verbs/status_full/orient.rs:400`). Without this, a permanently-live mark turns into a permanent false blocker line. |
| D5 | The hook **never overwrites a live mark**. If the agent already declared a `gate` or `question` wait, that mark stands and the hook writes nothing; the hook only fills an empty slot. | Protects D3's signal. The agent-declared wait is the high-value case; a hook firing later in the same turn would downgrade it to `turn-end` and erase exactly the distinction D3 exists to create. |
| D6 | The `subject` a `turn-end` mark carries is the last non-empty line of the turn's final assistant text block, trimmed and truncated. | R125 requires `subject` non-empty. That line is the one thing a human reader can use to recall what the session was doing, and it is already reachable at zero extra I/O (see Integration Points). Truncation keeps it renderable inline on the five `ah-3` surfaces. |

### Agent's Discretion

- The truncation length for D6, and whether truncation is by characters or by
  grapheme-safe boundary. Constraint: the result must render on one line on the
  session preamble and the compact capsule without wrapping.
- The exact spelling of D3's third value (`turn-end` is the working name).
  Constraint: lowercase, hyphenated, and it must not collide with `gate` or
  `question`.
- Whether the hook reuses the session record's stored `transcript_path`
  (`session_init.rs:401`) or the existing `resolve_transcript_for` glob. Constraint:
  one resolution path, not two, and it must not add a second transcript read per turn.

## Terms

| Term | Meaning in this feature |
|------|-------------------------|
| turn end | The `Stop` hook event: the main agent finished responding and control returned to the human. Not `SubagentStop`, and not fired at all when the human interrupts. |
| declared wait | A mark written by `bee state waiting-on set`, i.e. the agent said out loud it is waiting. Kind `gate` or `question`. |
| inferred wait | A mark written by the Stop hook under D1. Kind `turn-end`. Indistinguishable in shape from a declared wait (D2); distinguishable only by its kind (D3). |

## Specific Ideas And References

- The user's trigger case: the agent stops and asks a question in prose without
  calling `AskUserQuestion`, and nothing in bee records that the run is waiting.
  Confirmed as the observed behavior, not a rule the user wants imposed on the
  agent — this feature changes bookkeeping only, never how the agent is told to ask.
- Explicitly *not* wanted: forcing the agent to call `AskUserQuestion`. The user
  ruled that out as a soft rule that will be forgotten again.

## Existing Code Context

From the quick scout only. Downstream agents read these before planning.

### Reusable Assets

- `packages/bee-rs/crates/bee/src/verbs/workflow_store/record.rs` — `build_waiting_on(kind, subject, session)` validates and builds the mark; `WAITING_ON_STALE_SECONDS = 900.0` at `:522`. This is the validator D3 widens.
- `packages/bee-rs/crates/bee/src/verbs/state_group/waiting_on.rs` — the agent-facing `set`/`clear` verbs and D3 target resolution (session-scoped first, feature-scoped when a feature is live). The hook reuses this resolution, never a second one.
- `packages/bee-rs/crates/bee/src/hooks/session_close/perf.rs:71` — `resolve_transcript_for(root, session_id)` already resolves and reads the transcript on **every** Stop for the perf rollup. D6's last line is reachable at zero added I/O.
- `packages/bee-rs/crates/bee/src/hooks/session_close/nudges.rs:356-358` — `if ctx.event != "Stop" { return Ok(None); }`, the existing pattern for a Stop-only branch inside this hook.

### Established Patterns

- Record wins, projection is rebuilt from it — a set or clear writes the workflow record, then rebuilds the lane and default-state projections so the mark is visible immediately (`waiting_on.rs` header comment).
- Best-effort hook side effects — `prompt_context.rs:338` `clear_and_reap_waiting_on_best_effort` logs and continues on failure rather than failing the hook. The setter follows the same shape: a failed write never fails the Stop hook.

### Integration Points

- `packages/bee-rs/crates/bee/src/hooks/session_close/mod.rs` — `run_inner` already branches `SessionEnd` (`:113`) and `PreCompact` (`:119`); everything else, including `Stop`, falls through to `advisory()` at `:228`. The new setter hangs off a Stop-only branch here.
- `packages/bee-rs/crates/bee/src/verbs/status_full/orient.rs:400` — the blockers push that D4 narrows.
- `docs/knowledge/areas/workflow-state/workflow-records-and-projections.md` — R125 (closed kind vocabulary), R126 (the three ways a mark ends), R128 (the agent-facing doors), R129. R125 and R128 both move under D3 and D1.

## Canonical References

- `docs/history/awaiting-human/CONTEXT.md` — D1-D5 of the mark itself; `:120-122` is this feature's own origin note.
- `docs/knowledge/areas/workflow-state/workflow-records-and-projections.md` — R109, R125, R126, R128, R129.
- `docs/history/traceable-runs/CONTEXT.md` D8 — `awaiting-approval` covers every moment the agent waits on a human. D1 here is the literal reading of it.
- `docs/history/awaiting-human/reports/ah-3-rework.md` — the enumeration of the five surfaces that name a live wait, and why `status --brief` is deliberately excluded.

## Outstanding Questions

### Resolve Before Planning

None.

### Deferred To Planning

- [ ] Does anything outside `packages/bee-rs` persist or assert on the two-value `kind` vocabulary — the hook manifests, the Codex/OpenCode renderings, `registry_payload.json`, or a skill's prose? — answered by grepping for the literal strings `"gate"`/`"question"` near `waiting_on` across `skills/`, `.claude/`, `.codex/`, and the generated registry.
- [ ] Should `stop_hook_active` be read to skip a stop that is itself the continuation of a blocked Stop hook? — answered by checking whether the existing bypass block net (`nudges.rs:349`) can re-enter and produce a doubled mark.
- [ ] Does a `turn-end` mark interact with the 30-minute nudge dedup cache (`reads.rs:441`, `INJECT_INTERVAL_MS`)? — answered by reading whether the setter belongs before or after `should_inject`.

## Deferred Ideas

- A `PreToolUse` hook on `AskUserQuestion` that marks the wait the moment the
  question is asked. Deferred: an `AskUserQuestion` turn never reaches `Stop` — the
  tool answer returns inside the same turn — so this feature's Stop-only mechanism
  cannot cover it. Filed to the backlog (`proposal`, P3, layer `hooks`) for
  re-triage after this merges.
- A dashboard reading the mark. Out of scope by `awaiting-human` D5, which ships
  the data source only; unchanged here.

## Handoff Note

CONTEXT.md is the source of truth. Decision IDs are stable. Planning reads locked
decisions, code context, canonical references, and deferred-to-planning questions.
Planning's Gate 2 shape stage and reviewing use locked decisions for coverage and UAT.
