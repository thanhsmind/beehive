---
name: bee-principle-never-invent-behavior-neither-side-has
description: "Apply when you resolve a merge or rebase conflict that has no clean answer. Never write a compromise neither side contains — pick one and record it, or escalate to whoever owns the tradeoff."
---

# Never Invent Behavior Neither Side Has

A hunk conflict with no clean answer is not an invitation to design. Do not
write the compromise: a threshold halfway between the two, a fallback neither
commit tested, a "safe default" invented on the spot.

Side A sets a retry count of 3, side B sets 5, and neither commit message
explains why. Writing 4 to split the difference ships a number nobody chose,
nobody tested, and nobody can defend — and the merge commit makes it look
considered.

If preserving both is impossible and neither side's stated goal clearly wins,
that is a decision for whoever owns the tradeoff. Flag it and ask, or pick one
side and record it as an explicit, named guess — never let a guess enter the
history disguised as a resolution.

**Why:** a merge resolution carries no author and no reasoning. Behavior that
enters the tree that way has no test, no owner, and nobody who remembers
choosing it.

**Depth:** `.bee/expertise/merges.md` § Never invent behavior neither side has.
