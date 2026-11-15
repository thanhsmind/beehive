# Clippy in test — Context

**Feature slug:** clippy-in-test
**Date:** 2026-11-16
**Scope:** Small
**Domain types:** RUN

## What was asked

The owner asked to add clippy to beehive's `commands.test`, before the cargo
tests. Per trust-layers D5 the check ran once first: with `-D warnings` it is
red (fleet 3 warnings, bee 1036 warnings and 1 error); without it only the one
deny-level error fails (`never_loop` in `verbs/blind/mod.rs`). The owner chose
option 1: fix that one error, then add clippy without `-D warnings`.

## What will be done

Fix the `never_loop` error. Make `commands.test` a two-entry list, clippy
first. Teach the two readers that assume a string (the release script and the
proof_gate parity test) to handle the list, and add the clippy run to CI.

## Locked Decisions

| ID | Decision | Store id |
|---|---|---|
| D1 | `commands.test` = [`cargo clippy --manifest-path packages/bee-rs/Cargo.toml` (no `-D warnings`), the unchanged cargo test command]. The one deny-level error is fixed first. | d6c90a97 |
| D2 | `scripts/release.sh` runs each entry in order and stops at the first red. proof_gate accepts a string or a list: the cargo test entry equals every cargo test line in ci.yml and windows.yml; each other entry appears as a run line in ci.yml. windows.yml gains no clippy. | c44134a1 |
| C1 | Test contract clippy-in-test. | 9a9521ab |

## Read set

### Knowledge

| Concept | Grade at read | Why it matters |
|---------|---------------|----------------|
| docs/knowledge/patterns/20260826-a-rule-living-in-n-places-needs-one-test-that-reads-all-n.md | pattern | the parity test that pins config against both workflows |
| docs/knowledge/areas/verify-pipeline/suite-topology-and-discovery.md | suspect | `commands.test` as the one declared test path |

### Code

| Path | What it decides |
|------|-----------------|
| `packages/bee-rs/crates/bee/src/verbs/blind/mod.rs` | the dossier section loop clippy flags as `never_loop` |
| `packages/bee-rs/crates/bee/tests/proof_gate.rs` | config and workflow parity |
| `scripts/release.sh` | the release test gate that runs `commands.test` |
| `.github/workflows/ci.yml` | the push and PR gate |
| `.bee/config.json` | `commands.test` |

## Out of scope

Cleaning the 1039 clippy warnings and adding `-D warnings`. Clippy on the
Windows job.
