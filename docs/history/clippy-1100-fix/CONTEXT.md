# Clippy 1.100 fix — Context

**Feature slug:** clippy-1100-fix
**Date:** 2026-11-16
**Scope:** Small
**Domain types:** RUN

## What was asked

The owner asked to push and release. Release 2.108.0 published, but the CI
workflow on main went red at the clippy step: CI stable is rustc 1.100.0, the
local default is 1.98.0, and 1.100.0 adds `clippy::needless_bool` at
`herding/wave_ledger.rs:368`. A red CI is a fix-first item.

## What will be done

Fix the one lint and drop the unused `serde` dependency of `fleet` that the
same toolchain reports, proven with clippy 1.100.0 installed beside the
default toolchain.

## Locked Decisions

| ID | Decision | Store id |
|---|---|---|
| D1 | Return `!all_dry_run` in `is_empty_fleet`; remove `serde` from `crates/fleet/Cargo.toml`; prove with `cargo +1.100.0 clippy -- -D warnings`. | 7bb9fb4c |
| C1 | Test contract clippy-1100-fix. | e6d916ce |

## Read set

### Knowledge

| Concept | Grade at read | Why it matters |
|---------|---------------|----------------|
| docs/knowledge/areas/verify-pipeline/suite-topology-and-discovery.md | suspect | the clippy entry of commands.test and its CI step |

### Code

| Path | What it decides |
|------|-----------------|
| `packages/bee-rs/crates/bee/src/herding/wave_ledger.rs` | `is_empty_fleet`, the flagged function |
| `packages/bee-rs/crates/fleet/Cargo.toml` | the unused `serde` dependency |

## Out of scope

Pinning the Rust toolchain so local and CI cannot drift: the release-binaries
workflow adds cross targets to `stable`, so a pin needs its own change.
