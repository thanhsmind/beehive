# skill-report-stamps — locked context

## Why

`refs/ponytail` (DietrichGebert/ponytail) ships six skills in 383 lines with
zero reference files. Reading them as *craft* surfaced habits bee's
report-shaped skills do not have. Three real bee reports were checked as
baseline evidence and none of them ends with a count line:

- `docs/history/bee-footprint/reports/review-1.md` — ends on a "Residuals" paragraph
- `docs/history/budget-fence-removal/reports/stale-rule-pointers.md` — ends on a "Completeness" paragraph
- `docs/history/worktree-session-routing/reports/wsr-1.md` — ends on "Deviations" prose

Three reports, three endings, no scoreboard. The reader must read the whole
document to learn how big the result was.

## Locked decisions

1. **A report-shaped skill ends with ONE required line.** It carries the
   counts. The empty case is written out verbatim in the skill, because an
   unwritten empty case is what makes an agent pad instead of saying
   "nothing found".

2. **Findings that are countable get a one-line stamp and a closed tag
   list.** Every tag names its replacement or action. This applies to
   grooming, whose job is ponytail-audit's job. It does NOT apply to
   `bee-reviewing`'s per-finding schema: a P1 security finding earns its
   depth, and that schema stays as it is. Only the *summary* line is added
   there.

3. **A skill that prints numbers names the number it must never invent, and
   why.** Ported from `ponytail-gain`'s honesty boundary ("the unbuilt
   version was never written, so there is no baseline to subtract from").
   Grooming predicts impact and reports an entropy score; it may not state a
   savings figure that was not counted.

4. **A Boundaries block routes what it refuses.** Naming the out-of-scope
   concern is half the rule; naming where it goes instead is the other half.

## The shapes — locked, not the worker's to redesign

**bee-reviewing**, required last line of the synthesis report:

```
<N> finding(s) — P1 <a>, P2 <b>, P3 <c> · axis: spec <s>, standards <t>.
```

Empty case, verbatim: `No findings. Scope clean — <N> file(s), <M> capped cell(s) verified.`

**bee-grooming**, one line per candidate:

```
<tag> <what to cut>. <what replaces it>. [<path>]
```

Closed tag list, and nothing outside it: `dead:` `stale:` `dupe:` `stub:`
`prune:` `structural:`. Every row must name a replacement; `dead:` names
`nothing`.

Required last line: `<N> candidate(s) — <k> proposed, <r> ranked out. entropy <e> (<trend>).`
Empty case, verbatim: `Nothing worth killing. Ship.`

**bee-capturing**, the close line:

```
captured: <what settled> → <where it landed>.
```

Empty case, verbatim: `nothing settled.`

## Out of scope

- Shrinking `bee-reviewing`'s finding schema. Depth there is correct.
- The skill A/B benchmark harness (ponytail's `benchmarks/agentic/`). That is
  separate, larger work; it was presented to the user and is not this feature.
- Any other bee skill. Three files only.
