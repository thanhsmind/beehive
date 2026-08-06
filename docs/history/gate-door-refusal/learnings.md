# gate-door-refusal — learnings (captured 2026-08-06)

One cell, `gdr-1`, capped 2026-08-04 under decision `20969403`. It changed one
thing: what the high-risk execution-gate refusal *says*. It deliberately changed
nothing about what the gate refuses.

## What shipped

Both refusal arms — the pre-lock peek and the post-lock recheck — returned a bare
internal error that the command surface rendered as *unsupported argument shape*.
A caller with perfectly correct flags was told their flags were wrong. Both arms
now return a stated refusal naming the missing or stale advisor consult, listing
each failed condition separately, and naming the recording flow. Both call one
shared helper, so the two arms cannot drift apart.

Spec homes: `areas/advisor-protocol/triggers.md` B3a and P7 (the refusal and its
anchors); `areas/workflow-state/gates.md` — B9a already described this behavior,
and now carries the evidence that made it true, plus the Open Gap below.

## The finding worth keeping

`advisor-protocol/triggers.md` had asserted since 2026-07-19 that the approval
"refuses with a corrective message naming each failed condition." That was the
intent, and it was false in code for over two weeks — the message named nothing.
The spec was not wrong about the design; it was wrong about the tense. A spec
written from a plan describes what will be true, and nothing re-checks it later.

This is the near-mirror of a pattern promoted the same day from `counter-teeth`
([arm a refusal only after its own remedy is proven to work](../../knowledge/patterns/20260806-arm-a-refusal-only-after-its-own-remedy-is-proven-to-work.md)),
and it is worth reading beside it rather than promoted again: there, a refusal
pointed at a remedy that could not run; here, a refusal pointed at nothing at
all, and a spec asserted otherwise.

## The gap this feature chose not to close

The refusal is now honest and still unsatisfiable from the command line: the
freshness precondition it describes is unported, so the arm refuses every
high-risk execution approval unconditionally, and the verb it names for recording
a consult is declared but not built. The approval comes from the human or waits
on that port.

That was weighed and declined on the record (`20969403`): porting the
precondition and building the recording verb is itself high-risk work, so it
deadlocks against the door it would repair. What shipped was the honesty, never
the unblock — stated now as an Open Gap on `gates.md` rather than left for a
reader to discover from a refusal they cannot satisfy.

## Process notes

- The cell was dispatched directly with its id after an earlier attempt was
  blocked on a worktree-binding mismatch; the registered-worker completion door
  was cleared with a named inline reason recorded on the trace. Visible, not
  silent — which is the discipline working.
- The feature has no `CONTEXT.md` and no `plan.md`; a single-cell small-lane fix
  approved by one decision is the whole record, and that is proportionate.
- Tests green at cap; `state_group` covers the refusal shape, asserting it is a
  stated refusal naming the high-risk cause and not the generic argument-shape
  error.
