---
type: bee.area
title: "Workflow State — the capture queue, its pending membership, and the threshold that turns an offer into a blocker"
description: "What sits in the capture queue and what takes an entry out of it, why a pending queue is an offer the agent makes rather than a demand, and the two thresholds — a count and an age — past which the orientation surface stops offering and starts blocking."
timestamp: 2026-08-06
bee:
  id: workflow-state-capture-queue-and-the-blocker-threshold
  lifecycle: active
  areas: [workflow-state]
  required_context: [areas/workflow-state/overview.md]
  decisions: ["c8e25271 (a settlement is stubbed the moment it settles; the flush is offered, never forced)", c2a7bd4f item 2 (prose-rule-audit batch A — the capture-queue offer gains blocker teeth past a count and an age threshold), "counter-teeth D2 (thresholds are ten pending stubs or an oldest stub older than seven days; constants in code for this batch, config keys deferred)", counter-teeth D6 (a test proving the counter computes correctly lands before the flip to refusal)]
  sources: ["counter-teeth cell ct-3 (trace .bee/cells/ct-3.json, 2026-08-04 — escalation reuses the existing pending membership; status_full 49 passed, 0 failed)", docs/history/counter-teeth/CONTEXT.md, "packages/bee-rs/crates/bee/src/state.rs (DEFAULT_CAPTURE_QUEUE_THRESHOLD + capture_queue_threshold — the two constants, moved here from the retired status_full/orient.rs:134-181 during the rust-port split)", "packages/bee-rs/crates/bee/src/verbs/drivers/close.rs (capture_queue_door_detail — the escalation predicate; capture_queue_pending — pending membership, stub rows minus flush rows; moved here from the retired status_full/records.rs:215)"]
  authoritative_for: "workflow-state: the capture queue's pending membership and its escalation from offer to blocker"
---

# Workflow State — the capture queue, its pending membership, and the threshold that turns an offer into a blocker

A settlement is recorded the moment it settles: a one-line stub goes into the
capture queue that same turn, and the full merge into the area specs happens
later, in one pass. That deferral is deliberate — it keeps a settlement from
costing a whole documentation session in the middle of other work. What the
deferral must not become is a queue that grows forever while every session
politely offers to drain it and no session ever does.

## Behaviors & Operations

**B50 — A pending capture queue is an offer until it is old or large, and then
it is a blocker (counter-teeth D2, 2026-08-04).** Trigger: the orientation pass
a session runs when it routes, starts, or resumes work. What happens: the queue's
*pending* entries are the stubs that have no matching flush recorded against
them — the same membership every other reader of the queue uses, computed once
and never re-derived by the orientation surface itself. While any entry is
pending, orientation says so and offers to drain it, naming the flush as
something the human may choose. Past either of two thresholds, the same fact
stops being narration and joins the blockers: **ten or more pending entries**,
**or** an oldest pending entry more than **seven days** old. The two are
independent — either one alone escalates, and neither cancels the other. What
each actor observes: a queue drained in the ordinary rhythm is never anything
but an offer; a queue that has been quietly deferred past a week, or past ten
settlements, is reported as work in the way of everything else, and the agent
routes to draining it. Both thresholds are constants in the build for now:
counter-teeth scoped config keys out deliberately, and that remains open work
rather than an oversight.

## Business Rules

- R101 — The capture queue's pending set is stubs minus flushes, and it escalates
  from an offer to a blocker at ten or more pending entries, or an oldest pending
  entry older than seven days — whichever comes first; the two thresholds are
  independent, and both are build constants, not configuration (counter-teeth D2,
  cell ct-3, 2026-08-04).

## Edge Cases Settled

- An empty queue produces neither an offer nor a blocker — silence, not a line
  saying nothing is pending.
- A queue that crosses the count threshold and the age threshold at once
  escalates once, not twice: the blocker states the pending count, and the age is
  what made it a blocker only when the count alone would not have.
- The thresholds decide how the queue is *reported*; nothing about them drains,
  reorders, or expires a stub. A stub leaves the pending set only by being
  flushed into a spec.

## Pointers (implementation)

- Escalation predicate and both constants (B50/R101):
  `CAPTURE_QUEUE_BLOCKER_MIN_PENDING` (10) and
  `CAPTURE_QUEUE_BLOCKER_MAX_AGE_DAYS` (7.0) with
  `capture_queue_blocker_line` and `capture_queue_oldest_pending_at_ms` in
  `packages/bee-rs/crates/bee/src/verbs/status_full/orient.rs:134-181` — the test
  is `over_count || over_age`. Pending membership comes from
  `capture_queue_summary` (`status_full/records.rs:215`), which the orientation
  surface consumes rather than recomputing; the non-escalated offer line is
  rendered at `status_full/render.rs:246-252`. Red-first per counter-teeth D6.
  Evidence: trace `.bee/cells/ct-3.json` (status_full 49 passed, 0 failed,
  2026-08-04).
