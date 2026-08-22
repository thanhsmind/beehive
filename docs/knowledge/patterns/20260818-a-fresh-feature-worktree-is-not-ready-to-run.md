---
type: bee.pattern
title: A fresh feature worktree is not ready-to-run — budget its bootstrap gaps
description: "A worktree created by `bee worktree new` used to arrive with two known gaps. The binary gap is CLOSED as of 2026-08-21 — the bootstrap now provisions `.bee/bin/bee` itself. The dirty-copy gap is still live: the bootstrap drops untracked cell-record copies under `.bee/cells/` on a tracked path, which later blocks `bee worktree merge` as WORKTREE_DIRTY (remove the copy via git clean before merging — the main store holds the authoritative record). Three deliveries hit the binary gap and two hit the dirty-copy gap on the same day before this was named."
timestamp: 2026-08-18
bee:
  id: pattern-20260818-fresh-worktree-not-ready-to-run
  lifecycle: active
  areas: [worktree-parallelism]
  sources: ["porting-protocol cell pp-1 (worker copied .bee/bin/bee from main; merge blocked on untracked .bee/cells/pp-1.json, cleaned via git clean; 2026-08-18)", "pocock-nuggets cell pn-1 (same binary copy, same dirty-copy clean)", "review-axes cell ra-1 (same pair; docs/history/review-axes/promote-proposals.md pattern candidate)", "backlog: teach bee worktree new to close both gaps"]
---

**The binary gap is CLOSED (store-reach-gaps D2, 2026-08-21).** It read: a
worker dispatched into a fresh feature worktree cannot run the regen chain,
its verify command, or `bee cells finish`, because `.bee/bin/bee` is a
gitignored build artifact a fresh checkout does not have — so copy it from the
main checkout at the identical commit and record the copy as a deviation. The
bootstrap now provisions the binary itself (a symlink where the platform
allows one, a mode-preserving copy otherwise), so no worker brief needs to
budget that step any more. Kept here as history: three deliveries paid for it
before it was named, and the remedy above is what a pre-2026-08-21 worktree
still needs.

The same bootstrap also copies the feature's cell record into the
worktree's `.bee/cells/`. That path is tracked in this repo, so the
copy sits untracked and `bee worktree merge` later refuses with
WORKTREE_MERGE_WORKTREE_DIRTY. The copy is not state — the main
checkout's store holds the authoritative, capped record — so the fix
is `git clean -f -- .bee/cells/<id>.json` in the worktree before
merging (the direct-edit guard blocks a bare `rm`; `git clean` on the
bootstrap copy, with the reason recorded, is the sanctioned spelling).

Budget the dirty-copy gap into any dispatch prompt or worker brief that
targets a fresh worktree. The durable fix is `bee worktree new` closing its
own gaps: it now closes the binary one, and the cell-copy one is still filed
in the backlog.
