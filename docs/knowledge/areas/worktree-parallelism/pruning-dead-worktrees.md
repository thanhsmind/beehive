---
type: bee.area
title: "Worktree Parallelism — pruning: bee worktree prune and the fail-closed dead-worktree classifier"
description: "Why a worktree that never crossed the merge return path needs its own reclaim path, the eight independent conditions every one of which must hold before a worktree is judged dead, why every condition keeps rather than guesses on any doubt, and why liveness reads live session records instead of the workspace record's own ownership fields."
timestamp: 2026-08-06
bee:
  id: worktree-parallelism-pruning-dead-worktrees
  lifecycle: active
  areas: [worktree-parallelism]
  required_context: [areas/worktree-parallelism/returning-and-the-merge-gate.md]
  decisions: [worktree-reclaim D2 (bee worktree prune sweeps what merge never saw), "worktree-reclaim D2a (every classifier probe fails closed; a base ref that does not resolve refuses the whole run rather than guessing at mergedness; three keep-conditions added on review — branch identity/detached HEAD, no interrupted operation, no precious gitignored state)", "worktree-reclaim D2b (liveness comes from session records, not the workspace record's own ownership fields, which are never written for a worktree)", "worktree-reclaim D3/D3a (prune's removal reuses the same shared teardown helper as merge cleanup and unregister — see returning-and-the-merge-gate.md and entering-creating-and-registering.md)"]
  sources: ["docs/history/worktree-reclaim/CONTEXT.md and plan.md (D2, D2a, D2b, wr-2, wr-3)", commit 396b8cb7 (the fail-closed dead-worktree classifier) and commit f290c08d (bee worktree prune), packages/bee-rs/crates/bee/src/verbs/worktree/prune.rs, worktree-reclaim cell wr-5 (the reclaimable count surfaced at orientation and in the session preamble above a floor of one; docs/history/worktree-reclaim/promote-proposals.md)]
  authoritative_for: "worktree-parallelism: bee worktree prune, the fail-closed dead-worktree classifier, and the reclaimable-count surfacing"
---

# Worktree Parallelism — Pruning: `bee worktree prune`

The return path (`returning-and-the-merge-gate.md`) reclaims a worktree only when its owner
comes back through `worktree merge`. A worktree whose branch merged some other way — squashed
elsewhere, cherry-picked in by hand, or simply abandoned mid-feature — never crosses that path,
so it never gets cleaned up, no matter how dead it actually is. `bee worktree prune` is the
second reclaim path: a standalone sweep, run on demand, over every worktree the store knows
about, whether or not it was ever merged through.

## `bee worktree prune` (D2, D5)

`bee worktree prune [--dry-run] [--older-than-days N] [--json]`, run from the ordinary MAIN
checkout only — never from inside a worktree, since prune enumerates and removes *other*
worktrees, and a linked worktree cannot prune itself:

- **Enumerates from the grant registry UNION the workspace-record store**, not the grant
  registry alone. A grant-driven scan can never reach a workspace record whose grant is already
  gone — exactly the shape `worktree unregister`'s old, partial teardown left behind before
  D3/D3a: grant dropped, record orphaned. Unioning the two closes that gap for every record
  already stranded, not just the ones created after the fix.
- **Measures every worktree against one base commit, resolved once per run**, from the main
  checkout's own current branch. A detached main HEAD refuses the whole run outright — prune
  needs a named branch to measure mergedness against, and it will not guess one.
- Classifies each enumerated worktree dead-or-kept (below), then removes every one classified
  dead: worktree directory, branch, grant, and workspace record, through the same shared
  teardown helper `worktree merge` cleanup and `worktree unregister` both call.
- **`--dry-run` classifies and reports; it removes nothing** — no lock, no git mutation, no
  registry write. The report still names the reclaimable size per worktree, so a dry run answers
  the question a human actually has ("how much would this get back") without touching anything.
- **The report is one line per worktree, naming why**: removed and how much was reclaimed, kept
  and which condition kept it, or (dry run) how much is reclaimable and why it would go. A
  worktree whose branch-delete step fails after its directory is already gone is reported as
  *removed, with a caveat* — the directory and its reflog are already gone, so calling that
  "kept" would be a lie.
