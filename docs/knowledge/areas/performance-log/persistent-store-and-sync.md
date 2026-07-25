---
type: bee.area
title: Performance Log — Persistent Store and Automatic Sync
description: "Automatically rolling up every project's coding-session activity into one shared, append-only, per-machine log — safe to re-run, and never in the way of a session ending."
timestamp: 2026-07-22
bee:
  id: performance-log-persistent-store-and-sync
  lifecycle: active
  areas: [performance-log]
  required_context: [areas/performance-log/sections-lifecycle-and-measurement.md]
  decisions: [D1 0a459671, D 62a7c7fd]
  sources: [docs/history/perf-log/CONTEXT.md, docs/history/perf-log/plan.md, "cells perf-log-1, perf-log-2, perf-log-3 (capped, verified)", "docs/specs/performance-log.md#R6", "docs/specs/performance-log.md#R7", "docs/specs/performance-log.md#R9", "docs/specs/performance-log.md#R10", "docs/specs/performance-log.md#E4", "docs/specs/performance-log.md#P1", "docs/specs/performance-log.md#P3", "docs/specs/performance-log.md#P5"]
  authoritative_for: "performance-log: persistent store and automatic sync"
---

# Performance Log — Persistent Store and Automatic Sync

Where a section (`sections-lifecycle-and-measurement.md`) is the
operator-visible, hand-named record, this concept owns the machine underneath
it: the one shared, append-only, per-machine store every section lands in,
and the automatic mechanism that keeps it populated from real session
activity without the operator ever having to run anything. The read-only
cross-project view built from this same store is `cross-project-matrix.md`.

## Behaviors & Operations

- **Populating the store (sync).** Triggered on demand, or automatically
  when a coding session ends. Every project's session activity records are
  scanned, each session is rolled up (per-model tokens, running time,
  parallelism), and one row per session is written into the **persistent
  log** — the single stored file that holds all performance data. Rows are
  keyed by session, so re-writing a session replaces its row rather than
  duplicating it (the automatic end-of-session write and any re-run are
  safe to repeat). A size+modified-time cache makes repeat scans fast —
  only records that changed since the last scan are re-read. The automatic
  end-of-session write parses only the just-ended session (never a full
  re-scan); it is best-effort and never delays or fails a session's end.
  Backfilling once brings the whole history into the log.

## Business Rules

- **R6 — One shared location, per-machine, project-tagged.** All sections
  append to one shared log for the whole machine; each section is tagged
  with its project and branch so entries from many projects coexist and
  stay attributable. The location honors the operator's standard per-user
  configuration location and can be redirected for testing. (D1 ·
  `0a459671`)
- **R7 — The store is append-only and machine-readable; reports are
  rendered on demand.** New sections are only ever appended; the
  human-readable report is produced from the store when asked, never kept
  as the store itself. (D1 · `0a459671`)
- **R9 — The persistent log is the source of truth; the view only reads
  it.** All performance data is written into one persistent log (rows are
  derived from the activity records the operator never has to track). The
  matrix view reads that log and never re-scans activity records at view
  time. Session rows are keyed by session, so writing the same session
  again replaces its row — the automatic end-of-session write can fire
  repeatedly with no double counting. (implementation-confirmed; D
  `62a7c7fd`)
- **R10 — The automatic refresh is best-effort and never blocks a
  session's end.** The end-of-session regeneration runs only when a cache
  already exists (so the one-time full scan is never paid at session end),
  and any failure in it is swallowed — a session always ends cleanly
  regardless of the matrix. (implementation-confirmed)

## Edge Cases Settled

- **E4 — Empty / missing log** — reading and rendering report "nothing
  logged yet".

## Pointers (implementation)

- **P1** — Aggregation core, per-session rollup + cross-project scan
  (`collectSessionRollups`/`scanProjects` + mtime/size cache), persistent
  store (`sessionRecord`/`upsertSessionRecords`/`readSessionRecords`/
  `syncSessionsToLog`), matrix build-from-log grouped by last folder
  (`buildMatrixFromLog`, `projectName`), HTML (`renderMatrixHtml`/
  `writeReport`), and section schema (`bee-perf/v1`):
  `packages/bee/lib/perf.mjs` (mirrored to
  `.bee/bin/lib/perf.mjs` and the `.claude`/`.agents` distribution trees).
- **P3** — Automatic write + refresh at session close: `maybePerfRefresh`
  in `packages/bee/hooks/bee-session-close.mjs` (Stop + PreCompact) — upserts the
  current session row into the log then rebuilds the HTML from the log;
  best-effort/fail-open, parses only the one current transcript.
- **P5** — Global store: `~/.config/beehive/performance.jsonl` (manual
  sections), `~/.config/beehive/performance.html` (matrix report),
  `~/.config/beehive/cache/scan-cache.json` (per-transcript rollup cache).
  `XDG_CONFIG_HOME` honored, `BEEHIVE_PERF_DIR` override for tests.
