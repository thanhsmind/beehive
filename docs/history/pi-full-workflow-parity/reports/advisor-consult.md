# Advisor consult: Pi full workflow parity

## Scope

Five high-risk perspectives checked `CONTEXT.md`, `approach.md`, and `plan.md` before Gate 2.

The runtime executed `hat-value` and `hat-user-impact` through herding. It prepared `hat-facts-gaps`, `hat-risks`, and `hat-alternatives`, but this Pi tool set has no Agent tool. The leader ran those three checks directly. This is a named environment deviation, not a reduced review scope.

## Structure

BLOCKERS: none.

WARNINGS: `pfp-1` owns many files, but they form one state transition and share dispatcher, registry, and behavior tests. Splitting them would add file overlap and intermediate public-command drift.

Every locked decision lands in a cell. Both cells have bounded files, directive actions, required contracts, and executable proof. The dependency graph is acyclic. `pfp-2` depends on the public command from `pfp-1`.

## Risks

One blocker was found and fixed before this report. Multi-record close could close early records before finding a later planned-next record. The plan now requires a complete preflight before any close mutation. One unsafe target refuses the whole operation.

The remaining high-risk controls are:

- Dismiss accepts pause only.
- Planned-next remains on guarded adoption.
- Mailbox records remain for audit history.
- Projection rebuild excludes closed workflows.
- The public registry remains hand-maintained and receives its contract tests.

## Value and user impact

The herding advisors found no excess scope. The two-cell shape is the smallest complete path.

The CLI user gains one narrow action for a pause record. Pi then runs the same installed command. After close, `bee orient` no longer reports a stale handoff from a closed workflow.

## Alternatives

- Projection filtering alone hides the blocker but leaves contradictory mailbox state.
- Projection deletion alone is not durable because rebuild recreates it.
- Reusing adopt would mix pause dismissal with claim transfer.
- Clearing all handoffs at close would lose planned-next authority.

## Verdict

Proceed. The plan has no unresolved blocker or critical cell flag.

Herding evidence:

- `/home/thanhsmind/Projects/goglbe/beehive/.bee/mailbox/job-1789307669788-889374-1/report-1.md`
- `/home/thanhsmind/Projects/goglbe/beehive/.bee/mailbox/job-1789307669788-889376-1/report-1.md`
