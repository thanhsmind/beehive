---
type: bee.area
title: "Bee Herding — presence, the wake report, and how autonomy is earned"
description: "The away/back mark with exactly two effects, the single bounded report that back renders and no second back can duplicate, the seven derived health counters with two-sided bands and a first-class not-measurable verdict, and the narrow fail-closed silence-is-consent mode that gates always outrank."
timestamp: 2026-08-28
bee:
  id: bee-herding-presence-wake-reports-and-earned-autonomy
  lifecycle: active
  areas: [bee-herding]
  required_context: [areas/bee-herding/overview.md, areas/bee-herding/the-supervisor-observer-and-its-interventions.md]
  decisions: ["slp-supervisor-heartbeat 9f5cd250 (presence is an explicit away/back mark with exactly two effects; on back exactly ONE WakeReport of at most 10 lines in four sections, plus one push notification; gates and bypass levels untouched)", "slp-supervisor-heartbeat 66c4c251 (reports carry health metrics with two-sided bands; raising silence-is-consent is EARNED and the human still flips the switch; assumptions sort by impact-if-wrong descending)", "slp-supervisor-heartbeat c706053e (a NARROW opt-in silence-is-consent mode: only when explicitly enabled, only for non-gate queued asks, user-configured timeout, every auto-proceed logged and rendered prominently)", "slp-supervisor-heartbeat ea02cb68 (the 2x-estimate overrun signal is SKIP-UNTIL-PRESENT: bee adds no estimate field to the cell schema)", "slp-supervisor-heartbeat a8f4b8ab (the night queue sorts by the confidence x door predicate: a one-way door at low confidence always waits)"]
  sources: [docs/history/slp-supervisor-heartbeat/CONTEXT.md, docs/history/slp-supervisor-heartbeat/plan.md, "slp-supervisor-heartbeat cells sup-8, sup-9 (the presence mark's two effects, the one-report-per-window door; traces in `.bee/cells/`, 2026-08-27)", "slp-supervisor-heartbeat cells sup-10, sup-11 (derived counters with two-sided bands, fail-closed silence-is-consent; traces in `.bee/cells/`, 2026-08-27)", "capture stub 3b7b9e9c (Phase 3 shape: presence effects, the single-report door, report ordering)", "capture stub 4ad73d79 (Phase 4 shape: derived counters, the blocked-rate denominator, the named-variant consent predicate)"]
  authoritative_for: "bee-herding: the presence mark, the wake report, supervisor health metrics, and silence-is-consent"
  owns.code: [packages/bee-rs/crates/bee/src/verbs/supervisor.rs]
---

# Bee Herding — presence, the wake report, and how autonomy is earned

**Presence is a mark, not a permission.** The human says *away* when they stop
watching and *back* when they return. That mark has **exactly two effects**
(9f5cd250), and naming them exhaustively is the whole safety argument:

1. It sets the **report window** — the current window opens on *away* and
   closes on *back*, which becomes the last closed window.
2. It stamps non-urgent asks **queued**, so they fire no notification while the
   human is away.

Nothing else moves. Gates do not open. Bypass levels do not change. Permission
posture does not shift. A presence flag that quietly widened what the machine
may do would be permission control hiding in a convenience switch, and that is
the thing this design refuses.

The **urgent class is exempt** from the quiet queue. Danger still wakes you.

## Exactly one report per window

**`back` is the only door that renders a wake report**, and it renders one per
window. A second *back* on the same window returns the report already stored;
it can never add a second one. Two reports for one absence would force the
human to reconcile them, which is the opposite of the job.

The report is bounded on both sides: **four headings**, at most **six content
lines**, a floor of nine lines and a ceiling of ten. The floor exists so the
report cannot degrade into a stub; the ceiling exists so it cannot become
something you skim.

The four sections are: what happened, what was decided, what needs you, and
next action.

Content is ordered by **impact-if-wrong, descending** (66c4c251):

| Rank | Content |
|---|---|
| 4 | urgent alerts |
| 3 | waiting on a gate |
| 2 | escalations |
| 1 | ordinary interventions |

Within a rank, a **one-way door sorts before a reversible one**. When more
content exists than the ceiling allows, the list ends on an explicit **"+N
more"**. Nothing is ever dropped silently — a report that quietly truncated
would be a report you could not trust.

