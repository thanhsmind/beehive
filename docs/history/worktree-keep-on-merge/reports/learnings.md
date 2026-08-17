# worktree-keep-on-merge — learnings (2026-08-17)

Feature: flip `bee worktree merge` to keep the worktree by default, queue a
`worktree-cleanup` entry, prune drains and resolves it, `worktree list`
surfaces the pending marker. Cells wkm-1..3, merged to main at 8ae9ffda.

## What settled

- worktree-keep-on-merge D1 (supersedes worktree-reclaim D1) — keep by
  default; `--cleanup` / `worktree_cleanup_on_merge: true` opt back in;
  `--no-cleanup` explicit keep wins over config. Specs synced:
  `returning-and-the-merge-gate.md`, `pruning-dead-worktrees.md`,
  `entering-creating-and-registering.md`.

## Learnings

1. **Shared git index breaks file-disjoint parallelism at commit time.**
   Two workers in one worktree on disjoint files still collided on the
   checkout's single index (staged edits swept into a sibling's commit; one
   edit wiped and redone). Promoted to
   `docs/knowledge/patterns/20260817-parallel-workers-in-one-worktree-share-one-git-index.md`.
   Next swarm in a single worktree: one execution worker at a time, or
   per-worker `GIT_INDEX_FILE`.
2. **The decision-log deferral guard false-positives on filenames.** Logging
   D1 was refused three times because the store's own filename
   (`deferred-queue.jsonl`) contains "defer"; the decision had to name the
   file indirectly. Cheap fix candidate: exempt code-spans/filenames from the
   deferral heuristic.
3. **The old binary's merge still tears down by default** until the next
   release ships this change; `--no-cleanup` was passed explicitly on this
   feature's own merge for that reason. Self-hosting repos see new behavior
   only after `.bee/bin/bee` is rebuilt/released.

## Friction filed

- Deferral-guard false positive (learning 2) — worth a backlog row if it
  recurs; recorded here first.
