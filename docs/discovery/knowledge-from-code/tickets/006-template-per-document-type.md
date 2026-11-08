---
type: grilling
status: open
blocked-by: 002-area-cut-without-losing-reasons
---

## Question

Which knowledge document types get a named template that tells the
agent what information to add and where, and what does the cut area
concept's template hold? Today bee has templates for CONTEXT.md, plan,
the nine-section area spec, the research brief and the mini-brief, but
not for the cut area shape D2 (93e1001d) names (map, boundaries,
invariants, entry points, pointers), not for the vision, and not for a
glossary. Two sub-questions decide the shape:

1. Does a business rule in an area concept carry its own fields —
   status (active, temporary, retired), the decision that settled it,
   and a verification pointer (the test or verify feature that proves
   it) — or does the concept's frontmatter (`decisions`, `anchors`,
   `verified_commit`) carry that for the whole concept and a rule stays
   one bullet?
2. Which prompts go inside each template: a "what to add here" line per
   section (the question the section answers for a rebuilder, D2's
   homes rule) so the writer fills information rather than shape, and a
   deletion line ("remove this bullet if a test already states it").

Agent recommendation: per-rule fields only where the rule can differ
from its concept — status and verification pointer on the bullet,
decision at the concept level; every template section opens with the
one question it answers and names the source that may feed it (the
Gather Sources table of `skills/bee-capturing/references/area-spec.md`
already does this for the area spec).

## Answer
