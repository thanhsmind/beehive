# Full Suite Once Per Feature — Context

**Feature slug:** full-suite-once
**Date:** 2026-11-07
**Scope:** Standard
**Domain types:** RUN | CALL

## What was asked

The owner rule of 2026-11-07 (decision dc19f610, touches test-doctrine
58ec9664): the full declared suite (`commands.test`) runs ONCE per feature,
at the feature's end, before the merge to main. A cell proves itself with its
related tests only. A leader never re-runs a worker's recorded proof and
never runs the full suite per cell. `scripts/release.sh` and CI keep their
own full runs.

## Why the rule was not followed (measured)

1. No rule wrote it down. test-doctrine D1 gives the agent the test scope
   with no upper bound.
2. The session preamble prints the full `commands.test` under "Standard
   commands". Planners copy it as cell verify: 31 of 81 recent cargo cells
   carry the bare full suite.
3. `packages/bee/LEADER.md:324` says "run `commands.test` over what changed"
   before ending substantial work; agents read it as the full suite.
4. `docs/knowledge/areas/verify-pipeline/suite-topology-and-discovery.md`
   says `commands.test` runs fresh at cells finish, close and worktree merge
   (false since test-doctrine D7), and says the full run is scheduled once a
   day (false: `.github/workflows/ci.yml` runs on every push).
5. Parallel full runs on one machine take 3-11 min instead of about 20 s, and
   some herding tests flake under that load.

## Locked Decisions

| ID | Decision | Store id |
|----|----------|----------|
| D1 | The owner rule above. | dc19f610 |
| D2 | Cell plan check: `bee cells add` and `bee cells update` refuse a cell whose verify runs the bare full suite (a command segment equal to `commands.test`, or a bare `bee test`, with no test filter), with a FIX line that names a scoped form. A non-empty `full_suite_reason` on the cell (a cross-cutting change) allows it, and the reason stays on the cell. | 82a5bff7 |
| D3 | The feature-end door is `bee worktree merge`. `bee test` stamps its record with the checkout HEAD. Merge reads the worktree's fresh green `bee test` record for a feature whose diff touches code, and refuses without it, naming `bee test` in the worktree. `--full-suite-waiver "<reason>"` passes and writes a `merge` audit row. A docs-only diff and a no-test repo pass. The door reads proof and spawns nothing, so test-doctrine D7 holds. | feddf5c3 |

## Why merge, not close

Merge is the one door every feature passes before main. `bee close` runs
after the merge, on main, so a proof it records would come too late to keep
red code out of main. Keeping D7 (doors read proof, never spawn the suite)
keeps the door fast and keeps the run under the agent's eye, where a red
result is the next work item.
