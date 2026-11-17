---
name: bee-principle-split-on-a-real-dependency
description: "Apply when a plan proposes phases, slices or an expand-migrate-contract sequence. Each split needs a real dependency — ask the removal test of every one before you keep it."
---

# Split on a Real Dependency

A phase feels safe, but it has a cost: an in-between state the tree must
hold, a bridge to keep working, and a later step to remove it. Only a real
dependency pays for that cost — a persisted-data migration, a consumer
outside this change (another live worktree, an external client, a published
contract), or a compatibility window.

Without one, the change lands as one coordinated change, and the compiler and
the tests prove it. Ask the removal test of every phase and slice: if we skip
it and go straight to the final state, what breaks? If nothing breaks, it is
not a phase. A temporary bridge that does stay in the tree is filed with
`bee backlog add`, names its removal condition, and is cited from the owning
concept.

**Check:** for each phase or slice, can you name what breaks without it?

**Why:** a split with no dependency behind it does the work twice and leaves
half-states nobody needed.

**Depth:** `.bee/expertise/planning.md` § Split only on a real dependency.
