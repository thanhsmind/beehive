---
name: bee-principle-the-deletion-test
description: "Apply when you judge whether a suspect module, wrapper or layer earns its complexity. Imagine deleting it and inlining its behavior at every call site."
---

# The Deletion Test

Delete the module in your head and inline what it does at every call site.
Then look at the call sites.

If the complexity reappears at each of the N callers, the module was doing
real work — deleting it would move the complexity, not remove it. If the
complexity simply vanishes with no trace anywhere, the module was a
pass-through: an interface wrapped around nothing, paying for indirection and
returning no behavior. A three-line `OrderValidator` around one if-check
disappears cleanly — inline it and remove the module. A `RetryingClient`
around a flaky call does not: its five callers would each write their own
backoff loop.

**Why:** "is this abstraction worth it" is an argument. "What happens at the
call sites when it is gone" is an observation, and it settles the argument.

**Depth:** `.bee/expertise/architecture.md` § The Deletion Test.
