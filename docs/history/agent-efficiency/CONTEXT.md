# Agent Efficiency — Context

Asked: "vậy hãy làm toàn bộ" (2026-11-04), after the leader named five
friction points seen while building slp-article-gaps, agent-anti-patterns and
anti-pattern-principles.

## Feature Boundary

In scope: the seven items of D1. Out of scope: the 107 orphaned scribing-debt
cells of old features (D2, filed as a backlog item). The "stale backlog row"
the leader reported was a reading mistake: PBI p-05d2a4f4 carries a later
`done` status event; nothing to build.

## Locked Decisions

- **D1 (c4da7108).** One cell each:
  1. `bee feature start` opens a feature in one verb (worktree, lane record,
     route) and `bee feature finish` lands it (worktree merge, scribing
     stamp, close); `bee route --set` accepts no risk flags without a
     placeholder value.
  2. `bee herding run` caps the cell itself when the worker reported done
     with a commit and a green proof line.
  3. After a vendored prompt source changes, `bee dispatch prepare` works
     with no hand rebuild of `.bee/bin/bee`.
  4. A leader goal-check miss is recorded as `bee cells leader-check
     --verdict gap`, which reopens the cell, so the leader check row counts
     real misses.
  5. The Prior rounds digest carries a block or reopen reason in full.
  6. The worker cell prompt drops boilerplate the worker skill already
     carries, without dropping any contract the worker must keep.
  7. The 22 pending capture stubs are flushed and the 20 unapplied promote
     proposals are applied or recorded as skipped.
  None of these verbs approves a gate.
- **D2 (69e23234).** The 107 orphaned scribing-debt cells are outside this
  feature; no rule that drops knowledge debt is built here.

## Evidence

- About 40 bee calls per small feature; retries on `--flags ""`, `--owner`
  phase mismatch, control-plane verbs refused inside a worktree.
- 3 of 3 herding runs ended "worker reported success ... without capping it".
- `dispatch prepare` refused with "prompt skew" after `worker-cell.md`
  changed, until `.bee/bin/bee` was rebuilt and copied by hand.
- 268 leader checks, all ok; the sag-1 miss went through `cells reopen`.
- The Prior rounds digest cut the sag-1 block reason after about 150 chars.

## Outstanding Questions

None.
