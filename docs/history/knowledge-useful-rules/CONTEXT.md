# Knowledge that agents can use — Context

**Feature slug:** knowledge-useful-rules
**Date:** 2026-11-10
**Shaping session:** complete (the owner approved the five rules in chat: "Làm đi")
**Scope:** Standard
**Domain types:** READ | RUN | ORGANIZE

## Feature Boundary

bee gives every agent that writes knowledge the same five writing rules from
one home, prints a concept's "Read when" line in the pack, warns in
`bee knowledge check` when a concept breaks a rule a machine can see, records
on each cap which concepts changed a decision, and reports which concepts are
used and which are loaded but never used. It ends at the mechanism: rewriting
the existing concepts to the new rules is a separate backlog item.

## Locked Decisions

| ID | Decision | Rationale (only if it changes implementation) |
|----|----------|-----------------------------------------------|
| D1 | Title and description carry the words a reader searches for: the error text, the symptom, the mechanism. (71241c48) | Search and the relevance ranker read title, description and body tokens. |
| D2 | Every area and pattern concept body opens with one line `Read when: <situation>`; the pack prints that line in place of the title when it exists. (71241c48, contract f4b38727) | The pack shows a path and one line per concept. |
| D3 | An entry keeps the why, the boundary and the rejected option with how it failed; it never restates the how that code or a test shows. (71241c48) | |
| D4 | Anchors name files, never directories; a concept over 300 lines splits along its lifecycles. `bee knowledge check` warns on both and on a missing Read-when line, and never blocks. (71241c48, contract e8899530) | A directory anchor grades the concept suspect on most commits. |
| D5 | A cap records which concepts changed a decision; the report lists use counts and concepts loaded five or more times with no use. Silence is recorded, never refused. (71241c48, contracts 721d06f5, e759fbaa) | knowledge-in-the-flow D1 (9dca3714) forbids a mandatory knowledge-ref check. |
| D6 | The rules have one home, the consolidate rule list, rendered into the worker brief. (contract 90988b99) | The worker writes the delta at cap and never runs consolidate. |

### Agent's Discretion

- The `Read when:` line is searched in the first 10 non-blank, non-heading body lines.
- The never-used threshold is five loads.

## Read set

### Knowledge

| Concept | Grade at read | Why it matters |
|---------|---------------|----------------|
| docs/knowledge/areas/okf-profile/freshness-and-document-homes.md | suspect | The document homes rule, anchors, the templates rule the new rules extend. |
| docs/knowledge/areas/okf-profile/context-and-promote.md | suspect | The pack assembler and consolidate's thin-layer rules (B5). |
| docs/knowledge/areas/workflow-state/cells-completion-judge-and-archive.md | suspect | The cap trace and the mistakes answer the knowledge-used answer copies. |
| docs/knowledge/areas/okf-profile/critical-bar.md | suspect | The knowledge report and its outcome measure. |

### Code

| Path | What it decides |
|------|-----------------|
| `packages/bee-rs/crates/bee/src/verbs/knowledge/consolidate.rs` | The thin-layer rule list. |
| `packages/bee-rs/crates/bee/src/verbs/drivers/prepare.rs` | The one assembler's learned-context lines and the worker prompt variables. |
| `packages/bee-rs/crates/bee/src/verbs/knowledge/check.rs` | Profile warnings. |
| `packages/bee-rs/crates/bee/src/verbs/cells/handlers_close.rs` | The cap trace and the mistakes answer. |
| `packages/bee-rs/crates/bee/src/verbs/knowledge/report.rs` | The knowledge report. |

## Canonical References

- `docs/history/knowledge-in-the-flow/CONTEXT.md` — D1 flow not gate, D7 thin layer.
- `docs/history/feature-pack-areas/` — area concepts enter the pack under a history anchor.

<!-- bee:not-a-deferral: template section names with no open question under them, not a promise to act later -->
## Outstanding Questions

### Resolve Before Planning

- (none)

### Deferred To Planning

- (none)
<!-- /bee:not-a-deferral -->
