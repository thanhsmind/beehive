---
name: bee-principle-outcome-oriented-execution
description: "Apply during a planned rewrite or migration with explicit phase boundaries. Converge on the target architecture instead of keeping every intermediate state smooth with throwaway compatibility code."
---

# Outcome-Oriented Execution

Aim at the intended, verifiable end state, not at keeping each intermediate
state smooth. Holding every step fully stable usually means writing
temporary compatibility code, and that code outlives the migration as debt.
Converge on the target architecture and prove it at the boundaries the plan
names.

Breakage between steps is acceptable when it is planned, scoped and
reversible: say where it is allowed and which step must be green again. Keep
focused checks on the areas you are actively changing, and run the full
static and runtime verification before you call the migration done.

**Check:** which phases of this plan may be red, which step restores green,
and what is the full verification that closes it?

**Not when:** there is no real dependency forcing phases (the change lands
whole, see split-on-a-real-dependency), or a user-facing surface that another
worktree or client uses would sit broken mid-migration.

**Why:** a smooth middle is paid for in shims, and a declared break that ends
at a proved end state is cheaper than a shim that never leaves.

**Depth:** `.bee/expertise/planning.md` § Outcome-oriented execution.
