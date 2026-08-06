# hook-teeth — learnings (captured 2026-08-06)

Six prose rules gained mechanical enforcement on 2026-08-04 (cells `bh-1`..`bh-6`, parent decision
`e1e41ec8`). The knowledge sync for those cells was never run at the time; this file and the area
spec merges dated 2026-08-06 are that repair, written from the cell traces and the shipped source,
not from `plan.md`.

## What shipped

| Cell | Door | Spec home |
|---|---|---|
| bh-1 | The approved plan document freezes; the write guard resolves the feature from the path, lane record first | `areas/hook-runtime/governed-paths-and-the-intake-gate.md` B27/R26 |
| bh-2 | Claiming onto a red base refuses; one escape, a fix-first reason on the trace | `areas/workflow-state/claims-and-ownership.md` B45/R96 |
| bh-3 | Authoring units into an ungated feature refuses the whole batch; docs lane exempt | `areas/workflow-state/cells-authoring-and-revision.md` B46/R98 |
| bh-4 | Sessions persist their start manner; adoption refuses on resume and compaction | `areas/workflow-state/handoff.md` B47/R97 |
| bh-5 | Re-lane transitions validated at record time; each refusal names its rule | `areas/workflow-state/sessions-lanes-and-identity.md` B48/R99 |
| bh-6 | Completion verifies the unit's own commit trailer over the feature branch | `areas/workflow-state/cells-completion-judge-and-archive.md` B49/R100 |

Evidence: all six capped green; `bh-6`'s cap ran the full suite at 1058 passed, 0 failed. Commits
recorded on the traces: `7ef3a1f7` (bh-2), `95fe412d` (bh-5), `08e95a4e` (bh-6); `bh-1`, `bh-3` and
`bh-4` recorded no sha on their traces.

## What generalised

One pattern cleared the promotion bars and is now in the bundle:
[`A new enforcement door treats absent evidence as silence, not as violation`](../../knowledge/patterns/20260806-a-new-enforcement-door-treats-absent-evidence-as-silence.md).
Five of the six doors independently chose the same three-state answer — refuse on violation, warn on
unknown, exempt what cannot carry the evidence — which is what makes it a rule rather than a
coincidence.

## What did not generalise

- **No deviations, no friction on any of the six traces.** The feature ran clean; there is no
  pitfall to harvest from it, and none was invented.
- **`bh-2`'s one red attempt was environmental**, not a defect: a load-flaky concurrency run, green
  on re-run, filed as a P3 backlog row at the time. It is recorded here so the trace's
  `verdict: tests-red` is not later mistaken for a real regression.
- **Wave sequencing was file-overlap bookkeeping**, not a lesson: `bh-3` and `bh-6` waited on
  `bh-2` because they share one test file. The `SMALLER PATH` check in `plan.md` already refused to
  merge them into one unit, to keep two public-contract flips separately revertible.

## Debt this repair leaves behind

- `hook-teeth` never produced a promote proposal file; the area merges above were written by hand
  from traces and source. The proposal path (`bee knowledge promote --work <id>`) needs a
  `bee.work-item` concept, and this feature has none.
- The feature ran without a route record, by recorded deviation `399d72e1` — the installed binary of
  the day still carried the route granted-arm defect.
