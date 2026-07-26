# Stage: reviewing (`bee-reviewing`)

**Purpose** — Run an independent multi-agent review gate — severity findings,
artifact verification, and human UAT — over an immutable, user-chosen scope.

**When it runs** — **Only on explicit user request** ("review this", "review
today's work", a named list, a diff range, "review everything unreviewed before
release"). Never auto-triggered by a finished cell or feature, and never by the
words "merge" / "ship" / "release" alone — those get a *report + one question*
first.

## Inputs
- The review session (`reviews show --id`), `CONTEXT.md` / `plan.md` per feature in
  scope, the cumulative diff, capped cells and traces, `status --json`.

## Outputs
- A review session record (`reviews create/record`), findings graded **P1 / P2 / P3**.
- Artifact verification results, UAT records.
- Backlog rows for P2/P3, a session decision (`pending` / `blocked` / `approved`).
- Optional `walkthrough.md` via bee-briefing.

## Gate
**Gate 4** — P1>0: "P1 findings block merge. Fix before proceeding?"; P1=0:
"Review complete. Approve merge?" **This gate lives only inside a review session.**

## State touched
[`reviews create/show/record/candidates/status`](../register.md#logs--caches-read-mostly),
[`backlog add --type review-finding`](../register.md#beebacklogjsonl). Does **not**
call generic `state set` for the feature phase.

## Key rules
- **Scope is frozen before any reviewer is dispatched** — `reviews create` fails
  closed on missing verification evidence.
- Reviewer panel depth scales to the *session scope* risk (1 / 4 core / full wave,
  cap 6), not the originating feature's lane.
- **Gate bypass never creates or auto-approves a review session** — only the merge
  question, and only when P1=0 and UAT all pass.
- **Never continue past open P1s** without explicit user acknowledgment.

## Source
`skills/bee-reviewing/SKILL.md`
