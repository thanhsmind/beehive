---
name: bee-principle-minimize-reader-load
description: "Apply when reviewing or shaping code that is hard to trace. Count the layers between a question and its answer and the hidden state a reader must hold; collapse one-caller wrappers and shrink mutable scope."
---

# Minimize Reader Load

Maintainability is the work a reader does to understand the code, on two
independent axes: how many layers sit between a question and its answer, and
how much hidden or mutable state the reader must keep in their head. A flat
file with fifty globals can be as hard to reason about as a six-layer adapter
stack, so guard both.

Collapse a layer that does not earn its keep: a wrapper with one caller, an
adapter with no second implementation, indirection built for a future that
never came. Shrink state scope toward pure functions, locals over fields,
fields over module state, and derive a value instead of syncing a copy. Name
an invariant once at the boundary, not in every consumer. Before adding a
layer or a piece of state, ask whether it removes at least as much load
somewhere else.

**Check:** can a new reader say where X comes from, and what can change X, in
under thirty seconds?

**Not when:** the layer is the seam that hides a real decision from several
callers, or a second implementation exists or is planned (see the deletion
test). A boundary that compresses meaningful work is the point, not the load.

**Why:** code is read far more than it is written, and line counts and
cyclomatic complexity are proxies for the cost that matters. Working memory is
finite for a human reader and for an agent's context window alike.

**Depth:** `.bee/expertise/review.md` § Reader load.
