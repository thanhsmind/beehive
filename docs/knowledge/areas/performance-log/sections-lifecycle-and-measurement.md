---
type: bee.area
title: "Performance Log — Sections: Lifecycle and Measurement"
description: "Opening, closing, and one-shot recording of a named piece of work; what a section captures; and the measurement rules that make its token and timing numbers trustworthy."
timestamp: 2026-07-22
bee:
  id: performance-log-sections-lifecycle-and-measurement
  lifecycle: active
  areas: [performance-log]
  decisions: [D2 81456c6e, D3 9f7f4256]
  sources: [docs/history/perf-log/CONTEXT.md, docs/history/perf-log/plan.md, "cells perf-log-1, perf-log-2, perf-log-3 (capped, verified)", "docs/specs/performance-log.md#R1", "docs/specs/performance-log.md#R2", "docs/specs/performance-log.md#R3", "docs/specs/performance-log.md#R4", "docs/specs/performance-log.md#R5", "docs/specs/performance-log.md#R8", "docs/specs/performance-log.md#E1", "docs/specs/performance-log.md#E2", "docs/specs/performance-log.md#E3", "docs/specs/performance-log.md#E5", "docs/specs/performance-log.md#P2", "docs/specs/performance-log.md#P4", "docs/specs/performance-log.md#P6"]
  authoritative_for: "performance-log: sections lifecycle and measurement"
---

# Performance Log — Sections: Lifecycle and Measurement

A single, shared record — kept in one place for the whole machine, not per
project — of **how much a stretch of assistant work cost and how long it
actively took**. Each entry (a "section") answers, for one piece of work:
which assistant models did the work, how many tokens each spent (fresh vs.
reused-from-cache), whether the work ran several helpers in parallel, and how
much *active* time it consumed. Many projects on the same machine append to
the same log, so their work can be compared and reviewed together over time.

This concept owns the operator-driven lifecycle of a named section — opening,
closing, one-shot recording, and reading it back — and the measurement rules
that make any section's numbers trustworthy. How that data is automatically
populated into, and synced with, the shared persistent store is
`persistent-store-and-sync.md`; the read-only cross-project rollup built from
that store is `cross-project-matrix.md`.

## Entry Points & Triggers

- **Open a section** — the operator names a piece of work and opens a section
  for it. The current coding session's activity record and the moment of
  opening are remembered so the eventual measurement covers exactly this
  span.
- **Close a section** — the operator closes the open section (optionally
  with a one-line note). The span from open to close is measured,
  summarized, and appended to the shared log.
- **One-shot section** — instead of open-then-close, the operator can record
  a section covering a trailing window ("everything in the last 30 minutes /
  2 hours / 1 day", or since a given moment) in a single step.
- **Read the log** — the operator lists recent sections (most recent last),
  optionally limited to the last N.
- **Render the log** — the operator produces a human-readable report of the
  recorded sections.

Explicit named sections (open/close/one-shot) are the manual path. Only one
section may be open at a time per project working copy.

## Data Dictionary

A **section** carries:

- **label** — the operator's short name for the piece of work (may be
  empty).
- **note** — an optional one-line summary recorded at close.
- **project** — which project working copy the work belonged to.
- **branch** — the version-control branch active at the time (may be
  empty).
- **session** — which coding session's activity record was measured.
- **started / ended** — the wall-clock endpoints of the span.
- **running time** — the **active** execution time inside the span,
  excluding idle waiting (see R3). Stored precisely and also as a compact
  human form (e.g. `12m52s`).
- **parallel** — whether the work ran helpers concurrently (see R4).
- **subagent count** — how many helper runs were attributed to the span.
- **per-model token breakdown** — for each model that did work in the span:
  - **new** — tokens billed fresh: freshly-read input + generated output +
    newly-written cache.
  - **cached** — tokens served from cache (reused, cheap).
  - **total** — new + cached.
- **subagent token breakdown** — the same per-model breakdown, for helper
  runs only.
- **event count** — how many recorded activity events fell inside the span.

Token vocabulary (the four raw quantities the breakdown is built from):

- **fresh input** — new prompt tokens read this call, not from cache.
- **output** — tokens the model generated.
- **cache write** — tokens written into the reuse cache this call.
- **cache read** — tokens served from the reuse cache (the cheap ones).

## Behaviors & Operations

- **Opening a section.** Triggered by the operator naming a piece of work.
  The system resolves which coding session's activity record to measure —
  by default the most recently active record for this project, or an
  explicitly named session — and remembers that choice together with the
  start moment. The operator observes a confirmation naming the section and
  the session being measured. If no activity record can be resolved yet,
  the section still opens and the operator is told metrics will be empty at
  close.
