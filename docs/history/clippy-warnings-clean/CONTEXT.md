# Clippy warnings clean — Context

**Feature slug:** clippy-warnings-clean
**Date:** 2026-11-16
**Scope:** Small
**Domain types:** RUN

## What was asked

The owner asked to clean the clippy warnings that clippy-in-test left: 1039
findings in 185 files on the default targets (892 machine-applicable, 147
manual; 597 of them collapsible_if). When clean, clippy runs with
`-D warnings` in the declared test path and in CI.

## What will be done

Apply clippy's machine fixes, fix the rest by hand, allow two signature-design
lints in one workspace table, then add `-- -D warnings` to the clippy entry of
`commands.test` and the ci.yml clippy step. Behavior does not change; the full
test suite is the pin.

## Locked Decisions

| ID | Decision | Store id |
|---|---|---|
| D1 | Clean until `cargo clippy --manifest-path packages/bee-rs/Cargo.toml -- -D warnings` exits 0 on the default targets; then the clippy entry of commands.test and the ci.yml step gain `-- -D warnings`. Behavior does not change. | cf6ce5e3 |
| D2 | `too_many_arguments` and `result_large_err` are allowed once in `packages/bee-rs/Cargo.toml` `[workspace.lints.clippy]`, members opt in with `[lints] workspace = true`. Every other finding is fixed in code; no item-level `#[allow]`. | e88c22c7 |
| C1 | Test contract clippy-warnings-clean. | 4a500653 |

## Read set

### Knowledge

| Concept | Grade at read | Why it matters |
|---------|---------------|----------------|
| docs/knowledge/areas/verify-pipeline/suite-topology-and-discovery.md | suspect | names beehive's commands.test list (clippy-in-test D1/D2) |
| docs/knowledge/patterns/20260826-a-rule-living-in-n-places-needs-one-test-that-reads-all-n.md | pattern | proof_gate pins commands.test against ci.yml |

### Code

| Path | What it decides |
|------|-----------------|
| `packages/bee-rs/Cargo.toml` | the workspace and its lint table |
| `packages/bee-rs/crates/bee/src/` | 1036 of the findings |
| `packages/bee-rs/crates/fleet/src/` | 3 of the findings |
| `.bee/config.json`, `.github/workflows/ci.yml` | the clippy command that gains `-D warnings` |

## Out of scope

Lints on test targets (`--all-targets`), because commands.test checks the
default targets only. Redesigning the 16 long parameter lists and the large
error type.
