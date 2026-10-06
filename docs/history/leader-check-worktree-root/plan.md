# Plan: Leader-check resolves artifacts in the feature worktree

## Summary

`bee cells leader-check` checks artifact paths against the main checkout, so a
file that exists only in the feature's worktree is refused before merge. Merge
needs the leader check, so today the only way through is a
`leader-check-deferral` decision (PBI p-3983800c). The fix makes the artifact
checks use the same worktree root that the commit-diff check in the same
function already uses.

Mode: `tiny` — 0 risk flags: none
Why this is the least workflow that protects the work: one function in one
source file, one red-first test, and every existing refusal message stays.

## Load-bearing claims

| # | Claim | Label | Anchor | Verbatim evidence |
|---|-------|-------|--------|-------------------|
| 1 | The existence check joins the artifact onto the store root | read | `packages/bee-rs/crates/bee/src/verbs/cells/leader_check.rs:239` | `let p = root.join(&answer.artifact);` |
| 2 | The overlap check compares raw strings | read | `packages/bee-rs/crates/bee/src/verbs/cells/leader_check.rs:254` | `.any(\|a\| files_changed.iter().any(\|fc\| fc == &a.artifact));` |
| 3 | The commit-diff check in the same function already resolves the worktree | read | `packages/bee-rs/crates/bee/src/verbs/cells/leader_check.rs:274` | `let history_root = commit_trailer_history_root(root, feature);` |
| 4 | That helper finds the feature's granted worktree | read | `packages/bee-rs/crates/bee/src/verbs/cells/finish_support.rs:346` | `crate::verbs::status_full::find_granted_worktree_for_feature(fallback_root, feature)` |
| 5 | The served door hands leader-check the main root | read | `packages/bee-rs/crates/bee/src/verbs/drivers/prepare.rs:43` | `(Roots::Ordinary(main_root), here)` |

## Cells (current slice)

```json
[
  {
    "id": "lcwr-1",
    "feature": "leader-check-worktree-root",
    "title": "Resolve leader-check artifacts against the feature worktree",
    "lane": "tiny",
    "role": "code",
    "deps": [],
    "decisions": [
      "7e738c47-e9fa-4d43-9d91-623e9b7b4ebf"
    ],
    "files": [
      "packages/bee-rs/crates/bee/src/verbs/cells/leader_check.rs",
      "packages/bee-rs/crates/bee/src/verbs/cells/tests.rs"
    ],
    "read_first": [
      "packages/bee-rs/crates/bee/src/verbs/cells/leader_check.rs",
      "packages/bee-rs/crates/bee/src/verbs/cells/finish_support.rs"
    ],
    "affects_skills": [],
    "affects_specs": [],
    "action": "In record_leader_check, resolve the artifact root once with commit_trailer_history_root(root, feature) — the same root the commit-diff check already uses. Check artifact existence against that root. Before the files_changed overlap compare, turn an absolute artifact path under that root into a repo-relative path and compare both sides through normalize_cell_path. Keep every refusal message unchanged. Red first: add a test in verbs/cells/tests.rs using merge_ready_granted_worktree that writes a file only in the worktree and records a leader check whose artifact names it (once relative, once absolute); it must fail before the fix and pass after.",
    "verify": "PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml --bin bee leader_check",
    "must_haves": {
      "truths": [
        "A leader check whose artifact exists only in the feature's granted worktree is accepted",
        "An absolute worktree path whose relative form is in files_changed passes the overlap check",
        "An artifact missing from both checkouts is still refused with the same message"
      ],
      "artifacts": [
        {
          "path": "packages/bee-rs/crates/bee/src/verbs/cells/leader_check.rs",
          "substantive": "artifact checks use commit_trailer_history_root"
        }
      ],
      "key_links": [
        "record_leader_check uses one root for artifact existence, overlap, and commit diff"
      ],
      "prohibitions": [
        "No new code comments",
        "No change to refusal message text"
      ]
    },
    "behavior_change": true
  }
]
```

## Proof

- `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml --bin bee leader_check` green, with the new worktree test red before the fix.
