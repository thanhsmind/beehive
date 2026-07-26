---
type: bee.pattern
title: "A resolver added beside the old one is dead code until every call site is swept — the seam ships when the LAST consumer moves, not when the new function lands"
description: A cell that adds a new function beside an old one and calls the migration started ships nothing observable; an honest worker block that names the real sweep size beats a false-green cap that hides it.
tags: [architecture, migration, refactor, dead-code, worktree-parallelism, advisor]
timestamp: 2026-07-25
bee:
  id: pattern-20260725-a-resolver-added-beside-the-old-one-is-dead-code
  lifecycle: active
  sources: ["multisession-native cell multisession-native-17 (resolveContext(cwd) added beside resolveRoots with zero production call sites reading its new fields; trace .bee/cells/multisession-native-17.json, commit bd8f755, 2026-07-25)", multisession-native re-slice decision 89a4a87b (msn-18 honest block and the 18a/18b/18c/18d re-slice that followed), docs/history/multisession-native/reports/advisor-digest-slice4.md (condition 3/F4)]
  polarity: pitfall
  critical: true
---

# A resolver added beside the old one is dead code until every call site is swept — the seam ships when the LAST consumer moves, not when the new function lands

`multisession-native-17` added `resolveContext(cwd)` — the new topology
resolver meant to split coordination stores onto a shared control plane —
beside the existing `resolveRoots()`, refactored the latter into a
byte-identical compat wrapper over the same core, and capped green: tests
passed, the new function existed, the cell's own scope was satisfied on
paper. But not one production module actually read `controlRoot` for
anything yet. Every coordination-store call site — sessions, claims, the
workflow record, leases, recovery, compaction, the CLI dispatcher itself —
still resolved its own root the old way. The feature the cell was building
toward (a linked worktree's coordination state visible from main) did not
exist yet, and nothing about the green cap said so.

The worker assigned the next cell — wiring `resolveContext` into the write
guard's lane read — did not half-land a fix scoped to one call site. It
recognized that closing the gap honestly meant re-rooting roughly a dozen
call sites across six modules, named that scope, and **blocked** rather than
shipping a cell that moved one reader while a dozen others stayed on the old
resolver — a state indistinguishable from "still broken" for every reader
except the one it touched. The re-slice that followed (decision `89a4a87b`)
split the real sweep into four ordered cells (state.mjs's own sites and the
highest-risk guard read; the cells/reservations/recovery/compaction/
projection sweep; the CLI dispatcher, kept standalone rather than folded into
a later cell to avoid a false-green window; the onboarding migration for
data stranded by the earlier three) — and only after all four closed did the
feature the resolver was built for actually exist.

**Rule.** A new resolver, adapter, or schema added beside an old one is not
"the seam landing" — it is dead code with a passing test suite until the LAST
call site that reads the old path is moved to the new one. Auditing "does the
new function work" one unit test at a time will pass every review while the
real migration hasn't started; the question that catches it is "how many
production call sites read the OLD path today, and does this cell's own
`files` list cover closing every one of them — or does it just add the new
option beside them?" When a plan or cell scope answers "cover every one" with
anything less than the true count, the honest move is the worker's here: name
the real sweep size and block, rather than cap a green that only moved one
reader. A false green here does not fail loudly later, either — the two
paths keep coexisting silently, agreeing on every input they happen to share
and diverging exactly where a linked worktree's state used to live before
the split, which is the one case nobody's test suite was written to notice
until this pattern was caught.
