---
type: bee.area
title: Performance Log — Cross-Project Matrix
description: "The read-only, per-project rollup view built from the shared persistent log, needing no prior tracking and grouped so different checkouts of the same project collapse into one row."
timestamp: 2026-07-22
bee:
  id: performance-log-cross-project-matrix
  lifecycle: active
  areas: [performance-log]
  required_context: [areas/performance-log/persistent-store-and-sync.md]
  decisions: [D 62a7c7fd]
  sources: [docs/history/perf-log/CONTEXT.md, docs/history/perf-log/plan.md, "cells perf-log-1, perf-log-2, perf-log-3 (capped, verified)", "docs/specs/performance-log.md#R11", "docs/specs/performance-log.md#P7"]
  authoritative_for: "performance-log: cross-project matrix"
---

# Performance Log — Cross-Project Matrix

The full cross-project performance matrix: every project's coding sessions,
scanned and rolled up into one report. This view is read-only — it never
measures anything itself, it only reads what `persistent-store-and-sync.md`
has already written into the shared persistent log — and it needs no prior
tracking, since it reads whatever real work already happened.

## Entry Points & Triggers

- **Build the matrix** — the operator asks for the full cross-project
  performance matrix. Every project's coding sessions are scanned and
  rolled up into one report; with the HTML option a self-contained web page
  is written to the shared location (a project-by-project matrix).
- **Automatic refresh** — at the end of every coding session the matrix
  report is regenerated on its own, so the operator never has to run
  anything to keep it current.

The matrix and its automatic refresh are the primary, zero-effort path,
alongside the manual named-section path owned by
`sections-lifecycle-and-measurement.md`.

## Behaviors & Operations

- **Building the matrix (read-only view).** The matrix is **read from the
  persistent log**, never by re-scanning activity records at view time.
  Rows are grouped by **project name — the last folder of the project's
  path** — so different checkouts of the same-named folder collapse into
  one row (their full paths are kept and shown on hover). Each row shows
  sessions, active time, per-model token totals (new/cached), cache ratio,
  parallel-session count, and last activity. The operator observes either a
  self-contained HTML file they open, or a printed per-project summary. If
  the log is empty the first time, it is backfilled once. A window filter
  limits the matrix to sessions active since a given moment. An empty or
  missing log yields an empty matrix, never an error.

## Business Rules

- **R11 — Projects are grouped by their last folder name.** The matrix
  groups rows by the final segment of each project's path (its folder
  name), so it reads at a glance; two different paths that end in the same
  folder name merge into one row, and every underlying full path is
  retained and shown on hover. (D `62a7c7fd`)

## Pointers (implementation)

- **P7** — Tests: `packages/bee/tests/test_perf.mjs`, and the
  perf blocks in `packages/bee/tests/test_bee_cli.mjs`.
