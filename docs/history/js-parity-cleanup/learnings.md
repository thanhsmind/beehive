# js-parity-cleanup — learnings (captured 2026-08-06)

Nine cells landed 2026-08-04 removing the last structural traces of the retired
Node runtime from the Rust port. Six were pure cleanup (`jp-1`, `jp-2`, `jp-3`,
`jp-6`, `jp-7`, `jp-8`) and carried no behavior; three changed behavior and are
the ones that owed a knowledge sync. That sync never ran; this file and the spec
merges dated 2026-08-06 are the repair, written from the cell traces and verified
against the shipped source.

## What shipped (the behavior-changing three)

| Cell | Change | Spec home |
|---|---|---|
| jp-4 | The stored approvals map merges over the defaults only when it is an object; every other shape takes the defaults whole | `areas/workflow-state/gates.md` B53/R104 |
| jp-5 | Seven duplicated length helpers plus three inline copies folded into one module; display truncation counts characters | `areas/rust-runtime/text-measurement-and-the-two-counting-units.md` R8/R9/R10 (new concept) |
| jp-9 | The question-heading guard restored to the platform validator's own counting unit; the surviving exception's rationale corrected | `areas/hook-runtime/write-guard-request-shapes.md` B28/R27 + a new Open Gap |

Evidence: all three capped green — 999, 1004 and 1006 tests passing respectively.
None of the three recorded a commit sha on its trace, and all three carry an
empty `files` array with the real file list under `trace.files_changed` (18 files
for jp-5 alone).

## What generalised

One pattern cleared the bars:
[A sweeping consolidation eats the call site that was different on purpose](../../knowledge/patterns/20260806-a-sweeping-consolidation-eats-the-call-site-that-was-different-on-purpose.md).
jp-5 converted every length helper to the new default — correctly — including
the one guard whose measure is dictated by an external validator, which no
consolidation can distinguish from drift. The suite stayed green throughout,
because the two counting units agree on every input the tests used.

## What the judge pass bought

Both of jp-9's fixes came from an independent judge pass over jp-4 and jp-5
(`bc2e2d44`), on work everyone involved correctly called a pure refactor. It
found the swept guard and, separately, that the surviving exception's comment
named the wrong artefact: it claimed to keep a manifest's path list reproducible
while that file's own test asserts the opposite. A refactor with a green suite is
exactly the shape of change a review is assumed not to need.

## What did not generalise

- **No deviations and no friction on any of the three traces.** The recorded
  deviations for this feature are process-level: it ran without a route record
  (`bda043af`) because the route verb was broken under worktree grants — the same
  defect `counter-teeth` ct-1 fixed hours later — and it closed through a worktree
  merge verify because the close verb was unported for lane-carrying repositories
  (`0248c1fd`).
- **jp-3's number-equality addendum** (`3218bf25`) was fixed narrowly at two call
  sites the same day. Real, recorded, and too specific to promote.
- **jp-7's comment sweep was incomplete** by roughly forty occurrences
  (`a146590b`), with follow-up cells recommended by directory. That is open work,
  not a lesson.

## Debt this repair leaves behind

- An over-long question heading containing any non-ASCII character reaches a
  delegation branch with nothing behind it — recorded as an Open Gap on
  `write-guard-request-shapes.md`. It is safe (nothing is truncated blindly) but
  it is not a decision.
- The split-brain approvals field and one surviving code-unit length call in
  `feedback.rs` were filed to the backlog by the same judge pass (`bc2e2d44`).
- D6 scoped the live delegation signals out of this feature deliberately; they
  still signal *delegate to Node* at runtime with no Node behind them.
