# Knowledge docs in the flow — Context

**Feature slug:** knowledge-docs-in-the-flow
**Date:** 2026-11-08
**Shaping session:** complete (consumes docs/discovery/knowledge-from-code/MAP.md D9, D10, D11, D12)
**Scope:** Standard
**Domain types:** READ | RUN | ORGANIZE

## Feature Boundary

Make documentation a working part of the bee flow: a feature's CONTEXT.md
records the knowledge concepts and code entry points it was read against,
shape approval refuses a CONTEXT.md without that section, every knowledge
document template asks for information instead of shape, a business rule
bullet can carry its own status and verification pointer, and
`bee knowledge check` refuses an area concept with no decision link. It
ends before any content change to bee's own area concepts beyond the five
that carry no decision link today (the content cut is PBI p-3a756890).

## Locked Decisions

| ID | Decision | Rationale (only if it changes implementation) |
|----|----------|-----------------------------------------------|
| D1 | Documentation is a working part of the flow: shaping and planning record a pulled read set, cap and close update what changed, every document type has a named template that says what to add and where, substance over volume. (map D9, 5c5ae146) | Binds the mechanics below to the shape and plan steps; cap and close already hold their doors (knowledge-freshness-mechanic D4). |
| D2 | The read set lives in `docs/history/<feature>/CONTEXT.md` as a `## Read set` section with two tables: knowledge (concept path, freshness grade at read time, one line on why it matters) and code (path, what it decides). Shaping fills it from `bee knowledge context` and `bee knowledge search`; planning extends it and each cell's `read_first` cites it. Shape approval checks the section exists and every knowledge path resolves. Nothing is pushed into worker briefs. (map D11, f3c5ad0f) | The parser sits beside `parse_locked_decision_ids`; the refusal sits in the shape and merged gate approval beside the preview refusal. |
| D3 | Every knowledge document type is written into a named template whose sections each open with the one question the section answers for a rebuilder and name the sources that may feed it, plus a deletion line. A rule bullet carries status (active, temporary, retired) and a verification pointer only where it can differ from its concept; the decision link stays on the concept. The cut area template holds map, boundaries, invariants, entry points, pointers. (map D12, 2bc56fb6) | Templates are the skills' reference files; the cut area template is authored here and used by the later content cut (PBI p-3a756890, trigger `pbi-p-3a756890-the-content-cut-of-bee-s-o__e2f6e8c3`). |
| D4 | `bee knowledge check` refuses an area concept that carries no decision link (empty or absent `bee.decisions`) as a profile error with a remedy naming `bee decisions log` and the concept's `decisions` list. (map D10, e2f6e8c3, the check half) | The five concepts that fail today are fixed in this feature; the cut half of D10 is PBI p-3a756890 (trigger `pbi-p-3a756890-the-content-cut-of-bee-s-o__e2f6e8c3`). |
| D5 | Knowledge is pulled on demand; nothing new is pushed into briefs. (knowledge-freshness-mechanic D1, 2b11f025) | `brief.rs` and the dispatch prompt renderer are not touched. |
| D6 | The mechanic is portable: it lives in the bee binary and the skills, and reaches hosts through onboarding with no host-side configuration. (knowledge-freshness-mechanic D7, 6a502ad0) | No bee-repo-only path in the check or the gate. |

### Agent's Discretion

- The exact grammar of the `## Read set` tables, as long as the knowledge
  table's first column is a repo-relative path under `docs/knowledge/` and
  the parser is prefix-tolerant like the Locked Decisions parser.
- The escape for a CONTEXT.md written before this feature: a reasoned
  `<!-- bee:no-read-set: <reason> -->` marker in CONTEXT.md passes the
  check; the refusal names it.
- Whether the knowledge table's grade column is checked for staleness at
  approval (recommended: printed in the refusal text only, never a block).
- The wording of each template's section question and deletion line.
- Whether a rule bullet's status and verification pointer use a fixed
  inline form (`— status: temporary; proves: <path>`) or a sub-bullet.

