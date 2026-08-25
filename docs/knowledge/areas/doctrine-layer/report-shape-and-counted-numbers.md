---
type: bee.area
title: Doctrine Layer — report shape and counted numbers
description: "The shape a report-shaped skill's output takes: one required closing count line with its empty case written verbatim, a one-line stamp over a closed tag vocabulary for countable findings, a Boundaries block that routes every concern it refuses, and the rule that a skill printing numbers must name the figure it may never invent."
tags: [doctrine-layer, communication, reporting, honesty]
timestamp: 2026-08-25
bee:
  id: doctrine-layer-report-shape-and-counted-numbers
  lifecycle: active
  areas: [doctrine-layer]
  required_context: [areas/doctrine-layer/overview.md, areas/doctrine-layer/the-communication-contract.md]
  decisions: ["213799df (a report-shaped skill ends with ONE required last line carrying the counts, and the empty case is written out verbatim in the skill body)", "1bcb1cec (countable findings are reported as ONE line over a closed tag vocabulary, and every tag names its replacement or action)", "44175a6f (a skill that prints numbers names the number it may never invent, and says why it cannot be known)", "3a9a303e (a Boundaries block names the out-of-scope concern AND where it routes instead)"]
  sources: ["docs/history/skill-report-stamps/ (small lane, cells srs-1/srs-2/srs-3, 2026-08-25)", "skills/bee-reviewing/SKILL.md (the count line and the Boundaries block this concept describes)", "skills/bee-grooming/SKILL.md (the stamp, the closed tag list, the count line and the honesty boundary)", "skills/bee-capturing/SKILL.md (the close line and its verbatim empty case)", "the ponytail skill collection by DietrichGebert, read out-of-tree — six skills whose endings and honesty boundary the craft was distilled from"]
  authoritative_for: "doctrine-layer: the shape of a skill's report output and the honesty of its numbers"
---

# Doctrine Layer — report shape and counted numbers

## Purpose

The communication contract governs the shape of a *turn* — what crosses from execution into the
conversation. It does not govern the shape of a *report*: the artifact a review, an audit, or a
capture pass leaves behind. Left unspecified, that artifact ends wherever the writer ran out of
things to say, so a reader cannot learn the size of the result without reading all of it, and a
clean result gets padded with weak observations to look like work.

This concept is the ruleset for that artifact: how it ends, how a countable finding is written,
what it refuses and where it sends what it refuses, and which of its numbers are allowed to exist.

## Entry Points & Triggers

- A skill that produces a report of findings, candidates, or settlements finishes a round.
- A skill that would print a number describing benefit — time saved, lines saved, a percentage —
  is about to state it.
- An author adding or editing a report-shaped skill decides how its output ends.
- A round that found nothing — the trigger this concept exists for, because it is the one the
  writer is most tempted to fill.

## Data Dictionary

| Element | Meaning |
|---|---|
| **Report-shaped skill** | A skill whose product is an artifact enumerating findings, candidates, or settlements, rather than a change to the work itself. Reviewing, grooming, and capturing are the three today. |
| **The count line** | The single required last line of a report, carrying the counts and nothing else after it. Its content is per-skill; its obligation is not. |
| **The empty case** | The verbatim wording a report uses when it found nothing, written into the skill body rather than left to the writer. An unwritten empty case is what produces padding. |
| **The stamp** | The fixed one-line shape a countable finding takes, so findings can be counted and compared rather than read as prose. |
| **Closed tag vocabulary** | The complete, exhaustive list of tags a stamp may carry. A candidate matching no tag is not yet a candidate. Every tag obliges the row to name a replacement or an action. |
| **The forbidden number** | The figure a report may never state because it was never measured — the cost of a thing that was kept, the benefit of code never written. Named explicitly, with its reason, inside the skill that would otherwise invent it. |
| **Counted number** | A figure the report actually counted: candidates, files, hits, findings per severity, a recorded score with its trend. These are the only numbers a report may print. |
| **Boundaries block** | The section naming what a skill does NOT do, where each excluded concern is routed instead, and how the skill is turned off. |

The three report-shaped skills and their required last lines:

| Skill | Count line | Empty case, verbatim |
|---|---|---|
| Reviewing | `<N> finding(s) — P1 <a>, P2 <b>, P3 <c> · axis: spec <s>, standards <t>.` | `No findings. Scope clean — <N> file(s), <M> capped cell(s) verified.` |
| Grooming | `<N> candidate(s) — <k> proposed, <r> ranked out. entropy <e> (<trend>).` | `Nothing worth killing. Ship.` |
| Capturing | `captured: <what settled> → <where it landed>.` | `nothing settled.` |

## Behaviors & Operations

**Ending a report.** Trigger: a report-shaped round finishes. The report's last line is the count
line, with nothing after it. What the reader observes: the size of the result is legible from the
final line alone, without reading the body. What the reader never observes: a report that trails
off into a summary paragraph, leaving them to total the findings themselves.

**Reporting an empty round.** Trigger: the round found nothing. The skill's verbatim empty-case
wording is written, alone, and the round stops. What the reader observes: a short, unambiguous
"nothing here". What the reader never observes: weak candidates manufactured to fill a line that
looked too short, or a clean result dressed up as a partial one.

