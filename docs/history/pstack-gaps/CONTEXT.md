# pstack Gaps — Context

**Feature slug:** pstack-gaps
**Date:** 2026-09-01
**Shaping session:** complete (agent-shaped under `gate_bypass: full`)
**Scope:** Standard
**Domain types:** ORGANIZE, RUN

## Feature Boundary

Close the three rows of `docs/history/research/pstack-distill.md`'s dependency
matrix that `pstack-adoption` deliberately left open: the `/how` trace flow, the
`/why` provenance flow, and the CONFLICT row — bee's rule ids are annotations no
human can speak. Backlog items `p-3203f84b`, `p-aa23f10e`, `p-abfd7136`.

It ends at those three surfaces. It does NOT add a route class, a lane, a CLI
verb, a skill, or a guard; it does not touch gates, worktrees, or the cell
lifecycle. The `research`-class read-only guard stays out of scope — that is
`p-69bee217`, still open by choice.

## Locked Decisions

| ID | Decision | Rationale |
|----|----------|-----------|
| D1 | The trace flow and the provenance flow are TWO named procedures in ONE new home, `skills/bee-researching/references/trace-and-provenance.md`. `bee-researching/SKILL.md` gains one reference-table row; the `research` class playbook in `bee-planning/references/planning-reference.md` gains a pointer line. Nobody transcribes the steps. | Both are research procedures and bee-researching already owns a reference table that is loaded on demand (`SKILL.md:93`). One file, two `##` sections, keeps the single-home rule that `pstack-adoption` D1 established for playbooks and costs one row, not two. |
| D2 | The trace procedure ("Trace") fans out to 2-4 read-only workers over DISJOINT entry points through the dispatch door, and the leader folds them into ONE account naming the runtime path with `path:line` anchors. Fan-out is a ceiling of 4 and a floor of 2; a trace with one entry point runs inline and says so. | pstack's `/how` splits explorers for coverage; bee already owns the mechanism (`bee dispatch prepare --kind gather`) and the disjoint-scope law (AGENTS.md, "Work in parallel"). Naming the ceiling stops the flow becoming an excuse for a swarm. |
| D3 | The provenance procedure ("Provenance sweep") names SEVEN evidence categories a why-question sweeps, and every category that returned NOTHING is reported BY NAME. An unswept category is named as unswept, never omitted. The seven: decision log, git history, `docs/history/`, the knowledge bundle, code comments and doc-comments, tests, external tracker (issues/PRs). | pstack's `/why` reports empty searches; bee already applies the same honesty rule to `bee-reviewing`'s "No findings. Scope clean" line and to `pstack-adoption` D5's dismissed bucket. Seven is the list bee's own stores actually support — it is derived from what exists here, not copied from pstack. |
| D4 | Rule ids become spoken: the EXISTING index — the `## AGENTS.md rule homes` section of `docs/knowledge/areas/doctrine-layer/router-triage-and-the-agents-md-duplication-boundary.md` — gains a one-line spoken form per rule beside its existing section pointer and `applied_at` list. No new file, no new CLI verb. | Rung 1 (Reuse). The registry already exists, is parsed by `knowledge/ownership.rs`, and is already validated by `bee knowledge check` (`unknown_rule_ref`, `duplicate_rule_home`). What is missing is a sentence per rule and a way to say its name — not a second registry. A new verb would be rung 4 work for a rung 1 gap. |
| D5 | `AGENTS.md` (source: `packages/bee/AGENTS.block.md`) states the invocation law: a user may invoke any rule id by name mid-run, and the agent's reply MUST name the decision or step the rule changed — or state plainly that it changed nothing. The law names the index path. | pstack's leverage is not the list, it is the obligation on the answer. Without the "name what changed" half, "apply never-build-on-red" is a no-op an agent can acknowledge and ignore. The "changed nothing" branch keeps the law honest instead of inviting a fabricated change. |
| D6 | One new text-reading test, `packages/bee-rs/crates/bee/tests/rule_index_parity.rs`, asserts the `<!-- rule: <id> -->` markers in `AGENTS.md` and in `packages/bee/AGENTS.block.md` are the same set, that set equals the index section's rows, and that every row carries a non-empty spoken line. | The index and the markers are a rule living in three places (`docs/knowledge/patterns/20260826-a-rule-living-in-n-places-needs-one-test-that-reads-all-n.md`). `knowledge check` covers ref→home, not marker↔row parity, and not the spoken line's existence. The test reads files as text — no crate import — the pattern `specs_fence.rs` and `route_class_parity.rs` already use. |

### Agent's Discretion

The exact wording of each procedure's steps and of each rule's spoken line is
planning's, bounded by D1-D5. Where the invocation law sits inside
`AGENTS.block.md` is planning's call, provided it is one place.

## Terms

| Term | Meaning in this feature |
|------|-------------------------|
| Trace | The named procedure that answers "how does this actually run?" with a runtime path, not a file list. |
| Provenance sweep | The named procedure that answers "why is this the way it is?" over seven evidence categories. |
| Spoken form | One line per rule id, in the plain words a user would say, so the id can be invoked out loud. |
| Invocation law | The AGENTS.md rule that a spoken rule id obliges the reply to name what it changed. |

## Specific Ideas And References

- `docs/history/research/pstack-distill.md` — the dependency matrix these three
  rows come from, and its § "The one CONFLICT worth naming" (the source of D4/D5).
- `docs/history/research/pstack-xia.md` — the deeper prior study. Where the two
  disagree, it wins.
- `docs/history/pstack-adoption/CONTEXT.md` — the four rows already shipped.
  D1 here reuses its single-home-plus-pointer shape.

## Existing Code Context

### Reusable Assets

- `skills/bee-researching/SKILL.md:93` — reference table, loaded on demand. D1's row lands here.
- `skills/bee-planning/references/planning-reference.md:216-223` — the `research` class playbook. D1's pointer lands here.
- `docs/knowledge/areas/doctrine-layer/router-triage-and-the-agents-md-duplication-boundary.md` § "AGENTS.md rule homes" — the existing rule registry, 10 rules, each already naming its AGENTS.md section. D4 extends these rows.
- `packages/bee-rs/crates/bee/src/verbs/knowledge/ownership.rs:130-146` — `parse_agents_rule_homes`, the parser that reads that section. D4 must not break its line shape.
- `packages/bee-rs/crates/bee/tests/route_class_parity.rs`, `tests/specs_fence.rs` — the text-reading test pattern D6 copies.

### Known Constraints

- `AGENTS.md` is rendered from `packages/bee/AGENTS.block.md`; both carry the markers and both must move together (`bee dev regen`).
- Every skill file has generated copies under five plugin trees and the regen chain rewrites one shared release manifest — skill-touching cells are SERIAL, never concurrent (`pstack-adoption` plan rows 14-15).
- `parse_agents_rule_homes` treats a line starting with `- ` as a new rule and an indented block as its `applied_at`. A spoken line must not be mistaken for a rule row.
