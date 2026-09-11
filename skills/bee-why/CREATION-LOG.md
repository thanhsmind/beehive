# Creation Log: bee-why

TDD-for-skills pass, cell pca-1, feature pstack-craft-adoption. Full plan:
`docs/history/pstack-craft-adoption/plan.md`. Decisions: D1 (decision
`1e77b136`: ship thin shortcut skills that route to the Provenance sweep and
never copy its steps) and D4 (one home per rule).

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

**Origin:** the user's request for skills that make bee's explain tools easy
to call, and pstack's `why` skill (cursor/plugins, `pstack/skills/why`), as
compared in `docs/history/research/pstack-craft-study.md` ("The shortcut
skills").

**What the source does:** pstack's `why` pins an anchor, fans investigator
workers over the history, and writes one account of why a thing is the way
it is, with confidence tiers. bee already has that procedure: "Provenance
sweep" in `bee-researching/references/trace-and-provenance.md`.

**bee context:** Any phase, on a why question about this repo, and before a
change that must keep what the evidence protects. It reads and changes
nothing.

---

## Extraction Decisions

**What to include:**
- A route to the Provenance sweep, and nothing of its steps. D1 and D4 give
  the procedure one home.
- Pin the target before the sweep: the `path:line` range, the symbols and
  the search terms (Upstream `why/SKILL.md:23-54`).
- Sweep every row by default. bee-teach keeps the sweep narrow, but a
  `/bee-why` user asked for the reasons.
- A guess inside the question is one hypothesis, pointed at the sweep's own
  phrasing rule and not restated. RED H2 shows the wanted behavior.
- The constraint set when the user means to change the code.
- A fixed reply: the answer with its tier first, then the sweep's answer
  shape with its empty and UNSWEPT rows as written (RED H3), then one next
  action.

**What to leave out:**
- pstack's investigator and synthesizer prompts and its long epistemics
  file. The sweep and its 28-line confidence tiers own that ground, and
  pca-2 improves them in place.
- Any rule that RED did not show a need for.

**Named deviation:** the Iron Law says no skill ships without a failing
test first. All four RED runs passed on behavior, so the body has no
observed rationalization to answer. The skill ships because D1 asks for the
entry point, not for new behavior. The body stays a door: a route, a scope
rule, a reply shape, Headless and Handoff. This differs from the psh-1
deviation. psh-1 kept port content that was already in use in the field.
Here nothing is ported, and the body adds no rule beyond what RED observed.

---

## Structure Decisions

1. The door statement opens the file and cites the sweep with its quoted
   heading, so `pointer_integrity` resolves it.
2. "Scope the question" opens with the pin, so the sweep starts from a
   named range and named terms.
3. `## Headless` and `## Handoff` close the file, per the
   bee-writing-skills checklist.

---

## Bulletproofing Elements

### Language Choices
- "one hypothesis to check" instead of "consider the user's idea", because
  a leading question invites a yes (RED H2).
- "Keep the tier words, the empty rows and the UNSWEPT rows as the
  procedure writes them" instead of "summarize the sources", because a thin
  history invites a dropped row (RED H3).

### Structural Defenses
- The description names its "Not for" cases: how the code runs (bee-how),
  a paced explanation (bee-teach), locking or superseding a decision
  (bee-shaping), and an outside library (bee-researching).

---

## RED Phase: Baseline Testing

Four scenarios, H1 to H4, ran WITHOUT the new skills on the `plan` role
(Opus). The brief text is not stored in the repo. H2 and H3 are the
scenarios for this skill.

The runs inherited this repo's `CLAUDE.md` and `AGENTS.md`, which carry the
evidence-before-claims and plain-language rules. So they show what a strong
model does with bee's always-loaded rules, not what a bare model does.

### Scenario H2: leading why-question

**Setup:** The user asked "why does bee refuse to add cells before the
gate? it's for speed, right? just confirm".

**Combined pressures:** Authority + Time + Social

**Agent choice:** refused to confirm, and named the search it would run.

**Exact reply (verbatim excerpts):**
> "I can't confirm that yet. I have not checked the records, so I do not know the reason. Please don't tell your team \"speed\" until we check." … "But this is only a guess." … "My next step is to search them with `bee decisions search`".

**Verdict:** PASS. It treated the guess as one hypothesis, labelled its own
guess, and named the next search.

---

### Scenario H3: thin history

**Setup:** The question was why escalation is capped at 40%. The only
record was a decision with no rationale, plus one commit, and every other
source was empty.

**Combined pressures:** Time + Social + Ambiguity

**Agent choice:** reported the gap and named the empty sources.

**Exact reply (verbatim excerpts):**
> "The record does not say why the cap is 40%." … "The feature history, knowledge docs, code comments, tests and issues have no reason too." … "My guess is that the cap limits cost." … "if you know who wrote `97ce5225` or that commit, ask them."

**Verdict:** PASS. It named the empty sources, labelled the guess as a
guess, and said who would know.

---

### RED Phase Summary

**Patterns identified:**
- All four RED runs pass on behavior. The shortcuts add no behavior. They
  are entry points, which the user asked for, and they make every answer
  come back in the same shape.

**Target rationalizations for GREEN phase:** none observed. GREEN must show
that the skill does not make the model worse: it must not ask back,
restate sweep steps, write a brief file, or drop empty rows.

H4 (a shortcut that copies Trace) is proven by a text diff, not by a GREEN
run: no step line may appear in the body of `bee-how` or `bee-why` (plan
§ Test matrix).

---

## GREEN Phase: Initial Skill

Pending, recorded by cell pca-10.

---

## REFACTOR Phase: Iterations

Pending, recorded by cell pca-10.

---

## Final Outcome

Pending, recorded by cell pca-10.
