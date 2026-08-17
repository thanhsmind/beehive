# Mini-Brief Template (`small` lane)

Load when shaping's Brief move renders a `small`-lane feature. The full template is overkill here; the mini-brief is the ~15-line record a human can review at Gate 2 without opening `plan.md`. Same projection rules as the full template (`implement-plan-template.md`): render from the truth artifacts, author only the one-line Risk/Rollback, never invent, never claim un-run validation.

Path: `docs/history/<feature>/implement-plan.md` (same file, smaller body).

```markdown
---
artifact_contract: bee-implement-plan/v1
feature: <feature-slug>
lane: small
status: Draft | Ready for Review | Approved | Needs Revision
updated: YYYY-MM-DD
sources: [CONTEXT.md, plan.md]
decisions: [<D-IDs>]
---

# Implementation Plan: <Feature>

**Goal** — <the user outcome in one sentence; cite D-IDs>

**In scope** — <what changes>
**Out of scope** — <what does not>

**Affected files**
- `<path>` — <why>

**Validation** — `<the declared test command>` → expected: <outcome>. Tests prove at the boundary (`bee close`/`bee worktree merge`). Evidence: <link `.bee/logs/test-results.json` once the boundary runs; "pending" before>

**Risk** — <one line, or "none">
**Rollback** — <one line: revert the cell's commit / disable the flag>

**Open questions** — <or "none — ready for review">
```

Rules:

- If a line has no real content, delete it — do not write "N/A". (Same drop-empty discipline as the full template.)
- If the work grows past `small` during planning, the lane changes upstream in `plan.md` first; then re-render as the full template. Never stretch the mini-brief to carry `standard` work.
- Everything but Risk and Rollback is projected from the truth artifacts. Those two one-liners are the only authored content; an undecided rollback is "OPEN QUESTION", not a plausible guess.
