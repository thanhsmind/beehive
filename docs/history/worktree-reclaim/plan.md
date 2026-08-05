# worktree-reclaim — work shape

Lane `high-risk` · class `feature` · flags `public-contracts`,
`covered-contract-change`, `multi-domain`, `cross-platform` · 7 product files.

Locked decisions: [CONTEXT.md](CONTEXT.md) D1–D4, amended by D1a, D2a, D2b, D3a,
D4a and D5 in `.bee/decisions.jsonl` (tag `worktree-reclaim`). Baseline on
`wt/worktree-reclaim`: `cargo test --release` → `1257 passed, 3 ignored`.

## What the review pass changed

The first draft argued that `git branch -d` made the worst case a rebuild of
`target/`. Two review passes proved that false, and the plan below is the
answer to them, not a patch on top:

- `git worktree remove --force` runs **before** `branch -d`
  (`merge.rs:307` then `:318`). `branch -d` protects commits *on the branch*.
  Commits on a detached HEAD have no ref, and their only reflog is
  `.git/worktrees/<id>/logs/HEAD` — deleted with the directory, immediately
  gc-eligible. That is permanent loss, and it is reachable through a clean tree
  and a merged branch.
- The liveness condition was **inert**. All 14 workspace records carry
  `write_owner_session: null` and `attached_sessions: []`, including the
  worktree a live session was running in, because the only writer hardcodes
  `workspace_id = "main"` (`state_group/policy.rs:128`).
- "Clean" is `git status --porcelain` without `--ignored`, by decision D8a
  (`merge.rs:54`). The ignored set here includes `.bee/HANDOFF.json`,
  `.bee/capture-queue.jsonl`, `.bee/state.json` and any local `.env` —
  no object, no reflog, gone forever.
- A parsed `rev-list --count` fails **open**: any git failure reads as `0`,
  i.e. "merged", for every worktree at once.

So the shape is: build the classifier as a fail-closed instrument first, prove
each keep-reason, and only then let anything act on it.

## Smaller path check

*Is there a cheaper shape that still honors every locked decision?*

Considered folding the classifier into the prune subcommand. **FAIL** — the
classifier is the entire safety argument; a cell that ships a subcommand and a
classifier together has no red-first moment for the eight keep-reasons.

Considered skipping the shared teardown helper. **FAIL** — D3 needs the
registry teardown from `unregister`, which has no merge; three callers copying
five removal steps is how the workspace record got orphaned.

Considered dropping the wr-0 test pins. **FAIL** — three of the four
`perform_cleanup` outcome shapes are unpinned, so a lift that collapses their
key order lands green.

## Slice 1 — reclaim, end to end

### wr-0 — pin what the lift can break

`perform_cleanup` builds five outcome shapes with four distinct key orders by
insertion (`merge.rs:288, :296, :312, :320, :344`). Exactly one is pinned
(`tests.rs:217`, the check-failed branch — a bare tempdir). Add pins for the
dirty branch (`ok, code, reason, status`), the remove-failed branch, and the
branch-delete-failed branch, whose `removed: true` sits in a middle slot no
uniform formatter would reproduce.

Pure test cell. Proof: three new tests green against unmodified source.

Files: `verbs/worktree/tests.rs`.

### wr-1 — one teardown, explicit removal (D3, D3a)

Lift the **five** steps out of `perform_cleanup` — directory, branch, grant,
workspace record, and `release_all_for_holder` (`merge.rs:342`), which the
first draft miscounted as four — into one helper. Directory removal is an
explicit non-default parameter, and the helper asserts the current directory is
not inside the tree it removes. `run_unregister` (`registry.rs:218`) reaches
the registry half only: grant, workspace record, holds release, never the
directory.

The `Map` construction stays in `perform_cleanup`; only the side-effect calls
move. That is the constraint that keeps wr-0's key orders intact.

