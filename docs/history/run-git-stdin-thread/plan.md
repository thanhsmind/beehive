---
artifact_contract: bee-plan/v1
mode: tiny
---

# Plan: run-git-stdin-thread

## Summary

Write git's stdin from a thread so a long patch-id stream cannot deadlock the
merge door. One inline cell on main.

## Load-bearing claims

| # | Claim | Label | Anchor | Verbatim evidence |
|---|-------|-------|--------|-------------------|
| 1 | The write happens before the read | read | packages/bee-rs/crates/bee/src/verbs/worktree/git.rs:116-121 | `if stdin.write_all(input).is_err() {` / `match child.wait_with_output() {` |
| 2 | The typo exists | read | docs/knowledge/areas/worktree-parallelism/returning-and-the-merge-gate.md:156 | `A cleaner rebase is not stale proof` |

## Cells — current slice

```json
[
  {
    "id": "rgs-1",
    "feature": "run-git-stdin-thread",
    "title": "Write git stdin from a thread so a long patch-id stream cannot deadlock",
    "lane": "tiny",
    "role": "code",
    "status": "open",
    "deps": [],
    "decisions": ["0154bcb7-5daa-480c-aee1-af90bbc8143f"],
    "files": ["packages/bee-rs/crates/bee/src/verbs/worktree/git.rs", "docs/knowledge/areas/worktree-parallelism/returning-and-the-merge-gate.md"],
    "read_first": ["packages/bee-rs/crates/bee/src/verbs/worktree/git.rs"],
    "affects_skills": [],
    "affects_specs": ["docs/knowledge/areas/worktree-parallelism/returning-and-the-merge-gate.md"],
    "behavior_change": false,
    "action": "Per run-git-stdin-thread D1: in run_git_stdin, take the child's stdin, move the input into std::thread::spawn which does write_all and drops the handle, call child.wait_with_output on the parent, then join the thread and ignore its result. Keep GitOut and every fail-open path. Per D2 no new test. Fix the typo 'A cleaner rebase' to 'A clean rebase' in the concept. No code comments.",
    "verify": "`PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee verbs::worktree::tests` is green; `rg -c 'A clean rebase is not stale proof' docs/knowledge/areas/worktree-parallelism/returning-and-the-merge-gate.md` prints 1; `.bee/bin/bee dev comment-baseline --check` is clean.",
    "must_haves": {
      "truths": ["run_git_stdin never blocks on a full stdout pipe", "every existing worktree test stays green"],
      "artifacts": [{"path": "packages/bee-rs/crates/bee/src/verbs/worktree/git.rs", "substantive": "stdin written from a spawned thread, stdout read with wait_with_output"}],
      "key_links": ["phases.rs callers unchanged"],
      "prohibitions": ["No change to the GitOut shape", "No new code comments"]
    }
  }
]
```
