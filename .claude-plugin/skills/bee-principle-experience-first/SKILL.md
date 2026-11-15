---
name: bee-principle-experience-first
description: "Apply when product, UX or feature-scope tradeoffs come up. Choose user delight over implementation convenience; ship fewer polished features over more rough ones."
---

# Experience First

The product is the experience, and every technical decision either helps or
hurts it. When implementation convenience conflicts with user delight, choose
delight.

Say no to a thousand things: every feature, control and option must earn its
place. Ship less and ship better, since a polished experience with three
features beats a rough one with ten. Prototype before committing, because a
design decision is cheaper in throwaway markup than in production code. Sweat
the details: transitions, alignment, spacing, feedback and error states.
Tighten the core loop: each feature serves the central workflow or gets out of
the way.

The user is whoever consumes the work: the end user for a UI, the colleague who
imports it for a library or an internal API, and the engineer who maintains the
code next. Weigh their experience the same way and explain impact from their
seat. Foundations serve the experience; foundational thinking governs the
sequence of work, and this principle governs the target.

**Check:** whose experience does this change touch, and where convenience and
delight conflicted, which one did you pick?

**Not when:** a locked decision or a stated constraint already settled the
tradeoff, or the surface is one the owner called throwaway. Polish nobody asked
for is scope (see smallest-honest-shape).

**Why:** a feature that is convenient to build and unpleasant to use is paid for
by every user, every time.

**Depth:** `.bee/expertise/thinking.md` § Experience First.