- **`--older-than-days` overrides the default age threshold (7 days)** for one run; there is no
  config key behind it, and no lock bookkeeping of its own is needed to honour a permanent
  opt-out — `git worktree lock` already is one: `git worktree remove --force`, prune's first
  removal step, refuses outright on a locked tree. Prune's own report names locking as the way
  to keep a worktree out of every future sweep, not just this one.

## The classifier: every condition keeps on any doubt (D2, D2a, D2b)

A worktree is judged **dead** only when every one of eight independent conditions holds. Any one
of them failing to confirm — a missing file, an unreadable record, a failed git command, an
unparseable value, a real divergence — is not evidence the worktree is safe to keep, and it is
never read as permission to remove it. It is a reason to **keep**, full stop, and the report
names which condition did it. Deletion has no retry, so the classifier is built the opposite way
from most of this codebase's guards, whose failure mode is "the real gate runs again in a
moment, try again" — here, the failure mode is permanent, so every probe fails **closed**.

| # | Condition | Why it exists |
|---|---|---|
| 0 | The workspace record is readable, and names a real branch and a real root | The classifier's first move is establishing what worktree it is even looking at. An unreadable or missing record, a branch field that reads `null` — exactly what a freshly re-registered worktree looks like — or a missing root all keep, because nothing after this point can be trusted once the record itself cannot be. |
| 1 | The branch is provably merged into the freshly resolved base commit | Checked as an ancestor test whose only two outcomes are "yes" and "not provably yes" — a real divergence and a git failure read identically, both as "not merged." An earlier count-based shape read a git failure as the literal empty string, which parsed to zero — "merged," for every worktree in the run, at once. That is a failure that reads as permission; this replaces it with one that reads as a keep. |
| 2 | The worktree's actual checked-out branch is the one the record claims | Confirmed live, off the worktree itself, never off the stored record alone. A **detached HEAD** keeps outright: it is the condition standing between prune and a permanent, silent loss. `git worktree remove` runs BEFORE `git branch -d` in the removal sequence, which means `branch -d`'s safety promise — refusing to delete a branch carrying unmerged commits — only ever protects commits that are actually *on* the branch ref. A commit made while the worktree sat on a detached HEAD is on no ref at all; its only history is a reflog file scoped to that one worktree, deleted the instant the directory goes, and immediately eligible for git's ordinary garbage collection after that. A clean tree and a "merged" branch both look perfectly safe on top of that loss, so detached HEAD gets its own explicit keep-condition rather than being folded into "merged." A mismatch between the recorded branch and what is actually checked out also keeps, logged as the reason rather than resolved either way. |
| 3 | The tree is clean — no tracked-modified or untracked files | The same clean-tree check the return path already uses. A failed check keeps rather than guesses the tree is clean. |
| 4 | Nothing precious is sitting only in the gitignored part of the tree | The clean-tree check above runs `git status --porcelain` without `--ignored`, deliberately — the same definition of "dirty" the return path uses. That means anything gitignored is invisible to it, and this store keeps two things there on purpose: a paused session handoff, and a queue of not-yet-promoted capture entries. Either one present keeps, checked directly off the filesystem rather than trusted to the clean-tree check, because both are exactly the kind of state someone would be furious to lose without a word of warning. |
| 5 | No git operation is mid-flight | A rebase, cherry-pick, merge, or bisect left in progress in the worktree's own git administrative state keeps, hard — an interrupted operation has no business being deleted out from under it. |
| 6 | No live session holds or is attached to the worktree | Read from session records kept at the main checkout, never from the workspace record's own ownership fields. Every workspace record — worktree or main — carries fields meant to answer exactly this question, but the only code path that ever writes them hardcodes the answer `"main"` for every checkout, always. On a worktree those fields are therefore never anything but empty, including for the one worktree that matters most: the one a session is genuinely working inside right now. Trusting them would mean prune could never tell a truly idle worktree from an occupied one, so liveness is answered by scanning session records instead and asking whether any live one either names this workspace directly, or resolves to a location that sits under this worktree's own root, with a heartbeat still fresh inside a liveness window sized in hours — long enough that a gate answer, a review, or a closed laptop lid does not read as dead, unlike the much shorter staleness window used elsewhere for retryable claims. An unreadable session record, or one with no readable heartbeat at all, counts as live: this is a deletion, not a retry, so the scan cannot rule a session out and must not skip it. |
| 7 | The last commit is old enough | A worktree can be merged, clean, and session-free, and still be one its owner means to return to tomorrow. The age threshold defaults to a week and is overridable per run; a commit date that cannot be read or does not parse keeps rather than guesses its own age. |

