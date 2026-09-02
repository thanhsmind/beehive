# Worktree and close

Feature work happens in its own git worktree so the user's main checkout stays
clean, and lands through `bee worktree merge`. `bee close` is the separate
question of whether the feature is finished: it reports a list of named doors and
which of them still block.

## Sub-features

- `worktree-new` creates and registers a sibling worktree on its own branch.
- `worktree-list` reports the grants and which are pending a merge.
- `worktree-merge` merges the branch back into main and records the merge.
- `close-dry-run` reports the close doors and runs nothing.
- `close-doors` names each blocking door and what settles it.
- `close-green` reports no blocking doors once every door is settled.

## How to get to it (user POV)

- Run `bee worktree new --feature <slug> --json` from the main checkout.
- Run `bee worktree list --json`.
- Run `bee worktree merge --id <id> --json` from the main checkout.
- Run `bee close --feature <slug> --dry-run --json`, then without `--dry-run`.

## Driving it with control-bee

Preconditions:

- A launched sandbox, `control-bee doctor` fully `ok`.
- A feature started — see [feature-gates](./feature-gates.md).

- **Create the worktree.** Run
  `control-bee cli -- worktree new --feature wt-demo --json`. The payload reports
  `id: "repo--wt--wt-demo"`, a `worktreeRoot` that is a **sibling** of the
  sandbox inside the run dir, and `branch: "wt/wt-demo"`. Confirm with
  `control-bee sh -- ls ..` — the run dir now holds `repo` and
  `repo--wt--wt-demo`.
- **The grant is registered.** Run `control-bee cli -- worktree list --json`. It
  reports `grants` with `"repo--wt--wt-demo": true`, `merged_pending` `false`,
  and a `main_root` pointing at the sandbox.
- **Work inside the worktree.** Aim the harness at it with `VERIFY_CWD`. Run
  `printf 'work\n' | VERIFY_CWD=repo--wt--wt-demo control-bee put WORK.md`, then
  `VERIFY_CWD=repo--wt--wt-demo control-bee sh -- git add -A` and
  `VERIFY_CWD=repo--wt--wt-demo control-bee sh -- git commit -m "add WORK.md"`.
- **Merge it back from main.** Run
  `control-bee cli -- worktree merge --id repo--wt--wt-demo --json` — without
  `VERIFY_CWD`, because a worktree cannot merge itself. The payload reports
  `ok: true`, `merged: true`, the branch, and a `verify` field reading
  `"proven (<N> cell(s))"`, `"unchecked (no capped cells)"` or `"skipped"`. When
  bee committed `.bee` bookkeeping first, a `bookkeeping_commit` object carries
  `committed: true` and its sha.
- **Confirm the merge landed.** Run `control-bee sh -- ls WORK.md` and
  `control-bee sh -- git log --oneline -n 4`. The file exists in the main
  sandbox and the log's newest entry is a merge commit naming the worktree and
  the branch. `control-bee cli -- worktree list --json` now reports
  `merged_pending` `true` for that id.
- **The worktree is KEPT by default.** Teardown runs only when `--cleanup` is
  passed for that merge, or the repo sets `worktree_cleanup_on_merge: true`;
  `--no-cleanup` is an explicit keep and beats both
  (`worktree/handlers.rs:413-437`, where an absent config key reads `false`).
  A non-boolean config value refuses the merge rather than guessing.
- **A merge refuses on recorded debt, before git runs.** Three zero-mutation
  checks fire ahead of `git merge` (`worktree/phases.rs`):
  `WORKTREE_MERGE_PROOF_DEBT` when a capped cell carries no proof line
  (`:219-226`), `WORKTREE_MERGE_DISSENT_DEBT` (`:260-267`), and
  `WORKTREE_MERGE_ADVISOR_NUDGE_DEBT` (`:296-303`). A `standard`/`high-risk`
  feature with an unapproved uat gate refuses too (`:478-488`) unless
  `--skip-uat` is passed or config `uat_before_merge` is `false`.
