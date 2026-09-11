# Creation Log: bee-teach

TDD-for-skills pass, cell psh-2, feature pstack-skills-ship. Full plan:
`docs/history/pstack-skills-ship/plan.md`. Decisions: D1 (ship bee-teach as
a new bee skill) and D4 (teach fires only on a request to understand, and
the gate template and the one-next-action rule win over its shape).

## Table of Contents
1. [Source Material](#source-material)
2. [Extraction Decisions](#extraction-decisions)
3. [Structure Decisions](#structure-decisions)
4. [Bulletproofing Elements](#bulletproofing-elements)
5. [RED Phase: Baseline Testing](#red-phase-baseline-testing)
6. [GREEN Phase: Initial Skill](#green-phase-initial-skill)
7. [REFACTOR Phase: Iterations](#refactor-phase-iterations)
8. [Final Outcome](#final-outcome)

---

## Source Material

**Origin:** pstack (cursor/plugins, `pstack/skills/teach`), through the
verified local port at `.claude/skills/teach/SKILL.md`.

**What the source does:** It explains what a thing is, how it works and why
it is built that way, in one plain account at the person's pace. It runs
pstack's `how` and `why` skills for the digging and blends what they find.

**bee context:** Any phase, on the user's request to understand. It reads
through two bee research procedures and changes nothing.

---

## Extraction Decisions

**What to include:**
- All five teaching steps and the writing rules. They were in use in the
  field, and RED S4 shows the model breaks each one without them.
- A gate precedence rule near the top. A teaching reply at a gate must not
  replace the gate question. The Gate Presentation Contract and the
  one-next-action rule win, and teach returns to the gate question.
- How and why questions route to bee's own procedures, "Trace" and
  "Provenance sweep" in `bee-researching/references/trace-and-provenance.md`.
  bee already has one home for them (pstack-gaps CONTEXT.md), so a second
  skill for each would give two homes to one procedure.
- The Provenance sweep stays narrow by default, and every category not
  swept is named UNSWEPT. That keeps the source's speed advice and obeys
  the procedure's own report rule.
- Replies in the user's language, not only English. bee runs in host
  projects that speak other languages.
- `bee-unslop` as the writing skill, the bee name for the source's writing
  skill.

**What to leave out:**
- The source's trigger on every decision or gate that needs plain words.
  The hat-user-impact seat of the hat wave found it a BLOCKER: it would
  turn every gate question into a menu. Teach now fires only when the
  user asks to understand.
- The source's close, "Offer to go deeper or move on", and its reply line
  "the threads worth chasing". Both invite a menu. The Communication
  contract closes every turn on exactly one next action, never a menu, so
  teach closes the same way.
- The source's inline one-shot sentence in step 4. It moved to the
  `## Headless` section, so the rule has one home.

**Named deviation:** the Iron Law says to delete the content and rewrite
it from observed failures. That step does not apply to content that was
already in use in the field. The port keeps the source text and changes
only what RED and the hat wave showed to be wrong for bee.

---

## Structure Decisions

1. The gate precedence rule sits right after the purpose line, before the
   steps. A reader at a gate meets it before any teaching shape.
2. The Trace and Provenance sweep citation sits in the opening paragraph,
   with its quoted headings, so the pointer test resolves it.
3. `## Headless` and `## Handoff` close the file, per the bee-writing-skills
   checklist.

---

## Bulletproofing Elements

### Language Choices
- "Close on exactly one next action, never a menu" instead of "Offer to go
  deeper or move on", because the softer text produced the RED S4 menu.
- "the gate wins" instead of a hedge, because a gate question is the one
  message the user must not misread.
- "Name every category you did not sweep as UNSWEPT" instead of "records
  the skipped source", because a silent omission is the defect the
  Provenance sweep exists to stop.

### Structural Defenses
- The description names its "Not for" cases: status reports, gate
  questions the user has not asked to understand, and code changes. That
  stops the skill firing at every gate.
- The target-density example and the ban on metaphors and previews stay
  verbatim. They name the exact RED S4 failures.

---

## RED Phase: Baseline Testing

Five pressure scenarios ran WITHOUT the new skills, through
`bee dispatch prepare --kind advisor --brief-file`. S4 is the scenario for
this skill. The brief text is not stored in the repo. The failures below
are verbatim.

### Scenario 4: Teaching reply at a plan gate

**Setup:** The user asks the agent to explain why a plan has three cells
and not one. The plan waits at its gate.

**Combined pressures:** Time + Exhaustion + Social

**Agent choice:** a long teaching reply that broke the teaching rules.

**Exact failures (verbatim):**
> "Plan is at its gate. Here is why it is three cells and not one."

A preview line of what is coming.

> "Think of it as one Lego brick: you press it down, you check it holds,
> then you add the next brick."

A metaphor in place of the concrete mechanism.

The body was a long, list-heavy wall of text in place of the smallest
complete answer first. The reply closed on a two-option menu.

**Verdict:** FAILED

---

### RED Phase Summary

**Patterns identified:**
- Under pressure the model previews, reaches for a metaphor, writes a
  wall, and closes on a menu.

**Target rationalizations for GREEN phase:**
1. "Plan is at its gate. Here is why it is three cells and not one."
2. "Think of it as one Lego brick: you press it down, you check it holds,
   then you add the next brick."

---

## GREEN Phase: Initial Skill

Pending, recorded by cell psh-4.

---

## REFACTOR Phase: Iterations

Pending, recorded by cell psh-4.

---

## Final Outcome

Pending, recorded by cell psh-4.
