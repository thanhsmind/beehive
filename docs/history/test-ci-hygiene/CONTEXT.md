# Test CI hygiene — Context

**Feature slug:** test-ci-hygiene
**Date:** 2026-11-19
**Scope:** Tiny
**Domain types:** RUN

## What was asked

Backlog item p-814029a8: the fleet crate declares `pub mod fake` with no gate,
so `FakeBackend` and its fault-injection API compile into the library that the
bee binary links.

## What will be done

Gate the module behind a `test-support` feature (also on under `cfg(test)`),
turn the feature on for fleet's own integration tests and for the bee crate's
tests through dev-dependencies, and prove a release build of the bee binary no
longer holds the fake.

## Locked Decisions

| ID | Decision | Store id |
|---|---|---|
| D1 | `fake` is declared under `cfg(any(test, feature = "test-support"))`; fleet and bee enable the feature through dev-dependencies. | c597c5b7 |
| C1 | Test contract fake-backend-gated: no `FakeBackend` symbol in a release bee binary, and every FakeBackend test still runs and passes. | b0bcc68f |

## Read set

### Knowledge

| Concept | Grade at read | Why it matters |
|---------|---------------|----------------|
| docs/knowledge/areas/rust-runtime/overview.md | suspect | names the compiled runtime the fleet crate links into |

### Code

| Path | What it decides |
|------|-----------------|
| `packages/bee-rs/crates/fleet/src/backend.rs` | the ungated `pub mod fake` |
| `packages/bee-rs/crates/fleet/Cargo.toml` | where the feature and dev-dependency go |
| `packages/bee-rs/crates/bee/Cargo.toml` | bee's dev-dependency on fleet |

## Out of scope

Any change to FakeBackend behavior.
