# Outcome scorecard — Context

**Feature slug:** outcome-scorecard
**Date:** 2026-11-17
**Scope:** Standard
**Domain types:** BUILD

## What was asked

After the "coding agents made you a manager" review, the owner asked whether
bee scores its work and counts rework, and chose option A: extend
`bee report outcomes` into a per-feature scorecard, read-only, and fix the
gap where a herded worker's reaction never reaches its cell.

## What will be done

`bee report outcomes --by-feature` prints one row per feature with rates,
cycle time, gate reopens, fix-first cells and worker cost. Herded caps keep
the worker's reaction. Every gate change leaves an audit row so reopens can
be counted.

## Locked Decisions

| ID | Decision | Store id |
|---|---|---|
| D1 | `bee cells finish --from-job` copies the herding result's `reaction`, `skipped`, `unknowns`, `found` and `stopped` onto the cap exactly as the native cap path does. | 459dfff7 |
| D2 | Every `bee gate` / `bee state gate` set, manual or auto, appends one audit row of a new kind `gate` with note `gate <name> set to <true|false> by <actor>`; the existing `gate-bypass` row for auto sets stays. | 459dfff7 |
| D3 | `bee report outcomes --by-feature [--since <7d|ISO>]` prints, per feature: cells, rework rate (reworked cells / cells), judge revision rate (NEEDS_REVISION records / judge records), reaction coverage (cells with a reaction / cells), cycle time (lane `created_at` to the close record's `closed_at` in `.bee/usage/<feature>.json`, else to the newest `trace.capped_at`), gate reopens (audit rows of kind `gate` or `gate-bypass` whose note sets a gate to false), fix-first cells (cells whose optional `fixes_feature` names this feature), and summed `usage.cost_usd` and `usage.total_tokens` from `.bee/mailbox/*/job.json` with this feature. A value with no source prints `not recorded`, never 0. A header line puts features closed beside the overall rework rate. `--since` keeps features whose lane `created_at` is at or after the bound. Read-only, never refuses on data. | 459dfff7 |
| D4 | A cell may carry an optional top-level `fixes_feature: <slug>` naming the feature whose defect it fixes; planning docs say a fix-first cell sets it. | 459dfff7 |

## Read set

### Knowledge

| Concept | Grade at read | Why it matters |
|---------|---------------|----------------|
| docs/knowledge/patterns/20260818-a-green-count-is-not-evidence-that-your-new-test-ran.md | confirmed | testers quote filter counts |
| docs/knowledge/areas/workflow-state/cells-authoring-and-revision.md | suspect | the cell fields D4 extends |

### Code

| Path | What it decides |
|------|-----------------|
| `packages/bee-rs/crates/bee/src/verbs/report_outcomes.rs` | the outcome report |
| `packages/bee-rs/crates/bee/src/verbs/cells/finish_support.rs` | the cap built from a herding job |
| `packages/bee-rs/crates/bee/src/verbs/state_group/set_gate.rs` | gate sets and their audit row |
| `packages/bee-rs/crates/bee/src/audit.rs` | audit kinds |
| `packages/bee-rs/crates/bee/src/herding/run.rs` | per-job usage on job.json |
