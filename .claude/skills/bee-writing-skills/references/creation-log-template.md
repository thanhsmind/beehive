# Creation Log: [Skill Name]

<!-- Copy this template to your skill directory as CREATION-LOG.md -->
<!-- Complete every section — this is the evidence that TDD-for-skills was followed -->

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

**Origin:** [Where the technique/process came from — khuym, gsd-core, superpowers, claudekit, repository-harness, gstack, compound-engineering, internal, etc.]

**What the source does:** [Core behavior of the source, 1–3 sentences]

**bee context:** [Which bee stage(s) this skill implements or supports]

---

## Extraction Decisions

**What to include:**
- [Item 1] — because [addresses observed failure / enforces critical constraint / etc.]
- [Item 2] — because [reason]

**What to leave out:**
- [Item A] — [project-specific / repetitive / the model already knows this / etc.]
- [Item B] — [reason]

---

## Structure Decisions

1. [Structural decision + rationale] — e.g., "HARD-GATE before the feasibility matrix because plausibility language is the most common escape"
2. [Structural decision + rationale]

---

## Bulletproofing Elements

### Language Choices
- "[Specific phrase used]" — instead of "[softer alternative]" — because [prevents rationalization X]
- "MUST" / "NEVER" / "No exceptions" where discipline is enforced
- Implementation intention format: "When X, IMMEDIATELY do Y"

### Structural Defenses
- [Defense + what it prevents] — e.g., "Rationalization table pre-refutes the 'I'm being pragmatic' escape hatch"
- [Defense + what it prevents]

---

## RED Phase: Baseline Testing

<!-- Run these scenarios WITHOUT the skill loaded. Document results verbatim. -->
<!-- Scenario templates and the 7 pressure types: references/pressure-test-template.md -->

### Scenario 1: [Name]

**Setup:**
```
[Full scenario text — include concrete options A/B/C]
```

**Combined pressures:** [Time + Authority + Sunk Cost, etc. — minimum 3]

**Agent choice:** [Option A / B / C]

**Exact rationalization (verbatim):**
> "[Agent's exact words — quote precisely; 'agent was wrong' is useless]"

**Verdict:** FAIL / PASS

---

### Scenario 2: [Name]

(same fields)

---

### Scenario 3: [Name]

(same fields)

---

### RED Phase Summary

**Patterns identified:**
- [e.g., "Agent consistently invoked time pressure to justify skipping the gate"]

**Target rationalizations for GREEN phase:**
1. "[Exact quote 1]"
2. "[Exact quote 2]"

---

## GREEN Phase: Initial Skill

**The SKILL.md addressed:**
- [Which specific rationalization each key section targets]

**Re-ran the same scenarios WITH the skill:**

| Scenario | Result | Notes |
|---|---|---|
| Scenario 1 | PASS / FAIL | [notes if still failing] |
| Scenario 2 | PASS / FAIL | |
| Scenario 3 | PASS / FAIL | |

**Overall GREEN result:** All pass / Required iteration [N]

---

## REFACTOR Phase: Iterations

### Iteration 1 (repeat per iteration)

**New rationalization discovered:**
> "[Agent's exact words during GREEN testing]"

**Fix applied:**
- [What was added/changed: explicit negation, rationalization-table row, red-flag entry, section moved earlier]

**Re-test result:** [All scenarios re-run — PASS/FAIL each]

---

## Final Outcome

**Iterations required:** [N]

**Signs of bulletproofing observed:**
- [Agent cited specific skill sections as justification under maximum pressure]
- [Meta-test answer: "the skill was clear, I should follow it"]

**Known residual risks:**
- [Loopholes deemed acceptable + why]

**Validation run:** [`node --check` on scripts, link check, frontmatter check — commands + results]
