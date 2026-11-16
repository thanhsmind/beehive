# Cargo build-dir tests — Context

**Feature slug:** cargo-build-dir-tests
**Date:** 2026-11-16
**Scope:** Small
**Domain types:** RUN

## What was asked

The owner asked to fix the CI verify chain, red on main since 2026-11-12:
the same 105 tests fail on GitHub Actions and pass locally.

## What was found

CI runs rustc 1.100.0, the local default is 1.98.0. Run locally on 1.100.0,
104 tests fail with the same messages: `built bee binary not found at
.../release/build/bee/<hash>/out/bee` and `CARGO_BIN_EXE_bee is unset`.
Cargo 1.100 puts test executables in `target/<profile>/build/<pkg>/<hash>/out/`
instead of `target/<profile>/deps/`; the bin stays at `target/<profile>/bee`.
Ten hand-rolled lookups pop directories from `current_exe()`, and assert_cmd's
`cargo_bin` guesses the same way, so all of them look in the wrong place.

## Locked Decisions

| ID | Decision | Store id |
|---|---|---|
| D1 | Integration tests use `env!("CARGO_BIN_EXE_bee")`. Unit tests in the bin crate use one shared `cfg(test)` helper that walks the ancestors of `current_exe()` for the first `<dir>/bee` file. `cargo_bin("bee")` and every per-file `current_exe` lookup go. | dd93aeaf |
| C1 | Test contract cargo-build-dir-tests. | 1e9b2c01 |

## Read set

### Knowledge

| Concept | Grade at read | Why it matters |
|---------|---------------|----------------|
| docs/knowledge/areas/verify-pipeline/suite-topology-and-discovery.md | suspect | home of how the suite runs; gains the binary-lookup rule |

### Code

| Path | What it decides |
|------|-----------------|
| `packages/bee-rs/crates/bee/tests/*.rs` (9 files) | per-file `binary()` lookups |
| `packages/bee-rs/crates/bee/src/verbs/{plan_group,upkeep_group,report,design_group}.rs`, `verbs/status_full/tests.rs`, `onboard/tests.rs` | unit tests that run the built binary |
| `packages/bee-rs/crates/bee/src/main.rs` | module root for the shared helper |

## Out of scope

Pinning the Rust toolchain (filed in the backlog).
