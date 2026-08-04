# Native worktree integration — the transaction, the threat model, and the attestation

The acceptance procedure owned by cell `worktree-isolation-4` (see `plan.md` in this
directory). It lived in `skills/bee-swarming/references/swarming-reference.md` until the
doctrine diet moved it here: no hook, no test, and no gate reviewer checks any of it —
the typed halt names below appear in no source file — so it is a specification for work
that is not built yet, not a rule an orchestrator obeys on every wave. Every word is
preserved; nothing was rewritten in the move.

## Native Worktree Integration Transaction

This is the orchestrator-owned goal-check for an eligible Claude Code native
worktree wave. It is a consistency and recovery protocol, not a security
boundary: a same-UID worker is cooperative but fallible, worker-reported Git
identity is informational, and Git metadata is evidence rather than authority.
Normal eligibility remains an opted-in wave of at least two workers. The sole
one-worker exception is the post-enablement `worktree-isolation-4` acceptance,
and its serialized prerequisites (`worktree-isolation-1` →
`worktree-isolation-2` → `worktree-isolation-3`) must already be capped.

### Protected pre-dispatch record

Before dispatch — before worker output or a worker result can exist — record the
main checkout's pre-main SHA and a control-plane attestation outside the worker's
editable worktree:

- canonical `commonDir`, canonical `worktreePath`, and metadata-derived
  `worktreeId`;
- the initial symbolic `headRef` (detached HEAD is ineligible) and `baseCommit`;
- normalized cell `declaredPaths` and the actually held `reservedPaths`.

If the runtime cannot capture and retain this record, halt with
`WORKTREE_ATTESTATION_UNAVAILABLE`; it is ineligible for worktree mode. Never
accept a branch, base, path, id, or candidate supplied only by worker text.

### Re-attest before integration

After `[DONE]`, derive the candidate from the protected worktree id and fresh Git
metadata. Re-resolve the canonical common dir and worktree path, validate the
metadata backlink, require the same symbolic ref, and reject detached HEAD. Any
identity or backlink mismatch halts as `WORKTREE_IDENTITY_MISMATCH`. Then run
`git merge-base --is-ancestor <baseCommit> <candidate>`; failure is
`WORKTREE_BASE_ANCESTRY_MISMATCH`. Finally obtain
`git diff --name-only <baseCommit>..<candidate>`, apply the same logical path
normalization used by reservations, and require the result to be a subset of
the attested `reservedPaths`; an extra path is
`WORKTREE_RESERVED_DIFF_MISMATCH`.

Every typed halt preserves the worktree, branch/ref, candidate commit, and
attestation. The orchestrator does not reinterpret a worker's result wording to
continue.

### Merge, verify, and provenance

From the attested main checkout, capture `pwd` and pre-main HEAD, then run exactly
`git merge --no-ff --no-commit <candidate>`. On a merge conflict, run
`git merge --abort`, prove HEAD still equals pre-main HEAD, and preserve the
worker recovery state. Run the cell's targeted checks while the merge is
uncommitted; on targeted red, run `git merge --abort` and again prove main
history still equals pre-main HEAD. Only green targeted checks permit the merge
commit.

On committed main, capture this provenance as one attributable record:

- `pwd`;
- pre-main HEAD and post-main HEAD;
- merged-commit ancestry (`git merge-base --is-ancestor <candidate> <post-main>`);
- the exact full repository verify command;
- full verify output and exit status.

Run that exact full repository verification only from the committed main
checkout. An unexpected post-commit red immediately runs
`git revert -m 1 --no-edit <post-main>` before any later work. Record the new
revert commit, confirm main is no longer carrying the merge's changes, and
preserve the worker worktree/ref. Revert is non-destructive: never reset or
rewrite main history to hide the failed merge.

### Conservative disposition and cleanup

Automatic cleanup is a conjunction, not a best-effort tail. Immediately before
cleanup, require worker `git status --porcelain` to be empty, the recorded
committed-main full verify to be green, and
`git merge-base --is-ancestor <candidate> <main-head>` to prove the candidate is
reachable. Only then use the non-force commands
`git worktree remove <worktreePath>` followed by `git branch -d <headRef>`.
Failure of either command preserves whatever recovery identity remains and is
reported; it never falls through to a force variant.

`[BLOCKED]`, `[HANDOFF]`, abandonment, identity mismatch, merge conflict,
targeted or full red verification, post-commit revert, and any incomplete or
unknown outcome all suppress automatic cleanup. They preserve the worktree,
symbolic ref/branch, HEAD, candidate, attestation, and the reason integration
stopped. A feature close, capped cell, worker log, timeout, or absent process is
not cleanup authorization.

### Explicit destructive drop

A destructive drop is a separate operator action, never an automatic recovery
step. Before asking for explicit operator authorization, record the current
status, dirty/untracked diff, HEAD, candidate reachability from main, and a
recovery ref or patch stored outside the worktree being dropped. The approval
must identify that captured recovery artifact and the exact worktree/ref to
destroy. Without both explicit operator authorization and successful recovery
capture, preserve everything. Even with approval, report the resulting recovery
identity; a force removal or branch deletion must never appear in the automatic
cleanup path above.

Acceptance tests use deterministic temporary Git repositories to inject identity
mismatch, out-of-scope diff, merge conflict, targeted red, post-commit full red,
`[BLOCKED]`, `[HANDOFF]`, abandonment, cleanup suppression, and revert behavior.
No live checkout is used as a fault-injection target.


## Threat model and protected attestation

A same-UID worker is cooperative and fallible, not a security principal. Git
metadata is consistency evidence, never independent authorization or a security
boundary against that worker. Worker-reported id, branch, base, path, and commit
are informational only; the orchestrator derives the candidate from the protected
attestation and freshly read Git metadata.

After `[DONE]` and before any merge, re-resolve the attested worktree and require:

1. canonical path, native id, `commonDir`, forward link/backlink, and symbolic
   `headRef` still match the attestation. A detached HEAD returns
   `WORKTREE_IDENTITY_MISMATCH`; any path/id/common-dir/ref/backlink mismatch also
   returns `WORKTREE_IDENTITY_MISMATCH`.
2. the candidate is the freshly read worktree HEAD and
   `git merge-base --is-ancestor <baseCommit> <candidate>` succeeds. A
   non-descendant returns `WORKTREE_BASE_ANCESTRY_MISMATCH`.
3. the NUL-delimited `git diff --name-only <baseCommit>..<candidate>` is a subset
   of attested `reservedPaths` after the same logical normalization used by
   reservations. Any extra path returns `WORKTREE_RESERVED_DIFF_MISMATCH`.

These are typed identity halts: stop integration, preserve the worktree and
branch, and never reinterpret worker result wording as authority. Transactional
merge, verification, revert, cleanup, and destructive-drop disposition remain the
acceptance procedure owned by `worktree-isolation-4` and the swarming reference.
