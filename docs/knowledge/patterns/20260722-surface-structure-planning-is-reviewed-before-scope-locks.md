---
type: bee.pattern
title: "A plan shaped by a document's literal structure gets fresh-eyed before scope locks"
description: "When a plan's shape comes from file counts or heading lists rather than a content model, review that assumption at design time — not at validation, not at execution."
tags: [planning, review, decomposition, scope]
timestamp: 2026-07-22
bee:
  id: pattern-20260722-surface-structure-planning
  lifecycle: active
  areas: [okf-profile]
  sources: [docs/history/learnings/20260722-okf-foundation.md, "docs/history/okf-foundation/CONTEXT.md (D9/D23 withdrawn, D25/D29/D30 superseded)"]
  polarity: pitfall
  critical: false
---

# A plan shaped by a document's literal structure gets fresh-eyed before scope locks

A plan derived from a document's **surface structure** — its file count, its heading list, its line
count — always looks executable and cheap. The mismatch between that structure and what the content
actually *is* does not surface until execution, and then it arrives as scope blowup or orphaned
content.

Two of two such plans in `okf-foundation` were wrong, and both were cheap to kill at design time:

- A retrofit shaped by **file count** — stamp frontmatter onto all 593 `.md` files under `docs/`.
  Reframing "concept" as *any non-reserved `.md` inside a curated `docs/knowledge/`* deleted the
  retrofit, three incompatible legacy frontmatter schemas, 14 prose-header learnings, a
  filename-with-spaces hazard, and a `timestamp` double-source conflict — in one move.
- A split shaped by **heading list** — five concepts for the 1464-line `workflow-state.md`. Its
  headings are a BA template (Purpose / Behaviors / Business Rules / Edge Cases / Pointers), not a
  topic map, so the five names left ~700 lines homeless, with multi-session coordination — the
  second-largest cluster in the file — having no destination at all.

## The rule

When a plan's shape is derived from a document's literal structure rather than from a content-level
model of what governs what, run a fresh-eyes review of **that structural assumption** before locking
scope. Do not wait for validation or execution to discover the map is wrong.

**Corollary — a homeless-content signal means the axis is wrong, not that the work is hard.** When a
proposed decomposition leaves content with no destination, stop and re-derive the split axis. Trying
harder along a wrong axis produces a dumping-ground concept, which is how "two artifacts both claim
to be the source of truth" begins.

**Corollary — prove a risky mechanism on the smallest artifact that shares the real one's
STRUCTURE**, not merely something small. `advisor-protocol.md` (202 lines) carries the identical
nine-section template as `workflow-state.md` (1464 lines), so it exercised every migration path —
frontmatter carry-over, numbered-anchor redistribution, pointer stub, anchor map — at a seventh of
the risk.
