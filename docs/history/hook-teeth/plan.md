# hook-teeth — plan

Lane: standard (flags: public-contracts, proof-weakening; ~10 product
files). Route record absent by deviation 399d72e1. Decisions D1-D7 in
CONTEXT.md, all cited below.

## Shape — six cells, three waves (file-overlap forced)

Wave 1 (disjoint, parallel): bh-1, bh-2, bh-4, bh-5.
Wave 2: bh-3 (shares cells/handlers_write.rs + cells/tests.rs with bh-2).
Wave 3: bh-6 (shares cells/tests.rs with bh-2/bh-3; handlers_close.rs).

- **bh-1 — plan.md freeze (D1).** write_guard: deny Edit/Write to
  docs/history/<feature>/plan.md when that feature's approved_gates.shape
  is true (feature resolved from the path, lane-aware). Files:
  src/hooks/write_guard/checks.rs (+ write_guard/tests.rs).
- **bh-2 — red-base claim refusal (D2).** cells claim reads
  .bee/logs/test-results.json; red → refuse naming the failing command,
  --fix-first <reason> escapes onto the trace; missing file warns only.
  Files: src/verbs/cells/handlers_write.rs, src/verbs/cells/tests.rs.
- **bh-3 — gated cells add refusal (D3).** cells add refuses in a gated
  phase without approved execution; docs-lane and post-gate flows
  untouched. Files: same as bh-2 (wave 2).
- **bh-4 — adopt fresh-boundary refusal (D4).** handoff adopt refuses
  when the calling session's start source is resume/compact; persist the
  source on the session record in session-init if absent. Files:
  src/verbs/workflow_store/handoff.rs, src/hooks/session_init.rs (or
  where session records are written), + their tests.
- **bh-5 — re-lane transition validation (D5).** route --set over an
  existing record: downward only, once per feature, high-risk never
  demotes, hard-gate flag blocks demotion, promotion free. Files:
  src/verbs/state_group/workflows.rs, src/verbs/state_group/tests.rs.
- **bh-6 — commit trailer check at finish (D6).** finish verifies a
  commit with the cell id trailer on the feature branch (granted
  worktree else main) when files_changed non-empty; --commit-pending
  <reason> escapes onto the trace. Files:
  src/verbs/cells/handlers_close.rs or finish_support.rs, cells/tests.rs
  (wave 3).

Every flip is red-first per D7: the counter/condition test lands before
the refusal. Existing tests asserting the softer behavior are replaced by
the stronger assertion in the same cell, named in the done-report
(proof-weakening acknowledged).

## SMALLER PATH check

Cheaper shape honoring D1-D7? Dropping bh-4's session-source persistence
would make D4 unenforceable (no counter to key on) — not cheaper, just
hollow. Merging bh-2+bh-3 couples two public-contract flips in one
revert unit for one shared file — the wave split already pays the
serialization cost without coupling the commits. PASS.

## Verify

commands.test at every cap via cells finish; wave-final full suite in
the worktree; merge re-runs verify as the semantic-conflict gate.

## Later slices

None — one slice, three waves.
