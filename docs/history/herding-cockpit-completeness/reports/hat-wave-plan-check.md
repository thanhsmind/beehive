# Hat wave — plan check (herding-cockpit-completeness)

Wave: 2026-09-06, three seats, all returned inside the 10-minute ceiling.

| Seat | Dispatch | Channel | Result |
|------|----------|---------|--------|
| hat-facts-gaps | 559549ba | claude-agent (opus) | returned |
| hat-alternatives | 7d04885a | claude-agent (opus) | returned |
| hat-user-impact | fd9e7d9d | herding pane w1:pN, agy-flash, job hat-user-impact-hcc | returned as `malformed_result` (UTF-8 BOM on result-1.json); report-1.md read and used |

Dropped seats: none. Diversity: two models, two channels.

```text
PLAN CHECK
Work: slice 1 of docs/history/herding-cockpit-completeness/plan.md

STRUCTURE
BLOCKERS: none
WARNINGS: cell-completeness test filter for job_verbs.rs unnamed / plan.md Test matrix / filters added per slice
WARNINGS: key-links --continue refusal on a cancelled mark implied, not owned / run.rs:2449-2455 checks paused_limit_at only / owned by slice 1 "Mark reads"

CELLS  (reviewed: 0 — cells are drafted after the gate; MANDATE 2 runs at drafting)
CRITICAL FLAGS: none
MINOR FLAGS: none
CLEAN CELLS: n/a
```

## Findings folded into plan.md

- Facts seat: claims table rows 1–17 all match their anchors. Two unbacked claims fixed: the poll tick does not read job.json today (run.rs:1140-1197, :2097-2144) and `execute_continue` does not read a mark (run.rs:2424-2500). Plan now adds one `read_mark` helper, a tick read, and a `ContinueRefusal::Cancelled`.
- Facts seat: tmux named key must go without `-l` (pane_verbs.rs:311-318). Plan states it.
- Alternatives seat: keep on all six questions — job.json fields, new trait method, 5-line pid poll, two `RunOutcome` variants, shared `mark_orphans` helper, `verbs/worktree/git.rs::run_git`.
- User-impact seat: `retryable` must be `false` on `blocked` (D3 lists it) — plan now emits it on every envelope except `done`/`dry_run`; cancel FIX line names the pid and a re-run finalises a `cancel_pending` job; `mark_reason` separates D1 (`user`) from D5 (`process_restarted`); sweep and stalled transitions print one line each; `git.changed_paths` unions the porcelain paths.

## Open questions carried (bypass full: recommended option proceeds)

Plan Open Questions 3–6: query-verb writes on transition, one-poll-interval interrupt latency, the `stalled` word clash with herdr's prompt stall, BOM tolerance (backlog).
