---
name: bee-principle-boundary-discipline
description: "Apply when you wire validation, error handling or a framework adapter. Check input once where it enters, trust typed data inside, and keep decisions in pure functions."
---

# Boundary Discipline

Input from outside — CLI arguments, config, files, the network, another
process — is untrusted until it is parsed. Parse and validate it once, at the
edge, into the types the rest of the code uses. Past the edge, trust those
types; a check deep in a call chain means the edge did not finish its job.

Keep decisions in pure functions that take typed data and return a result,
and keep the shell that calls them thin. Never let the boundary's private
types, such as a wire format or a storage row, leak through the public
surface.

**Check:** is this data crossing a boundary right now, and can this logic be
a pure function the shell calls?

**Not when:** the code is the boundary itself. There, check defensively and
return errors without assuming anything.

**Why:** validation scattered through trusted code is noise that gives a
false sense of safety, and logic tied to the framework cannot be tested
without it.

**Depth:** `.bee/expertise/architecture.md` § Explicit Boundaries and Contracts.
