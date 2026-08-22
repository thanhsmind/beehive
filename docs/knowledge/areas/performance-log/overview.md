---
type: bee.area
title: "Performance Log — purpose, persistent store, and measurement"
description: "Capturing and aggregating session performance metrics, token usage, and durations into a shared persistent log, rendered as cross-project matrix reports."
timestamp: 2026-08-22
bee:
  id: performance-log-overview
  lifecycle: active
  areas: [performance-log]
  required_context: []
  decisions: ["D1 0a459671", "D 62a7c7fd"]
  sources: [docs/history/perf-log/CONTEXT.md, docs/history/perf-log/plan.md, "docs/specs/performance-log.md#R1", "docs/specs/performance-log.md#R6"]
  authoritative_for: "performance-log: purpose, persistent store, and measurement"
  owns.code: [packages/bee-rs/crates/bee/src/verbs/timings.rs, packages/bee-rs/crates/bee/src/hooks/session_close/perf.rs]
  owns.skills: []
  owns.tests: []
---

# Performance Log — Purpose, Persistent Store, and Measurement

## Purpose

The performance log measures and tracks session timings, token consumption, and model
activity across projects. It maintains a persistent, append-only store and generates
cross-project rollups so developers and operators can monitor execution efficiency
without disrupting active workflows.

## How this area is split

- Section definitions, lifecycle, and measurement: `sections-lifecycle-and-measurement.md`.
- Persistent storage and background sync: `persistent-store-and-sync.md`.
- Cross-project matrix reporting: `cross-project-matrix.md`.
- CLI self-timing and instrumentation: `cli-self-timing.md`.

## Entry Points & Triggers

- **Session close hook** — automatically captures session timings and tokens at session end.
- **Timing inspection** (`bee timings`) — displays recorded duration and performance metrics.
- **Report rendering** — generates HTML performance matrices from the persistent store.

## Data Dictionary

| Element | Meaning |
|---|---|
| persistent log | Machine-local append-only log recording session metrics across projects. |
| scan cache | Cache of processed session files to accelerate repeat rollups. |
| matrix view | Rendered HTML summary showing project activity, token usage, and wall-clock times. |

## Actors & Access

- **Session harness** — automatically records metrics at lifecycle events.
- **Operator** — inspects performance timings and views rendered matrices.

## Business Rules

- Metric recording is fail-open and must never block or delay session termination.
- Data in the persistent log is append-only and keyed by session identity to prevent duplication.

## Pointers (implementation)

- Timing queries and commands: `packages/bee-rs/crates/bee/src/verbs/timings.rs`.
- Session close performance hook: `packages/bee-rs/crates/bee/src/hooks/session_close/perf.rs`.
