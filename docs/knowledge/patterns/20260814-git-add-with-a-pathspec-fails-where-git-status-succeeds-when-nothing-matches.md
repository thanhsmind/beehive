---
type: bee.pattern
title: git add with a pathspec fails where git status succeeds when nothing matches
description: "`git status --porcelain -- <root>` reports quietly on a root that matches nothing, but `git add -A -- <root>` exits non-zero with 'pathspec did not match any files' — a multi-root bookkeeping commit must filter its pathspecs to ones that exist on disk or are tracked before add/commit."
timestamp: 2026-08-14
bee:
  id: pattern-20260814-git-add-pathspec-fails-on-empty-match
  lifecycle: active
  areas: [worktree-lifecycle]
  sources: [".bee/cells/archive/traceable-runs/trun-4.json (deviation, red test first)", "packages/bee-rs/crates/bee/src/verbs/worktree/merge.rs (commit_main_bookkeeping pathspec filter)"]
  polarity: pitfall
  evidence: wired
  evidence_ref: "packages/bee-rs/crates/bee/src/verbs/worktree/merge.rs (commit_main_bookkeeping)"
  signature: git-add-pathspec-did-not-match-any-files
---

# git add with a pathspec fails where git status succeeds when nothing matches

`git status --porcelain -- <root>` and `git add -A -- <root>` disagree about an
empty match: status reports nothing and exits zero; add fails outright with
`pathspec 'X' did not match any files`. Any commit step driven by a LIST of
candidate roots — bookkeeping auto-commits, sweep commits, scoped syncs — hits
this the first time one root legitimately has nothing in it (the ordinary case
for `docs/history/<feature>/` on a worktree that never wrote there).

## The rule

- Never pass a speculative pathspec list straight to `git add`. Filter each
  root first: keep it only if it exists on disk or `git ls-files -- <root>`
  shows tracked content.
- `git status`'s silence is not evidence `git add` will accept the same
  pathspec — the two commands have different empty-match contracts.
- Found red-first in trun-4 (traceable-runs): `commit_main_bookkeeping` filters
  its roots before add/commit.
