# Reviewing Reference

Companion to SKILL.md — judgment lives there; reviewer prompts, the
finding schema, and the acceptance wording live here.

## Reviewer prompts

Common shape, every reviewer:

```text
You are the <X> reviewer. Review only your focus area. Lead with findings.
For each: severity, file/line evidence, failure scenario, smallest credible fix.
Do not rewrite code.
```

Core four (always dispatched, parallel) — append the focus line to the
common shape:

| Reviewer | Focus line |
|---|---|
| `code-quality` | Correctness, readability, type safety, error handling. Cite file/line evidence for every claim. |
| `architecture` | Boundaries, coupling, API design, maintainability, drift from plan.md structure. |
| `security` | Auth, authorization, secrets in code or logs, injection, permissions, data exposure. |
| `test-coverage` | Missing edge cases, regression paths, weak or tautological assertions, untested behavior changes. |

Conditional reviewers — scan the diff once, mechanically (file paths and
hunks, not vibes); spawn every matched trigger in the same parallel
wave, same isolation contract, same prompt shape. Cap the wave at six
total; if more triggers match, fold the extra lens into the closest core
reviewer's focus line and say so in the synthesis.

| Reviewer | Spawn when the diff touches | Focus line |
|---|---|---|
| `performance` | ORM/query calls inside loops, caching layers, pagination, hot-path data access | Query patterns, N+1 exposure, cache correctness, unbounded result sets. Flag only measurable risks with the triggering code cited. |
| `api-contract` | routes, serializers, public response shapes, exported type signatures, versioned endpoints | Client-visible breaking changes, envelope drift, missing versioning, silent field removals — checked against locked decisions. |
| `data-migration` | migration files or schema definitions only (`**/migrations/**`, `db/migrate/*`, `schema.*`, `*.sql` DDL) | Destructive DDL, backfills on large tables, NOT NULL without default, irreversibility, deploy-order coupling. |
| `reliability` | retries, timeouts, queues, background jobs, webhooks, external service calls | Failure paths: what happens on timeout, partial failure, replay, and double delivery. Missing idempotency and dead-letter handling. |

Personas stay thin lens contracts — no failure-mode catalogs. The model
already knows the domain; the trigger and the lens are the value.

## Finding schema

Every distinct issue becomes one finding:

```markdown
### [P<N>] <problem title>   (autofix_class: gated_auto | manual | advisory)

## Plain-Language Summary
<1-3 sentences a non-specialist understands>

## What The Code Does Today
- <current behavior, with source>

## Why This Is A Problem
- <requirement, locked decision, or invariant broken>

## Concrete Failure Scenario
- <realistic steps and the incorrect outcome>

## Evidence
File: `path`
Line(s): <line>
Snippet: <small relevant snippet>
Why this proves the issue: <one sentence>

## Proposed Fix
Recommended: <smallest credible fix>
Tradeoff: <if any>

## Acceptance Criteria
- [ ] <specific testable condition>
```

`autofix_class` routes the work — `gated_auto` (concrete fix applied
after orchestrator judgment), `manual` (needs design input), `advisory`
(report-only) — and never bypasses judgment or the merge question.

## Human UAT

For each SEE/CALL/RUN decision in CONTEXT.md, verbatim:

```text
UAT Item <i>/<n> - Decision <D-id>:
"<deliverable>"
Can you confirm this works? [Pass / Fail / Skip]
```

- Fail → create a P1 fix cell, then rerun this item after the fix caps.
- Skip → record the user's stated reason before moving on.
- Intermittent failure is a Fail, not a Skip.
