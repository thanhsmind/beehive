---
type: bee.area
title: "Worktree Parallelism — the staging mixing ground the user tests at"
description: "The invariant 'staging = main + Σ features awaiting UAT', the three triggers that keep it true, why staging is disposable and its history is garbage, and the teeth that keep it a mixing ground instead of a back door into main."
timestamp: 2026-08-17
bee:
  id: worktree-parallelism-staging-mixing-ground
  lifecycle: active
  areas: [worktree-parallelism]
  required_context: [areas/worktree-parallelism/returning-and-the-merge-gate.md]
  decisions: ["staging-lane D0 (the main/staging/worktree topology becomes first-class bee mechanics, not convention, 2026-08-17)", "staging-lane D0a (lifecycle detail on D0: the invariant, the three update triggers, disposability, 2026-08-17)", "uat-gate-before-merge D1 (the uat gate is the 'awaiting UAT' signal staging reads, 2026-08-17)"]
  sources: ["docs/history/staging-lane/plan.md", ".bee/decisions.jsonl (0f87be54 D0, d20ef88e D0a, 16c7ba64 uat-gate-before-merge D1)", "packages/bee-rs/crates/bee/src/verbs/staging/mod.rs", "packages/bee-rs/crates/bee/src/verbs/worktree/phases.rs (WORKTREE_MERGE_STAGING_FORBIDDEN, staging_rebuild_suggested)", "packages/bee-rs/crates/bee/src/hooks/write_guard/checks.rs (staging_worktree_commit_denial)"]
  authoritative_for: "worktree-parallelism: bee staging add/rebuild/status, the staging.json record, and the disposable staging mixing ground"
---

# Worktree Parallelism — The Staging Mixing Ground

Three checkouts now have a fixed job each: main stays clean and only ever receives a
UAT-passed feature branch; a feature worktree is the only place code truth changes; one
staging worktree is the disposable mixing ground where finished features sit together
for the user to test at ONE place — one port, one build. Staging is mechanics, not
convention (staging-lane D0): the CLI enforces the shape it used to just recommend.

## The invariant

**Staging = main + Σ features awaiting UAT, at every moment** (staging-lane D0a). Every
feature currently merged into staging and not yet UAT-approved belongs there; nothing
else does. The "awaiting UAT" set is derivable, never hand-kept — it is exactly the
features that are staged AND whose `uat` gate (uat-gate-before-merge D1) is still
pending. A feature whose uat gate is approved, or whose branch has already landed on
main by some other route, drops out of the staged set on its own — the invariant
re-derives rather than needing to be told.

## The three update triggers (D0a)

1. **A feature becomes ready for user testing** — `bee staging add --feature <slug>`
   merges that feature's branch into staging (lazily creating staging from main's
   current HEAD the first time this runs) and rebuilds.
2. **A fix lands on the feature branch after user feedback** — `bee staging add` again;
   git carries over only the new commits, so re-running is cheap and safe.
3. **Main moved** — a feature merged to main changes what "current main" means for
   every feature still under test. `bee staging rebuild [--without <slug,...>]` resets
   staging hard to main and re-merges every feature still staged and awaiting uat,
   minus any named exclusions, then rebuilds. Skipping this trigger leaves staging on a
   stale base and hides interaction bugs that would otherwise surface before they reach
   main.

Outside those three, staging never updates itself: no pull, no rebase, no direct
commit — its only inputs are "reset hard to main" and "merge one feature branch in."

## Disposable — history is garbage

Staging's branch may live forever or be deleted and lazily re-created; only its
CURRENT state has meaning, never its commit history. That is what makes trigger 3 free
to run: `reset --hard main` throws away staging's own history on purpose, every time,
because nothing about that history was ever the point. Fixes belong on the feature
branch; staging only ever receives them secondhand, through another `staging add`.

## The teeth (D0 iron rules)

Guidance teaches the why here; the CLI carries the actual enforcement:

1. `bee worktree merge` refuses `WORKTREE_MERGE_STAGING_FORBIDDEN`, zero mutation, when
   the worktree/branch being merged IS staging itself — no escape flag. Merging staging
   straight into main would defeat the whole point of testing at a disposable mixing
   ground; this direction has no hatch, by design.
2. A direct `git commit` inside the staging worktree is refused by the write guard
   unless `BEE_STAGING_MACHINERY=1` — the env marker `bee staging add`/`bee staging
   rebuild` set only around their own merge commits. The remedy the refusal names: fix
   on the feature branch, then `bee staging add` again.
3. Staging is only ever created from main — the add command owns creation, and no path
   accepts a different base.
4. After any successful `worktree merge` to main, when a staging record already exists
   the merge result carries `staging_rebuild_suggested: "bee staging rebuild"` — a
   nudge toward trigger 3, never a forced rebuild.

## Conflict policy

A merge conflict while staging a feature (`add` or `rebuild`) aborts that one merge and
reports `STAGING_MERGE_CONFLICT`, naming the feature and the conflicting files. On
`rebuild`, the remaining features still merge and build — one broken feature never
blocks testing the others. The remedy is always on the feature branch; staging itself
is never patched to route around a conflict.

## Pointers (implementation)

- Store: `.bee/runtime/staging.json` on main — `{branch, worktree_root, created_at,
  base_sha, staged: [{feature, branch, last_merged_sha, at}]}`. CLI-only writes; see
  `docs/handbook/register.md`.
- Commands: `staging_add`, `staging_rebuild`, `staging_status` in
  `packages/bee-rs/crates/bee/src/verbs/staging/mod.rs`.
- Build hook: config `commands.staging_build` (optional; a skip is a visible note, never
  an error).
- Merge-side teeth: `WORKTREE_MERGE_STAGING_FORBIDDEN` and `staging_rebuild_suggested` in
  `packages/bee-rs/crates/bee/src/verbs/worktree/phases.rs`.
- Commit guard: `staging_worktree_commit_denial` in
  `packages/bee-rs/crates/bee/src/hooks/write_guard/checks.rs`.
