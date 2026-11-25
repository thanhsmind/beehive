# knowledge-upkeep-nov25 — brief

Lane: docs. Class: docs.

## Asked

Run the knowledge upkeep pass (`bee-knowledge-upkeep`) on the bundle.

## Found

`bee upkeep due --kind knowledge` reported a run due: no prior run, and 17
suspect concepts loaded 5 or more times. No concept had a wrong report and no
concept graded orphan. The pass picked the five suspect concepts loaded most:
`areas/advisor-protocol/consult-loop.md`, `areas/project/overview.md`,
`areas/workflow-state/workspace-protocol.md`, `areas/rust-runtime/output-spill.md`,
`areas/workflow-state/outcome-scorecard.md`. One read-only reader per concept
checked each claim against the concept's own sources.

## Will write

Revise in place, inside `docs/knowledge/` only, the claims the sources prove
stale:

- `consult-loop.md` P1: the worker prompt template moved out of
  `swarming-reference.md`; the `Advisor:` line is in
  `packages/bee/prompts/worker-cell.md`.
- `overview.md` Build table: `commands.test` is an array run in order (clippy,
  then cargo test); Linux CI runs clippy then the test suite, Windows CI runs
  the test suite.
- `output-spill.md`: the threshold comes from the merged config layers, not
  `.bee/config.json` alone; `.bee/team.json` cannot set it.
- `workspace-protocol.md`: the corrections its reader returns.

Concepts that held get `bee knowledge confirm`.

## Not in scope

Product code. The read-only worker-guard refuses a quoted `>` in an rg
pattern; that defect is filed as a bee issue report, not fixed here.
