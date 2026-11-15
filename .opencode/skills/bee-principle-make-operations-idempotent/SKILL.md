---
name: bee-principle-make-operations-idempotent
description: "Apply when you design a command, a lifecycle step or a processing loop that runs amid crashes, restarts and retries. Converge to the same end state whatever a partial earlier run left behind."
---

# Make Operations Idempotent

Design every state-changing operation to converge on the correct state however
many times it runs and wherever it starts from. Ask of each one: what happens
if this runs twice, and what happens if the previous run crashed halfway?

Start by scanning for state that already exists, clean what is stale, and
adopt what is live, rather than assuming a blank slate. Compare by content, not
by creation order. Let a lock heal itself by detecting that its holder is gone.
Let failed work respawn cleanly, with fresh input regenerated after each cycle.

**Check:** run it twice in a row, then crash it at every point it can stop;
does each re-run reach the same end state?

**Not when:** the operation is a pure read, or its single effect is already
atomic and safe to repeat; a reconciliation step there is code that guards
nothing (see subtract-before-you-add).

**Why:** commands, lifecycle steps and loops run where crashes, restarts and
retries are normal; if partial state changes the next run's outcome, every
restart becomes a debugging session.

**Depth:** `.bee/expertise/operations.md` § Make operations idempotent.
