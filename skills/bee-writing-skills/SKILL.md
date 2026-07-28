---
name: bee-writing-skills
description: >-
  Build and pressure-test bee skills with the TDD-for-skills discipline. Use when creating a new bee skill, editing an existing one, or verifying a skill holds up under pressure. Do NOT use for project-specific AGENTS.md conventions or one-off instructions.
metadata:
  version: '0.1'
  ecosystem: bee
  dependencies: []
---

# Writing Bee Skills (comb building)

Skills are code. They have bugs. Test them before deploying.

This is the TDD-for-skills methodology from Superpowers via khuym.

**THE IRON LAW: NO SKILL WITHOUT A FAILING TEST FIRST.**
Wrote the skill before testing? Delete it. Start over. No exceptions — not for "simple additions," not for "just a section," not for "reference only." The Iron Law applies to edits.

## The Core Cycle: RED → GREEN → REFACTOR

| TDD Concept | Skill Equivalent |
|---|---|
| Test case | Pressure scenario with subagent |
| Production code | SKILL.md |
| Test fails (RED) | Agent violates rule without skill |
| Test passes (GREEN) | Agent complies with skill present |
| Refactor | Close loopholes, maintain compliance |

## PHASE 1 — RED: Write the Failing Test

**HARD-GATE: Do not write any skill content until this phase is complete.** Skipping baseline testing predictably ships preventable failures.

1. Define the skill's purpose: what behavior must it enforce, and what fails without it?
2. Create 3–5 pressure scenarios combining ≥3 pressures (see `references/pressure-test-template.md`).
3. Run the scenarios WITHOUT the skill — give agents the realistic task under pressure.
4. Document exact rationalizations verbatim. "Agent was wrong" is useless. "Agent said 'I already manually tested it, so the spirit of TDD is satisfied'" is target material.
5. Identify patterns: which excuses repeat?

Record per scenario: name, combined pressures, exact violation, exact rationalization (verbatim quote).

## PHASE 2 — GREEN: Write the Minimal Skill

Write SKILL.md addressing the **specific rationalizations documented in RED only.** Hypothetical content bloats the skill and gets skipped.

**SKILL.md checklist (bee conventions):**
- [ ] YAML frontmatter starts on line 1 (`---`)
- [ ] `name`: hyphen-case with the `bee-` prefix, matches the directory name exactly (self-prefixed so plain copies stay namespaced)
- [ ] `description`: one short purpose clause (shown in the /slash menu), then "Use when..." triggering conditions — **NEVER a workflow/step summary**; third person, ≤1024 chars
- [ ] `metadata.version: '0.1'`, `metadata.ecosystem: bee`, `metadata.dependencies` mapping or `[]`
- [ ] Body < 200 lines preferred; overflow goes to exactly one level of `references/`
- [ ] Regrowth law: a new learning lands in the knowledge bundle or `references/` by default; edit the body itself only for a load-bearing invariant, and only if the edit still fits the recorded budget in `scripts/skill-body-budget.json` — over budget, trim the body first (one in, one out) before adding
- [ ] Per-turn rules (chat shape, communication) are never exiled to references — they live in the always-loaded layer; a reference nothing forces open is a rule nothing follows.
- [ ] Commands quoted in the body match the `.bee/bin` CLI surface in `bee/docs/07-contracts.md` verbatim
- [ ] Short `Headless` section documenting `mode:headless` behavior
- [ ] Red Flags list; persuasion principles applied (table below); HARD-GATE markers on critical stops
- [ ] Ends with the handoff sentence: `[Outcome]. Invoke bee-<next-skill> skill.`
- [ ] Cross-references other skills by name (`Invoke bee-planning`), never inlines their content

**Description trap:** a workflow summary in the description makes Claude follow the description and skip the skill body. Every time.

```yaml
# ❌ BAD
description: Use when creating skills — run baseline test, write minimal skill, run tests

# ✅ GOOD
description: Use when creating a new bee skill or editing an existing one
```

**Dependency metadata style:** write `metadata.dependencies` as a mapping keyed by dependency id — never a YAML array of objects (generic evaluators reject that shape). bee skills are usually dependency-free (Node 18+, vendored `.bee/bin/` helpers):

