---
name: bee-principle-laziness-protocol
description: "Apply when refactoring, judging diff size, or tempted to add an abstraction, a layer or a signal threaded through types. Lean toward deletion and the smallest change that solves the problem."
---

# Laziness Protocol

Writing code is cheap for you, which makes over-engineering easy. Borrow a
human maintainer's fatigue to counter it: get the most result from the least
code and complexity.

Look for removals before additions when asked to refactor or improve. Keep
the call hierarchy flat: a rich interface that hides real work is not a deep
chain, but if answering one question means tracing more than three files or
layers, flatten it. Make each decision once, behind one source of truth, and
pass the result on as a simple flag. Make the smallest change that solves the
problem, since fewer lines beat elegant boilerplate. When a task asks you to
thread a new signal through types, schemas or pipeline layers, stop and look
for a more direct path. Remove the small leaks early: a pass-through, a
representation that escapes its owner, a choice made twice. They compound into
a permanent coordination cost.

**Check:** what is the smallest diff that solves this, and would a human
maintainer find the result exhausting to keep up?

**Not when:** the layer or duplicate you would remove still has a caller, a
test or a contract leaning on it (see Chesterton's fence). Laziness trims
structure, never the proof: tests and honest error handling stay.

**Why:** code you did not write has no bugs and no reader, and every extra
layer is paid for by each later change.

**Depth:** `.bee/expertise/architecture.md` § Laziness Protocol.
