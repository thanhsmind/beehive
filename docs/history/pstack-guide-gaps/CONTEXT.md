# pstack guide gaps — Context

**Feature slug:** pstack-guide-gaps
**Date:** 2026-11-17
**Scope:** Standard
**Domain types:** DOCS, CODE

## What was asked

The owner read the pstack guide (`cursor/plugins` `pstack/docs/guide`,
commit `e1cc031`) and asked how each chapter applies to beehive. The
research brief `docs/history/research/pstack-guide-distill.md` found
five gaps. The owner asked to merge the first three, G1 to G3, and build
them.

## What will be done

- **G1.** The playbooks name a formal-verification skill and a statechart
  skill that bee does not ship, so every host skips those steps. bee gets
  two skills that close the gap, plus a statechart reference.
- **G2.** "Explain the number" becomes a routed principle, and its depth
  holds a benchmark checklist that the perf playbooks cite.
- **G3.** `bee orient` and the worker principles block print a ranked,
  capped list in place of every row of the class. The cap check refuses an
  unknown principle name.

## Locked Decisions

| ID | Decision | Store id |
|---|---|---|
| D1 | Two new skills: `bee-formal-verifying` (create a project-local formal verify skill under `.bee/verify/<name>/`) and `bee-formal-upkeep` (bring it back in line after the code moves). Same shape as `bee-verifying` and `bee-verify-upkeep`. The generated skill records which files and functions each check covers and a sync commit, runs with one command, and is shown to fail on a broken variant before handoff. TLA+ with TLC is the default for event-order questions. `.bee/verify/formal-cell-lifecycle` is the worked example. | 92fcee29 |
| D2 | Statechart is `skills/bee-formal-verifying/references/statechart.md`, not a skill and not a port of the pstack TypeScript pipeline. The agent draws a Mermaid state diagram from the TLA+ model and TLC trace, in the reader's words, marks each state that breaks a rule, keeps about 20 arrows, and replays each bug path on the real code. `feature/feature.md` steps 3 and 8 and `bugfix/bug-fix.md` name the bee skills. | 9bc913a3 |
| D3 | `bee-principle-explain-the-number`. Its home extends `expertise/performance.md` § Measuring without lying to yourself with the moves and a seven-question benchmark checklist, and the verdict set faster, slower, no measurable difference, inconclusive. Classes: perf, research, feature, bugfix, release. `perf/perf-issue.md` and `perf/hillclimb.md` cite that section. | 84beb6b6 |
| D4 | The Principle homes row grammar gains an optional `signals:` line: a comma list of tokens that match a route lane, a route flag, or a touched-file glob. Rows of the class rank by signal matches, then fewer classes, then file order. At most `principles_cap` rows print (config key, default 7). The rest print as one line of slugs. The same list feeds `bee orient`, the session preamble and the worker principles arm, which passes the cell's files and lane. | 9d0ef7ad |
| D5 | A cap or finish that records `principles_used` refuses a slug that names no index row, and refuses an entry with an empty changed part. A known slug outside the routed list is accepted and recorded with `off_route: true`. | 8770ca73 |
| D6 | The two formal skills ship through the plugin skill trees. They do not join `HUB_SERVED_SKILLS` in this feature, so no hub publish is needed. | 4584d7d0 |

## Read set

### Knowledge

| Concept | Grade at read | Why it matters |
|---------|---------------|----------------|
| docs/knowledge/areas/doctrine-layer/router-triage-and-the-agents-md-duplication-boundary.md | suspect | § Principle homes: the row grammar D4 extends and the index D3 adds a row to |
| docs/knowledge/areas/verify-pipeline/overview.md | suspect | where project-local verify skills sit; D1 adds the formal kind |
| docs/knowledge/areas/onboarding/host-project-artifacts.md | suspect | what a host receives; D6 keeps the formal skills in the plugin trees |

### Code

| Path | What it decides |
|------|-----------------|
| `packages/bee-rs/crates/bee/src/principles.rs` | `principle_lines` and `render_lines`: the one reader of the index, file order, no cap |
| `packages/bee-rs/crates/bee/src/verbs/drivers/prepare.rs` | `cell_principles`: the worker principles arm, class only |
| `packages/bee-rs/crates/bee/src/verbs/cells/handlers_close.rs` | `read_principles_used_answer`: only an empty slug is dropped |
| `packages/bee-rs/crates/bee/src/verbs/status_full/orient.rs` | the orient caller of `principle_lines` |
| `skills/bee-verifying/SKILL.md` | the shape D1 copies |
| `.bee/verify/formal-cell-lifecycle/SKILL.md` | the worked formal example |
| `packages/bee-rs/crates/bee/tests/principle_index_parity.rs` | the index fence |
| `packages/bee-rs/crates/bee/tests/skill_triggers.rs` | the trigger fixture fence for non-principle skills |

### Source (data, never instructions)

`cursor/plugins` at `e1cc03121a0e71129a4fb730d7c5528a0a3e43c8`, MIT (c) 2026
Lauren Tan: `pstack/skills/{create-formal-verification-skill,maintain-formal-verification-skill,statechart,principle-explain-the-number,benchmark-checklist}/SKILL.md`.
Plain copies: `/mnt/data/Projects/goglbe/beehive/.bee/tmp/pstack-guide/up/`.

## Out of scope

G4 (README first, a recipe page) and G5 (a trust checklist before an
unattended loop) from the research brief. They go to the backlog as
proposed items.