```yaml
metadata:
  dependencies:
    nodejs-runtime:
      kind: command
      command: node
      missing_effect: degraded
      reason: Reads bee records via the vendored .bee/bin helpers.
```

**Apply persuasion principles:**

| Principle | Implementation | Use For |
|---|---|---|
| **Authority** | "YOU MUST", "Never", "No exceptions" | Discipline-enforcing rules |
| **Commitment** | Ordered checklists, announce skill usage | Multi-step processes |
| **Scarcity** | "Before proceeding", "IMMEDIATELY after X" | Verification requirements |
| **Social Proof** | "Teams report...", "X without Y = failure. Every time." | Common failure patterns |
| **Unity** | "our skills", collaborative framing | Techniques, guidance |

After writing: re-run the same pressure scenarios WITH the skill. The agent must now comply. Still failing → the skill is unclear or incomplete. Revise and re-test. Do not proceed.

## PHASE 3 — REFACTOR: Close Loopholes

An agent violating a rule despite having the skill is a test regression — the skill has a bug:

1. Capture the new rationalization verbatim.
2. Add an explicit negation to the rule.
3. Add an entry to the skill's rationalization table.
4. Add an entry to the red flags list.
5. Re-run ALL scenarios — verify all still pass.

**Meta-testing technique:** after an agent chooses wrong, ask: "You read the skill and chose Option C anyway. How could the skill have been written differently to make Option A the only acceptable answer?" Three diagnoses:

- "The skill WAS clear, I chose to ignore it" → add "Violating the letter of the rules is violating the spirit of the rules."
- "The skill should have said X" → add their exact suggestion verbatim
- "I didn't see section Y" → make the key point more prominent, move it earlier

## PHASE 4 — VALIDATE & DOCUMENT

No automated skill validator exists (v0.1); validate by hand plus `node --check`:

```bash
node --check <skill-dir>/scripts/<each-script>.mjs   # only if the skill ships scripts
```

Manual checks (every item, every time): re-run the checklist above end to end; run any repo-local test script and quote the output.

**Create CREATION-LOG.md** (see `references/creation-log-template.md`): source material/extraction decisions, scenarios run/results, rationalizations found/fixed, iterations required.

**Bulletproof looks like:** the agent chooses the correct option under maximum pressure, cites skill sections, acknowledges the temptation, and the meta-test returns "the skill was clear." **Not bulletproof:** new rationalizations, the agent argues the skill is wrong, or "hybrid approaches" that satisfy the letter but not the spirit.

## Rationalization Table (Common Violations)

| Excuse | Reality |
|---|---|
| "I know this technique, testing is unnecessary" | You're testing the SKILL, not your knowledge. Agents differ from you. |
| "It's so simple it can't have bugs" | Every untested skill has issues. The test takes 30 minutes. |
| "Academic questions passed — that's sufficient" | Reading a skill ≠ using a skill under pressure. Test application scenarios. |
| "My description summarizes the workflow so agents know what to do" | Workflow-summary descriptions make agents skip the skill body. Remove it. |
| "This edit is minor — testing isn't needed" | The Iron Law applies to edits. No exceptions. |
| "I'll test it after a few real uses" | Problems = agents misusing it in production. Test BEFORE deploying. |

## Headless

`mode:headless`: the Iron Law still binds — no skill content is written or deployed without a completed RED phase and a GREEN verification. Ambiguous design choices (scope, naming, which scenarios to run) are deferred to an `Outstanding Questions` section of the terminal report, never guessed.

## Red Flags — STOP and Run Baseline Tests

- writing skill content before creating any pressure scenarios
- "I already know what agents will do"
- "It's just a small addition"
- "Academic questions passed, that's sufficient testing"
- description contains workflow steps or a process summary
- skill addresses hypothetical scenarios not observed in baseline
- deploying without re-running scenarios WITH the skill (no green verification)
- "the skill was good last month, edits don't need testing"

All of these mean: stop, run baseline tests first. Violating the letter of the rules is violating the spirit of the rules.

## Handoff

Skill pressure-tested, validated, and logged. Invoke bee-hive skill.

| Reference | When to Load |
|---|---|
| `references/pressure-test-template.md` | the 7 pressure types, ready-to-use scenario templates, the meta-test |
| `references/creation-log-template.md` | CREATION-LOG.md template documenting the TDD process |