## Terms

| Term | Meaning in this feature |
|------|-------------------------|
| read set | The knowledge concepts and code entry points a feature was read against at shaping and planning, recorded in CONTEXT.md. |
| section question | The one sentence at the top of a template section that says what the section answers for a rebuilder. |
| deletion line | The sentence in a template that tells the writer when to remove a bullet (a test already states it; nobody would miss it). |
| decision link | A non-empty `bee.decisions` list on an area concept. |
| cut area template | The area concept shape the content cut will use: map, boundaries, invariants, entry points, pointers. |

## Read set

### Knowledge

| Concept | Grade at read | Why it matters |
|---------|---------------|----------------|
| docs/knowledge/areas/okf-profile/freshness-and-document-homes.md | confirmed | The document homes rule and the grade this feature prints in the read set. |
| docs/knowledge/areas/okf-profile/concept-model-and-authoring.md | confirmed | Concept frontmatter contract; `bee.decisions` is the field D4 checks. |
| docs/knowledge/areas/workflow-state/gates.md | suspect | Shape and merged gate approval, the preview hash, where the read-set refusal sits. |
| docs/knowledge/areas/workflow-state/cells-completion-judge-and-archive.md | suspect | The cap sync door; unchanged here, cited by D1. |
| docs/knowledge/areas/doctrine-layer/router-triage-and-the-agents-md-duplication-boundary.md | suspect | Rule homes and the render chain (`bee dev regen`); skill edits must keep parity. |
| docs/knowledge/areas/decision-memory/owner-brief-contract.md | confirmed | The brief this feature's Lock wrote. |

### Code

| Path | What it decides |
|------|-----------------|
| packages/bee-rs/crates/bee/src/verbs/drivers/context_table.rs | The CONTEXT.md Locked Decisions table parser; the read-set parser is its sibling. |
| packages/bee-rs/crates/bee/src/verbs/state_group/set_gate.rs | Shape and merged gate approval; `gate_preview_refusal` is the refusal the read-set check joins. |
| packages/bee-rs/crates/bee/src/verbs/knowledge/check.rs | `check_bundle` and its finding rows; the no-decision-link error joins the profile errors. |
| packages/bee-rs/crates/bee/src/verbs/knowledge/frontmatter.rs | Canonical frontmatter emit and parse; `bee.decisions` round-trips through it. |
| packages/bee-rs/crates/bee/src/devtools/skill_trees.rs | Renders skills/ into the runtime skill trees; a skill reference edit must regen. |
| skills/bee-shaping/references/context-template.md | The CONTEXT.md template that gains `## Read set`. |
| skills/bee-capturing/references/area-spec.md | The area template that gains section questions, deletion lines, rule bullet fields and the cut area shape. |
| skills/bee-planning/references/planning-reference.md | Where `read_first` is defined; it gains the citation of the read set. |

## Specific Ideas And References

- Owner direction 2026-11-08: the flow itself must say what to read at
  shaping and planning, update knowledge at close as a door, and give
  writers templates that ask for information rather than shape; a document
  nobody would miss is deleted.
- The owner's reference document (an AI-agent workflow framework, 2026-11-08)
  proposes rule records with status and a verification link, a product
  direction document, and a read-then-verify order; D3 takes the rule fields,
  the rest stays on the map.
- Measured 2026-11-08: 140 area concepts, 5 with no decision link; 46
  confirmed, 77 suspect, 17 unverified after the freshness seed.

## Existing Code Context

### Reusable Assets

- `packages/bee-rs/crates/bee/src/verbs/drivers/context_table.rs` —
  `parse_locked_decision_ids`: heading-scoped, prefix-tolerant pipe-table
  parser; the read-set parser copies its shape.
- `packages/bee-rs/crates/bee/src/verbs/state_group/set_gate.rs` —
  `gate_preview_refusal` (used at :937 and :950) is the model for a
  zero-mutation refusal before a shape or merged approval.
