# Worktree-first — main stays clean

Status: approved direction (owner, 2026-07-31); supersedes the lane-first
routing default in worktree-parallelism ("Routing rule", "Lane-first
refinement").

## The policy inversion

Old default: feature work starts in the main checkout; a worktree is
granted late (execution gate) and only on genuine file overlap.

New default: **any code-touching feature branches from the start.** The
main checkout is for integration, docs-lane work, release machinery, and
reading — not for feature edits. Concurrent sessions each live in their
own worktree; they never share a git index or a dirty tree.

| Situation | Where the work lives |
|---|---|
| Feature work, any lane except `docs` and `tiny` | Sibling worktree `<repo>--wt--<feature>` on branch `wt/<feature>`, created at feature start |
| `tiny` lane, no other live session | Main checkout allowed |
| `tiny` lane, another live session present | Worktree, same as features |
| `docs` lane, release machinery, merges | Main checkout |
| Explicit owner override | `--in-main` on feature start, recorded as a decision — never silent |

Everything below the policy already exists and is unchanged: the trust
model (a worktree gets its own store only via a grant it cannot forge),
store tiers, cross-worktree holds, `worktree new` / `register` /
`merge` / `list`, and the merge verify gate.

## Machine changes

1. **Feature start takes the branch.** Starting a non-exempt feature
   (route recorded with a code-touching lane) creates and grants the
   worktree in the same step and tells the session to move there —
   `bee orient` in the main checkout answers with the worktree path as
   `next.command` until the session runs there. No step in shaping or
   planning requires the main checkout.
2. **The main checkout refuses feature edits.** The write guard denies a
   source write in the main checkout when the active feature is
   non-exempt and holds a granted worktree — the refusal names the
   worktree path and the `--in-main` override. Docs-lane paths and the
   exempt cases pass unchanged.
3. **Landing is one gesture.** `bee worktree merge --id <id>` stays the
   only road back to main: staged transaction, verify gate, cleanup.
   `bee close` runs inside the worktree; its green output names the merge
   command as the next action.
4. **Orient knows both sides.** In main: which features live in which
   worktrees (from the registry), and where the current session should
   go. In a worktree: the feature, the branch, and the merge-back state.

## What this buys

- Two sessions on two features never interleave commits or share a dirty
  index — the collision the shared-checkout default produced even with
  reservations and lanes.
- Main is always releasable: nothing lands except through the merge
  gate's verify.
- A abandoned feature is one `worktree remove` — no archaeology in main.

## Out of scope

- Wave-internal worker worktrees (opt-in isolation for one feature's
  parallel workers) — unchanged, orthogonal.
- The herding cockpit — already worktree-per-item; unchanged.
- Auto-merge of any kind: landing stays a human gesture.
