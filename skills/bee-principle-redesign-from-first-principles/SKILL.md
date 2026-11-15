---
name: bee-principle-redesign-from-first-principles
description: "Apply when integrating a new requirement into an existing design. Redesign as if the requirement had been a foundational assumption from day one, instead of bolting it on."
---

# Redesign From First Principles

Do not bolt a change onto the design that was there. Redesign as if the
requirement had been a foundational assumption from the start, so the result
reads like what you would have built had you known on day one.

Read every affected file and hold the current design whole. Ask what you
would build from scratch with this requirement, and take that as the target.
Carry the change through every reference (types, docs, examples, rationale),
so nothing still describes the design without it. Think about the redesign as
one picture and deliver it in small steps.

**Check:** if you were writing this from scratch with the new requirement,
would it look like what you are about to leave behind, and which reference
still describes the old design?

**Not when:** the requirement is a one-off that does not change what the
design is, or you have not read the affected files yet. Redesigning what you
have not understood is a rewrite on a guess (see Chesterton's fence).

**Why:** a bolt-on is cheap once and costly at every later change, because
each special case beside the old design is one more thing no one can explain.

**Depth:** `.bee/expertise/architecture.md` § Redesign From First Principles.
