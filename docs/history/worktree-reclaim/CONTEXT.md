# worktree-reclaim — locked context

Worktrees survive the work they were made for. On 2026-08-05 this machine
held 13 of them, 4.5 GB, every branch already `ahead=0` against `main`.
Nothing in bee reclaimed them, because reclamation was opt-in.

**Recorded deviation** — these decisions were locked from the user's direct
answers to two questions in-session, not from a `bee-shaping` interview. The
scope arrived already decided; an interview would have re-asked what was
answered. Each decision is in `.bee/decisions.jsonl` under tag
`worktree-reclaim` (2026-08-05T09:2x).

## Evidence

- 13 worktrees, 4.5 GB (`beehive--wt--worktree-finish` alone 1.6 GB); the
  weight is `packages/bee-rs/target` per tree, not source.
- Every `wt/*` branch measured `ahead=0` against `main` — merged, reclaimable,
  nothing at risk.
- `bee worktree merge` removes nothing unless `--cleanup` is passed
  (`verbs/worktree/merge.rs:278` `perform_cleanup`); without it the result only
  carries `cleanup_suggested_command` (`merge.rs:368`).
- No `prune`, `rm`, or `gc` subcommand exists; removal is only a side effect of
  `merge --cleanup` (`handlers.rs:648` `try_native`).
- `bee worktree unregister` drops the grant but not the workspace record —
  observed live: grants went to `{}` and 13 orphan
  `.bee/runtime/workspaces/beehive--wt--*.json` stayed behind.

## Locked decisions

### D1 — cleanup is the default, not the favour

`bee worktree merge` runs its cleanup path by default on a green (or
verify-skipped) merge: worktree directory, branch, grant and workspace record
all go. A `--no-cleanup` flag opts one merge out. `.bee/config.json`
`worktree_cleanup_on_merge` (default `true`) opts a whole repo out. The old
`--cleanup` flag stays accepted as a no-op, so every existing script keeps
working.

Every refusal that guards cleanup today survives untouched: a dirty worktree, a
textual conflict, or a red verify still refuses, and `git branch -d` is still
never `-D`.

### D2 — `bee worktree prune` sweeps what merge never saw

A worktree is **dead** when all of these hold:

- its branch is fully merged into the base ref (`rev-list --count base..branch`
  is `0`),
- the tree holds no tracked-modified and no untracked files,
- no live session owns or is attached to it (owner heartbeat fresh within the
  15-minute staleness window),
- its last commit is older than an age threshold.

Prune drops all four artifacts per dead worktree — directory, branch
(`branch -d`, never `-D`), grant, workspace record — and reports one line per
worktree naming why it was removed or kept. It carries `--dry-run` and an age
flag.

### D3 — a teardown either finishes or does not start

`bee worktree unregister` drops the workspace record in the same call it drops
the grant. A partial teardown leaves the registry lying about what exists.

### D4 — a 4.5 GB leak announces itself

`bee orient` and the session preamble surface reclaimable worktrees as named
open work once the count crosses a threshold, exactly as orphaned scribing debt
and unapplied promote proposals are surfaced today. The line names the count,
the reclaimable size, and the command that reclaims it.

## Out of scope

- Sharing one cargo `target/` across worktrees. It would cut the 4.5 GB at the
  source, but it changes build isolation — a separate decision, not this one.
- Reclaiming the 13 orphan workspace records already on disk. They are cleared
  by one prune run — but only because `prune` enumerates
  `.bee/runtime/workspaces/` as well as the grants (plan.md, wr-3). A
  grant-driven scan alone never visits a grantless orphan.

## Amendments

An adversarial review pass proved D2's safety argument unsound and D4's cost
unaffordable. Six amendments are logged in `.bee/decisions.jsonl`, all tagged
`worktree-reclaim`, and the plan is written against them:

- **D1a** — cleanup-by-default fires only on a merge that merged something;
  the `ALREADY_UP_TO_DATE` arm removes nothing. A non-boolean `--no-cleanup`
  value is refused, never ignored.
- **D2a** — every classifier probe fails closed; `merge-base --is-ancestor`
  replaces a parsed count; three keep-conditions added (attached HEAD matching
  the recorded branch, no interrupted rebase/cherry-pick/merge/bisect, no
  `.bee/HANDOFF.json` or non-empty capture queue).
- **D2b** — liveness comes from session records, not the workspace record's
  ownership fields, which are never written for a worktree.
- **D3a** — the teardown helper takes directory removal as an explicit
  parameter; `unregister` never reaches it.
- **D4a** — the preamble line names the count and the command, not the size,
  and runs no git at session start.
- **D5** — `generated/registry_payload.json` is hand-edited here, reason
  recorded: its declared regen chain needs Node, and this repo has none.
