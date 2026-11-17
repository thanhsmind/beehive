---
name: bee-principle-crash-site-versus-fault-site
description: "Apply when you have found where the code throws, crashes or asserts and are about to fix it there. Trace the bad value back to where it was created and fix it at its origin."
---

# Crash Site Versus Fault Site

The crash site is where the invalid state was *detected*. The fault site is
where it was *created* — often far earlier: a null returned three calls up, a
config misread at startup, a cache poisoned yesterday.

When you know where it crashed, trace the bad value backward to its origin and
fix it there. Ask why at each step until the answer is a cause and not another
symptom. A null-check at the crash site silences the alarm and leaves the
fault standing: the same bad value keeps flowing, and the next place it lands
has no guard.

When you find the origin, search for the same shape elsewhere — the same
misread, the same unchecked return. Fix every instance in the same cell.

**Restart bugs: suspect state before code.** Code does not change between
runs; state does. When something fails after a restart, suspect stale
persistent state first — config files, caches, lock files, serialized state.
If clearing a state file restores the behavior, the fix is validating that
state where it is read or written, not the code that tripped over it.

**A workaround that needs a paragraph to justify it means the code is
wrong.** Fix the code, not the comment. Without a way to make it fail, you
cannot tell a fix from a coincidence: get the repro first
(`bee-principle-reproduce-first`), and when the cause stays unclear, add
observation before you guess.

**Check:** where was this value created, and what else does that origin feed?

**Why:** the stack trace names the detector, not the cause. Treating the two as
the same thing turns one bug into a series of local patches, each one hiding
the evidence that would have found the real origin.

**Depth:** `.bee/expertise/debugging.md` § Crash site versus fault site.
