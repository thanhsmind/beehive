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

Skills are code. They have bugs. Test them before deploying —
TDD-for-skills, from Superpowers via khuym. Rules stated bare — decision
IDs: `references/provenance.md`; quoted headings resolve in the named
reference.

**THE IRON LAW: NO SKILL WITHOUT A FAILING TEST FIRST.** Applies to edits
too — no exceptions, not for "simple additions," "just a section," or
"reference only."

## The Core Cycle

| Phase | Action | Gate |
|---|---|---|
| RED | 3-5 pressure scenarios, ≥3 combined pressures, run WITHOUT the skill; record exact verbatim rationalizations, not paraphrases | HARD-GATE: no skill content written before this completes |
| GREEN | Minimal SKILL.md addressing only the RED rationalizations (hypothetical content bloats and gets skipped); re-run the same scenarios WITH the skill | Still failing → the skill is unclear; revise, do not proceed |
| REFACTOR | Any new rationalization → explicit negation + a rationalization-table row + a red-flag entry; re-run ALL scenarios | An agent violating a rule with the skill present is a skill bug, not agent error |
| VALIDATE | Manual checklist below, end to end; `node --check` on any shipped scripts; write CREATION-LOG.md | No automated skill validator exists (v0.1) |

Scenario templates, the 7 pressure types, and the meta-test:
`references/pressure-test-template.md` ("The 7 Pressure Types").
CREATION-LOG.md template: `references/creation-log-template.md`.

## SKILL.md checklist (bee conventions)

- [ ] YAML frontmatter starts on line 1 (`---`); `name` hyphen-case with the `bee-` prefix, matches the directory exactly
- [ ] `description`: one purpose clause, then "Use when..." triggers — NEVER a workflow/step summary (agents follow the description and skip the body); third person, ≤1024 chars
- [ ] `metadata.version: '0.1'`, `metadata.ecosystem: bee`, `metadata.dependencies` mapping or `[]` (never a YAML array of objects — `references/checklist-examples.md` ("Dependency metadata style"))
- [ ] Information density, not length: overflow to exactly one level of `references/`; a body line earns its place only by changing agent behavior — a line that doesn't belongs in `references/`
- [ ] Prose follows the instruction-spec standard — imperative rules, trigger framing, constraint first, one word one meaning, token economy: `references/prompt-style.md` ("The seven laws"); edits to existing skills also honor its frozen-heading and protocol-vocabulary guardrails
- [ ] **Regrowth law:** a new learning lands in the knowledge bundle or `references/` by default; edit the body itself only for a load-bearing invariant
- [ ] **Per-turn rules (chat shape, communication) are never exiled to references** — they live in the always-loaded layer; a reference nothing forces open is a rule nothing follows
- [ ] Commands quoted in the body match the `.bee/bin` CLI surface in `bee/docs/07-contracts.md` verbatim
- [ ] Short `Headless` section; Red Flags list; persuasion principles applied (`references/checklist-examples.md` ("Persuasion principles")); HARD-GATE markers on critical stops
- [ ] Ends with the handoff sentence: `[Outcome]. Invoke bee-<next-skill> skill.`
- [ ] Cross-references other skills by name (`Invoke bee-planning`), never inlines their content

Description-trap example: `references/checklist-examples.md` ("Description trap").

## Headless

`mode:headless`: the Iron Law still binds — no skill content is written or
deployed without a completed RED phase and a GREEN verification. Ambiguous
design choices (scope, naming, which scenarios to run) are deferred to an
`Outstanding Questions` section of the terminal report, never guessed.

## Red Flags — STOP and Run Baseline Tests

writing skill content before creating any pressure scenarios · "I already
know what agents will do" · "it's just a small addition" · "academic
questions passed, that's sufficient testing" · description contains
workflow steps or a process summary · skill addresses hypothetical
scenarios not observed in baseline · deploying without re-running scenarios
WITH the skill (no green verification) · "the skill was good last month,
edits don't need testing"

All of these mean: stop, run baseline tests first. Violating the letter of
the rules is violating the spirit of the rules. Rationalizations already
seen and their reality: `references/rationalization-table.md`.

## Handoff

Skill pressure-tested, validated, and logged. Invoke bee-hive skill.

| Reference | When to Load |
|---|---|
| `references/prompt-style.md` | the seven prompt-style laws and the edit guardrails, load before writing any skill prose |
| `references/pressure-test-template.md` | the 7 pressure types, ready-to-use scenario templates, the meta-test |
| `references/creation-log-template.md` | CREATION-LOG.md template documenting the TDD process |
| `references/checklist-examples.md` | description trap, dependency-metadata YAML, persuasion-principle table |
| `references/rationalization-table.md` | common violation excuses and their reality, extended during REFACTOR |
| `references/provenance.md` | decision IDs + rationale behind each body rule |
