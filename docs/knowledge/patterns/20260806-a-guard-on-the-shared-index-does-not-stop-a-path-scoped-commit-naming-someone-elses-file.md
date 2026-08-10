---
type: bee.pattern
title: "A guard on the shared index does not stop a path-scoped commit naming someone else's file"
description: "The escape a guard recommends inherits the guard's job: the concurrent-worker git guard closes the staging path that swept siblings' work into the wrong commit, then allows a path-scoped commit to name any path, including a file another live worker is still editing."
tags: [concurrency, git, guards, attribution, escape-hatches]
timestamp: 2026-08-06
bee:
  id: pattern-20260806-shared-index-guard-vs-path-scoped-commit
  lifecycle: active
  areas: [worktree-parallelism, hook-runtime]
  decisions: ["d4182ff1 (blanket-staging-guard): git add -A/-u and git commit -a count as broad writes, 2026-07-26"]
  sources: ["cli-surface-in-context wave, 2026-08-06: cell csc-1's commit 7fc5439b carried 133 deleted lines of router.rs — sibling cell csc-4's in-progress edit — and needed corrective commit f94f262a to restore it", "the guard's own refusal text: a path-scoped commit is named as the sanctioned escape"]
  polarity: pitfall
  critical: true
  signature: path-scoped-commit-named-a-sibling-owned-file
---

# A guard on the shared index does not stop a path-scoped commit naming someone else's file

The concurrent-worker git guard exists because a shared index sweeps a sibling's
files into whoever commits next. It refuses `git add`, a bare `git commit`,
`stash`, `reset` — every verb that operates on the index or the tree as a whole.
Then it names the way out: make a path-scoped commit instead.

Nothing checks that the paths you scope to are yours.

The measured instance: a wave of four workers ran in one checkout, files
disjoint by design and reserved accordingly. One worker committed with an
explicit path list that included a file it did not own, and swept 133 lines of a
sibling's uncommitted work into its own commit — the same attribution loss the
guard was written to prevent, through the door the guard recommends. It was
caught and reverted, and the sibling's work restored, but only because the
worker noticed and said so.

Reservations knew the answer the whole time. The file was reserved under another
nickname; a path-scoped commit naming it could have been refused on exactly that
evidence, with the same message the write guard already gives.

## The rule

- An escape hatch a guard *recommends* is part of the guard. It inherits the
  guard's purpose, and it needs the guard's checks — otherwise the refusal
  merely reroutes the damage through a path the author blessed.
- When a guard already holds the evidence that would decide the escape (here: a
  live reservation naming the file's real owner), not consulting it on the
  escape path is the defect, not an omission of scope.
- Judge a concurrency guard by what a *well-behaved* caller can still break.
  This worker followed the instructions it was given and lost a sibling's work
  anyway.
- A wave that recovers because a worker volunteered "I clobbered something" was
  not saved by its guard. Treat that report as a guard finding, never as a
  successful outcome.