Only when every one of these eight clears does a worktree count as dead. Reaching the end of the
list with no keep-reason found is the *only* way to answer dead — there is no separate "looks
safe enough" shortcut, with one deliberate exception: the orphan verdict below, which answers
dead a different way because there is nothing left in either artifact to ask the eight conditions
about.

## The orphan verdict: when the record is the only thing left (wov-1)

A worktree whose directory is gone AND whose branch is gone is not "unmerged" — it is not
anything: there is no tree to check for cleanliness, no branch for `merge-base --is-ancestor` to
even name, no admin dir to hold an interrupted operation. Condition 1 (merged into base) would
misread this shape forever: `git merge-base --is-ancestor` can never exit `0` for a branch that
does not exist, so a plain "not provably merged" keep would hold onto a bare workspace record
until the end of time, never confirming and never releasing it.

`classify_worktree` checks this conjunction — directory absent (`worktree_root.exists()` is
false) AND branch absent (`branch_exists`, `worktree/git.rs`) — BEFORE condition 1 runs, and
answers dead immediately when both hold, with no ancestry probe asked or needed: the workspace
record is the only artifact left, so there is no directory to remove, no commits to strand, and
no ignored files to lose. Either artifact alone still keeps, through the ordinary path below it:

- directory gone, branch still real — falls through to condition 2 (detached HEAD), since reading
  `HEAD` from a directory that is not there fails the same way a detached HEAD does; the branch
  may carry commits no other ref protects, so this keeps.
- directory still standing, branch gone — falls through to condition 1 itself, which reads a
  branch that does not exist the same way it reads a real divergence: not provably merged, so this
  keeps; the tree may hold uncommitted or ignored work the branch never saw.

`run_prune_core`'s removal step reads `Verdict::Dead`'s own `orphan` flag to skip `git worktree
remove`/`git branch -d` entirely for this verdict — running either against a target that was
never there would fail and the record would come back *kept*, not removed. It drops straight to
the registry-only teardown (`teardown_worktree(.., None)`, `merge.rs`) that `run_unregister`
already uses: the grant, the workspace record, and the holds all release; nothing git-shaped runs.

## Nobody runs a verb they never hear about (wr-5)

A reclaim path that has to be remembered is a reclaim path that does not run.
Orientation therefore names the count itself: when more than one worktree is
reclaimable — merged, clean, and idle past the age threshold — both the
orientation report and the session-start briefing carry one line stating how
many there are and naming the dry run as the way to see what would go. It is a
report and nothing more: it refuses nothing, blocks nothing, and never removes
anything.

The floor is deliberate. At zero the line would be noise; at exactly one it
would be nagging about a single stale directory that costs nothing to leave.
The line appears from two upward, where the count itself is the signal that the
reclaim path has stopped being used at all. The count is cheap by
construction — the grant registry plus one metadata check per candidate, with no
repository command and no directory walk — so orientation pays for it on every
call without noticing.

## Pointers (implementation)

- Reclaimable-count surfacing (wr-5): `reclaimable_worktree_ids` and the shared
  floor constant `RECLAIMABLE_WORKTREES_SHOWN_FLOOR` (1) in
  `packages/bee-rs/crates/bee/src/verbs/status_full/mod.rs:104`; the orientation
  blocker line at `verbs/status_full/orient.rs:252-258` and the preamble section
  at `hooks/session_preamble/render.rs:337-346`, wired in
  `session_preamble/budget.rs:581-585`. Both read the one scan and share the one
  floor, so the two surfaces cannot disagree. Tests:
  `session_preamble/tests.rs:357` (two or more name the count and the command)
  and `tests.rs:385` (a single reclaimable worktree, and a too-young one, stay
  silent).
- The classifier: `classify_worktree`, `PruneCheck`, `Verdict` in
  `packages/bee-rs/crates/bee/src/verbs/worktree/prune.rs`. Evidence: commit 396b8cb7.
- The subcommand: `run_prune` / `run_prune_core` in the same file, wired at
  `verbs/worktree/handlers.rs`'s command dispatch. Evidence: commit f290c08d.
- The shared teardown removal path prune calls into: `teardown_worktree` in
  `verbs/worktree/merge.rs` — see `returning-and-the-merge-gate.md` and
  `entering-creating-and-registering.md` for its other two callers.
