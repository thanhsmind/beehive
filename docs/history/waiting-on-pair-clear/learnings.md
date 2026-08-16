# Learnings — waiting-on-pair-clear (2026-08-16)

Thin harvest: 1 capped cell (wpc-1), tiny bugfix lane, no deviations.

- A setter that writes a pair owes a clear that clears the pair:
  `waiting-on set` wrote `waiting_on` + `run_state: "awaiting-approval"`,
  but the clear nulled only the mark — `run_state` kept showing a stale
  approval badge after the wait ended. Fix: pair-clear, with the
  `run_state` reset guarded on `== "awaiting-approval"` so a foreign
  value written by another path survives.
- The regression test asserts both halves: pair-clear happens, and a
  foreign `run_state` value is left untouched.

Nothing promoted: the rule was synced into
docs/knowledge/areas/workflow-state/workflow-records-and-projections.md
(lines ~473-476) during the fix itself; promote proposal reviewed,
nothing new to merge.