*back* also sends **one** push notification, on the same best-effort channel the
urgent alert uses.

## Health counters are derived, banded, and honest about not knowing

The report carries a metrics line built from **seven derived counters**. Derived
is literal: nothing is accumulated or persisted as a counter. Each figure is
recomputed at report time from stores that already exist — the cell records,
the decision log, the mailbox, and the observation store. A persisted counter
would be a second truth to keep in sync.

Every counter reports three things: a **two-sided band** (both too-low and
too-high are findings), an explicit **sample count**, and one verdict from
`below-band`, `in-band`, `above-band`, or **`not-measurable`**.

`not-measurable` is **first class**. It never renders as `in-band`. A metric
with no data is not a healthy metric, and collapsing the two would let an empty
window read as a good one.

Two counters carry rules worth stating outright:

- **Blocked rate** takes its denominator as the **union** of cells claimed and
  cells blocked in the window. A cell swept into blocked has no claim stamp at
  all, so blocked-over-claimed can exceed one — a "rate" above 100% is a broken
  denominator, not a bad week.
- The **2x-estimate** counter is **skip-until-present** (ea02cb68). Cells carry
  no estimate field and bee adds none for this. The counter computes overrun
  only where an estimate already exists and otherwise reports the literal state
  *no estimate recorded* — never a zero, which would read as "nothing overran".

## Silence-is-consent fails closed

There is one narrow mode where the machine may proceed on silence (c706053e).
Its defaults are the safety:

- It is **off** unless a configuration record explicitly says enabled with a
  timeout of at least one second. Every other reading — missing, malformed,
  partial, ambiguous — is **off**. This is the deliberate opposite of the
  notification channel, which fails **open**: a failed notice may not silence
  the observer, and a failed switch may not grant it consent.
- Eligibility is refused **by named reason**, never by a bare no. The refusal
  names which one applies: a gate, an urgent item, an escalation, an unknown
  kind, a one-way door held at low confidence, an item that was never queued,
  or one already consented. A boolean would leave nobody able to say why the
  machine waited.
- **Gates always wait.** So does a one-way door at low confidence (a8f4b8ab).
  No timeout reaches either.
- Every auto-proceed is written to the decision log first. If that write fails,
  the item **stays queued** — it does not proceed. An unrecorded auto-proceed
  is indistinguishable from a machine acting on its own.
- Each auto-proceed is rendered **prominently** in the wake report. The metrics
  line is the reason the report floor moved from eight lines to nine.

This narrow mode was generalized by slp-human-up's **83baf03f** into a
delegated-decision tier: the supervisor may decide a matter on the
human's behalf only when all four hold — small scope, reversible,
proven observation, inside protocol — one decision per message with a
recorded rollback path; unclear still always escalates.

## Autonomy is earned, and the human still flips the switch

Widening this mode is not automatic and never self-granted. The bar is a clean
run — **zero human-reversed one-way decisions across forty to sixty tasks** —
and even then the change is the human's gesture (66c4c251). The counters make
the case; they never cast the vote.

## Diagram

```mermaid
stateDiagram-v2
    [*] --> Watching
    Watching --> Away: away
    Away --> Away: non-urgent ask queued, no notice
    Away --> Notified: urgent alert (exempt)
    Notified --> Away
    Away --> Consenting: consent sweep, mode explicitly enabled
    Consenting --> Away: refused by named reason (gate, urgent, one-way low confidence, ...)
    Consenting --> Proceeded: timeout elapsed and decision-log write succeeded
    Consenting --> Away: decision-log write failed, stays queued
    Proceeded --> Away
    Away --> Rendering: back
    Rendering --> Reported: one report for this window
    Reported --> Watching
    Reported --> Reported: a second back returns the same report
```

## Pointers

- Presence, report rendering, counters, and the consent sweep: `packages/bee-rs/crates/bee/src/verbs/supervisor.rs`.
- Verb surface: `bee supervisor away | back | presence | report | metrics | consent-sweep`.
- Stores: `.bee/supervisor/presence` and `.bee/supervisor/reports`, under the control root.
- Counter inputs: `.bee/cells/`, `.bee/decisions.jsonl`, the mailbox, and the observation store.
- Companion page: [The supervisor observer and its interventions](the-supervisor-observer-and-its-interventions.md).