- **Reclaim dead worktrees.** `control-bee cli -- worktree prune --json` removes
  worktrees whose branch is fully merged and whose tree holds nothing precious;
  every probe fails CLOSED, so an unreadable file keeps the worktree
  (`worktree/prune.rs:769-805`). It must run from MAIN.
- **Close reports its doors without running anything.** Run
  `control-bee cli -- close --feature demo-note --dry-run --json`. The payload is
  a `doors[]` array; each entry has `door`, `blocking` and `detail`. Snapshot
  before and after and confirm the state is unchanged.
- **Count the doors, do not assume them.** A dry run reports TWELVE on a `tiny`
  lane: `tests`, `scribing-debt`, `capture-queue`, `mistakes`, `dissent-debt`,
  `advisor-nudge-debt`, `uat`, `pattern-check`, `knowledge-freshness`, `impact`,
  `routing`, `doc-deferral`. `judge-debt` is a thirteenth, lane-gated to
  `standard`/`high-risk` (`drivers/close.rs:1594`). The builder is
  `build_close_report_doors` (`close.rs:1495-1786`); the remaining doors are
  added by `close_handler` (`close.rs:2403-2413`). A recipe that names only the
  door it cares about goes stale the next time one is added — read the array.
- **A blocking door names its remedy.** With one capped cell and no reflection,
  the `mistakes` door reports `blocking: true` and a `detail` naming
  `bee mailbox reflect`. The `tests` door reports `blocking: false` with
  `1 capped cell(s) all carry a proof line`. On this `small` lane the `uat` door
  reports `blocking: false` with `clear — this lane is exempt from the
  close-time uat door`, whatever the uat gate says.
- **Settle it and close green.** Run
  `control-bee cli -- mailbox reflect --no-mistakes --json`, then
  `control-bee cli -- close --feature demo-note --json`. No entry of `doors[]`
  has `blocking: true`.
- **Proof.** Run `control-bee snapshot closed`. `git-log.txt` shows the merge
  commit, `cells/archive/demo-note/` holds the retired cell, and the `close`
  call's recorded `.out` file shows every door with `blocking: false`.

## Gotchas

- `bee worktree merge` needs `--id`, not `--feature`. The id is the directory
  basename (`repo--wt--<slug>`), and `bee worktree list` is where to read it.
- Merge must run from the main checkout. Running it from inside any linked
  worktree — including the one being merged — is refused.
- bee places worktrees at `../<repo-basename>--wt--<feature>`, outside the
  sandbox repo. `control-bee cleanup` removes the whole run dir for exactly this
  reason; removing only the sandbox would strand them.
- Merge refuses a dirty main or worktree tree. A bootstrapped gitignored `.bee`
  store alone does not count as dirty.
- **The merge-time `uat` refusal almost never fires.** It runs only when
  `.bee/config.json`'s `uat_stop` resolves to `"merge"`
  (`verbs/worktree/phases.rs:432`). The default is `"close"` (`uat.rs:44`), so on
  a stock repo `bee worktree merge` never refuses for an unapproved uat gate —
  with zero capped cells or a hundred. The check is not keyed on capped cells at
  all. To drive that refusal, set `uat_stop: "merge"` in the sandbox's
  `.bee/config.json` first.
- The door list is not fixed. `judge-debt` appears only on `standard` and
  `high-risk` lanes, and the `uat` door appears only where `uat_stop` places it —
  on a `small` lane it reports `clear — this lane is exempt from the close-time
  uat door`. Assert on the doors you drove, never on a door count.
- `bee close`'s `mistakes` door reads the feature's capped cells first
  (`trace.no_mistakes` or a non-empty `trace.mistakes`), and falls back to the
  closing run's mailbox only when cell debt remains. In practice the cell half is
  unreachable from the CLI — see the `--no-mistakes` gotcha in
  [cells-and-proof](./cells-and-proof.md) — so `bee mailbox reflect
  --no-mistakes` is what actually settles it.
- A green non-dry-run close archives the feature's cells into
  `.bee/cells/archive/<feature>/` and auto-commits `.bee/` bookkeeping. Snapshot
  the cells directory before closing if you need its pre-close contents.
- `close --dry-run` claims to run nothing. Prove that by diffing snapshots, not
  by trusting the flag.
