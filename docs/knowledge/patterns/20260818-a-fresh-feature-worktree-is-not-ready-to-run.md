---
type: bee.pattern
title: A fresh feature worktree is not ready-to-run — budget its two bootstrap gaps
description: "A worktree created by `bee worktree new` arrives with two known gaps: the gitignored `.bee/bin/bee` binary is absent (copy it from the main checkout at the same commit before any regen/verify/finish step), and the bootstrap drops untracked cell-record copies under `.bee/cells/` on a tracked path, which later blocks `bee worktree merge` as WORKTREE_DIRTY (remove the copy via git clean before merging — the main store holds the authoritative record). Three deliveries hit the binary gap and two hit the dirty-copy gap on the same day before this was named."
timestamp: 2026-08-18
bee:
  id: pattern-20260818-fresh-worktree-not-ready-to-run
  lifecycle: active
  areas: [worktree-parallelism]
  sources: ["porting-protocol cell pp-1 (worker copied .bee/bin/bee from main; merge blocked on untracked .bee/cells/pp-1.json, cleaned via git clean; 2026-08-18)", "pocock-nuggets cell pn-1 (same binary copy, same dirty-copy clean)", "review-axes cell ra-1 (same pair; docs/history/review-axes/promote-proposals.md pattern candidate)", "backlog: teach bee worktree new to close both gaps"]
---

A worker dispatched into a fresh feature worktree cannot run the regen
chain, its verify command, or `bee cells finish`: `.bee/bin/bee` is a
gitignored build artifact and a fresh checkout does not have it. The
working remedy is to copy the binary from the main checkout at the
identical commit before the first bee command, and record the copy as a
deviation — a tooling prerequisite, never a source change.

The same bootstrap also copies the feature's cell record into the
worktree's `.bee/cells/`. That path is tracked in this repo, so the
copy sits untracked and `bee worktree merge` later refuses with
WORKTREE_MERGE_WORKTREE_DIRTY. The copy is not state — the main
checkout's store holds the authoritative, capped record — so the fix
is `git clean -f -- .bee/cells/<id>.json` in the worktree before
merging (the direct-edit guard blocks a bare `rm`; `git clean` on the
bootstrap copy, with the reason recorded, is the sanctioned spelling).

Budget both gaps into any dispatch prompt or worker brief that targets
a fresh worktree; the durable fix is `bee worktree new` closing both
gaps itself (filed in the backlog).
