# Knowledge Skill Pair — Context

**Feature slug:** knowledge-skill-pair
**Date:** 2026-11-13
**Shaping session:** complete
**Scope:** Standard
**Domain types:** RUN | ORGANIZE

## Feature Boundary

bee gains a create skill and a periodic upkeep skill for `docs/knowledge/`, the cap answer that says a loaded concept was wrong, and two more printed writing rules; it ends before onboarding notices, config-held thresholds and any scheduler.

## Locked Decisions

These are fixed. Planning must implement them exactly — cited, never reinterpreted.

| ID | Decision | Rationale (only if it changes implementation) |
|----|----------|-----------------------------------------------|
| D1 | Knowledge takes the two-skill shape of the verify map. `bee-knowledge-setup` builds `docs/knowledge/` once per repo. `bee-knowledge-upkeep` audits it periodically. Both carry `disable-model-invocation: true` and the same frontmatter shape as `bee-verifying` and `bee-verify-upkeep`. | Owner direction 2026-11-13. |
| D2 | `bee-knowledge-setup` reads the tree and never interviews the user for content. It uses the existing verbs (`bee knowledge bootstrap` when `docs/specs/` holds specs, `bee knowledge new` otherwise, then `bee knowledge seed-anchors`), writes the orientation entry, and proves the result with `bee knowledge check --strict` and `bee knowledge index --check`. A repo that already has a bundle routes to `bee-knowledge-upkeep`. In-flow writes stay with `bee-capturing`. | No new verb: the create path already exists as verbs with no skill that owns the once-per-repo run. |
| D3 | `bee-knowledge-upkeep` has the three outcomes of `bee-verify-upkeep` (clean, changed, blocked) and this pass: bundle present; signals; pick; source wave of one read-only reader per picked concept; revise; validate; ship in one worktree and one cell, or stop. Its edit scope is `docs/knowledge/` only; a concept that describes behavior the code no longer has is doc drift or a product regression, and a regression is reported, never papered over. | Mirrors the verify upkeep pass so the two audits read alike. |
| D4 | Revision rules of an upkeep run. (a) Revise at most 5 concepts per run, picked by signal from concepts that were loaded for work; more candidates wait for the next run. (b) New text is grounded in the concept's own `bee.sources` and `bee.anchors`; the failing work is only the signal. A claim whose source cannot be read again is cut or marked as a guess. (c) Revise in place, headline (title, description, `Read when:`) first, never append a correction. (d) A noisy signal — an environment red, chance — never triggers a revision. (e) Dedupe, merge and retire belong to upkeep. | ReHarness (arXiv 2610.12668): gain is flat from k=5 to 20 and falls above 20; grounded revision keeps 95-98% of claims sourced; noisy reward gives almost no gain; the paper has no delete or merge step. |
| D5 | `bee cells cap` takes `--knowledge-wrong "<docs/knowledge path> — <what was wrong>"`, recorded on `trace.knowledge_wrong` as `{path, wrong}`, and `--report` accepts a `knowledge_wrong` array. It is never refused. `bee knowledge report` lists concepts by wrong count, in text and JSON. | knowledge-in-the-flow D1 forbids a mandatory knowledge check, so this is an answer, never a door. It follows the shape of `--knowledge-used` (decision 721d06f5). |
| D6 | The printed writing rules gain two: state the condition under which a claim holds, and mark a guess as a guess; record how to find a volatile detail, never the detail itself. Their one home stays `KNOWLEDGE_RULES`. Every place that says "five rules" is swept to the new count or to no count. | Extends decision 71241c48; knowledge-useful-rules D6 keeps writing rules in one home. |
| D7 | Timing. Read at shaping, planning and cell brief. Write at cap. Revise at cap when the cell proves a loaded concept wrong; when the cell cannot fix it, report it with `--knowledge-wrong`. Audit in an upkeep run. An upkeep run is due after a release, or when `bee knowledge report` shows a concept reported wrong, an orphan grade, a suspect concept loaded 5 or more times, or a concept loaded 5 or more times and never used. Signal rank for the pick: wrong count, orphan, suspect-and-loaded, loaded-never-used. The numbers are starting defaults and live in the upkeep skill text. | The 5-load threshold is the one `bee knowledge report` already uses. |

| D8 | Plan-check rulings, refining D5 and D7. (a) `bee knowledge report` JSON lists every loaded concept with its load count; orphan and suspect grades are read from `bee knowledge check --json`, never from the report. (b) Loaded-never-used is a weak signal while silent caps outnumber answered ones: it never makes a run due alone, and upkeep trims or retires on it only after the source reader confirms dead weight. (c) The worker-facing line asks for a loaded concept that misled the cell and that the cell did not fix in its commit. (d) The report text prints the newest non-empty note under each reported-wrong concept. (e) The setup skill writes the orientation file at `docs/knowledge/areas/project/overview.md`. | The report printed no grades and loads only for never-used concepts; 9 answered caps against 1359 silent; two worker lines disagreed; a count with no note cannot be triaged; no fixed orientation path. |

