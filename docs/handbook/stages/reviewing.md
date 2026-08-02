# Stage: reviewing (`bee-reviewing`)

**Purpose** — Run an independent multi-agent review gate — severity findings,
artifact verification, and human UAT — over an immutable, user-chosen scope.

**When it runs** — **Only on explicit user request** ("review this", "review
today's work", a named list, a diff range, "review everything unreviewed before
release"). Never auto-triggered by a finished cell or feature, and never by the
words "merge" / "ship" / "release" alone — those get `bee reviews status`, one
yes/no question, and silence left as unreviewed.

## Inputs
- The review session (`reviews show --id`), `CONTEXT.md` / `plan.md` per feature in
  scope, the cumulative diff, capped cells and traces, `bee status --json`.
- Each reviewer gets the cumulative diff and the in-scope features' `CONTEXT.md`
  and `plan.md` — and nothing else. Never session history.

## Outputs
- A review session record (`reviews create/record`), findings graded **P1 / P2 / P3**.
- Artifact verification results, UAT records.
- Backlog rows for P2/P3, a session decision (`pending` / `blocked` / `approved`).
- Optional `walkthrough.md` via `bee-shaping` (Brief walkthrough).
- If filing a finding fails, `docs/history/<feature>/reports/residual-findings.md`
  — nothing evaporates.

## Gate
**Gate 4** — P1>0: "P1 findings block merge. Fix before proceeding?"; P1=0:
"Review complete. Approve merge?" **This gate lives only inside a review session.**

## State touched
[`reviews create/show/record/candidates/status`](../register.md#beereviewsreview-idjson),
[`backlog add --type review-finding`](../register.md#beebacklogjsonl). Does **not**
call generic `state set` for the feature phase: closing a review closes the review,
not any feature — their already-closed state stays untouched.

## Key rules
- **Scope is frozen before any reviewer is dispatched** — `reviews create` freezes
  baseline, head, included and excluded, and checks verification evidence before a
  reviewer spends a token. If it refuses, surface its reason; never dispatch
  reviewers to compensate. Show the user the preview before dispatch.
- **The panel scales to the session scope's risk**, not the originating feature's
  lane: a small scope takes one correctness reviewer; high-risk content warrants
  the core four (`code-quality`, `architecture`, `security`, `test-coverage`) plus
  conditional reviewers, capped at six.
- **No synthesis before every reviewer returns.** Deduplicate; independent
  corroboration promotes one level; disagreement takes the conservative route;
  uncertain lands at P2.
- **Verify the artifacts, not the story.** Missing or vague verification evidence
  on a capped behavior-change cell is a **P1** — the behavior was never proven, and
  the remedy is re-verifying, never a backfill document. A weakened judge
  (softened assertions, skipped tests, a loosened verify command) is a **P1**.
- **After a P1 fix caps, re-review the delta AND sweep the scope for that defect
  class** — the same bug hides in siblings. A fix that crossed a boundary earns an
  expanded re-review *proposed to the user*, never silently chosen.
- **Gate bypass never creates or auto-approves a review session** — only the merge
  question, and only when P1=0 and UAT all pass.
- **Never continue past open P1s** without explicit user acknowledgment. Silence is
  not acknowledgment.

## Source
`skills/bee-reviewing/SKILL.md` + `references/reviewing-reference.md`;
craft in `.bee/expertise/review.md`
