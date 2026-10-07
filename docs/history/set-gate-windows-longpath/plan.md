# Plan: set-gate-windows-json

## Summary

The last Windows-only red: the set_gate
control-plane test compares the 8.3 short temp path (RUNNER~1) with the long path bee prints (runneradmin); the earlier fix handled only the
JSON escaping. The fix canonicalizes main before
both assertions.

Mode: `tiny` — 1 risk flag: cross-platform
Why this is the least workflow that protects the work: one test assertion.

## Load-bearing claims

| # | Claim | Label | Anchor | Verbatim evidence |
|---|-------|-------|--------|-------------------|
| 1 | The assertions use the raw main path | read | packages/bee-rs/crates/bee/src/verbs/state_group/set_gate.rs:3231 | "control plane: {}" |
| 2 | Windows CI fails at that line | ran | gh run view 37583701230 --log-failed | set_gate.rs:3230:13 |

## Cells (current slice)

```json
[
  {
    "id": "sgl-1",
    "feature": "set-gate-windows-longpath",
    "title": "Compare the set_gate control-plane output with the canonical main path",
    "lane": "tiny",
    "role": "code",
    "deps": [],
    "decisions": [
      "2fe20916-16d7-4c07-88a4-b089eaf26a60"
    ],
    "files": [
      "packages/bee-rs/crates/bee/src/verbs/state_group/set_gate.rs"
    ],
    "read_first": [
      "packages/bee-rs/crates/bee/src/verbs/state_group/set_gate.rs"
    ],
    "affects_skills": [],
    "affects_specs": [],
    "action": "In contract_control_plane_from_worktree_e32f3a66_state_verbs (set_gate.rs:3229-3231), take main through dunce::canonicalize before building both the JSON-escaped control_root text and the plain control plane line, so the Windows 8.3 short temp path matches the long path bee prints. No new code comments.",
    "verify": "PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml --bin bee verbs::state_group::set_gate",
    "must_haves": {
      "truths": [
        "Both control-plane assertions compare against the canonical main path"
      ],
      "artifacts": [
        {
          "path": "packages/bee-rs/crates/bee/src/verbs/state_group/set_gate.rs",
          "substantive": "dunce::canonicalize of main before both assertions"
        }
      ],
      "key_links": [
        "Windows CI verify-windows green for this test"
      ],
      "prohibitions": [
        "No new code comments"
      ]
    },
    "behavior_change": false
  }
]
```
