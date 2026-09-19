# stage-gate-phase-parity — learnings

**Date:** 2026-09-20
**Feature:** stage-gate-phase-parity (1 cell, `sgpp-1`, commit `5bcebcba2`)
**Lane:** tiny · bugfix · 1 product file

## What the work was

On the Pi runtime bee narrows the model's active tool list once per turn. The
policy behind that narrowing kept its own list of four phases that may write.
The write guard is the authority on when a write is allowed, and the copy had
drifted from it. The visible cost: in the tech-debt phase — the one phase whose
purpose is changing code — the model came back with no `edit` and no `write`.

The fix deletes the list and asks the guard.

## The finding that mattered

The first version of that fix was wrong, and everything around it agreed it was
right. The rule was written after reading one predicate, `is_gated_phase`, and
inferring the shape of the rest. Its unit tests passed. A live probe of the
shipped hook agreed with them. The rationale went into a plan, a decision record
and a doc comment.

Four lines above that predicate sat `is_terminal_phase`; one file over sat
`is_known_phase`. The guard sorts phases into four groups, not two. The shipped
rule handed write tools to the group where the guard refuses every write.

Nothing in the run could have caught it:

- The table test derived its expected value from the same predicate the
  implementation called, so a wrong rule and a wrong expectation agreed.
- The live probe showed what the hook answered, never what the guard would do
  with that answer.

An independent reviewer, dispatched read-only, found it by opening the sibling
predicates. Promoted as
`docs/knowledge/patterns/20260919-the-predicate-you-found-first-is-not-the-rule-read-its-siblings-before-you-derive-from-it.md`.

## The guard that nearly got acked away

`JUDGE_OBLIGATION` refused the cell because it touches `hooks/`: a guard and its
tests written by one author are one model, so a green suite proves only
self-agreement. The two exits were "raise the lane" or "record an ack". The ack
was taken — but the independent read it protects was run anyway, as a dispatched
reviewer, and it returned two P2 findings.

Had the ack been treated as the exit rather than the paperwork, this feature
ships a rule whose own decision record contradicts code four lines away, with
the wrong branch being the silent one.

The guard did its job by making the skip a named act. What made the difference
was reading the reason it gives rather than the flag it offers.

## Smaller notes

- **A proof command that was never run is not a proof command.** The cell's
  approved verify named `cargo test --lib`; this crate ships only a bin target,
  so the approved proof could not execute and the cap refused. Running the
  verify once at plan time, before it enters the packet, costs one command.
- **Red-first survives only if the refactor lands separately.** The extraction
  of `allowed_tools_for` and the rule change went in one edit, so the first test
  run would have been green on arrival; the old rule had to be pasted back to
  earn a real red. One edit per intent.
- **The bundle's own link style does not satisfy its own orphan check.**
  `links_to` counts `required_context`, a literal path in the body and a
  markdown path link — never `[[concept-id]]`, which is the form the capturing
  skill teaches and the patterns use throughout. A new concept linked the house
  way still reads as an orphan. Filed as friction; the "207 of 351 unlinked"
  figure in `check.rs` is probably this, not 207 real orphans.

## What shipped, stated honestly

The source is fixed and proven. The vendored binary the checkpoints execute
still carries the old rule until the next release build, so no Pi session sees
this change yet.
