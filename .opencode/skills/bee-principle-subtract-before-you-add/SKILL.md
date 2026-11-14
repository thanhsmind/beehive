---
name: bee-principle-subtract-before-you-add
description: "Apply when you sequence an addition, a refactor or a rewrite. Put the removal step first and build on the simpler base."
---

# Subtract Before You Add

Before a change adds anything, ask what can leave. Dead code, a validator no
contract asks for, a stub that holds no content: remove these first, so the
new work lands on a smaller base and every later step costs less.

Reach the minimum before you polish it, and build for use you have seen, not
for edge cases nobody has hit. Leave the design slightly simpler than you
found it — the same capability behind the same surface or a smaller one.

**Check:** is there a removal step ahead of the construction step in this
plan, and if not, what did you look at to decide there was nothing to
remove?

**Not when:** the thing you would remove still has a caller or an owner you
have not found. Find out why it is there first (see Chesterton's fence).

**Why:** addition is easy to justify one piece at a time, and the pile that
results is what makes the next change slow.

**Depth:** `.bee/expertise/planning.md` § Subtract before you add.
