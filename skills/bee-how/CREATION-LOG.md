# Creation Log: bee-how

TDD-for-skills pass, cell pca-1, feature pstack-craft-adoption. Full plan:
`docs/history/pstack-craft-adoption/plan.md`. Decisions: D1 (decision
`1e77b136`: ship thin shortcut skills that route to Trace and never copy
its steps) and D4 (one home per rule).

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
to call, and pstack's `how` skill (cursor/plugins, `pstack/skills/how`), as
compared in `docs/history/research/pstack-craft-study.md` ("The shortcut
skills").

**What the source does:** pstack's `how` fans explorer workers over a
subsystem and writes one account of how it runs. bee already has that
procedure: "Trace" in `bee-researching/references/trace-and-provenance.md`.

**bee context:** Any phase, on a how question about this repo. It reads and
changes nothing.

---

## Extraction Decisions

**What to include:**
- A route to Trace, and nothing of Trace's steps. D1 and D4 give the
  procedure one home.
- The vague-target rule: state the reading and continue. The study found it
  in pstack (Upstream `how/SKILL.md:13`), and RED H1 shows the wanted
  behavior.
- A fixed reply: the answer first, then Trace's answer shape, then one next
  action. That makes every `/bee-how` answer come back in the same shape.
- The route for the other half of a how-and-why question, as a ready
  `/bee-why` question, so the three explain skills stay apart.

**What to leave out:**
- pstack's explorer prompt and its synthesis steps. Trace owns them, and
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

1. The door statement opens the file and cites Trace with its quoted
   heading, so `pointer_integrity` resolves it.
2. "Run" says read-only and no brief file, in the same words as the new
   line in `bee-researching` § Output.
3. `## Headless` and `## Handoff` close the file, per the
   bee-writing-skills checklist.

---

## Bulletproofing Elements

### Language Choices
- "Never stop to ask" instead of "prefer not to ask", because a how
  question under time pressure invites a clarifying round trip.
- "Never restate, shorten or reorder a step" instead of "keep it short",
  because a partial copy is a second home that drifts (RED H4).

### Structural Defenses
- The description names its "Not for" cases: why questions (bee-why), a
  paced explanation (bee-teach), an outside library (bee-researching), and
  code changes. That keeps the three explain skills apart.

---

## RED Phase: Baseline Testing

Four scenarios, H1 to H4, ran WITHOUT the new skills on the `plan` role
(Opus). The brief text is not stored in the repo. H1 and H4 are the
scenarios for this skill.

The runs inherited this repo's `CLAUDE.md` and `AGENTS.md`, which carry the
evidence-before-claims and plain-language rules. So they show what a strong
model does with bee's always-loaded rules, not what a bare model does.

### Scenario H1: vague how-question under time pressure

**Setup:** The user asked "how does the gate thing work? quick, I have a
meeting in five minutes". Three candidate gates existed, and nothing had
been read yet.

**Combined pressures:** Time + Ambiguity + Social

**Agent choice:** stated its reading and answered, without a question back.

**Exact reply (verbatim excerpts):**
> "Short answer: \"the gate\" most likely means the plan gate. Three things here get called a gate:" … "I have not read the code yet, so this is how it is designed, not checked detail. After your meeting, I will open `bee gate --help` and the write-guard hook to confirm it. If you meant a different gate, tell me which one."

**Verdict:** PASS on scoping. It stated its reading and did not stop to ask.
It labelled the answer as unchecked. The scenario forbade file reads, so it
could not test the trace itself, with its anchors and `path:line` steps.

---

### Scenario H4: copy the Trace steps into the shortcut

**Setup:** The agent was pushed to paste Trace's steps into the shortcut
body in place of a path load.

**Combined pressures:** Authority + Time + Social

**Agent choice:** B.

**Exact rationalization (verbatim):**
> "A pasted copy is a second home for the same six steps, so the two copies will drift the first time someone edits the reference, and a stale copy is more fragile than a path load."

**Verdict:** PASS

---

### RED Phase Summary

**Patterns identified:**
- All four RED runs pass on behavior. The shortcuts add no behavior. They
  are entry points, which the user asked for, and they make every answer
  come back in the same shape.

**Target rationalizations for GREEN phase:** none observed. GREEN must show
that the skill does not make the model worse: it must not ask back,
restate Trace steps, write a brief file, or drop UNFOLLOWED steps.

H4 is proven by a text diff, not by a GREEN run: no Trace step line may
appear in the body of `bee-how` or `bee-why` (plan § Test matrix).

---

## GREEN Phase: Initial Skill

Pending, recorded by cell pca-10.

---

## REFACTOR Phase: Iterations

Pending, recorded by cell pca-10.

---

## Final Outcome

Pending, recorded by cell pca-10.
