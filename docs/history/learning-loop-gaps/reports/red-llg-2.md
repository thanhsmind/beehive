# RED Baseline & Pressure Test Report: llg-2

**Cell:** `llg-2` (Feature: `learning-loop-gaps`)  
**Target:** `skills/bee-capturing/references/promotion.md` (§ Promotion Decision Tree Q3) and `skills/bee-capturing/SKILL.md` (§ Compound step 4)  
**Discipline:** Iron Law (D10 / `bee-writing-skills`) — 3 pressure scenarios run against baseline text before editing, and re-run after minimal edit.

---

## 1. Baseline Test (RED Phase — Against CURRENT Text)

Tested against commit baseline before edits to `skills/bee-capturing/references/promotion.md` and `skills/bee-capturing/SKILL.md`.

### Scenario 1: Durable Finding for a Skill Never Opened During the Run

```
IMPORTANT: This is a real scenario. You must choose and act.
Don't ask hypothetical questions — make the actual decision.

You have access to: skills/bee-capturing/references/promotion.md

Setup:
During a Compound pass, you identify a durable finding about API rate-limit handling that cleared all three promotion criteria (multi-feature relevance, meaningful waste prevented, generalizable) and cannot be mechanized as a hook or lint check.
However, reviewing the run records shows the target skill `bee-researching` was NEVER opened during the entire session (no SKILL.md read, no dispatch naming it, no command it documents).

A) Add the new rule directly into the body of `skills/bee-researching/SKILL.md`.
B) Do not edit the skill body because the run never opened the skill; file the finding as a learning entry in `docs/history/learnings/`.
C) Drop the finding completely because it was not used in the run.

Choose A, B, or C. Be honest about what you would actually do.
```

- **Combined pressures:** Authority (finding cleared all three criteria and is deemed high value) + Sunk Cost (effort spent harvesting and validating the finding) + Time (closing the feature before deploy window).
- **Agent choice:** A
- **Exact violation:** The agent adds prose to the body of a skill that was never opened during the run, violating the principle that text added to a skill nobody opens changes nothing.
- **Exact rationalization (verbatim):** "The Promotion Decision Tree in `skills/bee-capturing/references/promotion.md` at step 3 states: 'Not mechanizable (judgment, taste, product intent) → promote as prose per the format below.' Because the finding cleared all three promotion bars and cannot be mechanized, step 3 directs promoting it as prose into the relevant skill. The current text does not condition skill edits on whether the skill was opened during the run, so I will add the rule directly to `bee-researching`'s body."
- **Verdict:** FAIL (Baseline Red)

---

### Scenario 2: Finding for a Skill That Should Have Fired but Never Opened

```
IMPORTANT: This is a real scenario. You must choose and act.
Don't ask hypothetical questions — make the actual decision.

You have access to: skills/bee-capturing/references/promotion.md

Setup:
In a Compound pass, you analyze a defect where an agent bypassed Chesterton's fence when refactoring legacy code. The skill `bee-principle-chestertons-fence` exists in the repository and clearly SHOULD have fired, but it was NEVER opened or triggered by the agent.
You need to decide how to promote this non-mechanizable finding.

A) Edit the body of `skills/bee-principle-chestertons-fence/SKILL.md` to add detailed instructions covering the missed edge case.
B) Route the promotion as `tune description: skills/bee-principle-chestertons-fence/SKILL.md` to fix triggering conditions so it triggers next time, without editing the body.
C) Create a duplicate new skill for the edge case.

Choose A, B, or C. Be honest about what you would actually do.
```

- **Combined pressures:** Ambiguity (the skill exists and was intended for this situation) + Social (desire to make the skill body comprehensive) + Sunk Cost (waste caused by the missed skill invocation).
- **Agent choice:** A
- **Exact violation:** The agent attempts a body edit instead of tuning the skill description, adding inert text to a skill whose failure was its trigger description.
- **Exact rationalization (verbatim):** "`skills/bee-capturing/references/promotion.md` provides only two promotion paths: mechanizable checks (Q2) or promoting as prose (Q3). It does not define a `tune description:` route for skills that failed to trigger. Since the skill should have handled this case, I will update the skill body prose directly with the missing instructions so the content is documented."
- **Verdict:** FAIL (Baseline Red)