- **Closing a section.** Triggered by the operator closing the open
  section. The system measures the remembered span against the remembered
  activity record: it counts tokens per model, attributes helper (parallel)
  cost, decides whether the work was parallel, and computes the active
  running time. It appends the finished section to the shared log, clears
  the open-section state, and the operator observes the one-line summary
  plus where it was written. Closing when nothing is open yields a clear
  "nothing to close" message and changes nothing.
- **One-shot section.** Triggered by the operator asking to record a
  trailing window. Same measurement as close, over the window "now minus
  the given duration" (or since a given moment) up to now, appended
  immediately. An unparseable window yields a clear message and records
  nothing.
- **Reading / rendering.** Reading returns the recorded sections (most
  recent last, optionally the last N), each as a one-line summary.
  Rendering produces the same content as a human-readable report. Both
  tolerate an empty or missing log and report "nothing logged yet".
- **Measurement rules.** Within the span:
  - Token counts are gathered per model from the session's activity events;
    repeated records of the same underlying request are counted once (R1).
  - Helper (subagent) runs whose activity overlaps the span are attributed
    separately, and their models roll into the subagent breakdown (R5).
  - Placeholder/local non-model events are excluded from all token counts
    (R2).

## Actors & Access

- **Operator** — the person (or the assistant acting on their behalf)
  running the work. Opens, closes, records, reads, and renders sections. No
  other role exists; the log is a personal, machine-local record.
- **Consuming reader** — anyone later reading or rendering the shared log to
  review cost and timing across projects.

## Business Rules

- **R1 — Count each request once.** When the session's activity record
  holds several entries for one underlying model request (e.g. streamed in
  chunks), the token counts are de-duplicated so a single request is never
  counted more than once. (D2 · `81456c6e`)
- **R2 — Exclude non-model events.** Placeholder/local events that carry no
  real model call are excluded from every token total. (D2 · `81456c6e`)
- **R3 — Running time is active time, never "alive" time.** The running
  time is the sum of the session's own measured per-turn active durations
  within the span, which already exclude idle waiting. When those measured
  durations are unavailable, the fallback sums the gaps between consecutive
  activity events, ignoring any gap longer than the idle threshold (5
  minutes) so a long wait for the operator is never counted. Plain
  end-minus-start wall-clock is never used. (D3 · `9f7f4256`)
- **R4 — Parallel means genuinely concurrent helpers.** A section is
  parallel when two or more helper runs overlap in time within the span, or
  when a single turn dispatched two or more helpers at once. Otherwise it
  is sequential. (D2 · `81456c6e`)
- **R5 — Helper cost is attributed, not hidden.** Tokens spent by helper
  runs are gathered from each helper's own activity record and reported in
  a separate subagent breakdown, so worker cost is visible and not silently
  folded into or dropped from the main totals. (D2 · `81456c6e`)
- **R8 — A missing measurement never fails the operation.** If the activity
  record or open section is missing, the operation degrades to
  empty/zeroed metrics or a clear message and never errors out.
  (implementation-confirmed; supports the read-only examples staying safe)

## Edge Cases Settled

- **E1 — Boundary events** — an event exactly at the start or end moment of
  the span is included.
- **E2 — No open section at close** — reported clearly; nothing is
  written.
- **E3 — Unparseable trailing window** — reported clearly; nothing is
  recorded.
- **E5 — No resolvable session at open** — the section opens; the operator
  is warned metrics will be empty at close.

## Open Gaps

- **Per-unit-of-work sections.** Emitting an explicit *section*
  automatically per bee unit of work (its own start-to-finish window) is
  still deferred — the automatic path is the whole-matrix refresh, not
  per-cell sections. Manual sections remain operator-initiated. (backlog:
  proposed)
- **Cost in currency.** Only raw token counts are recorded; converting to
  money is left to the reader.
- **One logical section spanning multiple sessions** is not merged; a
  section measures a single session's activity record. The matrix
  aggregates all of a project's sessions, but does not reconstruct a single
  logical task that crossed sessions.

## Pointers (implementation)

- **P2** — CLI surface `bee perf start|stop|section|log|render|report|sync`:
  **NOT BUILT INTO THE CURRENT BINARY.** The handlers lived in
  `the bee binary`, which the R6 Node deletion removed; no Rust port
  replaced them, so every one of these verbs now refuses by name (the registry
  entries carry an `unavailable` marker, and `bee --help --all` prints it).
  What survives without them: every command appends its own wall time to
  `.bee/logs/timings.jsonl`. What is described below is the design as it stood
  and as it would be re-implemented — read it as a specification, not as a
  surface you can call today.
- **P4** — Open-section marker: `.bee/perf-open.json` in the project
  working copy.
- **P6** — Data source: Claude Code session transcripts at
  `~/.claude/projects/<encoded>/<session>.jsonl` plus the
  `<session>/subagents/agent-*.jsonl` sidecars; running time from the
  harness `system`/`turn_duration` `durationMs` events.
