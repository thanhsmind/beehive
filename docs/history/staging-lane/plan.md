# staging-lane — plan

Lane: standard (flags: data-model, covered-contract-change). Locked
decisions: staging-lane D0 (topology becomes mechanics), D0a (lifecycle:
invariant "staging = main + Σ features awaiting UAT"; three update triggers;
created lazily, always from main; no self-updates; history is garbage).
Companion decision: uat-gate-before-merge D1 (the uat gate is the
"awaiting UAT" signal staging reads).

## Command surface

- `bee staging add --feature <slug>` — triggers 1 and 2 of D0a. Lazily
  creates the staging branch + worktree from CURRENT main (never from a
  feature branch), merges the feature's branch into staging, records the
  feature in the staged set, runs the configured build. Re-running after a
  fix re-merges the same branch (git takes only the new commits).
- `bee staging rebuild [--without <slug,...>]` — trigger 3. `reset --hard
  main`, then re-merge every staged feature still awaiting UAT (staged ∧ uat
  gate unapproved ∧ branch exists), minus `--without` exclusions, then
  build. Features whose uat gate is approved (or whose branch is merged to
  main) drop out of the staged set automatically — the invariant re-derives.
- `bee staging status` — the staged set, each feature's uat gate state,
  staging's base sha vs main (stale-base warning = trigger-3 reminder).
- Store: `.bee/runtime/staging.json` on main — `{branch, worktree_root,
  created_at, base_sha, staged: [{feature, branch, last_merged_sha, at}]}`.
  CLI-only writes.
- Build: config key `commands.staging_build` (optional). Absent → step
  skipped with a visible note, never an error.

## Teeth (D0 iron rules)

1. `bee worktree merge` refuses when the id/branch being merged IS the
   staging branch — typed `WORKTREE_MERGE_STAGING_FORBIDDEN`, zero mutation,
   no escape flag (catastrophic direction has no hatch; config removal is
   the only exit).
2. Direct commits on the staging worktree are refused by the write/commit
   guard unless carried out by bee's own staging machinery (env marker the
   staging commands set). Remedy names the rule: fix on the feature branch,
   `bee staging add` again.
3. Staging is only ever created from main (the add command owns creation —
   no path accepts a different base).
4. After any successful `worktree merge` to main, when a staging record
   exists the merge result carries `staging_rebuild_suggested:
   "bee staging rebuild"` — the trigger-3 nudge.

## Conflict policy

A conflict while merging a feature into staging (add or rebuild) aborts that
merge (`git merge --abort`), reports the feature typed
(`STAGING_MERGE_CONFLICT`, naming the feature and files), and — on rebuild —
continues with the remaining features so one broken feature never blocks
testing the others. Staging is disposable; the remedy is always on the
feature branch.

## Shape — one slice, 4 cells (serial, one worker at a time)

- sl-1: store + `staging add` (lazy create from main, merge, staged-set
  record, build hook, conflict abort+report). Files: new
  `verbs/staging/mod.rs` (+ registry payload entry), `runtime/staging.json`
  store module, config read.
- sl-2: `staging rebuild` + `staging status` (+ `--without`; auto-drop of
  UAT-passed features; stale-base detection). Depends sl-1.
- sl-3: teeth — merge refusal `WORKTREE_MERGE_STAGING_FORBIDDEN` (zero-
  mutation precondition beside the uat one), commit guard on the staging
  worktree, `staging_rebuild_suggested` nudge in merge output. Depends sl-1.
- sl-4: docs/specs/skills — handbook/register.md, worktree-parallelism area
  spec (new concept file `staging-mixing-ground.md`), bee-swarming/bee-hive
  wording where the close step mentions "user tests in the worktree" (now:
  user tests at staging), herding role-merge note. Depends sl-2, sl-3.

## Test scoping

`commands.test` at every cap; staging verbs get their own test module with a
scratch repo fixture (same pattern worktree tests use).

## Risks

- Registry payload hand-edit again (no generator) — same recorded fallback
  as ug-2.
- Staging build command runs host code — config-owned, never invented.
- Self-hosting lag: enforcement and commands ship with the next release.