Proof: unregister leaves no `.bee/runtime/workspaces/<id>.json`; wr-0's pins
and `tests.rs:217`/`:228` stay green unchanged.

Files: `verbs/worktree/merge.rs`, `verbs/worktree/registry.rs`,
`verbs/worktree/tests.rs`.

### wr-2 — the fail-closed classifier (D2, D2a, D2b)

Its own module, no subcommand yet. Given a granted id it returns dead-with-
reason or kept-with-reason, and **every unknown keeps**:

| Condition | Test | Fails closed by |
|---|---|---|
| merged into base | `git merge-base --is-ancestor <branch> <base>` | `status != Some(0)` is *not merged*; the run refuses outright if the base ref does not resolve (`git.rs:99` already returns `None`) |
| branch is real | `current_branch(worktree_root)` (`merge.rs:148`) equals the record's `branch` | `None` (detached HEAD) keeps; disagreement keeps and logs it |
| tree clean | `status --porcelain` | non-zero exit keeps, as `perform_cleanup` already does (`merge.rs:286`) |
| nothing precious ignored | `.bee/HANDOFF.json` absent, `.bee/capture-queue.jsonl` empty or absent | either present keeps, hard |
| no interrupted operation | no `rebase-merge`, `rebase-apply`, `CHERRY_PICK_HEAD`, `MERGE_HEAD`, `BISECT_LOG` in `.git/worktrees/<id>/` | any present keeps |
| no live session | session records under main naming this workspace with a fresh heartbeat, **or** a live session's root under the worktree root | unreadable record counts as **live**; window is `PRUNE_LIVENESS_SECONDS`, hours not the 15-minute `HEARTBEAT_STALE_SECONDS` |
| old enough | last commit older than the age threshold | unreadable date keeps |

Reuse, do not rewrite: `merge-base --is-ancestor` already runs at
`status_full/records.rs:625` and `reviews.rs:626` — the worktree crate needs
its own thin call through `worktree/git.rs:79` `run_git`, not a third
implementation of the idea.

Proof: one test per keep-reason plus the dead case; a test that a failing git
binary keeps everything.

Files: new `verbs/worktree/prune.rs`, `verbs/worktree/tests.rs`.

### wr-3 — `bee worktree prune` (D2, D5)

The subcommand over wr-2's classifier, routed at `handlers.rs:661`:
`bee worktree prune [--dry-run] [--older-than-days N] [--json]`. One line per
worktree with its reason. `--dry-run` removes nothing. The removal line names
the reclaimed size — this is where size is paid for (D4a) — and names
`git worktree lock` as the permanent, git-enforced opt-out: a locked tree
refuses `remove -f`.

Enumerate from grants **and** from `.bee/runtime/workspaces/`, so a grantless
orphan record is reachable — CONTEXT's out-of-scope line promises those 13 get
cleared, and a grant-driven scan never visits them.

Payload edit per D5: hand-edit `generated/registry_payload.json`, reason
recorded. The example must be inert and must succeed outside a git repo —
`tests/registry_dispatch.rs:131` executes it in a non-git scratch dir. Model it
on `worktree merge`'s deliberately inert `--id demo-feature-missing`.

Files: `verbs/worktree/prune.rs`, `verbs/worktree/handlers.rs`,
`verbs/worktree/mod.rs`, `verbs/worktree/tests.rs`,
`generated/registry_payload.json`.

### wr-4 — cleanup by default (D1, D1a)

Cleanup runs unless `--no-cleanup` is passed or `.bee/config.json` sets
`worktree_cleanup_on_merge: false` (the `close.rs:827` pattern: absent or
non-`false` means on). `--cleanup` stays accepted, does nothing.

Three things the first draft missed, each a defect if skipped:

- `--no-cleanup` must join `keys_known(&flags, &["id", "cleanup",
  "queue-wait-ms"])` (`handlers.rs:386`) or the whole merge returns `None`.
