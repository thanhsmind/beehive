# Monorepo manifest detect — Context

**Feature slug:** monorepo-manifest-detect
**Date:** 2026-11-15
**Scope:** Small
**Domain types:** RUN

## What was asked

The owner asked to add clippy to beehive's `commands.test`. Before that offer,
trust-layers D5 says to run clippy once and report the result. While checking,
the owner saw that the onboard detector found no clippy check for beehive: the
Rust workspace lives in `packages/bee-rs/Cargo.toml`, and the detector reads
root files only. The owner asked to fix the detector so it also finds
manifests in subdirectories, such as `packages/*/Cargo.toml`, through the bee
chain, because every monorepo host gains from it.

## What will be done

The convention detectors in `onboard/notices.rs` look for a convention
manifest at the root, then one and two directories down. A manifest found
below the root names its location in the proposed command.

## Locked Decisions

| ID | Decision | Store id |
|---|---|---|
| D1 | Convention manifests (Cargo.toml, pyproject.toml, go.mod, *.csproj) are searched at the root, then depth 1, then depth 2; inside a depth, sorted by relative path; the first match that satisfies the convention (for pyproject the `[tool.ruff` or `[tool.mypy` table) wins. The walk skips hidden directories and node_modules, target, vendor, dist, build. package.json, Makefile and composer.json stay root-only. | ac0a62df |
| D2 | A nested convention names its location: `cargo clippy --manifest-path <rel> -- -D warnings`, `ruff check <dir>`, `go -C <dir> vet ./...`, `mypy <dir>`, `pytest <dir>`, `dotnet test <rel csproj>`, `go -C <dir> test ./...`; source = the manifest's relative path. A root manifest keeps today's text byte for byte. Marker words do not change. | 2b3c81e2 |
| C1 | Test contract nested-manifest-detect. | 641b2eb3 |

## Read set

### Knowledge

| Concept | Grade at read | Why it matters |
|---------|---------------|----------------|
| docs/knowledge/areas/onboarding/host-project-artifacts.md | suspect | states that `detect_static_checks` reads root files only; it must follow D1 |

### Code

| Path | What it decides |
|------|-----------------|
| `packages/bee-rs/crates/bee/src/onboard/notices.rs` | `detect_commands`, `detect_static_checks`, `static_check_for`, `commands_notices` and their tests |

## Out of scope

A `cargo test` test convention for Cargo.toml (no convention exists today, root
or nested). Nested package.json, Makefile or composer.json scripts. Part (a),
the beehive `commands.test` change, is a separate owner decision.
