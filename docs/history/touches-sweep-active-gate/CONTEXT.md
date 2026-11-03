# Touches Sweep Active Gate — Context

**Feature slug:** touches-sweep-active-gate
**Date:** 2026-09-20
**Shaping session:** complete
**Scope:** Quick
**Domain types:** RUN

## Feature Boundary

The log-time touches-sweep stops queueing a citation stub when the decision
being cited is still active. It fires only when that decision has left the
active set. Nothing else changes: the retirement sweep on the other relation,
the existing file exclusions, and the impact door all keep their current
behaviour.

## Why now

The sweep queued 64 stubs that were drained by hand on 2026-09-20. Measured
against the active set at drain time:

- **37** cited a decision that was **still active** — a `touches:` relation
  relates without retiring, so those citations were never stale
- **27** were genuinely stale, and every one traced to a single retirement
  event (`0d11a415` retiring `9f5c6d17`), which the retirement sweep already
  covers on its own

So the touches half produced 37 false positives and no unique true positive.

It also feeds itself. Settling a batch means logging a decision, and that
decision needs a relation; pointing it at the same id — the honest target,
since the batch is about that decision — re-sweeps every document citing it.
Round one settled 64 and queued 7. Round two settled those 7 and queued 5.
Each round shrinks only because fewer documents cite the newer decision, not
because the loop terminates by design.

## Locked Decisions

Decision log: `00595ba4`.

| ID | Decision | Rationale (only if it changes implementation) |
|----|----------|-----------------------------------------------|
| D1 | The touches-sweep enqueues a stub ONLY when the cited decision is absent from the active set. A `touches:` whose target is still active queues nothing. | `00595ba4`. This is the whole fix: it removes the false-positive class and ends the self-feeding loop with one predicate, because a settling decision always cites a live decision. |
| D2 | The sweep that runs on the retiring relation is UNCHANGED. It is the half that proved itself — `doc-impact-synthesis` D1 records it finding 3 stale citations the same day. | Chesterton's fence: the sweep exists for a reason and that reason is sound on the retiring path. Only the analogy to `touches` was wrong. |
| D3 | The existing file exclusions stay exactly as they are: the generated decisions index, and the logging feature's own live history. | They solve a different problem and no evidence here touches them. |
| D4 | The impact door, the close-time sweep, and already-queued stubs are untouched. This changes what gets ENQUEUED at log time, nothing about what happens to a stub once it exists. | Keeps the change to one predicate at one call site. |

### Agent's Discretion

- Where the active-set membership is read, given the sweep already runs after
  the append and an `active_decisions(root, false)` call already exists a few
  lines below it.

## Terms

| Term | Meaning in this feature |
|------|-------------------------|
| Active set | The non-retired, non-redacted decisions, as every other reader of that set sees them. |
<!-- bee:not-a-deferral: The flagged word in this row is "later", inside a DEFINITION of what makes a citation stale — it describes decisions that arrive after the cited one, which is the whole distinction this feature turns on. It promises no future work. -->
| Stale citation | A document citing a decision that has left the active set. A document citing a live decision is not stale, however many later decisions relate to it. |
<!-- /bee:not-a-deferral -->

## Existing Code Context

### Reusable Assets

- `packages/bee-rs/crates/bee/src/verbs/decisions/verbs_read.rs:818-843` — the
  touches-sweep itself: `if let Some(ids) = &touches`, walking
  `sweep_decision_citations` per id and calling `add_capture_stub` per hit.
  The gate belongs here.
- `verbs_read.rs:845` — `let active = active_decisions(root, false)?;` already
  runs a few lines below the sweep, for the conflict-candidate scan. The same
  read answers D1.
- `verbs_read.rs:466-477` — `touches_sweep_excluded`, the existing file-based
  exclusions D3 preserves.

### Integration Points

- `do_supersede`'s own sweep — the half D2 leaves alone.
- The impact door at close, which reads stubs rather than creating them (D4).

## Canonical References

- `docs/history/doc-impact-synthesis/CONTEXT.md:37` — D1, the decision that
  created this sweep and recorded why: "Extends the proven supersede
  citation-sweep (found 3 stale citations same-day) to `touches` + close."

<!-- bee:not-a-deferral: These sections are this file's own record of what was resolved. "Deferred To Planning" is a fixed heading in the CONTEXT template and its one item is ANSWERED and checked off below, with a file:line behind it; "Deferred Ideas" is empty; the Handoff Note is template boilerplate naming what a planning agent reads. Nothing here promises later action. -->
## Outstanding Questions

### Resolve Before Planning

None.

### Deferred To Planning

Answered by reading the function during planning, before the gate.

- [x] **Is the existing `active_decisions` read reachable before the sweep?**
  **Yes, by hoisting.** The sweep and that read both sit after the
  `append_jsonl` at `verbs_read.rs:802`, and the read was below the sweep only
  by accident of ordering. Moving it above the sweep keeps the property its own
  comment at `:805-806` names — it reads the same active-set shape every other
  `active_decisions()` caller sees — so the sweep reuses it and no second read
  is added. The shipped cell prohibits a second read for exactly this reason.

## Deferred Ideas

None.

## Handoff Note

CONTEXT.md is the source of truth. Decision IDs are stable. Planning reads locked
decisions, code context, canonical references, and deferred-to-planning questions.
<!-- /bee:not-a-deferral -->
