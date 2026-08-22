---
type: bee.area
title: "Verify Pipeline — purpose, test execution, and proof discipline"
description: "The verification pipeline that proves correctness: running declared test commands, managing concurrency, ensuring hermetic runs, and validating skill pointer integrity."
timestamp: 2026-08-22
bee:
  id: verify-pipeline-overview
  lifecycle: active
  areas: [verify-pipeline]
  required_context: []
  decisions: ["contention-split D1-D6 (decision 1ce777d9)", "verify-scoping D1/D2 (decisions e39d3f89, 20534ea9)", "412e9b3a (commands.test is the one declared test path)"]
  sources: ["docs/history/test-economy/CONTEXT.md", "docs/specs/verify-pipeline.md#R1", "docs/specs/verify-pipeline.md#R2"]
  authoritative_for: "verify-pipeline: purpose, test execution, and proof discipline"
  owns.code: [packages/bee-rs/crates/bee/src/verbs/test_runner.rs]
  owns.skills: []
  owns.tests: [packages/bee-rs/crates/bee/tests/proof_gate.rs, packages/bee-rs/crates/bee/tests/concurrency.rs]
---

# Verify Pipeline — Purpose, Test Execution, and Proof Discipline

## Purpose

The verify pipeline guarantees that code changes, documentation, and skill definitions
satisfy their declared invariants before landing. It provides hermetic, contention-free
verification across worktrees and enforces proof-of-correctness gates across the bee
lifecycle.

## How this area is split

- Test execution and hermetic runs: `concurrency-and-hermetic-runs.md`.
- Skill reference and pointer verification: `skill-reference-pointer-integrity.md`.
- Historical suite discovery and caching: `suite-topology-and-discovery.md`, `suite-result-cache.md`.

## Entry Points & Triggers

- **Cell verify** — executed during `bee cells finish` and cell completion checks.
- **Merge verification** — runs declared test suites during `bee worktree merge`.
- **Knowledge verification** — runs `bee knowledge check` and `bee knowledge index --check`.

## Data Dictionary

| Element | Meaning |
|---|---|
| declared test command | Configured project test command in `.bee/config.json` (`commands.test`). |
| proof line | Triple `<command> — <result> — <scope reason>` recorded on cell completion. |
| pointer integrity | Verification ensuring all doc and skill cross-references resolve. |

## Actors & Access

- **Orchestrator and workers** — execute scoped verification and record proof lines.
- **CI / merge gate** — executes full project verification prior to integration.

## Business Rules

- A red verification result refuses cell completion and merge unconditionally.
- Proof lines must name fresh command output and valid scope justifications.

## Pointers (implementation)

- Test runner invocation: `packages/bee-rs/crates/bee/src/verbs/test_runner.rs`.
- Proof gate tests: `packages/bee-rs/crates/bee/tests/proof_gate.rs`.
- Concurrency tests: `packages/bee-rs/crates/bee/tests/concurrency.rs`.
