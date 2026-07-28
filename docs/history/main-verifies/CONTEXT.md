# Main Verifies — Context

**Feature slug:** main-verifies
**Date:** 2026-07-28
**Exploring session:** complete (two user philosophy decisions logged same day; bypass TOTAL)
**Scope:** Standard
**Domain types:** RUN (state machine guards), ORGANIZE (execution doctrine)

## Feature Boundary

Install the user's verification philosophy: **the delegator checks the work,
at the shippable unit.** Workers implement + commit + report — never run
suites. MAIN produces all proof: red evidence (bugfix repro) pre-dispatch at
authoring, and ONE feature-level verify (impacted over the feature's whole
diff, cache-assisted) when the full picture exists — before scribing. The cap
law's essence survives (no ship without green evidence); the proof's producer
and granularity move. Flags include proof-weakening **by design**: the
per-cell proof requirement is deliberately relocated, and the close-door gate
is what keeps the law honest.

## Locked Decisions

| ID | Decision | Rationale |
|----|----------|-----------|
| D1 | `cells cap` gains a sanctioned **feature-verify-pending** path: capping with `--feature-verify-pending` records `trace.feature_verify: "pending"` and requires NO per-cell verify evidence. The classic evidence path stays intact (other repos, transition, spot use). | Cells are work units, not ship units; their proof event moves to the feature boundary. |
| D2 | New verb `bee state feature-verify record --command <c> --output-file <f> --result green\|red` stamps a feature-verify record (feature, command, output sha, result, at) on the active feature's workflow record. Red records are storable (they document the red) but never satisfy D3. | The proof must be a machine-readable record, same discipline as validation-cache/advisor-ref. |
| D3 | **Close-door gate:** leaving `swarming` (any `state set` phase transition out, and `state scribing-run`) is REFUSED — typed, no bypass level lifts it — while any capped cell of the feature carries `feature_verify: "pending"` and the feature lacks a green feature-verify record newer than the last pending cap. Same door mechanics as `guardTestCellDebt`. | The relocated proof needs a relocated enforcement point, or the law is prose. |
| D4 | Doctrine flip: bee-executing worker loop becomes read → implement → commit → report (`[DONE]` carries diff + commit, no verify run, cap via the pending path); bee-swarming drops routine goal-check re-runs and wave-close impacted runs (smell-triggered spot checks stay, orchestrator judgment) — the ONE feature verify runs at final-slice close, is recorded via D2, then cells' pending markers are satisfied and the close proceeds. Bugfix repro red is MAIN-produced at authoring (et-4 precedent). Test AUTHORING stays batched at slice tail (unchanged); only running consolidates. | The two philosophy decisions, verbatim. |
| D5 | Red feature-verify → fix cells in the same feature (never un-cap), re-verify after; per-cell commits + `git bisect` are the localization tool. | Already the slice-red law, now at feature granularity. |

### Agent's Discretion

- Guard naming/wiring (mirror `guardTestCellDebt` at both doors).
- Record field shape; pending-marker satisfaction mechanics (record timestamp vs marker clear).

## Existing Code Context

- `packages/bee/bee.mjs:2920-2969` — `guardTestCellDebt` (both-doors guard precedent; D3 mirrors it).
- `packages/bee/lib/cells.mjs` — cap evidence requirements (D1 adds the pending branch); `workflow-store.mjs` — record home (route/close precedents).
- `skills/bee-executing/SKILL.md` (grandfathered 10225, zero headroom — body edits net-zero, detail to references) + `references/worker-details.md`; `skills/bee-swarming/SKILL.md` (migrated, ~12B headroom) + `references/swarming-reference.md`; `skills/bee-hive/references/routing-and-contracts.md` lane-table verify columns.
- Cache + impacted runner make the single feature verify cheap (25/25 cached precedent).

## Outstanding Questions

### Deferred To Planning
- [ ] Whether tiny/docs lanes keep their lighter existing paths untouched (recommendation: yes — feature-verify applies to lanes that dispatch workers).

## Handoff Note

Decision IDs stable; planning implements exactly.
