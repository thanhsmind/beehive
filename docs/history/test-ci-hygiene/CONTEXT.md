# Test and CI hygiene — Context

**Feature slug:** test-ci-hygiene
**Date:** 2026-11-19
**Backlog items:** p-9b2e8ee4, p-624e2d7d, p-814029a8
**Shaping session:** complete under gate bypass full (owner said go on the 2026-11-19 backlog sweep)
**Scope:** Standard
**Domain types:** RUN

## Feature Boundary

Tests stop writing into the crate folder. The opencode registry derivation
works on the opencode version the repo installs, and CI pins that version. The
fleet fake backend leaves the release binary.

## Locked Decisions

| ID | Decision | Rationale |
|----|----------|-----------|
| D1 | No test writes `packages/bee-rs/crates/bee/.bee/`. The dispatch authorize tests (and any other writer found through `verbs/mod.rs` timings append, near line 200) run with a temp root. The tracked `packages/bee-rs/crates/bee/.bee/logs/timings.jsonl` is removed from git. A test or a post-suite check asserts the folder is absent after the tests run. | p-9b2e8ee4. The stray file dirties worktrees and blocks `bee worktree merge`. |
| D2 | `tests/opencode_plugin_contracts.rs` registry derivation handles the opencode-ai 1.18.21 literal shape, with a regression test over both 1.18.16 and 1.18.21 shapes. CI (`.github/workflows/ci.yml` and `windows.yml`) pins the same opencode version that the repo tooling installs. | p-624e2d7d. A developer machine and CI must agree about green. |
| D3 | `fleet` gets a `test-support` cargo feature. `pub mod fake` in `crates/fleet/src/backend.rs` compiles only under `cfg(any(test, feature = "test-support"))`. The bee crate enables it only in `[dev-dependencies]`. Proof: a release build of bee does not contain the `FakeBackend` symbol. | p-814029a8. Integration tests link fleet as an external consumer, so a bare `cfg(test)` is not enough. |

## Read set

### Code

| Path | What it decides |
|------|-----------------|
| `packages/bee-rs/crates/bee/src/verbs/mod.rs` | D1 writer. |
| `packages/bee-rs/crates/bee/tests/opencode_plugin_contracts.rs` | D2. |
| `.github/workflows/ci.yml`, `.github/workflows/windows.yml` | D2 pins. |
| `packages/bee-rs/crates/fleet/src/backend.rs`, `packages/bee-rs/crates/fleet/Cargo.toml` | D3. |

## Handoff Note

CONTEXT.md is the source of truth. Decision IDs are stable.
