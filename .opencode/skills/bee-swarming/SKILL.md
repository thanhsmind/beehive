---
name: bee-swarming
description: >-
  Run approved cells to done — orchestrate bounded workers over gate-approved cells, or execute exactly one assigned cell inside a dispatched worker. Use when the merged shape+execution gate is approved and current-slice cells are open, or when running as a worker that received an assigned cell id.
metadata:
  version: '0.2'
  ecosystem: bee
  dependencies:
    bee-cli:
      kind: command
      command: .bee/bin/bee
      missing_effect: unavailable
      reason: Both roles drive the work through the vendored bee binary. The binary is vendored into the repo by onboarding; no Node runtime is involved.
---

# Swarming — run the approved work

A **worker** has an assigned cell id in its dispatch prompt; everyone else
is the **orchestrator**. `bee orient` shows where the work stands either way.

## Orchestrate

You launch workers and tend results; you do not implement. The wave runs
inside the feature's worktree (worktree-first — AGENTS.md). A `tiny` cell
may run inline in this session;
`small` and up always dispatches — one worker per cell
(`references/swarming-reference.md` ("Single execution worker in full")),
parallel by default: disjoint cells fan out concurrently
(reservations prove it, 3-4 live workers cap it), serial needs a named
file conflict. From two
cells up, state the one-line concurrency plan before dispatching.

1. `bee cells schedule --json` sets dispatch order — override only with a
   stated reason. Overlapping-file cells are fixed by scope or
   reservations, never by "spawn both carefully".
2. `bee dispatch wave --runtime <rt> --feature <f>` prepares the current
   wave of one feature in one call — claim, reserve, and payload per
   ready cell, refusals landing in `skipped` with a typed reason instead
   of aborting the batch (`--feature` omitted only resolves from a bound
   session lane or the default record's own feature; nothing resolving is
   a typed refusal, never an every-feature grab — pass `--limit <n>` to
   bound how many cells of the wave are claimed). One cell, or a cell
   needing its own worker name, takes
   `bee dispatch prepare --cell <id> --worker <name> --runtime <rt> --claim`
   instead. Judge and record the model tier first (`bee cells tier`;
   rubric: `references/swarming-reference.md`).
3. Spawn with exactly that payload — a whole wave goes out in ONE message,
   one tool call per cell. Never paste session history; never hand a
   worker two cells.
4. Tend: read each worker's Result form (the fenced
   `{outcome, commit, files, tests, deviations}` block its prompt
   requires), never its prose. Silence is not failure — inspect
   `bee cells list` and `bee reservations list` before assuming stuck.
5. On `[DONE]`: the worker's word is never the evidence. Goal-check on
   smell; `bee cells judge` for undeclared-file hits. At
   `standard`/`high-risk` the semantic judge is not judgment but a door:
   every `behavior_change` cell owes a `bee cells judge-record` verdict or
   `bee close` refuses (`judge-debt`), so run the slice judge before you
   reach for close.
6. Slice clean: `bee close --feature <slug> --dry-run` names every
   remaining door with the command that settles it; the final slice runs
   `bee close --feature <slug>`, which re-runs the declared tests
   (`bee test`) for the feature. Doors are never waived.

**`[BLOCKED]` rescue ladder:** (1) re-dispatch the same cell with the
missing context; (2) next model tier up — the ceiling is this session, so
the top rung hands the blocker to you; (3) surface it to the user with the
worker's diagnosis. If it invalidates the plan, return to bee-planning.

**Completion:** slice done with more approved work remaining → return to
bee-planning for the next batch (an approved plan stays frozen; planning
shapes the next batch, never reopens it). Final slice green → tell the
user execution is complete; capture is recorded as pending (bee-capturing
runs later, at the owner's pace) and landing is `bee worktree merge` from
main. Before declaring done: no active reservations, no in-flight
workers recorded.

The 65%-context handoff (AGENTS.md) holds mid-wave — never push through
the budget. When a unit finishes and approved work remains, continue
in-session; finishing a unit is never a reason to stop.

## Execute (worker)

Your dispatch prompt is the assignment: one cell, claimed for you, its
listed files reserved under your nickname. Everything else comes from CLI
outputs — when a verb refuses, its message names the fix.

1. Read `AGENTS.md`, then the cell's `CONTEXT.md` and plan (paths in the
   prompt). Conform before you code: scout adjacent patterns, reuse
   existing helpers, match the codebase's idiom. Authoring tests? Judge
   existing coverage first — `.bee/expertise/tests.md`.
2. Implement exactly the assigned cell. Reserve any additional path before
   writing (`bee reservations reserve`). Package installs and
   architectural changes are not yours to make — `[BLOCKED]` with the
   proposal.
3. When reality disagrees with the cell: a bug in touched code → fix it,
   record the deviation; a missing piece the outcome depends on → add it,
   record; blocking breakage in your path → fix, record; anything
   architectural → `[BLOCKED]`. Never reinterpret a locked decision to
   make the cell fit. An unexpected red or an unfamiliar mechanism
   mid-cell is a pull moment: `bee knowledge search --text "<symptom>"`
   surfaces matching patterns and area concepts before you guess.
4. Commit once: subject describes the change in imperative mood; the cell
   id rides the last line of the body.
5. `bee finish --id <cell> --outcome "<one line>" --files <a,b>
   --report '<json>'` — cap and release in one verb, `--report` carrying
   the same Result form you return (`{outcome, commit, files, tests,
   deviations}`), which finish validates key-for-key onto the trace.
   Finish runs the declared tests (`commands.test`): green caps; a red
   refuses, and the refusal carries the failing test excerpt — that red is
   now your work. Tests run at finish; close re-runs them for the feature.
6. Return exactly one token, first thing in your final message, and the
   Result form beside it — never in place of it:
   `[DONE]` (outcome, files, commit) · `[BLOCKED]` (what, why, your
   diagnosis) · `[HANDOFF]` (the 65% handoff, AGENTS.md — handoff file
   written before the token) · `[NOOP]` (cell missing or already capped). Never wait
   silently; never ask a blocking question — you run headless.

## Hard rules (both roles)

- Never spawn before the execution gate is approved; the orchestrator
  never edits source in a `standard`/`high-risk` wave.
- One cell per worker; the claim guard refuses a worker that claims, browses, or self-selects.
- Conflicts are fixed in scope or reservations, never by being careful.
- Never build on a red base — a red becomes its own fix-first cell.

## Headless

`bee-hive` ("Headless") governs; waves run without check-ins, and an
unrescuable blocker becomes an `Outstanding Questions` entry.

## References

| File | When to load |
|---|---|
| `references/swarming-reference.md` | Tier rubric, worktree dispatch transaction, prompt template details, result formats |
| `references/worker-details.md` | Deep worker mechanics: finish and its refusals, advisor consult, friction triggers |
| `.bee/expertise/tests.md`, `.bee/expertise/debugging.md` | Authoring tests; hunting a red |
| `.bee/expertise/INDEX.md` | The cell is domain work — stored data, a caller-facing contract, a trust boundary, a rollout, a speed budget, a surface people use: route from the index, load exactly one |
