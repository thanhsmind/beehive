# Reviewing Reference

Companion to SKILL.md — judgment lives there; reviewer role cards, the
finding schema, and the acceptance wording live here.

## Reviewer roles

Every dispatched reviewer receives a role card in the same shape:
**Purpose** (one line — the lens), **Scope** (set by the invoking
layer: the frozen diff, the in-scope `CONTEXT.md` and `plan.md`,
nothing else — never widened by the reviewer), and **Method** (the
numbered steps below, whose quoted names resolve to entries in
`.bee/expertise/review.md`). Role cards stay thin lens contracts — no
failure-mode catalogs. The model already knows the domain; the trigger
and the lens are the value.

Dispatch prompt shape, every reviewer:

```text
You are the <role> reviewer.
Purpose: <the role's Purpose line>
Scope: <the frozen scope the invoking layer hands you>
Method: follow the reviewer method; your step-2 lens is your Purpose.
Lead with findings. Do not rewrite code.
```

### Reviewer method (shared by all roles)

1. From the scope you were handed, write down what a correct change
   must handle before reading the diff — "Adversarial reading".
2. Read the diff through your Purpose's lens only, hunting what the
   diff does not say: absent handling, the cases the author was least
   likely to run, the edges just outside the changed region —
   "Adversarial reading".
3. Reproduce or trace every suspected defect before filing — "Verify
   before reporting"; what qualifies as a finding at all — "What a
   finding is".
4. Set severity by consequence, not offense — "Severity calibration";
   torn between two levels, take the lower and name the condition for
   the higher — "Severity is a spent signal".
5. Write each finding in the schema below: file/line, quoted behavior,
   sketched fix — "Evidence standards"; label anything unverified —
   "Label uncertainty exactly"; keep out-of-scope bugs in a separate
   follow-up note, never against the verdict — "Scope discipline".

### Core roles — always dispatched, in parallel

#### code-quality

- **Purpose:** Catch code that computes the wrong thing or mishandles failure — correctness, readability, type safety, error handling.
- **Scope:** Set by the invoking layer. Cite file/line evidence for every claim.
- **Method:** Reviewer method 1–5, step-2 lens as Purpose.

#### architecture

- **Purpose:** Catch structural damage — boundaries, coupling, API design, maintainability, drift from plan.md structure.
- **Scope:** Set by the invoking layer; drift is judged against the in-scope plan.md, never against memory.
- **Method:** Reviewer method 1–5, step-2 lens as Purpose.

#### security

- **Purpose:** Catch exposure — auth, authorization, secrets in code or logs, injection, permissions, data exposure.
- **Scope:** Set by the invoking layer.
- **Method:** Reviewer method 1–5, step-2 lens as Purpose; a verified exposure files at P1 — "Severity calibration".

#### test-coverage

- **Purpose:** Catch unproven behavior — missing edge cases, regression paths, weak or tautological assertions, untested behavior changes.
- **Scope:** Set by the invoking layer.
- **Method:** Reviewer method 1–5, step-2 lens as Purpose; a missing test is filed with the uncovered scenario named — "What a finding is".

### Conditional roles — spawned by diff triggers

Scan the diff once, mechanically — file paths and hunks, not vibes —
and spawn every matched role in the same parallel wave, same isolation
contract, same card shape. Cap the wave at six total; if more triggers
match, fold the extra lens into the closest core role's Purpose and say
so in the synthesis.

#### performance

- **Purpose:** Catch measurable slowdowns — query patterns, N+1 exposure, cache correctness, unbounded result sets.
- **Scope:** Set by the invoking layer; spawned when the diff touches ORM/query calls inside loops, caching layers, pagination, or hot-path data access.
- **Method:** Reviewer method 1–5; file only measurable risks, with the triggering code cited — "Evidence standards".

#### api-contract

- **Purpose:** Catch client-visible breakage — breaking changes, envelope drift, missing versioning, silent field removals.
- **Scope:** Set by the invoking layer; spawned when the diff touches routes, serializers, public response shapes, exported type signatures, or versioned endpoints.
- **Method:** Reviewer method 1–5; check every suspected break against the locked decisions in the in-scope CONTEXT.md before filing — "Verify before reporting".

#### data-migration

- **Purpose:** Catch irreversible schema damage — destructive DDL, backfills on large tables, NOT NULL without default, deploy-order coupling.
- **Scope:** Set by the invoking layer; spawned only for migration files or schema definitions (`**/migrations/**`, `db/migrate/*`, `schema.*`, `*.sql` DDL).
- **Method:** Reviewer method 1–5; state the rollback story for every flagged change — "Evidence standards".

#### reliability

- **Purpose:** Catch failure-path gaps — timeout, partial failure, replay, double delivery, missing idempotency and dead-letter handling.
- **Scope:** Set by the invoking layer; spawned when the diff touches retries, timeouts, queues, background jobs, webhooks, or external service calls.
- **Method:** Reviewer method 1–5; every finding states the concrete failure sequence, never "could fail" — "What a finding is".

## Finding schema

Every distinct issue becomes one finding, labeled with its axis —
`standards` (is the code well made: quality, architecture, security,
tests) or `spec` (does it do what the locked decisions promised). The
synthesis report stays ONE report, grouped by axis, spec-axis group
first; axes are never collapsed into one undifferentiated ranked list.

```markdown
### [P<N>] <problem title>   (axis: standards | spec, autofix_class: gated_auto | manual | advisory)

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