- A non-boolean value on `--no-cleanup` is **refused**, not ignored and not
  delegated (D1a). Today a mis-parsed flag fails safe; after the flip it fails
  destructive.
- `handlers.rs:475` gates a holds-ledger read on `cleanup`, and returns `None`
  on a corrupt ledger — under the flip that fires on every merge and lands on
  the router's refusal. Its comment ("only `--cleanup` can reach it") becomes a
  lie either way; fix the branch and the comment together.

The `ALREADY_UP_TO_DATE` arm (`phases.rs:240`) keeps today's behavior and
removes nothing (D1a).

Text surfaces that move with the behavior: `cleanup_suggested_command`
(`merge.rs:368`), the usage line (`mod.rs:9`), the `worktree.merge` description
in the payload, a `worktree_cleanup_on_merge` row in
`docs/config-reference.md` after `:164`, and the key in
`.bee/config-sample.json`.

Proof: merge without flags cleans up; `--no-cleanup` leaves the tree standing;
config off beats the absent flag; `--no-cleanup=yes` exits non-zero having
removed nothing; the up-to-date merge removes nothing.
`tests.rs:228` is renamed to what it now tests, not deleted.

Files: `verbs/worktree/handlers.rs`, `verbs/worktree/merge.rs`,
`verbs/worktree/mod.rs`, `verbs/worktree/tests.rs`,
`generated/registry_payload.json`, `docs/config-reference.md`,
`.bee/config-sample.json`.

### wr-5 — the leak announces itself, cheaply (D4, D4a)

The preamble runs **no git** at session start, and no size walk. Count granted
ids whose directory exists and whose grant is older than the threshold —
`read_dir` and `metadata`, the same cost shape as
`unapplied_promote_proposals` (`status_full/mod.rs:221`). The line names the
count and `bee worktree prune --dry-run`; prune itself prints the size.

Two constraints the cell must respect: the preamble holds ~279 bytes of
headroom against `PREAMBLE_BUDGET_BYTES = 5120` (`budget.rs:61`), pinned by
`the_preamble_stays_inside_its_budget_however_big_the_store_gets`
(`tests.rs:360`); and this repo already lost the session-start git fight once —
`orient` 850ms with the git-candidate file, 280ms without (`records.rs:477`).

Wired at `status_full/orient.rs:217-255` (`blockers[]`) and
`session_preamble/budget.rs:573`.

Files: `verbs/status_full/mod.rs`, `verbs/status_full/orient.rs`,
`hooks/session_preamble/render.rs`, `hooks/session_preamble/budget.rs`,
`hooks/session_preamble/tests.rs`.

## Order and overlap

Serial, for a named reason: wr-3 and wr-4 both write
`verbs/worktree/handlers.rs` and `generated/registry_payload.json`, and wr-1
through wr-3 build on each other.

`wr-0 → wr-1 → wr-2 → wr-3 → wr-4 → wr-5`

## Later slices — headlines only

- Sync `docs/knowledge/areas/worktree-parallelism/` — the merge-gate doc
  (`returning-and-the-merge-gate.md:84-99`) states the opt-in semantics D1
  replaces, and D8a's clean-tree scope now has a documented blind spot. Runs
  through `bee-capturing`, not as a cell.
- Persist the base **ref name** in the workspace record at create — `base_ref`
  is in hand at `create.rs:230` and dropped (`create.rs:285`). It would replace
  wr-2's base guess with a recorded fact. Its own feature.
- Share one cargo `target/` across worktrees — cuts the 4.5 GB at its source,
  changes build isolation. Explicitly out of scope (CONTEXT.md).

## Cost if the shape is wrong

A prune that removes work someone wanted. The guard is not review and not
`branch -d` — it is the classifier's eight conditions, every one of which keeps
on doubt. The cheap failure is the mirror: a threshold so shy nothing is ever
reclaimed, which costs one number and a second look at this file.
