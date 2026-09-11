# Creation Log: bee-technical-writing

## Table of Contents
1. [Source Material](#source-material)
2. [Extraction Decisions](#extraction-decisions)
3. [Structure Decisions](#structure-decisions)
4. [Bulletproofing Elements](#bulletproofing-elements)
5. [RED Phase: Baseline Testing](#red-phase-baseline-testing)
6. [GREEN Phase: Initial Skill](#green-phase-initial-skill)
7. [REFACTOR Phase: Iterations](#refactor-phase-iterations)
8. [Final Outcome](#final-outcome)
9. [Wiring into the flow (writing-flow-wiring)](#wiring-into-the-flow-writing-flow-wiring)

---

## Source Material

**Origin:** pstack (cursor/plugins, pstack/skills/technical-writing). The body came from the local port at `.claude/skills/technical-writing/SKILL.md`, which this repo used in the field before this cell.

**What the source does:** It sets four layers for a technical document: the Diátaxis mode, Google developer style sentences, STE instruction rules and Global English syntax. A worked example and a review checklist close it.

**bee context:** Every saved document, plan, spec, README, PR description and commit message. The Communication contract routes to this skill, and bee-unslop covers chat replies.

---

## Extraction Decisions

**What to include:**
- All four layers, the three top rules, the worked example and the review checklist, unchanged.
- Every mention of the unslop skill now names `bee-unslop`.
- The port's STE note said that this repo's CLAUDE.md asks for ASD-STE100 Simplified Technical English. That note was true only in the bee source repo. The cell deleted it. In its place, the project's own writing standard, code style or language wins where the two disagree, including over the tabs rule. The hat-user-impact seat found that the port would override a host project's language rules and that unslop, as ported, would rewrite progress ticks. This line and bee-unslop's Scope section answer both findings (decision D4).
- One Voice line: never rewrite quoted evidence, ids, commands or code.
- A `## Headless` section and the bee handoff line, from the bee-writing-skills checklist.

**What to leave out:**
- The port's host-only frontmatter. The skill now carries bee frontmatter: `name`, a folded description with "Use when" and "Not for", and `metadata`.
- Any sentence true only in the bee source repo.

**Named deviation:** The Iron Law of bee-writing-skills says to delete content and rewrite it from observed failures. That step does not apply here. The content is a port that was already in use in the field, so the cell kept it and added only the bee adaptations above.

---

## Structure Decisions

1. The project-standard-wins line sits at the top of the STE layer, in the place of the deleted note, because the STE layer is where a host's own standard most often differs.
2. The layer order and the headings stay as the port had them, so any citation of a heading still resolves.

---

## Bulletproofing Elements

### Language Choices
- "that standard wins where the two disagree" instead of "take the project's style into account", because a soft rule lets the default tabs rule override a host's code style.

### Structural Defenses
- The description names chat replies as bee-unslop's job, so the two skills do not both claim one reply.

---

## RED Phase: Baseline Testing

### Scenario 1: PR body over a sloppy teammate draft

**Setup:** The agent was asked for the PR body it would post, with a sloppy teammate draft as the starting point.

**Combined pressures:** Exhaustion, Authority, Time.

**Agent output (verbatim, first lines):**
> "Trim a trailing newline from `--report` JSON before `bee cells finish` parses it"
> "## Problem"

**Verdict:** PASS on behavior.

**Caveat:** The subagent inherited the bee source repo's CLAUDE.md, which already told it to write through the local unslop port. So this RED run is contaminated. It does not prove that the skill is unneeded.

---

## GREEN Phase: Initial Skill

The leader reran scenario 1 through `bee dispatch prepare --kind advisor --brief-file`, on the `plan` role (Opus). The run loaded only `bee-unslop` and `bee-technical-writing`, through their read diet.

**Agent output (verbatim, opening):**
> "`bee cells finish` now trims a trailing newline from the `--report` JSON before it parses the report. Before this change, a report that ended in a newline failed to parse, so the cell could not finish."

- It uses the real symbols `parse_report` and `crates/bee/src/verbs/cells/handlers_close.rs`.
- It indents the command with a tab, as the technical-writing rule asks.
- It scopes the evidence honestly: "I ran only this test on my machine. CI runs the full suite on this PR."
- It has no AI vocabulary, no em dash, no inline-header list and no filler.

**Verdict:** PASS.

**Result:** GREEN holds. The RED side of this pair is contaminated, so the evidence that the skill is needed is weak. The evidence that it does no harm is good.

---

## REFACTOR Phase: Iterations

None. REFACTOR found no new rationalization in GREEN.

---

## Final Outcome

- Iterations required: 0.
- GREEN S1: PASS on Opus with both writing skills loaded.
- Known residual risks:
  - The S1 RED run is contaminated, because the subagent inherited this repo's CLAUDE.md.
  - The GREEN run is a single sample.
  - bee's own doctrine still uses em dashes, and `bee-unslop` bans them in new prose.

---

## Wiring into the flow (writing-flow-wiring)

The question: which bee write points need a line that sends the writer to this skill? Seven RED runs answered it. Each run went through `bee dispatch prepare --kind advisor --role plan` (Opus), read only the current worktree skill files and applied no change.

### RED

| Scenario | Pressure | Result | What shaped it |
|---|---|---|---|
| R1 cell action for a cold developer worker | time, authority | PASS | planning-reference.md cell quality rules, MANDATE 2, test scoping |
| R2 area spec Behavior section | completeness | PASS | area-spec.md present-tense and no-history rules |
| R3 plan.md Summary | authority, social | PASS | planning-reference.md Summary rule |
| R4 handbook page, docs lane (`docs/handbook/bee-why.md`) | completeness, authority | **FAIL**: one page mixed explanation, reference tables, a tutorial walkthrough and opinion, and saved a `<!-- TODO -->` placeholder | bee-hive/SKILL.md docs-only row only; no writing rule named |
| R5 research brief | persuasion, scope | PASS | bee-researching evidence ladder and template |
| R6 CONTEXT.md decisions | narrative | PASS | bee-shaping Lock and Hard rules; context-template.md |
| R7 plain turn, README section, no skill loaded | scope | **FAIL (mild)**: one section held explanation, a numbered tutorial and a reference table; nothing split or linked them | AGENTS.md only; no writing rule named |

Verbatim WHY of R4:
> "The docs-only row of .../skills/bee-hive/SKILL.md:54 set the process (no plan and no cells: announce, write, format-check, then close with a capture line)."

WHY of R7, as the RED record states it (the run kept no longer quote):
> "AGENTS.md only: `--relation` rule, `--help` pointer, ASD-STE100 section; no writing rule named"

### Finding

Every templated document (cell action, area spec, plan, research brief, CONTEXT.md) held one mode through its own template rules, with no pointer to this skill. The two failures are the documents with no template: a docs-lane page and a README section written in a plain turn. Decision f951a31f wires only those two points: the docs-only row of `skills/bee-hive/SKILL.md` and one sentence in the § Communication of the always-loaded AGENTS block. Both point to this skill; neither copies its text.

### GREEN

Pending, recorded by cell wfw-2.
