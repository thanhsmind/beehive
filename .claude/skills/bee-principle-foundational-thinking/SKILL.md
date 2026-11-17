---
name: bee-principle-foundational-thinking
description: "Apply before writing logic: choosing core types and data structures, sequencing scaffold work before features, or asking what concurrent actors share. Settle the data shape first so the code that follows is obvious."
---

# Foundational Thinking

Get the data structures right before you write logic. The right shape makes
the code that reads and writes it obvious, and a change to it made late is a
rewrite where made early it is often a one-line diff. Define the core types
first, trace every access pattern against them, and choose the structure that
matches the dominant paths.

At code level, converge the structure, not every line: three similar
statements still beat a premature abstraction, and explicit beats clever.
Before two actors share state, ask what happens if the other changes it at
the same moment, and isolate them unless the answer is nothing. If a piece of
groundwork helps every later phase (CI, linting, test infrastructure, shared
types), do it first, in small single-purpose commits, after removing dead
weight.

**Check:** what are the core types and what touches them, and did you settle
them before the first function that uses them?

**Not when:** the change adds no structure and shares no state, such as a
one-line fix inside an existing shape. Settling a shape nobody needs yet is
speculation (see subtract-before-you-add).

**Why:** a data shape is a door held open or shut for everything built on it,
and it costs least to choose while nothing depends on it.

**Depth:** `.bee/expertise/architecture.md` § Foundational Thinking: Data Structures First.
