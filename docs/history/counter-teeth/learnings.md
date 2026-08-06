# counter-teeth — learnings (captured 2026-08-06)

Batch A of the prose-rule-audit: four advisory counters gained refusal teeth on
2026-08-04, plus the prerequisite fix that made one of those refusals honest
(cells `ct-0`..`ct-5`, parent decision `c2a7bd4f`). The knowledge sync never ran
at the time; this file and the spec merges dated 2026-08-06 are that repair,
written from the cell traces and verified against the shipped source.

## What shipped

| Cell | Change | Spec home |
|---|---|---|
| ct-0 | Made a worktree-dependent test hermetic — the green base the rest of the slice needed | (no spec surface: test hygiene) |
| ct-1 | Ported the route verb's granted-worktree arm; the argument-shape bail is gone | `areas/worktree-parallelism/control-plane-topology.md` (third un-ported-arm entry) |
| ct-2 | Close refuses uncaptured behavior-changing units absent a capture-deferral decision | `areas/workflow-state/gates.md` — behavior already documented; this feature is now cited as its evidence |
| ct-3 | The capture queue escalates from offer to blocker at ten pending stubs or seven days | `areas/workflow-state/capture-queue-and-the-blocker-threshold.md` B50/R101 (new concept) |
| ct-4 | Ceiling-tier assignment refuses past a 40% share, `--reason` overrides onto the trace | `areas/workflow-state/cells-authoring-and-revision.md` B51/R102 |
| ct-5 | Route-less claims warn once per session, then refuse | `areas/workflow-state/sessions-lanes-and-identity.md` B52/R103 |

Evidence: all six capped green. Commits on the traces: `f6398f8e` (ct-1),
`bf7f022f` (ct-2), `a5e564fa` (ct-4), `4a0d1b82`+`95ec0639` (ct-5); ct-0 names
`5e82b32e` in its outcome text and ct-3 records no sha.

## A correction this repair had to make

`sessions-lanes-and-identity.md` R80 still said a route-less claim "warns once —
a safety net, never a refusal." That was true before ct-5 and false after it, and
it had stood wrong in the bundle for two days. The sentence is replaced, not
annotated: a contradicted line in a spec is worse than a missing one, because a
reader has no reason to distrust it.

## What generalised

Two patterns cleared the promotion bars:

- [Arm a refusal only after its own remedy is proven to work](../../knowledge/patterns/20260806-arm-a-refusal-only-after-its-own-remedy-is-proven-to-work.md)
  — D5's whole reason for existing. The deny's remedy verb was broken under
  worktree grants, so shipping the deny first would have handed every refused
  caller a key that did not turn.
- [Scope a gating counter to the actor who can clear the fault](../../knowledge/patterns/20260806-scope-a-gating-counter-to-the-actor-who-can-clear-it.md)
  — D4's refinement (`64ad772d`), found during execution: a per-feature counter
  would have refused every worker in a swarm but the first, for a fault none of
  them could fix from inside a dispatched unit.

## What did not generalise

- **No deviations and no friction on any of the six traces**; every cell landed
  red-first per D6. There was no pitfall to harvest from the execution itself.
- **ct-0 was ordinary test hygiene** — a test resolving a root from the process
  working directory rather than the injected one. Real, fixed, and not worth a
  durable record beyond the commit.
- **The orchestrator ran ct-0 inline** because subagent capacity was exhausted,
  and the standard-lane review wave could not dispatch for the same reason
  (`91822ad7`). Both were recorded as named deviations at the time. That is the
  deviation discipline working, not a lesson about the work.

## Debt this repair leaves behind

- Both of ct-3's thresholds and ct-4's share ceiling are build constants;
  surfacing them as configuration was deliberately scoped out and remains open.
- counter-teeth ran without a route record of its own (deviation `3baa41f6`) —
  the same defect ct-1 fixed. The fix shipped in source; the installed binary of
  the day still carried the old arm, which is why `hook-teeth` inherited the same
  deviation hours later (`399d72e1`).
