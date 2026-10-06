# Plan: Leader-check matches aliased worktree paths

## Summary

Windows CI run 37500482820 refused the new absolute-path leader-check test.
The temp dir used the 8.3 short name `RUNNER~1`, while git reports the
worktree by its long name, so `strip_prefix` failed and the overlap check
refused. The fix canonicalizes both paths when the plain strip fails.

Mode: `tiny` — 1 risk flag: cross-platform
Why this is the least workflow that protects the work: one closure in one
function, with a Linux symlink test that has the same shape as the Windows
short name.

## Load-bearing claims

| # | Claim | Label | Anchor | Verbatim evidence |
|---|-------|-------|--------|-------------------|
| 1 | The overlap compare strips the raw worktree root only | read | `packages/bee-rs/crates/bee/src/verbs/cells/leader_check.rs:240` | `.strip_prefix(&history_root)` |
| 2 | Windows CI used the short-name temp path | ran | `gh run view 37500482820 --log-failed` | `C:\\Users\\RUNNER~1\\AppData` |

## Cells (current slice)

```json
[
  {
    "id": "lcwp-1",
    "feature": "leader-check-windows-paths",
    "title": "Match an absolute leader-check artifact through an aliased worktree path",
    "lane": "tiny",
    "role": "code",
    "deps": [],
    "decisions": [
      "6ca8f8a8-8015-41b7-8994-544c6cbe4848"
    ],
    "files": [
      "packages/bee-rs/crates/bee/src/verbs/cells/leader_check.rs",
      "packages/bee-rs/crates/bee/src/verbs/cells/tests.rs"
    ],
    "read_first": [
      "packages/bee-rs/crates/bee/src/verbs/cells/leader_check.rs"
    ],
    "affects_skills": [],
    "affects_specs": [],
    "action": "In record_leader_check's relative_artifact closure, when the raw strip_prefix of the worktree root fails, canonicalize both the root and the artifact and strip again; fall back to the raw artifact. Red first on Linux with a unix-only test that reaches the worktree through a symlinked alias of the temp dir (the same shape as a Windows 8.3 short name versus the long name git reports).",
    "verify": "PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml --bin bee leader_check",
    "must_haves": {
      "truths": [
        "An absolute artifact reached through an alias of the worktree root overlaps its relative files_changed entry",
        "The existing leader_check tests stay green"
      ],
      "artifacts": [
        {
          "path": "packages/bee-rs/crates/bee/src/verbs/cells/leader_check.rs",
          "substantive": "relative_artifact canonicalizes on a failed strip"
        }
      ],
      "key_links": [
        "relative_artifact feeds the files_changed overlap check"
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

- `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml --bin bee leader_check` green, with the symlink test red before the fix; Windows CI green on the next push.
