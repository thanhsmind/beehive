---
artifact_contract: bee-plan/v1
mode: small
---

# Plan: windows-test-build-2

## Summary

Make the bee test targets compile on Windows again. Gate the two sh-fake
`waggledance` fixtures with `#[cfg(unix)]` (D1), then read the real
Windows CI test result (C1).

Red, from CI run 45723393331 and reproduced locally with
`cargo check --tests --target x86_64-pc-windows-gnu --manifest-path packages/bee-rs/Cargo.toml -p bee`:
`tests/handoff_self_drive.rs:82-83` and `src/verbs/feature_group.rs:1019-1020`
(E0433 `cannot find unix in os`, E0599 `Permissions::from_mode`).

## Load-bearing claims

| # | Claim | Label | Anchor | Verbatim evidence |
|---|---|---|---|---|
| 1 | the handoff fixture makes a sh fake executable with PermissionsExt | read | packages/bee-rs/crates/bee/tests/handoff_self_drive.rs:82 | use std::os::unix::fs::PermissionsExt; |
| 2 | FakeWaggledance::install makes a sh fake executable with PermissionsExt | read | packages/bee-rs/crates/bee/src/verbs/feature_group.rs:1019 | use std::os::unix::fs::PermissionsExt; |

## Cells, current slice preview

```json
[
  {
    "id": "wtb2-1",
    "feature": "windows-test-build-2",
    "title": "Gate the sh-fake waggledance test fixtures with cfg(unix)",
    "lane": "small",
    "role": "code",
    "deps": [],
    "decisions": ["4f4447ee-7e4b-4786-852e-0a126371fe1a"],
    "files": [
      "packages/bee-rs/crates/bee/tests/handoff_self_drive.rs",
      "packages/bee-rs/crates/bee/src/verbs/feature_group.rs"
    ],
    "read_first": [
      "docs/history/windows-test-build-2/CONTEXT.md",
      "docs/knowledge/patterns/20260821-a-platform-job-red-on-a-build-error-is-an-unknown-number-of-failures.md"
    ],
    "affects_skills": [],
    "affects_specs": [],
    "action": "Add #![cfg(unix)] as the first line of packages/bee-rs/crates/bee/tests/handoff_self_drive.rs. In the mod tests of packages/bee-rs/crates/bee/src/verbs/feature_group.rs, add #[cfg(unix)] to FAKE_WAGGLEDANCE, the FakeDispatch enum, the FakeWaggledance struct and its impl blocks (including Drop), every #[test] fn that calls FakeWaggledance::install or names FakeDispatch, and any helper, const or use item that only those tests need, so the x86_64-pc-windows-gnu test check has no error and no unused or dead-code warning. Change nothing else; no test body rewritten; add no comment lines. Linux behavior and test count stay the same.",
    "verify": "PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo check --tests --target x86_64-pc-windows-gnu --manifest-path packages/bee-rs/Cargo.toml -p bee && PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee --test handoff_self_drive && PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee --bin bee -- verbs::feature_group",
    "must_haves": {
      "truths": [
        "cargo check --tests for x86_64-pc-windows-gnu reports no error",
        "the handoff_self_drive and verbs::feature_group tests still run and pass on Linux"
      ],
      "artifacts": [
        {"path": "packages/bee-rs/crates/bee/tests/handoff_self_drive.rs", "substantive": "the file starts with #![cfg(unix)]"},
        {"path": "packages/bee-rs/crates/bee/src/verbs/feature_group.rs", "substantive": "FakeWaggledance, its script, FakeDispatch and every test using them carry #[cfg(unix)]"}
      ],
      "key_links": [],
      "prohibitions": ["no production code change", "no test body rewritten", "no comment lines"]
    },
    "behavior_change": false
  },
  {
    "id": "wtb2-1t",
    "feature": "windows-test-build-2",
    "title": "Independently prove the Windows test build and the Linux tests",
    "lane": "small",
    "role": "test",
    "deps": ["wtb2-1"],
    "decisions": ["4f4447ee-7e4b-4786-852e-0a126371fe1a", "3487e31a-6f48-4fc1-8636-93e90d928314"],
    "files": [
      "packages/bee-rs/crates/bee/tests/handoff_self_drive.rs",
      "packages/bee-rs/crates/bee/src/verbs/feature_group.rs"
    ],
    "read_first": [
      "docs/history/windows-test-build-2/CONTEXT.md"
    ],
    "affects_skills": [],
    "affects_specs": [],
    "action": "Verify wtb2-1 independently: run the cell verify, confirm the x86_64-pc-windows-gnu test check shows no error and no warning in the two files, run cargo clippy --release --all-targets -p bee with -D warnings on Linux, and confirm the Linux test count for handoff_self_drive and verbs::feature_group did not drop against main. Run rg -n 'os::unix|from_mode' over packages/bee-rs/crates/bee and report any hit not under a cfg(unix) item. Edit only if wtb2-1 missed a site in the two files; add no comment lines.",
    "verify": "PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo check --tests --target x86_64-pc-windows-gnu --manifest-path packages/bee-rs/Cargo.toml -p bee && PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee --test handoff_self_drive && PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee --bin bee -- verbs::feature_group",
    "must_haves": {
      "truths": [
        "the Windows test check is clean with no warning in the two files",
        "Linux clippy all-targets is clean and the Linux test counts did not drop"
      ],
      "artifacts": [],
      "key_links": [],
      "prohibitions": ["no production code change", "no comment lines"]
    },
    "behavior_change": false
  }
]
```
