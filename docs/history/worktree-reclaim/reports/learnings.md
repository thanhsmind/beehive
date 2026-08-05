# worktree-reclaim — learnings

## A guard's failure direction is set by what failure costs

The worktree module had a documented fail-open convention: a spawn failure
yields a null status, so the guard passes (`worktree/git.rs:113`). That is
correct for the guards it was written for — their failure mode is a refusal and
a retry. This feature reused the same probes for **deletion**, where failure has
no retry. The first draft inherited the convention without noticing the change
of stakes, and a parsed `rev-list --count` would have turned any git failure
into "merged" for every worktree at once.

The rule that came out of it: a probe's failure direction belongs to the
consequence, not to the module. Copying a probe copies its failure direction —
re-derive it at every new call site.

## `branch -d` is not a safety net for a tree

The plan's original cost argument was "git refuses to delete an unmerged branch,
so the worst case is a rebuild of `target/`". Three facts break it:

- `git worktree remove --force` runs **before** `git branch -d`, so the
  directory is gone before the branch gets a vote.
- `branch -d` protects commits *on the branch*. Commits on a detached HEAD have
  no ref, and their only reflog lives in `.git/worktrees/<id>/logs/HEAD` —
  removed with the directory and immediately gc-eligible.
- `git status --porcelain` does not list ignored files. `.bee/HANDOFF.json`, the
  capture queue and any local `.env` are invisible to the clean-tree test and
  unrecoverable after it passes.

## A field that is never written is not a signal

D2's original liveness test read `write_owner_session` and `attached_sessions`
from the workspace record. Every one of the 14 records on disk carried `null`
and `[]` — including the worktree a live session was running inside — because
the only production writer hardcodes `workspace_id = "main"`
(`state_group/policy.rs:128`). The condition would have been silently inert, and
`--dry-run` would have printed the same wrong reason.

Before trusting a stored field as a signal, find its writer and read what it
actually writes.

## Fail-closed has a cost, and the cost must get its own verdict

The shipped classifier keeps all 13 orphan workspace records forever: their
branches were removed by hand, and an ancestry probe cannot exit 0 for a branch
that does not exist, so the fail-closed rule reads "not provably merged" and
keeps them. The rule is right; the gap is that the orphan case — directory gone
**and** branch gone, so the record is the only artifact left — was never given a
verdict of its own.

A fail-closed default converts every unmodelled case into "keep forever".
Enumerate the cases that keep, and check each one is a case you meant.

## Session-start cost is a budget someone already paid for

D4 asked for a preamble line naming the reclaimable size. That would have put N
git spawns and a walk of 4.5 GB on the session-start path — the exact regression
this repo had already measured and fixed (`orient` 850ms with the git-candidate
file, 280ms with it emptied, `status_full/records.rs:477`), against ~279 bytes
of headroom in a 5120-byte budget. The count and the command carry the whole
message; the size is paid for inside `prune`, where the user asked for it.

## Orchestrating a worktree feature costs a checkout dance

`bee dispatch prepare`, `decisions log` and `state set` all refuse inside a
granted worktree, while a dispatched subagent inherits the session's working
directory and the write guard refuses cross-worktree writes. One wasted dispatch
proved it. The working sequence per cell is: prepare and claim from main, enter
the worktree, spawn the worker, exit back to main. A granted worktree also ships
no `.bee/bin/bee`, so workers must call the main checkout's binary by absolute
path.
