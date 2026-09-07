# Worktree and close

Feature work happens in its own git worktree so the user's main checkout stays
clean, and lands through `bee worktree merge`. `bee close` is the separate
question of whether the feature is finished: it reports a list of named doors and
which of them still block.

## Sub-features

- `worktree-new` creates and registers a sibling worktree on its own branch, emitting enter transition intent.
- `worktree-enter` enters an existing verified and granted worktree with zero mutation, emitting enter transition intent.
- `worktree-linked-exit` emits zero-mutation exit-before-merge transition intent with typed continuation when merge runs inside a worktree.
- `worktree-relocate-pi` relocates the active Pi session across worktree boundaries without mutating process cwd.
- `worktree-relocate-recovery` handles post-exit merge refusal in Pi, keeping the worktree and reporting the re-entry command.
- `worktree-list` reports the grants and which are pending a merge.
- `worktree-merge` merges the branch back into main and records the merge.
- `close-dry-run` reports the close doors and runs nothing.
- `close-doors` names each blocking door and what settles it.
- `close-green` reports no blocking doors once every door is settled.

## How to get to it (user POV)

- Run `bee worktree new --feature <slug> --json` from the main checkout.
- Run `bee worktree enter --id <id> --json` from the main checkout.
- Run `bee worktree list --json`.
- Inside a linked worktree, run `bee worktree merge [--id <id>] --json` to emit exit intent before merge.
- Run `bee worktree merge --id <id> --json` from the main checkout to execute the merge.
- In Pi, run `/bee-worktree-new --feature <slug>`, `/bee-worktree-enter --id <id>`, or `/bee-worktree-merge [--id <id>]`.
- In Pi, run ordinary CLI `bee worktree new` or `bee worktree merge` in shell; the Pi belt intercepts `@@BEE_SESSION_TRANSITION@@` and relocates the session automatically after `agent_settled`.
- Run `bee close --feature <slug> --dry-run --json`, then without `--dry-run`.

## Driving it with control-bee

Preconditions:

- A launched sandbox, `control-bee doctor` fully `ok`.
- A feature started — see [feature-gates](./feature-gates.md).

- **Create the worktree.** Run
  `control-bee cli -- worktree new --feature wt-demo --json`. The payload reports
  `id: "repo--wt--wt-demo"`, a `worktreeRoot` that is a **sibling** of the
  sandbox inside the run dir, `branch: "wt/wt-demo"`, and a `sessionTransition`
  object with `operation: "enter-worktree"`, canonical `sourceCwd: main_root`,
  `targetCwd: worktreeRoot`, and `continuation: null`. When `PI_SESSION_ID` is set,
  `@@BEE_SESSION_TRANSITION@@` is emitted on stderr. Confirm with
  `control-bee sh -- ls ..` — the run dir now holds `repo` and
  `repo--wt--wt-demo`.
- **The grant is registered.** Run `control-bee cli -- worktree list --json`. It
  reports `grants` with `"repo--wt--wt-demo": true`, `merged_pending` `false`,
  and a `main_root` pointing at the sandbox.
- **Enter an existing worktree.** Run
  `control-bee cli -- worktree enter --id repo--wt--wt-demo --json`. The payload
  reports `ok: true`, `id: "repo--wt--wt-demo"`, and identical `sessionTransition`
  metadata. Confirm zero mutation: `control-bee sh -- git status --porcelain` and
  `VERIFY_CWD=repo--wt--wt-demo control-bee sh -- git status --porcelain` both
  remain completely clean.
- **Work inside the worktree.** Aim the harness at it with `VERIFY_CWD`. Run
  `printf 'work\n' | VERIFY_CWD=repo--wt--wt-demo control-bee put WORK.md`, then
  `VERIFY_CWD=repo--wt--wt-demo control-bee sh -- git add -A` and
  `VERIFY_CWD=repo--wt--wt-demo control-bee sh -- git commit -m "add WORK.md"`.
- **Exit intent from linked worktree merge.** Run
  `VERIFY_CWD=repo--wt--wt-demo control-bee cli -- worktree merge --json`
  inside the worktree (omitted `--id` defaults to current verified id). The payload
  reports `ok: true`, and `sessionTransition` with `operation: "exit-worktree-before-merge"`,
  canonical `sourceCwd: worktreeRoot`, `targetCwd: main_root`, and typed
  `continuation: { operation: "merge-worktree", noCleanup: false, skipUat: false, queueWaitMs: null }`.
  Confirm zero mutation on exit: `control-bee sh -- ls WORK.md` fails (unmerged),
  and `control-bee sh -- ls ../repo--wt--wt-demo` proves the worktree is intact and kept.