- `packages/bee-rs/crates/bee/src/verbs/knowledge/check.rs` — `check_bundle`
  (:557) builds `finding(path, code, message)` rows; profile errors such as
  `owns_missing` (:894) show the error shape and remedy wording.
- `packages/bee-rs/crates/bee/src/verbs/knowledge/frontmatter.rs` —
  `emit_frontmatter` (:90) and `parse_frontmatter` (:259); `bee.decisions`
  is a flat list and round-trips.
- `skills/bee-capturing/references/area-spec.md` — the Gather Sources table
  (:33-39) already names per-section sources; D3 generalises that form.

### Established Patterns

- A new door ships with a recorded escape and a remedy sentence that names
  the command that clears it (critical patterns 20260806, 20260825).
- Finding rows carry a code, a path and a remedy; `--json` output is a
  manifest, never file content.
- Skill edits land in `skills/` and are rendered by `bee dev regen`; the
  parity tests in `packages/bee-rs/crates/bee/tests/rule_index_parity.rs`
  red on drift.

### Integration Points

- `packages/bee-rs/crates/bee/src/verbs/state_group/set_gate.rs` — the
  shape and merged approval path gains the read-set refusal.
- `packages/bee-rs/crates/bee/src/verbs/drivers/context_table.rs` — gains
  `parse_read_set`.
- `packages/bee-rs/crates/bee/src/verbs/knowledge/check.rs` — gains the
  `missing_decision_link` profile error.
- `skills/bee-shaping/SKILL.md` and `references/context-template.md` —
  Explore runs `bee knowledge context` and `bee knowledge search` and Lock
  renders `## Read set`.
- `skills/bee-planning/SKILL.md` and `references/planning-reference.md` —
  `read_first` cites the read set; the plan's Discovery section extends it.
- `skills/bee-capturing/references/area-spec.md` and
  `references/concept-authoring.md` — section questions, deletion lines,
  rule bullet fields, the cut area template.
- `docs/knowledge/areas/okf-profile/` — the concept that documents the
  read set, the templates rule and the decision-link error.
- The five area concepts with no decision link, named in the plan.

## Canonical References

- `docs/discovery/knowledge-from-code/MAP.md` — the decisions this feature
  consumes.
- `docs/knowledge/areas/okf-profile/freshness-and-document-homes.md` — the
  document homes rule and the freshness grades.
- `docs/knowledge/areas/decision-memory/owner-brief-contract.md` — the
  brief contract Lock followed.

## Outstanding Questions

<!-- bee:not-a-deferral: template section names; both questions are planning's to answer from the code, not promises to act later -->
### Resolve Before Planning

- (none)

### Deferred To Planning

- [ ] Whether the read-set refusal reads CONTEXT.md from the lane's history
  root or the main checkout when run inside a worktree — planning reads how
  `build_routing_door` resolves the path.
- [ ] Whether `missing_decision_link` counts as a profile error for every
  area concept or only for concepts with `bee.anchors` — planning measures;
  today both sets give the same five.
<!-- /bee:not-a-deferral -->

<!-- bee:not-a-deferral: these ideas are owned by the knowledge-from-code map and PBI p-3a756890, cited inline -->
## Deferred Ideas

- The content cut of bee's own areas — PBI p-3a756890 (map D5, D10).
- Glossary (map ticket 004) and event catalogue (ticket 003) — wait on the
  pull-rate trigger `bee-knowledge-report-shows-a-pull-rate-r__28fb074d`.
- A product direction document per host project — on the map as fog.
<!-- /bee:not-a-deferral -->

## Handoff Note

<!-- bee:not-a-deferral: template prose describing what planning reads, not a promise to act later -->
CONTEXT.md is the source of truth. Decision IDs are stable. Planning reads locked
decisions, the read set, code context, canonical references, and deferred-to-planning questions.
Planning's Gate 2 shape stage and reviewing use locked decisions for coverage and UAT.
<!-- /bee:not-a-deferral -->
