# tci-2 — skill-lint pointer matcher reads a multi-heading parenthetical

**[DONE]**

The pointer check now accepts a quoted heading anywhere inside a parenthetical,
however many headings share it and whether or not the parenthetical wraps across
lines — so `("Silent Bookkeeping", "Progress ticks")` reads as the reachable
pointer it is. Before the fix the lint reported `bee-hive/SKILL.md has no pointer
to "Progress ticks"`; after it, the skill tree is clean. Reachability is still
the bar: a heading quoted outside every parenthetical is not a pointer, and an
absent one is still reported. `skill_lint` still exits 0 (T5's blocking check is
a separate cell).

**Files touched**

- `scripts/skill_lint.mjs` — `pointsTo` / `parentheticals` replace the literal
  `("<heading>")` substring test at the former line 107.

Full trace and verify evidence: `.bee/cells/tci-2.json`.

**Note for the feature owner** — check 1 (`ANCHOR_RE`, same file) captures only
the *first* quoted heading of a `references/x.md ("A", "B")` pointer, so a
dangling `"B"` goes undetected. Out of this cell's scope (it is a missed
detection, not a false report); worth its own cell if the slice wants it.
