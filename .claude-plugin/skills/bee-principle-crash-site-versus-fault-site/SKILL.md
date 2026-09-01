---
name: bee-principle-crash-site-versus-fault-site
description: "Apply when you have found where the code throws, crashes or asserts and are about to fix it there. Trace the bad value back to where it was created and fix it at its origin."
---

# Crash Site Versus Fault Site

The crash site is where the invalid state was *detected*. The fault site is
where it was *created* — often far earlier: a null returned three calls up, a
config misread at startup, a cache poisoned yesterday.

When you know where it crashed, trace the bad value backward to its origin and
fix it there. A null-check at the crash site silences the alarm and leaves the
fault standing: the same bad value keeps flowing, and the next place it lands
has no guard.

**Why:** the stack trace names the detector, not the cause. Treating the two as
the same thing turns one bug into a series of local patches, each one hiding
the evidence that would have found the real origin.

**Depth:** `.bee/expertise/debugging.md` § Crash site versus fault site.
