# Anti-Pattern Principles — Context

Asked: make anti-patterns 2 and 3 of the vhlam coding-agent-anti-pattern
article routed reminders, the way bee principles already work (user chose
option A, 2026-11-04).

## Feature Boundary

In scope: two `bee-principle-*` skills, their depth sections in
`expertise/planning.md`, their rows in the Principle homes index, and the
regenerated copies.

Out of scope: a principle for anti-pattern 1 (covered, see D1); any change
to the doctrine text that agent-anti-patterns D1 and D2 already wrote.

## Locked Decisions

- **D1 (a562288f).** Two craft principles route through `bee orient` to the
  classes `feature`, `refactor`, `perf`:
  `bee-principle-lock-the-contract` — a plan locks the contract a cell must
  keep and leaves the implementation to the worker (agent-anti-patterns D1);
  `bee-principle-split-on-a-real-dependency` — a phase, slice or
  expand-migrate-contract needs a real dependency, and the removal test is
  asked of each (agent-anti-patterns D2). Depth lives in
  `expertise/planning.md` under one heading each. Anti-pattern 1 gets no
  principle.

## Existing Code Context

- Template: commit d9eabd076 added two principles the same way.
- Index: `docs/knowledge/areas/doctrine-layer/router-triage-and-the-agents-md-duplication-boundary.md`
  § Principle homes.
- Parity proof: `packages/bee-rs/crates/bee/tests/principle_index_parity.rs`.

## Outstanding Questions

None.