**Writing a countable finding.** Trigger: a round produces a candidate that can be counted and
acted on. It is written as one stamped line carrying its tag, what to cut, what replaces it, and
where it lives. What the reader observes: rows they can rank, count, and act on. What the reader
never observes: a hedged paragraph that describes a concern without committing to an action.

**Declining to state a benefit figure.** Trigger: the report is about to quantify what an action
bought. Only counted numbers are printed; the benefit stays prose. What the reader observes: "3
unused functions across 2 files", and a predicted impact in words. What the reader never observes:
"saves ~4 hours a month" — a figure with no baseline behind it.

**Refusing a concern.** Trigger: something surfaces that the skill does not handle. The Boundaries
block names it as out of scope and names the destination that does handle it. What the reader
observes: a refusal with a forwarding address. What the reader never observes: a concern silently
absorbed because the skill had no place to send it.

## Actors & Access

| Actor | Observes |
|---|---|
| The person being served | A report whose last line states its size, whose empty case is unmistakable, and whose numbers are all countable ones. |
| The acting side (the assistant) | A fixed output shape per report-shaped skill, so the ending is never re-invented per round. |
| An author editing a report-shaped skill | The obligation that the skill body carries the count line, the verbatim empty case, and the Boundaries block — never a reference file, which nothing forces open. |

## Business Rules

1. **A report-shaped skill ends with ONE required line, and it is the last line.** Nothing follows
   it (213799df).
2. **The empty case is written verbatim into the skill body.** Not described, not paraphrased — the
   exact string the skill emits when it found nothing. An unwritten empty case is what makes a
   writer pad (213799df).
3. **A countable finding is one stamped line over a closed tag vocabulary, and every tag names its
   replacement or action.** A candidate matching no tag is not yet a candidate: hunt it into one or
   drop it (1bcb1cec).
4. **Depth and stamping serve different jobs, and the stamp never displaces depth.** A severity-P1
   review finding keeps its full per-finding schema; only the summary line is stamped. The stamp
   governs the countable-candidate case alone (1bcb1cec).
5. **A skill that prints numbers names the figure it may never invent, and says why it cannot be
   known.** What a kept thing costs was never measured, so there is no baseline to subtract from
   (44175a6f).
6. **Only counted numbers are printed.** Candidates, files, hits, findings per severity, a recorded
   score with its trend. Predicted benefit stays prose (44175a6f).
7. **A Boundaries block names the out-of-scope concern AND its destination.** Naming the exclusion
   is half the rule; naming where it goes instead is the other half (3a9a303e).
8. **These clauses live in the skill body, never in a reference file.** A rule nothing forces open
   is a rule nothing follows — the same placement law the per-turn rules obey.

## Edge Cases Settled

- **A clean result is a result.** The empty case exists so that finding nothing is a legitimate,
  short, complete report. Padding it into something longer is the failure mode this concept was
  written against.
- **Two artifacts, two endings.** Grooming's count line ends the round *in the conversation*; the
  rendered proposal file keeps its own single-recommendation ending. Both nouns say which artifact
  they govern, so neither rule reads as overriding the other.
- **Reviewing's per-finding schema was deliberately left long.** It was examined during this work
  and kept: a P1 security finding earns its depth, and shrinking it to a stamp would have traded a
  real safeguard for uniformity.

## Open Gaps

- Nothing mechanically checks that a report actually ends on its count line, or that an empty round
  used the verbatim wording. Adherence is self-applied, exactly like the communication contract's
  own seven rules, and this gap must never be reported as closed.
- The rules are stated for the three report-shaped skills that exist today. Whether a fourth
  inherits them automatically, or restates them, is unsettled.

## Pointers (implementation)

- The governing text lives in the three skill bodies: `skills/bee-reviewing/SKILL.md`
  ("Boundaries", and the count line at the end of the synthesis section),
  `skills/bee-grooming/SKILL.md` ("End every round on the count line", "Numbers — the honesty
  boundary", and the stamp folded into "Propose"), and `skills/bee-capturing/SKILL.md` (the close
  line in "Capture the moment it settles").
- The turn-level sibling is `areas/doctrine-layer/the-communication-contract.md`; that concept
  governs the conversation turn, this one governs the report artifact.
- The placement law that keeps these clauses in the body rather than a reference is
  `areas/doctrine-layer/placement-and-anchoring.md`.
- Landed by feature `skill-report-stamps` (cells srs-1, srs-2, srs-3; merged at `0544b9c`).
  Evidence: `.bee/cells/srs-1.json`, `.bee/cells/srs-2.json`, `.bee/cells/srs-3.json`, and
  `docs/history/skill-report-stamps/CONTEXT.md`, which records the baseline: three real reports
  (`docs/history/bee-footprint/reports/review-1.md`,
  `docs/history/budget-fence-removal/reports/stale-rule-pointers.md`,
  `docs/history/worktree-session-routing/reports/wsr-1.md`) each ending on a different prose
  paragraph, none carrying a count.
- Distilled from `refs/ponytail`, whose six skills each end on a mandatory line with its null
  wording spelled out, and whose `ponytail-gain` skill supplied the honesty boundary.
