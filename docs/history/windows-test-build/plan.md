---
artifact_contract: bee-plan/v1
mode: small
---

# Plan: windows-test-build

## Summary

Make the bee test targets compile on Windows again. Gate the unix-only
supervisor tests from a4e5c302f with `#[cfg(unix)]` (D1).

Red, reproduced locally:
`cargo check --tests --target x86_64-pc-windows-gnu --manifest-path packages/bee-rs/Cargo.toml -p bee`
gives exactly the three errors of CI run 43189304799:
`hook_contracts.rs:785` (`std::os::unix` in `supervisor_run`),
`hook_contracts.rs:800` (`Permissions::from_mode`),
`src/hooks/write_guard/tests.rs:6910` (`std::os::unix::fs::symlink`).

## Load-bearing claims

| # | Claim | Label | Anchor | Verbatim evidence |
|---|---|---|---|---|
| 1 | supervisor_run writes a POSIX shell script and makes it executable with PermissionsExt | read | packages/bee-rs/crates/bee/tests/hook_contracts.rs:785 | use std::os::unix::fs::PermissionsExt; |
| 2 | the symlink test calls a unix-only API | read | packages/bee-rs/crates/bee/src/hooks/write_guard/tests.rs:6910 | std::os::unix::fs::symlink(&s.managed, &link).unwrap(); |

## Cells, current slice preview

```json
[
  {
    "id": "wtb-1",
    "feature": "windows-test-build",
    "title": "Gate the unix-only supervisor tests with cfg(unix)",
    "lane": "small",
    "role": "code",
    "deps": [],
    "decisions": [],
    "files": [
      "packages/bee-rs/crates/bee/tests/hook_contracts.rs",
      "packages/bee-rs/crates/bee/src/hooks/write_guard/tests.rs"
    ],
    "read_first": [
      "docs/history/windows-test-build/CONTEXT.md",
      "docs/knowledge/patterns/20260821-a-platform-job-red-on-a-build-error-is-an-unknown-number-of-failures.md"
    ],
    "affects_skills": [],
    "affects_specs": [],
    "action": "Add #[cfg(unix)] to supervisor_run, to cross_project_supervisor_a_broken_waggledance_is_exit_two_within_four_seconds and cross_project_supervisor_a_registry_naming_the_project_grants_the_read in packages/bee-rs/crates/bee/tests/hook_contracts.rs, and to cross_project_supervisor_resolves_a_symlinked_registry_root in packages/bee-rs/crates/bee/src/hooks/write_guard/tests.rs. Change nothing else; add no comment lines. Linux behavior and test count stay the same.",
    "verify": "PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo check --tests --target x86_64-pc-windows-gnu --manifest-path packages/bee-rs/Cargo.toml -p bee && PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee cross_project_supervisor",
    "must_haves": {
      "truths": [
        "cargo check --tests for x86_64-pc-windows-gnu reports no error",
        "the cross_project_supervisor tests still run and pass on Linux"
      ],
      "artifacts": [
        {"path": "packages/bee-rs/crates/bee/tests/hook_contracts.rs", "substantive": "supervisor_run and its two callers carry #[cfg(unix)]"},
        {"path": "packages/bee-rs/crates/bee/src/hooks/write_guard/tests.rs", "substantive": "the symlink test carries #[cfg(unix)]"}
      ],
      "key_links": [],
      "prohibitions": ["no production code change", "no test body rewritten", "no comment lines"]
    },
    "behavior_change": false
  }
]
```
