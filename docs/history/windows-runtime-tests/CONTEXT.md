# CONTEXT: windows-runtime-tests

## Ask

The owner wants the Windows CI job green, proven on a Windows CI run of
the feature branch, merged to main and pushed. No release.

## Red

Windows CI run 45725981958 (branch wt/windows-test-build-2) compiles the
test targets and reports 6021 passed, 29 failed. The panic text of each
failure is in that run's log (`gh run view 45725981958 --log`).

## Locked decisions

- **D1** (a0beedae): three causes are real Windows product bugs and get
  product fixes; every other failure is a unix-shaped test that is made
  portable, never gated with `cfg(unix)`.
  - Product: `nested_checkout.rs` skips `.bee/tmp` by comparing against
    a string built with `/`.
  - Product: `hooks/write_guard/cross_project.rs` canonicalizes with
    `std::fs::canonicalize`, which gives the `\\?\` verbatim form on
    Windows, while the paths it compares against are plain.
  - Product: the same file keeps an allow-list entry or a shell path
    token only when it starts with `/` or `~`, so `C:\…` or `C:/…` is
    dropped.
  - Test: `ExitCode` Debug text, a fake hub that splits on `/`, a root
    compared to `std` canonicalize, tests that set `HOME` where bee reads
    `USERPROFILE` on Windows, backslash paths placed inside a bash
    command (Git Bash commands carry forward slashes), and wording
    tests that do not expect the `[owner-only: …]` suffix that a temp
    dir under the user profile earns.
- **D2** (a7c1afaf): the agy dispatch-prepare test isolates the global
  config layer and prints `config explain` for `team.agy.read` in its
  assertion.

## Read set

### Knowledge

| Concept | Grade at read | Why it matters |
|---------|---------------|----------------|
| docs/knowledge/areas/worktree-parallelism/path-identity.md | suspect | path compares go through dunce canonical forms; separators are the platform's job |

### Code

| Path | What it decides |
|------|-----------------|
| `packages/bee-rs/crates/bee/src/nested_checkout.rs` | which nested checkouts the scan skips |
| `packages/bee-rs/crates/bee/src/hooks/write_guard/cross_project.rs` | which paths belong to another project |
| `packages/bee-rs/crates/bee/src/hooks/write_guard/main.rs` | where the owner-only suffix is added |
| `packages/bee-rs/crates/bee/src/hooks/session_close/perf.rs` | the home dir bee reads per platform |
| `packages/bee-rs/crates/bee/src/config_layers.rs` | the global config layer path |

## Proof

Local: `cargo check --tests --target x86_64-pc-windows-gnu` and the
related Linux tests. Final: a Windows CI run of `wt/windows-runtime-tests`
with 0 failed in the bee test job. No local Windows runtime exists
(no wine), so the CI run is the repro surface, red and green.
