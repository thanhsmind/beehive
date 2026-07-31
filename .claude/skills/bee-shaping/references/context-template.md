# CONTEXT.md Template

Write this to `docs/history/<feature-slug>/CONTEXT.md`. Remove unused optional sections.
No TODOs, placeholders, or vague decisions. Decision IDs are stable forever —
downstream cells cite them (`per D2`), so never renumber.

```markdown
# <Feature Name> — Context

**Feature slug:** <slug>
**Date:** YYYY-MM-DD
**Shaping session:** complete
**Scope:** Quick | Standard | Deep
**Domain types:** SEE | CALL | RUN | READ | ORGANIZE

## Feature Boundary

<One concrete sentence describing what this feature delivers and where it ends.>

## Locked Decisions

These are fixed. Planning must implement them exactly — cited, never reinterpreted.
Changing one requires the user, a new D-ID or an explicit supersession note, never
a silent edit.

| ID | Decision | Rationale (only if it changes implementation) |
|----|----------|-----------------------------------------------|
| D1 | <specific decision, not a preference> | <why, or —> |
| D2 | <specific decision> | <why, or —> |

### Agent's Discretion

<What the user delegated to the agent, with constraints. Remove if none.>

## Terms

Pin a fuzzy domain word here the moment Socratic locking settles its meaning —
scribing seeds the spec's Data Dictionary from this table. Remove if none came up.

| Term | Meaning in this feature |
|------|-------------------------|
| <term> | <settled meaning, one line> |

## Specific Ideas And References

- <Mockup/example/reference the user mentioned, and what it means for the work.>

## Existing Code Context

From the quick scout only. Downstream agents read these before planning.

### Reusable Assets

- `<path>` — <what it does and how it applies>

### Established Patterns

- <pattern> — <where used and what to reuse>

### Integration Points

- `<path>` — <what new work connects to>

## Canonical References

- `<path-or-url>` — <what this defines>

## Outstanding Questions

### Resolve Before Planning

- [ ] <question> — <why it blocks planning>

### Deferred To Planning

- [ ] <technical question> — <what investigation answers it>

## Deferred Ideas

Out-of-scope ideas captured during shaping. Not lost, not planned.

- <idea> — <why deferred>

## Handoff Note

CONTEXT.md is the source of truth. Decision IDs are stable. Planning reads locked
decisions, code context, canonical references, and deferred-to-planning questions.
Planning's Gate 2 shape stage and reviewing use locked decisions for coverage and UAT.
```
