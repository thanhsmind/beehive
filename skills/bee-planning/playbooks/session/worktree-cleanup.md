# worktree-cleanup

You own the disk and the safety gate. Prune merged or abandoned git worktrees
and stale simulators to reclaim space. Deletion is irreversible, so every step
guards against deleting something in use or holding uncommitted work.

1. Snapshot and audit. Record `df -h /`, then run `bee worktree prune
   --dry-run` from the main checkout, never from inside a worktree. The
   lever already exists, so do not hand-build another
   (`bee-principle-build-the-lever`). It reads worktrees from bee's own
   registry and records, never from a hand-typed path, since a hand-typed path
   misses a worktree that lives somewhere else
   (`bee-principle-encode-lessons-in-structure`). It reports one line per
   worktree: the size it would reclaim, or the condition that keeps it.
2. The verdict is advice, not permission. The open sessions are the real artifact
   (`bee-principle-prove-it-works`). Get that set from the owner and cross-check every candidate,
   because a tool has marked safe a worktree its owner still used, so the
   owner's set wins.
3. Verify usage before deleting. For every worktree the report would remove
   that you doubt, run a gather (`bee dispatch prepare --kind gather`) that
   reads the session records and reports whether a session is open on it and
   which worktrees that session touches. A herded worker spawns `bee-arena` and
   repro trees into sibling worktrees, and those are in use even when their
   names never reach the owner's list. Keep the gather's output to a digest
   (`bee-principle-guard-the-context-window`), because session logs are bulk.
4. Pause on irreversible loss. A tracked uncommitted edit is lost the moment
   the directory goes, so show the diff and get a decision first; removing a
   clean worktree is recoverable from its branch, uncommitted work is not.
   Untracked throwaway is safe to drop, but name the files. Clean, merged and
   not in use proceeds; uncommitted work or in use pauses.
5. Prune the confirmed set with `bee worktree prune` (add `--older-than-days N`
   to move the age line for one run). It removes the worktree, its branch, its
   grant and its record together, and keeps any worktree that fails one of its
   conditions, so a worktree it keeps is held back, not skipped. Lock a
   worktree with `git worktree lock` to keep it out of every later sweep.
   Confirm with `df -h /` and re-list.
6. Simulators and other reclaimers, when the host has them. Simulators are
   usually the next-biggest win: `xcrun simctl --set testing delete all`
   (test clones), `xcrun simctl delete unavailable`, and `xcrun simctl runtime
   list` then `runtime delete <id>` for old runtimes. More when needed: Xcode
   `DerivedData` and `iOS DeviceSupport`, and package caches (pnpm, uv, brew,
   yarn). Clear only caches the owner has not said to keep, because a cache
   they chose to keep costs them a rebuild.

This is the one playbook that deletes user state with no code review to catch a
slip, so the gates above are the review.

Reply: `df -h /` before and after with space reclaimed, the worktrees pruned,
and a one-line reason for each held back (in use by which session, or
uncommitted work).

Proof line: `df -h /` before and after, and the `bee worktree prune` report,
verbatim.
