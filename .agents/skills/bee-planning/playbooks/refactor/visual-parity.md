# visual-parity

You own pixel-exact equivalence. The baseline is the spec; you do not touch
it. For "make X match Y exactly", styling-system migrations and porting a UI
across frameworks. Equivalence is verified by image diff, not by eye, because
an eye forgives the one-pixel shifts a diff reports.

1. Establish the baseline first, before any migration: a visual regression
   harness that screenshots the current component across its states, plus the
   target when matching two implementations. No baseline, no parity claim.
   It is a blocking prerequisite, not a follow-up, because a baseline taken
   after the migration records the migration.
2. State the anti-shortcut clauses and hold them: no harness modifications, no
   baseline tampering, no component restructuring to make a diff pass. A diff
   forced to zero by moving the goalposts proves nothing. If the baseline
   looks wrong, stop and ask; do not edit it.
3. Migrate one component at a time, because each is an independent artifact.
   Parallelize across worktrees with one owner per component, each dispatched
   as its own cell (`bee dispatch prepare --kind cell`), per
   `bee-principle-separate-before-serializing-shared-state`: giving each owner
   its own files removes the contention instead of queueing it. Shared
   primitives migrate first as a blocking phase, because every component
   built on one inherits its diff.
4. Verify each component against its baseline by image diff on the matching
   surface, driven through the project's `verify-app` skill when it has one,
   otherwise through the real interface itself. A nonzero diff is a fail;
   investigate the pixel delta and do not wave it through, because a
   remaining delta is a defect somebody will see. Rerun the diff per
   component until it is zero.
5. Land each component, or each safe batch, through the bee close and
   worktree merge path: the work is capped, merged by `bee worktree merge`,
   and the `uat` acceptance at `bee close` stays the owner's, because the
   owner is the one who looks at the screen.

Reply: components migrated, the diff result for each, the baseline harness
location, what's left.

Proof line: the image-diff result for each component against its baseline.
Proof floor: the cap refuses a result weaker than this class's floor and names it; the table is the cap's own (`bee cells finish --help`).
