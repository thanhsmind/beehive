---
name: bee-principle-make-illegal-states-unrepresentable
description: "Apply when you design a type or review a signature. Shape the types so a wrong value fails to compile instead of failing at runtime."
---

# Make Illegal States Unrepresentable

Let the compiler refuse what should never happen. Give each meaning its own
type: two ids that are strings underneath get one newtype each, validated
once when created, so swapping them is a compile error.

Model variants as a sum type, not a bag of optional fields. A record with
`completed: bool` and an optional `completed_at` admits "completed, never at
any time", which means nothing. Derive the flag from the one field, or spell
the variants out: open, or done at a time. If a bug makes you ask "can this
combination happen?", the type is too loose.

Build the type up from the values you want instead of carving them out of a
looser type with checks. A non-empty list is a head plus a rest, and a valid
time range is a start plus a duration — each cannot hold the illegal value,
and the interface callers need sits on top.

Parse outside data into those types at the edge
(`bee-principle-boundary-discipline`). Match every variant of a sum type
without a catch-all arm, so a new variant breaks each place that now has to
decide. When a schema owns a shape, derive the type from it instead of
writing a parallel one that drifts (`bee-principle-encode-lessons-in-structure`).

Do not lie to the compiler. A cast, an unsafe coercion or an assertion that
bypasses it is a crash waiting for its input. Prove the fact by validating or
narrowing, or accept the cast as a hazard and say so.

Strengthen a type only where partiality shows up: a runtime assertion, a null
check or a "cannot happen" panic marks where the type is too weak. Push that
check into the type, then stop. Prefer total functions — a sum of an empty
list is zero, so it takes the plain list; the first element has no answer, so
it demands the non-empty one. Extra precision beyond that costs reuse and buys
no safety.

**Check:** which two arguments or fields share a primitive type but mean
different things? Where did this cast or forced unwrap come from — trace it to
the boundary and validate there. Could a comment explain when this field
combination is valid? Then split it into a sum type. If a variant is added
next month, will the compiler name each place that needs a case?

**Not when:** the language cannot express the distinction at a reasonable
cost. Then validate once at the boundary and record why in the
owning concept. Nor when nothing would otherwise panic: keep the plain type.

**Why:** a runtime check protects the paths someone remembered to test; a
type protects every path, including the ones not written yet. Defining the
special case out of existence beats writing a handler for it.

**Depth:** `.bee/expertise/architecture.md` § Make Illegal States Unrepresentable.
