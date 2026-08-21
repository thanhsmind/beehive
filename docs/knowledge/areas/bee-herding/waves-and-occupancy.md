---
type: bee.area
title: "Bee Herding — waves over running workers, and counting the slots they occupy"
description: "A wave as a fan-out over workers that already exist, the append-only ledger written at the moment of the spawn, occupancy as a liveness question rather than a pane count, and why an unverifiable count refuses instead of guessing."
timestamp: 2026-08-20
bee:
  id: bee-herding-waves-and-occupancy
  lifecycle: active
  areas: [bee-herding]
  required_context: [areas/bee-herding/overview.md]
  decisions: ["herding-orchestration D2/D5 (the choreography is generic, behind a compiler-enforced boundary)", "herding-orchestration D7 (unverifiable is one of five worker states)", "herding-orchestration D9 (blocking threads, not an event runtime)", "herding-orchestration D10 (the wave ledger, written at spawn)", "herding-orchestration D11 (a wave is one value, not a sequence of calls)", "herding-orchestration D18 (dispatch records its own ledger row at spawn)"]
  sources: [docs/history/herding-orchestration/CONTEXT.md, "the first live D6 run on Linux, 2026-08-19"]
  authoritative_for: "bee-herding: waves, the wave ledger, and occupancy"
---

# Bee Herding — waves over running workers, and counting the slots they occupy

**A wave is a fourth entry point, and no role calls it.** Dispatch starts one
worker per iteration and never speaks to it again; a wave briefs several
ALREADY-RUNNING workers in one act and waits on all of them together. It carries
none of dispatch's guards — no arming marker, no classifier — so it is a fan-out
over workers that already exist, never the way ordinary backlog work is started.
It is invoked directly, by a human or by an agent that was told to.

## Data Dictionary

- **Wave** — one coordinated run over several workers, described as a single
  value rather than a sequence of calls: the worker list, the timeouts, and the
  failure policy (wait-for-all, first-success-cancel-rest, best-effort) all sit
  in that one value, so a scenario is something you hand over rather than
  something you perform (herding-orchestration D11).
- **Wave ledger** — the append-only record of what each wave did: one row per
  wave, one entry per worker carrying its name, its pane, its worktree, its brief
  and its outcome. It is the cockpit's memory of who was started, and it is
  written at the moment of the spawn rather than at the end
  (herding-orchestration D10).
- **Occupancy** — how many working slots are actually taken. It is answered by
  crossing the ledger's unresolved workers against the live pane list, and it
  carries the SOURCE of its own answer: a real crossing, or a degraded timer
  fallback used when the live list cannot be obtained.

## A wave is run once and recorded once

The coordination that drives it is deliberately generic — it knows nothing about
this tool's own vocabulary, and lives behind a boundary a compiler enforces
rather than a promise (herding-orchestration D2/D5). Workers run beside each
other on ordinary threads rather than on an event runtime, because a wave is a
handful of workers and each waiter is a blocking poll (herding-orchestration D9).

The entry point takes the worker list on its input, runs the whole choreography —
resolve and de-duplicate the targets, refuse any target that is not safe to
disturb, take a baseline, re-check each target immediately before handing it its
brief, then wait on all of them at the same time and aggregate what came back —
and appends exactly ONE ledger row for the whole wave.

Each worker's outcome is classified into a named bucket (finished, refused at
pre-flight, changed under us before the send, send failed, timed out, or
unverifiable afterwards) rather than into a bare pass/fail, because partial
failure is the normal case and the caller needs to know which kind it got. A
worker that fails does not stop the others.

## Occupancy is read, and an unverifiable read refuses

The dispatch role asks for the occupancy count instead of counting panes itself,
and it reads WHICH answer it got. On a real crossing it compares the count
against the four-slot cap as before. On the degraded fallback it cannot know
occupancy, so it reports one plain line saying so and dispatches nothing that
iteration.

The fallback fires exactly when the live pane list could not be obtained — which
is also when counting panes would have failed — so refusing is not a lost
opportunity, and dispatching on a count nobody can verify is the over-spawn the
ledger exists to prevent.

## Edge Cases Settled

- **A working agent that fails to name its own pane** used to leave a slot
  looking free, because the four-slot cap was enforced by the control model
  counting panes. **That hole is closed** — the cap now rests on the wave ledger,
  not on a pane count (herding-orchestration D10, D18). Dispatch records a row the
  moment it spawns, carrying the worker's pane id, so an agent that never names
  its own pane is still visible to the next iteration: the ledger knows the pane
  even when the pane does not know itself. Occupancy is a liveness question — the
  ledger's unresolved pane ids crossed against the multiplexer's own live pane
  list — and a one-hour timer survives only as an explicitly tagged FALLBACK for
  when that list cannot be obtained. Dispatch refuses to act on a fallback answer
  rather than guessing. The case is still worth knowing, because it names the
  class: a cap enforced by counting what you can see is only as good as the naming
  discipline of the things being counted.
- **"Idle" tracks the pane's own focus, not the work.** A worker's runtime status
  flips to idle or done according to whether that individual pane has been seen,
  not according to whether the work finished — a pane reported done while never
  being focused, and the multiplexer's own documentation states a coarser
  tab-level rule than its behavior actually follows. Any reading of a worker's
  status must therefore treat "done" as a fact about attention, never as evidence
  that the work is complete; that is why an explicitly UNVERIFIABLE outcome is a
  first-class answer rather than an error (herding-orchestration D7, which makes
  unverifiable one of the five worker states a backend must map its own
  vocabulary onto).

## Open Gaps

- **A wave cannot confirm that a worker finished.** Proven by the first live run
  on Linux: two workers were started in their own worktrees, took their briefs and
  answered correctly, and the wave still reported both as UNVERIFIABLE — which is
  the honest answer, because the only completion signal available tracks the
  pane's attention rather than the work (see Edge Cases). The consequence is that
  a wave over ordinary agent sessions reports overall failure even when every
  worker did its job, so today the ledger row and the pane's own output are what
  an owner reads, not the verdict. Closing this needs a completion signal the
  worker itself emits — which is exactly what the run verb's mailbox provides for
  the workers it starts itself.

## Pointers (implementation)

- The generic choreography lives in the fleet crate,
  `packages/bee-rs/crates/fleet/`; the wave entry point and the ledger's read and
  write sides are in `packages/bee-rs/crates/bee/src/herding/wave.rs`.
