# How to resolve merge conflicts

Craft for turning an in-progress merge or rebase conflict into a finished,
intent-preserving result — never a diff-guessing exercise, and never a
silent `--abort`.

## Where to look

| Situation / goal | Entry |
|---|---|
| A merge or rebase just landed in conflict | See the whole state first |
| About to resolve a hunk | Recover intent from primary sources, not the diff |
| Two sides changed the same code differently | Resolve each hunk: preserve, or pick and record |
| Tempted to fill a gap neither side wrote | Never invent behavior neither side has |
| The conflicts are resolved and staged | The checks are part of the resolution |
| Ready to close it out | Finish the merge; abort is a recorded escalation |
| Conflicts recur commit after commit during a rebase | Rebase: same discipline, once per commit |

## See the whole state first

When a merge or rebase conflict lands → do not open the first conflicted
file and start typing. Look at the whole state first: which paths are
conflicted, the commit sequence on both sides, and a full read of each
conflicted file end to end — including hunks git resolved automatically,
which can still be wrong. Touching one hunk before you have the map risks
fixing conflict two in a way that fights conflict five, three files over.

**Example:** `git status` reports six files in conflict. `git log
--oneline main..feature` and `git log --oneline feature..main` show the
two sequences are a rename plus a validation-rule change, colliding at
the same function. That shape, seen before opening a single file, says
the resolution has to keep both the new name and the new validation.

## Recover intent from primary sources, not the diff

When about to resolve a hunk → the diff shows what changed, never why.
Read the commit message on each side, the PR description if one exists,
and the linked ticket — the diff is evidence of a change, the message is
evidence of the reason for it. A hunk that looks like a straight
overwrite often turns out to be two independent fixes to the same line;
the `<<<<<<<` markers alone cannot tell you which.

**Example:** side A's hunk lowers a timeout from 30s to 10s with no
message; side B's commit message reads "fix flaky upload test by raising
client timeout to 45s." Reading only the diff, you'd average the two or
guess; reading the message tells you B's 45s is the fix and A's 10s was
an unrelated experiment that should not survive.

## Resolve each hunk: preserve, or pick and record

When two sides changed the same code differently → try to keep both
intents in the final hunk; most conflicts are two additive changes that
only collide because they touch nearby lines, not because they disagree.
When the two are genuinely incompatible, pick the side that matches the
merge's stated goal — the reason this merge is happening at all — and
write one line next to the resolution (the merge commit or a decision-log
entry) naming what you kept and what you dropped.

**Example:** side A renamed a parameter for clarity; side B added a new
call to the same function. Preserve both: apply B's new call using A's
renamed parameter. If instead the two disagree — A's PR says "always
require login," B's PR says "guest checkout is required for launch" —
pick the side matching this merge's stated goal (shipping the launch) and
note in the merge commit: "kept guest checkout per B; A's
always-require-login change is superseded, not merged."

## Never invent behavior neither side has

When a hunk conflict has no clean answer → do not write a compromise
neither side actually contains — a threshold halfway between the two, a
fallback neither commit tested, a "safe default" invented on the spot.
If preserving both is impossible and neither's stated goal clearly wins,
that is a decision for whoever owns the tradeoff, not a hunk to resolve
alone.

**Example:** side A sets a retry count of 3, side B sets it to 5, and
neither commit message explains why. Writing 4 "to split the difference"
ships a number nobody chose, tested, or can defend. Flag it and ask, or
pick one and record it as an explicit, named guess — never let it look
like a considered decision.

## The checks are part of the resolution

When the conflicts are resolved and staged → run the project's checks —
typecheck, then tests, then format, in the order the project declares
them — before treating the merge as done. A merge that resolves every
`<<<<<<<` marker but breaks the build has not been resolved; it has been
silenced. Fix whatever the merge broke in this same pass, not as a
follow-up task.

**Example:** after resolving five files, the test suite fails one test
that neither side's original commit touched — the resolution itself
introduced the break, by keeping side A's renamed field without updating
side B's call site to match. Fix it now, before finishing the merge, not
in a separate "fix the merge" cell afterward.

## Finish the merge; abort is a recorded escalation

When the resolution and checks are green → stage everything and finish
it — commit for a merge, continue for a rebase, repeating across every
remaining commit. Aborting is not the default off-ramp for a merge that
got hard; it throws away the resolution work already done and leaves the
original conflict waiting for the next person. Reach for it only as a
deliberate, recorded escalation — a decision that this merge should not
happen at all, made and stated out loud — never a quiet exit from a
conflict that was merely tedious.

## Rebase: same discipline, once per commit

When rebasing rather than merging → the same moves apply, replayed once
per commit instead of once for the whole branch: see the state of the
current conflicted commit, recover its intent from its own message — the
commit being replayed, not the branch it lands on — resolve each hunk,
never invent, run the checks, then continue the rebase, and repeat for
the next commit. The intent that governs each conflict is the intent of
the commit currently being replayed; a rebase touching ten commits asks
the same question ten times, each with a different answer.

## Related guides

Neighboring disciplines, each loaded only when its trigger applies —
never alongside this file by default.

- [debugging.md](debugging.md) — read when the checks fail after a
  resolution and the cause of the break is not obvious from the merge
  itself.
- [operations.md](operations.md) — read when the change the merge lands
  is risky enough to need its own rollout thinking, independent of how
  the conflict was resolved.