### Agent's Discretion

Skill body wording, section order inside each skill, and the trigger-fixture briefs. The skills are written through `bee-writing-skills`.

## Terms

| Term | Meaning in this feature |
|------|-------------------------|
| signal | A recorded fact that a concept may be wrong or dead weight: a `knowledge_wrong` entry, an orphan or suspect grade, or a load count with no use. |
| revise | Rewrite the concept's own text in place. Never an appended note. |
| loaded | The concept's path appeared in a pack or cell brief, as counted by `bee knowledge report`. |

## Read set

### Knowledge

| Concept | Grade at read | Why it matters |
|---------|---------------|----------------|
| docs/knowledge/areas/okf-profile/critical-bar.md | confirmed | Holds the knowledge-used cap answer and its report counts, which D5 extends. |
| docs/knowledge/areas/okf-profile/overview.md | suspect | Entry points of the bundle; names the freshness concept the skills cite. |
| docs/knowledge/areas/verify-pipeline/verify-pipeline-skill-trigger-cases.md | suspect | New skills need trigger rows in the fixture it describes. |
| docs/knowledge/areas/verify-pipeline/overview.md | suspect | Names the verify skill pair that D1 mirrors. |
| docs/knowledge/patterns/20260723-an-append-only-learning-artifact-transmits-obsolete-advice.md | — | The defect D4(c) exists to stop. |
| docs/knowledge/patterns/20260821-instruction-text-is-an-untested-code-path.md | — | A skill is instruction text; it needs its trigger test. |

### Code

| Path | What it decides |
|------|-----------------|
| `skills/bee-verifying/SKILL.md` | The create-skill shape D1 and D2 mirror. |
| `skills/bee-verify-upkeep/SKILL.md` | The upkeep pass D3 mirrors. |
| `packages/bee-rs/crates/bee/src/verbs/knowledge/consolidate.rs` | `KNOWLEDGE_RULES`, the one home of the writing rules (D6). |
| `packages/bee-rs/crates/bee/src/verbs/cells/finish_support.rs` | Where `knowledge_used` is parsed and written to the trace (D5). |
| `packages/bee-rs/crates/bee/src/verbs/knowledge/report.rs` | `build_knowledge_used`, the report section D5 extends. |
| `packages/bee-rs/crates/bee/tests/fixtures/skill-triggers.json` | Trigger cases every skill description is tested against. |
| `skills/bee-capturing/SKILL.md` | Owns the in-flow write and the cap-time re-confirm; gains the D7 revise-or-report line. |
| `.bee/expertise/knowledge.md` | The craft guide both skills point at. |

## Specific Ideas And References

- ReHarness, arXiv 2610.12668v1: forward read, backward revision of the retrieved reflections, forward write. The source of D4 and D6.
- The owner's model: "verify is created by one skill and updated by another; knowledge can have that form".

## Existing Code Context

### Reusable Assets

- `bee knowledge report` — already prints use counts, loaded-never-used at 5 loads, pull rate and outcome arms. On 2026-11-13 it showed 9 cells answered and 1359 silent, so the use signal is thin today.
- `bee knowledge check`, `confirm`, `seed-anchors`, `bootstrap`, `new`, `index` — every mechanical step the two skills call.

### Established Patterns

- A skill pair with opposite "Not for" lines in the two descriptions, tested by near-miss rows in `skill-triggers.json`.

### Integration Points

- `skills/` is the source; runtime skill homes are rendered output (`bee dev render-skill-trees`, `bee dev regen`).

## Canonical References

- `docs/history/knowledge-in-the-flow/CONTEXT.md` — D1 no mandatory knowledge gate.
- `docs/history/knowledge-useful-rules/CONTEXT.md` — D5 knowledge-used answer, D6 one home for rules.
- `docs/history/knowledge-freshness-mechanic/CONTEXT.md` — D3 the grades.

## Outstanding Questions

None open. The two technical questions shaping left for planning are answered in the Discovery section of `plan.md`: cap and finish share one flag struct and one parse, and the regen chain rewrites the rendered skill trees, the onboarding ledger and the release manifest.

## Out Of Scope

- An onboarding notice that offers the two skills, as verify has.
- Thresholds in `.bee/config.json`.
- A report line that says "upkeep due"; the skill reads the report itself.

## Handoff Note

CONTEXT.md is the source of truth. Decision IDs are stable. Planning reads the locked
decisions, the read set, the code context and the canonical references.
