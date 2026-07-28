# Foundation Fixes — Context

**Feature slug:** foundation-fixes
**Date:** 2026-07-28
**Exploring session:** complete (root-cause hunts with file:line evidence; bypass TOTAL)
**Scope:** Standard
**Domain types:** RUN (state machine, CI suite)

## Feature Boundary

Two foundation repairs, one slice, dispatched as a parallel wave: (A) stop the
state-clobber — workflows are never closed, so the projection's idle-bootstrap
picker resurrects zombie-active prior features on every SubagentStop; (B) stop
the Windows CI timeout — split the git-heavy `test_worktree_store.mjs` so no
suite approaches the 600s ceiling. Verification lands as ONE trailing test cell
(slice-tail batching — the model this feature also demonstrates).

## Locked Decisions

| ID | Decision | Rationale |
|----|----------|-----------|
| D1 | `startFeature` closes the outgoing feature's workflow record (`status: 'closed'`) when starting a new one; the enum value finally gets its writer. | Hunt evidence: `STATUS_VALUES` includes `'closed'` (`workflow-store.mjs:72`) but no production path ever sets it — every past feature stays `active` forever. |
| D2 | Defense in depth: `pickNewestActiveWorkflow` (`state-projection.mjs:149-159`) additionally excludes `phase: 'compounding-complete'` records — a zombie that predates D1's writer can never be chosen. | The picker firing via `bee-state-sync` on SubagentStop is the exact drift shape (3 incidents, all during/after worker stops, all landing on a prior closed feature at compounding). |
| D3 | `test_worktree_store.mjs` (13 tests, ~450 lines) splits into two auto-discovered suites at the topology/merge boundary (~:200): store/topology tests stay, merge-path tests move to a sibling suite file. No test deleted, no timeout raised. | Windows runs git-heavy suites 2–4x slower (`windows.yml:136-138`, prior bump 300→600s already spent); halving each suite's runtime fixes the ceiling on all platforms instead of masking it. |
| D4 | Both fix cells defer regen via `wave-barrier` ack and dispatch as a parallel wave; one trailing test cell covers the slice's net behavior. | Dogfoods parallel-default D1/D2 and slice-tail batching for the user's verification. |

## Existing Code Context

- `packages/bee/lib/state.mjs:2827-3011` — `startFeature` (creates new workflow :3001-3008, never closes prior).
- `packages/bee/lib/state-projection.mjs:210-277` — `rebuildStateProjection`; idle-bootstrap branch :260-277; picker :149-159.
- `.bee/bin/hooks/bee-state-sync.mjs:124-140` — SubagentStop/Stop writer (`.codex/hooks.json:87-103,127-135`).
- `packages/bee/lib/workflow-store.mjs:72` — `STATUS_VALUES` with orphan `'closed'`.
- `packages/bee/tests/test_worktree_store.mjs` — 13 async tests, real git worktrees; `.github/workflows/windows.yml:136-145` — timeout history + exclusions.

## Outstanding Questions

### Deferred To Planning
- [ ] Which suite owns workflow-store/state-projection regression tests today (fx-3 locates; extend, don't fork).

## Handoff Note

Decision IDs stable; planning implements exactly.
