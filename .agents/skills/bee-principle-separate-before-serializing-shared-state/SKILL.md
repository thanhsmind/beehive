---
name: bee-principle-separate-before-serializing-shared-state
description: "Apply when concurrent actors might write the same file, branch, key or record. Remove the sharing first; enforce one writer only when one shared target is a real invariant."
---

# Separate Before Serializing Shared State

List the mutable things more than one actor touches. For each, ask whether
they need the same object at all. Usually they are publishing independent
facts: give each actor its own file, key or record and merge them where they
are read.

Separation has to reach the file. Two writers of different fields in one JSON
file still share it. Only when one shared target is a real invariant do you
enforce a single writer through structure — an owning task, a lock, one write
path — never a convention callers must remember.

**Check:** can each actor own its own record, with the merge done at read
time?

**Not when:** the value is one global fact, such as a counter that must be
ordered across all actors. There, one structural writer is the design.

**Why:** a race shows up only under load and survives every quiet test, and
"we need a lock" is often a sign the sharing was never needed.

**Depth:** `.bee/expertise/architecture.md` § One Writer for Shared State.
