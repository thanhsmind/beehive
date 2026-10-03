# Plan: orient-merged-worktree

## Summary
After a feature is merged and waits for the user's test, `bee orient` and the
per-turn hint still say `bee worktree enter --id <id>`, because the worktree
grant is kept after merge. A merged branch now gives no enter step; a fresh
worktree still does.

Mode: `small` — 1 risk flag: covered-contract-change. One cell, three source files.

## Load-bearing claims

| # | Claim | Label | Anchor | Verbatim evidence |
|---|-------|-------|--------|-------------------|
| 1 | orient's main-side worktree context uses the grant alone | read | packages/bee-rs/crates/bee/src/verbs/status_full/orient.rs:166 | let Some((id, worktree_root)) = find_granted_worktree_for_feature(&root, &feature_str) |
| 2 | the per-turn hint uses the grant alone | read | packages/bee-rs/crates/bee/src/hooks/prompt_context.rs:932 | control_root.and_then(|cr| crate::verbs::status_full::find_granted_worktree_for_feature(cr, f)) |
| 3 | a worktree's branch is readable | read | packages/bee-rs/crates/bee/src/verbs/status_full/topology.rs:245 | pub(crate) fn read_worktree_branch(main_root: &Path, id: &str) -> Option<String> { |
| 4 | a git helper exists in status_full | read | packages/bee-rs/crates/bee/src/verbs/status_full/records.rs:595 | pub(crate) fn run_git(root: &Path, args: &[&str]) -> Option<(i32, String)> { |

## Cells — current slice (preview)

| id | title | files | deps | you see | proof |
|---|---|---|---|---|---|
| omw-1 | Skip the worktree enter step for a merged branch | topology.rs, orient.rs, prompt_context.rs, status_full/tests.rs | — | after merge, orient and the hint stop saying enter | status_full and prompt_context tests |

```json
[
  {
    "id": "omw-1",
    "feature": "orient-merged-worktree",
    "lane": "small",
    "role": "code",
    "change_class": "bugfix",
    "title": "Skip the worktree enter step for a merged branch",
    "deps": [],
    "decisions": ["D1", "e82a00f2-fc28-4cfa-b7f0-6782ff2d82bb", "ae42b741-d854-4330-9f11-eaf527823174"],
    "files": [
      "packages/bee-rs/crates/bee/src/verbs/status_full/topology.rs",
      "packages/bee-rs/crates/bee/src/verbs/status_full/orient.rs",
      "packages/bee-rs/crates/bee/src/hooks/prompt_context.rs",
      "packages/bee-rs/crates/bee/src/verbs/status_full/tests.rs"
    ],
    "read_first": ["packages/bee-rs/crates/bee/src/verbs/status_full/topology.rs"],
    "affects_skills": [],
    "affects_specs": [],
    "action": "Per D1: in packages/bee-rs/crates/bee/src/verbs/status_full/topology.rs, beside `pub(crate) fn find_granted_worktree_for_feature` (about line 198), add `pub(crate) fn find_unmerged_granted_worktree_for_feature(main_root: &Path, feature: &str) -> Option<(String, String)>`: it returns what find_granted_worktree_for_feature returns, except None when the worktree's branch (read_worktree_branch, about line 245) is merged into main HEAD. Merged means: `git merge-base --is-ancestor refs/heads/<branch> HEAD` exits 0 AND the branch tip sha is NOT on `git rev-list --first-parent HEAD` (it arrived through a merge commit). A fresh branch whose tip sits on the first-parent line, or any git failure, counts as not merged, so the enter step is kept. Use run_git from verbs/status_full/records.rs (about line 595). Then use the new function in orient_worktree_context's main-side branch (orient.rs about line 166, `let Some((id, worktree_root)) = find_granted_worktree_for_feature(&root, &feature_str)`) and in prompt_context.rs (about line 932, `control_root.and_then(|cr| crate::verbs::status_full::find_granted_worktree_for_feature(cr, f))`). Leave every other caller of find_granted_worktree_for_feature unchanged. Tests red first in packages/bee-rs/crates/bee/src/verbs/status_full/tests.rs: with a real git repo fixture, a granted worktree branch with one commit merged into main with `git merge --no-ff` is reported merged (the new function returns None), and a fresh branch created at main HEAD is reported unmerged (Some). Write no code comments.",
    "verify": "cd packages/bee-rs && cargo test --release -p bee --bin bee verbs::status_full && cargo test --release -p bee --bin bee prompt_context",
    "must_haves": {
      "truths": [
        "a granted worktree whose branch was merged through a merge commit yields no worktree enter step",
        "a fresh granted worktree branch at main HEAD still yields the worktree enter step",
        "a git failure keeps the enter step"
      ]
    }
  }
]
```
