# Plan: windows-path-tests

## Summary

Four tests are red on Windows CI only (run 37502285776, red since at least
37448854259). One is a real Windows bug: `bee close` compares worktree paths
byte for byte, so the 8.3 short temp path and the long path git reports do
not match. Two are test fixtures that compare or write paths the Linux way.
The fourth only fails because the first panics while holding a shared test lock.

Mode: `small` — 1 risk flag: cross-platform
Why this is the least workflow that protects the work: one product compare
reuses an existing helper; the rest are test fixes; Windows CI is the proof.

## Load-bearing claims

| # | Claim | Label | Anchor | Verbatim evidence |
|---|-------|-------|--------|-------------------|
| 1 | close compares the cwd worktree with == | read | packages/bee-rs/crates/bee/src/verbs/drivers/close.rs:830 | here == Path::new(&worktree_root) |
| 2 | A Windows-aware path compare already exists | read | packages/bee-rs/crates/bee/src/roots.rs:312 | pub(crate) fn same_path(a: &str, b: &str) -> bool { |
| 3 | The cwd guard unwraps the shared lock | read | packages/bee-rs/crates/bee/src/verbs/state_group/set_gate.rs:3039 | let guard = crate::verbs::drivers::TEST_CWD_LOCK.lock().unwrap(); |
| 4 | The status_full test failed only on the short vs long path | ran | gh run view 37502285776 --log-failed | RUNNER~1 |

## Cells (current slice)

```json
[
  {
    "id": "wpt-1",
    "feature": "windows-path-tests",
    "title": "Make the four Windows-only path tests pass",
    "lane": "small",
    "role": "code",
    "deps": [],
    "decisions": [
      "efa057d6-fbc5-4d56-aee4-1924c9959a92"
    ],
    "files": [
      "packages/bee-rs/crates/bee/src/verbs/drivers/close.rs",
      "packages/bee-rs/crates/bee/src/verbs/status_full/tests.rs",
      "packages/bee-rs/crates/bee/src/verbs/drivers/prepare.rs",
      "packages/bee-rs/crates/bee/src/verbs/state_group/set_gate.rs",
      "packages/bee-rs/crates/bee/src/verbs/cells/handlers_meta.rs",
      "packages/bee-rs/crates/bee/src/verbs/cells/handlers_write.rs"
    ],
    "read_first": [
      "packages/bee-rs/crates/bee/src/roots.rs",
      "packages/bee-rs/crates/bee/src/verbs/drivers/close.rs",
      "packages/bee-rs/crates/bee/src/verbs/status_full/tests.rs"
    ],
    "affects_skills": [],
    "affects_specs": [],
    "action": "Fix the four tests red on Windows CI run 37502285776 (all green on Linux). (1) Product: in close.rs docs_root_for_feature (close.rs:822-836) compare main_root with root and here with the granted worktree root using roots::same_path (roots.rs:312, it resolves a Windows 8.3 short name and the long name git reports to one path) instead of ==; this fixes contract_control_plane_from_worktree_e32f3a66_close_dry_run, whose cwd is the short temp path while git reports the long one. (2) status_full/tests.rs:796 unmerged_granted_worktree_distinguishes_fresh_from_merged_branch: compare the worktree id exactly and the path with roots::same_path rather than byte equality. (3) prepare.rs paseo_probe_tests::paseo_agent_ready_when_command_resolves_on_path (around prepare.rs:6336) formats a Windows path with backslashes into JSON unescaped; build the JSON string value with serde_json (e.g. serde_json::to_string of the path) so it is escaped. (4) set_gate.rs:3039 contract_control_plane_from_worktree_e32f3a66_state_verbs failed only with PoisonError because the close test panicked while holding TEST_CWD_LOCK; make every CwdGuard that takes TEST_CWD_LOCK (close.rs:6086, set_gate.rs:3039, cells/handlers_meta.rs:1260, cells/handlers_write.rs:3054) recover a poisoned lock with unwrap_or_else(|e| e.into_inner()) so one red test no longer cascades. No new code comments. Windows cannot be run here: prove on Linux that the touched tests stay green and the suite builds; Windows CI is the real proof after merge.",
    "verify": "PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml --bin bee -- verbs::drivers::close verbs::status_full verbs::drivers::prepare verbs::state_group::set_gate verbs::cells",
    "must_haves": {
      "truths": [
        "docs_root_for_feature matches the granted worktree through roots::same_path",
        "The status_full worktree test compares the path with same_path",
        "The paseo probe test writes a JSON-escaped path",
        "A poisoned TEST_CWD_LOCK no longer cascades into other cwd tests"
      ],
      "artifacts": [
        {
          "path": "packages/bee-rs/crates/bee/src/verbs/drivers/close.rs",
          "substantive": "same_path in docs_root_for_feature"
        }
      ],
      "key_links": [
        "Windows CI verify-windows job green for the four tests"
      ],
      "prohibitions": [
        "No new code comments",
        "No change outside these four tests and the one compare"
      ]
    },
    "behavior_change": true
  }
]
```

## Proof

- The cell verify green on Linux, the full suite green on Linux, then the Windows CI job green after push.
