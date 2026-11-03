# Plan: set-gate-windows-json

## Summary

One Windows-only red is left after windows-path-tests: the set_gate
control-plane test compares the raw main path with JSON output, where a
Windows path's backslashes are escaped. The fix builds the expected text from
the JSON string form of the path.

Mode: `tiny` — 1 risk flag: cross-platform
Why this is the least workflow that protects the work: one test assertion.

## Load-bearing claims

| # | Claim | Label | Anchor | Verbatim evidence |
|---|-------|-------|--------|-------------------|
| 1 | The assertion formats the raw path into JSON text | read | packages/bee-rs/crates/bee/src/verbs/state_group/set_gate.rs:3229 | "\"control_root\": \"{}\"" |
| 2 | Windows CI fails at that line | ran | gh run view 37582563405 --log-failed | set_gate.rs:3229:13 |

## Cells (current slice)

```json
[
  {
    "id": "sgw-1",
    "feature": "set-gate-windows-json",
    "title": "Match control_root in the set_gate test against the JSON-escaped path",
    "lane": "tiny",
    "role": "code",
    "deps": [],
    "decisions": [
      "c2bcea41-6dbc-42e8-80de-7ef1080bcd3b"
    ],
    "files": [
      "packages/bee-rs/crates/bee/src/verbs/state_group/set_gate.rs"
    ],
    "read_first": [
      "packages/bee-rs/crates/bee/src/verbs/state_group/set_gate.rs"
    ],
    "affects_skills": [],
    "affects_specs": [],
    "action": "In contract_control_plane_from_worktree_e32f3a66_state_verbs (set_gate.rs:3229), build the expected control_root text from serde_json::to_string of the main path string, so a Windows path's backslashes are escaped the way the JSON output escapes them. Leave the plain 'control plane:' assertion as it is. No new code comments.",
    "verify": "PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml --bin bee verbs::state_group::set_gate",
    "must_haves": {
      "truths": [
        "The control_root assertion compares against the JSON-escaped main path"
      ],
      "artifacts": [
        {
          "path": "packages/bee-rs/crates/bee/src/verbs/state_group/set_gate.rs",
          "substantive": "serde_json::to_string of the main path in the control_root assertion"
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
