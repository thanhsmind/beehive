# Plan-step hat wave — knowledge-in-the-flow

Date: 2026-11-09. Five seats opened (high-risk). Ceiling 10 minutes.

## Seats

| Seat | Outcome | What came back |
|---|---|---|
| hat-value | done | report, 0 missed decisions, 1 warning |
| hat-risks | dropped | seat model hit the account's monthly spend limit mid-run; findings taken from its run log before it stopped |
| hat-user-impact | dropped | same spend limit; findings taken from its run log |
| hat-facts-gaps | dropped | spawn failed on the same model; no output |
| hat-alternatives | dropped | missed the 10-minute ceiling; findings taken from its run log |

Three seats share one model and all three stopped on the spend limit, so
the wave has low diversity. The leader's own claims pre-flight (15 rows,
each opened at its anchor) stands in for the facts-gaps seat.

## Accepted findings (plan changed)

1. Drain cells wrote control-store stamps before the merge; an abandoned
   branch would hide the debt (risks). Drain cells now write
   `drain-ledger.md` only; the leader stamps after the merge.
2. The pack row must land in the control root's dispatch.jsonl, or the
   report misses features run in a worktree (risks). kitf-1 says so.
3. A delivery.md left from an earlier slice would read as consolidated
   (risks). kitf-4 compares its time with the newest capped cell.
4. `branch_changed_files` can fail; the merge line prints unknown (risks).
5. The pull rate would drop to zero after the skills switch to
   `knowledge pack` (user-impact). kitf-3 counts pack as a pull.
6. The close hint after a merge pointed at a step that had already passed
   (user-impact). kitf-4's not-consolidated line names the follow-up
   writer cell instead.
7. The read set needs a grade per concept and the pack carried none
   (alternatives). kitf-1 annotates grades on the pack verb output.
8. The 8-line truncate runs on the whole list (alternatives). kitf-1
   truncates knowledge lines before appending code lines.
9. Drain sizing: 114 old traces in one cell (value). Drain cells record
   `none` when nothing is clearly still true, instead of deep-diving.
10. The advisor shares a supported-vars arm with gather and reviewer
    (leader, from the read-side gather). kitf-2 splits it out.

## Dismissed

- "D4 anchors are written by instruction only, no verb writes them" (alternatives) — D1 forbids a check; the worker brief is the flow step by design.
- "Features planned before this ships lose their close-time proposal" (user-impact) — accepted risk: their cell commits carry deltas and `bee knowledge consolidate` reads archived cells at any time; recorded in the risk map.
- "Writer cell has no mechanical guarantee before merge" (alternatives) — D1: flow, not gate; the close and merge lines are the detection.
- "kitf-8c verify does not prove its must_have" (alternatives) — superseded by finding 1: the must_have is now the ledger row, which the cell writes and the leader checks at leader-check.