- **Merge it back from main.** Run
  `control-bee cli -- worktree merge --id repo--wt--wt-demo --json` — from the
  main checkout. The payload reports
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
- **Merge refusal recovery names re-entry in Pi.** In Pi, when a post-exit
  merge reconstructed on main refuses (due to uncommitted dirt, recorded proof
  debt, dissent debt, or an unapproved uat gate), the session remains on main,
  the worktree remains kept and granted, and the notification explicitly names
  the recovery re-entry command: `/bee-worktree-enter --id <id>`. Direct CLI
  `bee worktree merge` invocations from main refuse with standard typed error
  payloads without re-entry formatting; the operator or agent uses
  `bee worktree enter --id <id>` directly to return to the kept worktree.
- **Pi session relocation driving and fallback.** In a live Pi session with
  interactive TUI context (`ExtensionCommandContext`), direct commands
  (`/bee-worktree-new`, `/bee-worktree-enter`, `/bee-worktree-merge`) and
  deferred agent transitions fork the active conversation via
  `SessionManager.forkFrom` and switch via `ctx.switchSession`, preserving
  history without changing `process.cwd()`.
  Interactive extension command execution requires an interactive TUI context.
  If interactive TUI automation is absent in the driving environment (e.g. headless
  CI or non-interactive batch runners), record this environment limitation and
  use the real-extension Node contract suite (`cargo test --test pi_plugin_contracts`)
  as the explicit fallback. Never label that fallback `green:live` (it is `green:unit`).
- **Proof.** Run `control-bee snapshot closed`. `git-log.txt` shows the merge
  commit, `cells/archive/demo-note/` holds the retired cell, and the `close`
  call's recorded `.out` file shows every door with `blocking: false`.

## Gotchas

- `bee worktree merge` needs `--id`, not `--feature`. The id is the directory
  basename (`repo--wt--<slug>`), and `bee worktree list` is where to read it.
- `bee worktree merge` inside a linked worktree does NOT merge in place. It
  emits zero-mutation `exit-worktree-before-merge` transition intent with typed continuation
  parameters (`noCleanup`, `skipUat`, `queueWaitMs`). The merge itself executes
  only from main (or is reconstructed by the Pi belt after replacement on main).
- `bee worktree enter` is main-only. Invoking it inside any linked worktree
  refuses untyped, naming the checkout kind it was run from.
- Exit before merge never deletes the worktree. Teardown runs only after a
  successful merge on main when `--cleanup` is passed or
  `worktree_cleanup_on_merge: true` is configured.
- Pi session relocation does not change `process.cwd()`. It forks the active
  session via `SessionManager.forkFrom` and switches via `ctx.switchSession`.
- Automatic session relocation for agents in Pi occurs only after `agent_settled`.
  During an active turn, the belt intercepts `@@BEE_SESSION_TRANSITION@@` on stderr,
  strips it from the visible tool result, and holds the intent until the turn settles.
- Honest late-runtime failure boundary has three distinct phases:
  1. *Before fork:* Pre-switch validation failures (idle check, session ID mismatch,
     unknown schema, invalid operation, non-canonical path, non-existent target,
     unpersisted session, or missing/empty source session file) fail closed before
     any fork is created. The source session remains active with zero fork files created or deleted.
  2. *Fork exists plus cancellation or pre-teardown switch failure:* If
     `SessionManager.forkFrom` creates a forked session file, but `ctx.switchSession`
     fails before teardown begins (`!transitionTeardownOccurred`) or the user cancels
     the switch, the forked session file is explicitly removed (`rmSync`), cleanly
     preserving the source session without leaving orphaned session artifacts.
  3. *Post-teardown failure:* Once `ctx.switchSession` begins tearing down the
     old runtime context (`transitionTeardownOccurred`), any subsequent upstream
     or downstream failure during target runtime service reconstruction cannot
     promise rollback; the extension logs the Pi error honestly without deleting
     the fork or claiming impossible rollback safety.
- Version compatibility: an older Pi belt ignores additive `sessionTransition`
  JSON metadata and remains in the current checkout; a newer Pi belt with an older
  bee CLI warns of a version mismatch when transition metadata is absent and
  refuses to fork or switch.
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
