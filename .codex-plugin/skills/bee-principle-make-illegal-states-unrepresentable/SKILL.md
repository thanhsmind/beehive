---
name: bee-principle-make-illegal-states-unrepresentable
description: "Apply when you design a type or review a signature. Shape the types so a wrong value fails to compile instead of failing at runtime."
---

# Make Illegal States Unrepresentable

Let the compiler refuse what should never happen. Give each meaning its own
type: two ids that are strings underneath get one newtype each, validated
once when created, so swapping them is a compile error.

Parse outside data into those types at the edge. Match every variant of a
sum type without a catch-all arm, so a new variant breaks each place that now
has to decide. When a schema owns a shape, derive the type from it instead of
writing a parallel one that drifts.

**Check:** which two arguments or fields share a primitive type but mean
different things?

**Not when:** the language cannot express the distinction at a reasonable
cost. Then validate once at the boundary and record why in the
owning concept.

**Why:** a runtime check protects the paths someone remembered to test; a
type protects every path, including the ones not written yet.

**Depth:** `.bee/expertise/architecture.md` § Make Illegal States Unrepresentable.
