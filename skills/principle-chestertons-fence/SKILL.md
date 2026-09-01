---
name: principle-chestertons-fence
description: "Apply when you are about to delete or bypass something that looks pointless — a guard clause with no obvious trigger, a sleep, a flag nobody recognizes. Find out why it was put there before it comes down."
---

# Chesterton's Fence

Before you remove the thing that looks useless, find out why it exists.
Search the history, the linked issue, the test that fails without it. "I
don't see why this is here" is the argument for investigating, never the
argument for removing.

Two outcomes, both good: the reason is obsolete and the removal proceeds with
confidence, or the reason is alive and an incident was just declined. If the
reason is genuinely unrecoverable, remove it as an experiment with a watch on
what breaks — not as a cleanup.

**Why:** the code that looks pointless is exactly the code whose purpose was
never written down. Deleting on "I can't see the point" tests your visibility,
not the fence's value.

**Depth:** `.bee/expertise/thinking.md` § Chesterton's Fence.
