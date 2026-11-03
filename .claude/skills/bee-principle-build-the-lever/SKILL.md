---
name: bee-principle-build-the-lever
description: "Apply when the same non-obvious edit must land in many places — a rename across files, a migration over many records, a repeated rewrite. Do one unit by hand, then build the tool that repeats it."
---

# Build the Lever

When the work repeats and each unit is not obvious at a glance, a person
doing it by hand makes a different small mistake in every tenth unit. Build
the tool that does the work instead — a script, a codemod, a query, a
generator.

Do one unit by hand first. That unit is the specification: it shows what a
correct result looks like, including the edge the tool must respect. Then
build the tool, run it on that same unit, and require that its output
matches your hand result before it touches the rest. Keep the tool where the
next person can rerun it.

Skip the lever when the work is a handful of obvious edits — a tool that
costs more than the work it replaces is its own waste.

**Why:** a hand-done unit plus a tool that reproduces it is checkable; fifty
hand edits are fifty separate claims nobody verifies.

**Depth:** `.bee/expertise/planning.md` § Build the lever.
