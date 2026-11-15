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
matches your hand result before it touches the rest. Make the tool safe to
rerun (`bee-principle-make-operations-idempotent`), because a reviewer will.
Keep it where the next person can rerun it, and commit it when the work
outlives the session.

The tool is the artifact a reviewer can read and rerun; a hand-done change can
only be checked by redoing it. A deterministic tool beats fan-out: when one
pass can process every unit, run it yourself instead of sending workers to
hand-apply what a script can do. When you do fan out, write the recipe once —
the steps, the verification contract and the do-not-touch fences — in a file
every worker reads, outside their write scope so none can quietly edit the
contract, instead of re-explaining it in each brief and watching each copy
drift (`bee-fanout`).

Skip the lever when the work is a handful of obvious edits — a tool that
costs more than the work it replaces is its own waste. The bar is triviality,
not repetition: a one-off still earns a lever when the lever is what makes it
checkable, such as a query for an analysis or a rerunnable check for a
verification (`bee-principle-prove-it-works`). Build the smallest tool that does
or proves the job, never a framework (`bee-principle-laziness-protocol`).

**Check:** if you cited this principle, is there a script, codemod, generator
or recipe file in the diff? If not, you did not apply it.

**Why:** a hand-done unit plus a tool that reproduces it is checkable; fifty
hand edits are fifty separate claims nobody verifies.

**Depth:** `.bee/expertise/planning.md` § Build the lever.
