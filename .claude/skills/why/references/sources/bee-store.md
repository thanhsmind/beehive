# Bee Store and Project Records

## What this source contains

- Active and archived decision log entries (`bee decisions search`), recording explicit agreements, rationales, alternatives considered, supersedes/touches relations, and tags
- Locked feature decisions in `docs/history/<feature>/CONTEXT.md`, capturing product and architectural invariants locked before implementation
- Living area specifications in `docs/knowledge/areas/`, defining subsystem boundaries, domain models, and architecture contracts
- Captured learnings and retrospectives in `docs/history/learnings/` and `docs/knowledge/patterns/`, documenting failure modes, incident root causes, and structural lessons
- Feature plans and review records in `docs/history/<feature>/plan.md` and `docs/history/<feature>/review.md`

Bee's records provide explicit, durable documentation of why decisions were made, what tradeoffs were weighed, and what past issues shaped the system.

## How to search it

Search the decision log using the bee CLI:

```bash
# Search active decision records by keywords (OR-matched, ranked by hits)
.bee/bin/bee decisions search --text "<terms>"

# Search across both active and archived decisions
.bee/bin/bee decisions search --text "<terms>" --all

# Filter decisions by feature slug or area scope
.bee/bin/bee decisions search --feature "<feature-slug>"
.bee/bin/bee decisions search --scope "<area-name>"

# Filter decisions by cell ID
.bee/bin/bee decisions search --cell "<cell-id>"
```

Search locked feature decisions and historical context:

```bash
# Find CONTEXT.md across feature histories
fd CONTEXT.md docs/history/

# Search for specific terms or decisions inside CONTEXT.md files
rg -i '<term>' docs/history/*/CONTEXT.md

# Inspect a feature plan for rationale and rejected alternatives
rg -i '<term>' docs/history/*/plan.md
```

Search area specifications:

```bash
# Search living architecture specs in docs/knowledge/areas/
rg -i '<term>' docs/knowledge/areas/
```

Search captured learnings and pattern records:

```bash
# Search postmortems and captured learnings
rg -i '<term>' docs/history/learnings/

# Search critical patterns and failure mode analyses
rg -i '<term>' docs/knowledge/patterns/
```

## What good evidence looks like here

- An explicit decision log entry (`decide` record) detailing why an option was chosen and what alternatives were rejected
- A locked decision in `docs/history/<feature>/CONTEXT.md` (e.g., `D1`, `D2`) declaring intentional constraints or requirements
- An area specification in `docs/knowledge/areas/` stating an invariant, layering rule, or domain boundary
- A learning file in `docs/history/learnings/` explaining a bug, regression, or failure mode that motivated adding a guard or refactoring code
- A `supersedes:<id>` or `touches:<id>` relation linking an active decision back to prior rationale

## Common pitfalls

- **Treating superseded decisions as current law.** Check whether an earlier decision was superseded (`supersedes:<id>`). A newer decision or current area spec takes precedence over older rationale.
- **Confusing draft proposals with locked decisions.** Only decisions logged in the store or recorded in `CONTEXT.md` are locked commitments. Freeform notes or exploratory plans may reflect rejected ideas.
- **Missing archived decisions.** Older decisions are archived into `.bee/decisions-archive.jsonl`. Use `.bee/bin/bee decisions search --all` when a query returns no active hits.
- **Ignoring area specs in favour of isolated comments.** Subsystem contracts in `docs/knowledge/areas/` reflect synthesized domain truth and supersede one-off commit comments.

## What to return

Every decision record, CONTEXT.md requirement, area spec rule, or learning document bearing on the question:
- The exact text (quoted)
- The decision ID (e.g. `dec-123`), feature slug, or file path and line number
- Date, author/agent, and relation tags if available
- Whether it is direct (explicitly addresses the question) or circumstantial
