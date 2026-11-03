# Standing request: waggledance takes the cockpit-supervisor seat

Status: WAITING on the owner — the waggledance project has not opted into
orchestrator dispatch (enable it from waggledance's settings page). Once
enabled, resend the task below verbatim via `waggledance_dispatch`
(project `waggledance`, preset `claude-sonnet`).

Recorded in beehive as decision `b59e50c8` (touches `8fea3561`);
gate_bypass raised to `full` the same day (2026-08-30) on the owner's word.

## Task text to send

Request from the beehive repo (sent 2026-08-30 on the owner's order): take on the COCKPIT-SUPERVISOR role for the beehive project (/home/thanhsmind/Projects/goglbe/beehive). File this into waggledance's own intake (backlog/spec-drop) as a standing work request — the owner will activate the role themselves; do not start building beyond recording and triaging it.

Desired flow, owner-approved:

1. The human hands a spec to the waggledance supervisor.
2. The supervisor opens a LEAD agent in the target repo (beehive) and passes the spec.
3. The lead evaluates the spec and chooses the working flow itself (beehive runs bee: `bee orient` / `bee route` pick discovery/shaping/planning/swarming; specs enter via the backlog spec-drop convention — a foreign spec arrives as a proposed PBI whose id is the sender's correlation id, with a provenance line `from <repo>@<commit>`, and is not dispatchable until triage locks CONTEXT.md).
4. Merging to main stays a human gesture; the supervisor never merges.

Beehive-side facts already in place:

- bee's own supervisor is a locked OBSERVER (never routes) — the router seat is intentionally left to waggledance, per beehive decision 8fea3561 (the waggledance supervisor's read-only rollup is the one cross-project awareness layer); the move is recorded there as decision b59e50c8.
- beehive gate_bypass is now "full": an unattended lead may auto-approve Gates 1-2; Gate 3 (UAT/acceptance) still stops for the human.
- beehive's local herding cockpit is deliberately NOT bootstrapped; waggledance owns the seat.

Scope pointer: the deferred wd-supervisor items from beehive's slp-advisor-nudge closeout (the waggledance supervisor itself, the widened ask_state digest, the cockpit repository, weekly reports) are the natural backlog for this role — see docs/history/slp-advisor-nudge/CONTEXT.md in the beehive repo.
