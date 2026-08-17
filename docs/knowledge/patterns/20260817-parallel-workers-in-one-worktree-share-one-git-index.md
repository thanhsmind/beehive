---
type: bee.pattern
title: Parallel workers in one worktree share one git index — file-disjointness does not protect commits
description: "Two execution workers editing disjoint files inside the SAME worktree checkout still collide on the shared git index: one worker's stage/commit/amend cycle can sweep the sibling's staged edits into its own commit or wipe an in-progress edit. Disjoint files protect the working tree, never the index. Parallel cells in one worktree need serial commit windows, a per-worker GIT_INDEX_FILE, or one-worker-at-a-time dispatch."
timestamp: 2026-08-17
bee:
  id: pattern-20260817-shared-git-index-in-one-worktree
  lifecycle: active
  areas: [worktree-lifecycle]
  sources: ["worktree-keep-on-merge cells wkm-2/wkm-3 worker reports (2026-08-17): wkm-2's commit cycle swept wkm-3's staged registry.rs; a tests.rs edit was wiped and redone; both self-healed, final attribution clean (commits 3e32e605, 6ff041f8)", "sibling friction: merge-time bookkeeping sweep, backlog row 2026-08-16 (commit e9840f11)"]
---

Two bee-build workers ran in parallel inside one feature worktree on
deliberately disjoint files (`prune.rs`+`deferred_queue.rs` vs
`registry.rs`). The file split protected every *edit* in the working
tree — and protected nothing at commit time, because both workers
shared the checkout's single git index:

- Worker A's staged-but-uncommitted edit sat in the index when worker
  B committed, so B's commit (and each amend) swept A's file in.
- One of A's freshly written test edits was wiped by B's checkout
  churn and had to be redone.

Both workers noticed and self-healed — final commits carried the right
files — but only because each re-checked its diff before capping.

**The rule:** reserving disjoint files makes concurrent *editing*
safe, not concurrent *committing*. When more than one worker commits
in the same checkout, serialize the commit step (one worker commits at
a time), give each worker its own `GIT_INDEX_FILE`, or simply dispatch
one execution worker per worktree at a time. The cheapest reliable
shape today is the last one.
