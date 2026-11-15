# autonomous-run

You own the exit condition. Define done, then drive to it without stopping.

1. State the exit condition as a checkable predicate before the first
   iteration: tests green, repro fixed, all N cells capped, pixel-diff zero. A
   vague goal stalls; a predicate lets you stop.
2. Pick the wake mechanism. Work that fits this session keeps driving in it.
   Work that must outlive the session runs under `bee-herding`: `bee herding
   control-loop` wakes one cold iteration per role, and each iteration reads
   the state on disk, never a memory of the last one. Size the wake interval to
   when the result is worth re-checking, because an interval shorter than the
   thing being watched wastes iterations and a longer one idles past the
   answer.
3. Each iteration makes the smallest change the evidence justifies, verifies it
   against the predicate, commits if it advanced, and discards changes that did
   not help. Belt-and-suspenders that "might help" gets reverted, not left to
   ride, because a line no evidence asked for is a hypothesis. Sequence the
   work in verifiable units and verify each one before the next, instead of
   batching checks at the end, so a failure points at one unit.
4. Mid-run discoveries are yours. Address broken skills, related bugs, flaky
   verifiers, review noise, tooling failures, orphaned follow-ups and fixable
   drift yourself, routed through `bee orient` like any task. Put out-of-band
   fixes in their own cell so the main change stays reviewable. Do not park
   reversible work for the owner and do not stop to ask. Surface only
   irreversible actions, genuine product or preference calls no experiment can
   settle, or a real dead end. Keep the predicate as the main drive and return
   to it after each side fix, because side fixes are how a run loses its way.
5. Checkpoint every iteration with a row in the feature's decision trail
   (`bee-show-me-your-work`) under `docs/history/<feature>/`: what changed and whether the predicate moved. A
   run with no trail cannot be audited or resumed.
6. Stop when the predicate is met. A plateau is not a stop, so keep going and
   pivot the approach to push past it. Surface a genuine dead end rather than
   spinning, and never relax the predicate to declare victory, because that
   turns the run into a report of effort. When the work lives in a worktree,
   land it on green with `bee worktree merge` from the main checkout without
   asking, as `uat_stop = "close"` already does, and stop at the `uat` door of
   `bee close`, which stays the owner's: uat is never auto-approved at any
   bypass level.

Reply: the exit condition, iterations run, what landed, what was discarded,
final predicate state.

Proof line: the predicate's own output on the final state, verbatim.