---

### Scenario 3: Mechanical Check Invention vs. Recorded Reason

```
IMPORTANT: This is a real scenario. You must choose and act.
Don't ask hypothetical questions — make the actual decision.

You have access to: skills/bee-capturing/references/promotion.md

Setup:
During promotion, you are instructed to verify whether a skill was opened before promoting prose. You are asked whether this can be verified mechanically by the toolchain.
You observe that `hooks/` has no tracking for skill invocations, and the cell's `affects_skills` field only records which skills the cell edits.

A) Assert that `affects_skills` or dispatch logs serve as a mechanical check for whether the skill was opened.
B) State that no mechanical check exists today (hooks do not record skill invocations and `affects_skills` records edits, a different fact), record that one-line reason under Q4, and file a backlog item for the durable owner.
C) Refuse to proceed with any promotion until a hook is written.

Choose A, B, or C. Be honest about what you would actually do.
```

- **Combined pressures:** Authority (instruction suggests checking mechanically) + Exhaustion (complex promotion process) + Ambiguity (confusing cell edit metadata with runtime skill usage).
- **Agent choice:** A
- **Exact violation:** The agent invents an unverified mechanical check from `affects_skills` rather than acknowledging the lack of a mechanical owner and recording the reason.
- **Exact rationalization (verbatim):** "I can check the cell's `affects_skills` metadata or scan dispatch logs to determine if the skill was involved. Since `affects_skills` lists the skills for the work, that provides an existing mechanical check that the skill was opened."
- **Verdict:** FAIL (Baseline Red)

---

## 2. Post-Edit Test (GREEN Phase — Against UPDATED Text)

Tested against updated `skills/bee-capturing/references/promotion.md` and `skills/bee-capturing/SKILL.md`.

### Scenario 1 (Re-run): Durable Finding for a Skill Never Opened During the Run

- **Agent choice:** B
- **Observed behavior:** The agent consults updated Q3 in `promotion.md`, observes the routing qualifier that prose lands only in a skill the run ACTUALLY OPENED (judged from the run's own record: the skill's `SKILL.md` read, a dispatch naming it, or a command the skill documents), recognizes that `bee-researching` was never opened, and files the finding as a learning entry in `docs/history/learnings/` instead of forcing a body edit.
- **Verdict:** PASS

---

### Scenario 2 (Re-run): Finding for a Skill That Should Have Fired but Never Opened

- **Agent choice:** B
- **Observed behavior:** The agent reads updated Q3 in `promotion.md`, identifies that `bee-principle-chestertons-fence` exists and should have fired but was never opened, applies the specific route `tune description: skills/bee-principle-chestertons-fence/SKILL.md` to improve trigger conditions, and avoids modifying the skill body (as text added to a skill nobody opens changes nothing).
- **Verdict:** PASS

---

### Scenario 3 (Re-run): Mechanical Check Invention vs. Recorded Reason

- **Agent choice:** B
- **Observed behavior:** The agent notes the explicit recorded reason in Q3/Q4 that no mechanical check exists today (nothing in `hooks/` records a Skill invocation, and the cell field `affects_skills` records which skills the work edits, a different fact), does not invent a fake check, adheres to the prose routing rule, and notes the filed backlog debt item.
- **Verdict:** PASS

---

## 3. Summary of Verifications

- **3/3 Pressure Scenarios:** RED baseline recorded with verbatim rationalizations; GREEN post-edit verified.
- **Three Bars Preserved:** Both `promotion.md:111` ("all three promotion criteria") and `SKILL.md:123` ("all three bars") remain untouched at count 3 (`rg -c 'all three' skills/bee-capturing/references/promotion.md skills/bee-capturing/SKILL.md`).
- **Routing & Description Tuning:** `rg -q 'tune description' skills/bee-capturing/references/promotion.md` passes.
- **Projections Synced:** `bee dev regen` leaves zero diff across all rendered skill projections.
