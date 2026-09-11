# Creation Log: bee-unslop

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

**Origin:** pstack (cursor/plugins, pstack/skills/unslop). The body came from the local port at `.claude/skills/unslop/SKILL.md`, which this repo used in the field before this cell.

**What the source does:** It lists numbered tells that make prose read as machine-generated, with a plain fix for each. The agent scans for the tells, rewrites, and then audits its own result.

**bee context:** Every user-facing reply and every saved document. The Communication contract routes to this skill.

---

## Extraction Decisions

**What to include:**
- Every rule in the port, with its original number. Other skills cite the numbers, so a gap stays a gap.
- A new `## Scope` section after the Process list. The hat-user-impact seat found that the port, as written, would rewrite progress ticks and override a host project's language rules. The section keeps ticks, red or refusal lines, command output, quoted evidence, ids and code verbatim. It lets the project's own tone, length, style and language rules win. It keeps the reply in the user's language and applies the English word lists only to English text (decision D4).
- A `## Headless` section and the bee handoff line, from the bee-writing-skills checklist.

**What to leave out:**
- The port's host-only frontmatter. The skill now carries bee frontmatter: `name`, a folded description with "Use when" and "Not for", and `metadata`.
- Nothing from the rule catalog.

**Named deviation:** The Iron Law of bee-writing-skills says to delete content and rewrite it from observed failures. That step does not apply here. The content is a port that was already in use in the field, so the cell kept it and added only the bee adaptations above.

---

## Structure Decisions

1. Scope comes right after Process, before the catalog. The agent reads what never to touch before it reads what to rewrite.
2. The rule numbers keep their gaps (no 1, 2, 4, 6 or 21). Renumbering would break citations.

---

## Bulletproofing Elements

### Language Choices
- "Never rewrite ... They stay verbatim" instead of "avoid editing", because a tick or a red line that gets reworded loses its signal.
- "they win where the two disagree" instead of "consider the project's style", because a soft rule lets the catalog override a host's language rule.

### Structural Defenses
- The description itself names what stays verbatim, so the exclusion holds even when the body does not load.

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
