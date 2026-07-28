---
date: 2026-07-28
feature: skill-diet-wave2 + workflow-lifecycle
categories: [orchestration, workflow-state, doctrine-layer]
severity: high
tags: [parallel-at-scale, git-safety, feature-verify, thin-body]
---

# parallel at scale — two features, eleven workers, one afternoon

## What Happened

Two features ran concurrently in one checkout, eleven workers total.

**skill-diet-wave2** finished the thin-body migration: the last nine skills,
105,435 → 67,401 bytes (−36%). Every skill in the tree is now under the 8,192
budget and **zero grandfather exceptions remain** — the fence has no
exemptions left to explain.

**workflow-lifecycle** closed the drift that hit five times today: one
record-creation seam every start path calls, close-by-feature rather than
close-by-record-presence, and a `state workflows list/close` verb so clearing
stale records never again requires hand-editing bee's own state.

## The concurrency plan is computed, and the computation pays

The nine migrations were first scheduled as **nine serial waves** — every cell
declared the shared budget ledger, and the scheduler correctly refused to run
them together. Moving the ledger to the barrier (MAIN writes all nine entries
in one pass at close) made the cells genuinely disjoint and the same scheduler
returned **one wave**. The rule that mattered: a shared *ledger* is a barrier
artifact, exactly like a generated manifest — never a per-cell file.

## Git is a shared resource, and reservations do not cover it

Three incidents, each teaching a different half of R88:

1. Two workers used `git add` + commit; a sibling's commit swept their staged
   paths. Content survived, commit attribution did not.
2. A worker's whole-tree revert wiped a live sibling's in-progress edit **while
   that sibling held a valid file reservation** — reservations govern files,
   the tree is not a file.
3. `git commit -- <path>` turns out to refuse untracked pathspecs, which is
   what pushed workers toward staging in the first place.

Settled shape: a worker commits through its **own temp index**
(`GIT_INDEX_FILE` + read-tree/update-index/write-tree/commit-tree/update-ref),
never touching the repository's; `git add -N` scoped to its own new files is
the fallback; whole-tree verbs are forbidden outright. Found by workers, in
production, in one afternoon — no amount of design review would have produced
this list.

## The boundary verify earned its place three times

MAIN's single feature verify caught three defects no worker could have seen,
because each only existed once the pieces were assembled:

1. A byte-trim severed a bundle-branch clause a line-local fence requires.
2. Three new exports were never declared in the census allowlist.
3. Two new CLI commands shipped with registry examples that no suite executed.

Each became a fix cell (never an un-cap), each re-verified green. Cost: ~10
minutes. Value: three broken invariants that would have shipped.

## Recommendation

- **When cells serialize, look for the shared artifact before accepting it.**
  One ledger moved to the barrier turned nine waves into one.
- **Give parallel workers a private index, not rules about the shared one.**
  Coordinating access to a shared resource fails; removing the sharing works.
- **Mechanize what a wave keeps proving.** The hook already inspects commands;
  denying whole-tree git verbs while more than one worker is live is now a
  filed P1, not a suggestion.
