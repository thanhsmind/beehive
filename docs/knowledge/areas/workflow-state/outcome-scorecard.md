---
type: bee.area
title: Workflow State — the per-feature outcome scorecard (`bee report outcomes --by-feature`)
description: "How `bee report outcomes --by-feature [--since]` scores each feature across rework, judge revisions, reaction coverage, cycle time, gate reopens, fix-first cells, and worker cost, where each metric reads its source, and why missing sources print `not recorded` instead of zero."
timestamp: 2026-11-17
bee:
  id: workflow-state-outcome-scorecard
  lifecycle: active
  areas: [workflow-state]
  required_context: [areas/workflow-state/overview.md]
  decisions: ["outcome-scorecard D1 (docs/history/outcome-scorecard/CONTEXT.md, 2026-11-17 — report_from_job carries reaction, skipped, unknowns, found, and stopped onto herded caps)", "outcome-scorecard D2 (docs/history/outcome-scorecard/CONTEXT.md, 2026-11-17 — every gate set appends a gate audit row; gate-bypass stays for auto)", "outcome-scorecard D3 (docs/history/outcome-scorecard/CONTEXT.md, 2026-11-17 — read-only per-feature scorecard with not recorded for missing sources and --since bound)", "outcome-scorecard D4 (docs/history/outcome-scorecard/CONTEXT.md, 2026-11-17 — optional top-level fixes_feature: <slug> on fix-first cells)"]
  sources: [outcome-scorecard cells osc-1/osc-1t/osc-2/osc-2t/osc-3/osc-3t/osc-4/osc-4c (docs/history/outcome-scorecard/plan.md, 2026-11-17), packages/bee-rs/crates/bee/src/verbs/report_outcomes.rs, packages/bee-rs/crates/bee/src/verbs/cells/finish_support.rs, packages/bee-rs/crates/bee/src/verbs/state_group/set_gate.rs, packages/bee-rs/crates/bee/src/herding/run.rs, packages/bee-rs/crates/bee/src/verbs/drivers/close.rs]
  authoritative_for: "workflow-state: per-feature outcome scorecard metrics, sources, and missing-data rules"
  anchors: [packages/bee-rs/crates/bee/src/verbs/report_outcomes.rs, packages/bee-rs/crates/bee/src/herding/run.rs, packages/bee-rs/crates/bee/src/verbs/drivers/close.rs, packages/bee-rs/crates/bee/src/audit.rs]
  verified_at: 2026-11-25
  verified_commit: 9729d9879406757099df32ece356a4c21eb4239a
---

# Workflow State — the per-feature outcome scorecard (`bee report outcomes --by-feature`)

Read when: reading or extending `bee report outcomes --by-feature`, checking why a scorecard metric prints `not recorded`, or tracing where rework, cycle time, gate reopens, fix-first counts, or worker cost come from.

## Purpose & Boundaries

`bee report outcomes --by-feature [--since <duration|ISO>]` scores each feature from existing store records so an owner can tell whether agent work is improving over time, without changing default `bee report outcomes` output or mutating any store file (outcome-scorecard D3). It is read-only and never refuses on missing or partial data: any metric whose source is absent prints `not recorded` (`null` in `--json`), never `0`, so an unmeasured feature is never mistaken for a zero-defect one.

## Metrics & Sources

| Metric | Source(s) | Rule / Missing-source boundary |
|---|---|---|
| `cells`, `rework rate`, `judge revision rate`, `reaction coverage` | `.bee/cells/` and `.bee/cells/archive/` | Rates divide by `cells` or judge records; `0` denominator yields `not recorded`. Herded caps carry `trace.reaction` via `report_from_job` (outcome-scorecard D1). |
| `cycle time` | `.bee/lanes/<feature>.json` `created_at` to `.bee/usage/<feature>.json` `closed_at` | Falls back to newest `trace.capped_at` when no close usage record exists; `not recorded` when `created_at` or both end stamps are absent. |
| `gate reopens` | `.bee/audit.jsonl` rows of kind `gate` or `gate-bypass` whose note contains ` set to false` | Counts `gate` rows (outcome-scorecard D2); reads `gate-bypass` only when a feature has no `gate` rows so auto sets with both rows are not double-counted. |
| `fix-first cells` | Top-level `fixes_feature: <slug>` on cells (outcome-scorecard D4) | Counts cells naming this feature; `not recorded` when no cell in the store carries `fixes_feature`. |
| `cost` (`cost_usd`, `total_tokens`) | `.bee/mailbox/*/job.json` `usage` where `feature` matches | Summed across matching jobs; `not recorded` when no matching job carries usage. |

- **Filtering & ordering (outcome-scorecard D3).** `--since <7d|48h|ISO>` keeps features whose lane `created_at` is at or after the bound (dropping features with no lane `created_at`). Rows sort oldest-first, and the header pairs closed feature count beside the overall rework rate so throughput and rework stay visible together.

## Rejected Options

- **Git commit dates for cycle time:** rejected because squash merges and rebases rewrite commit timestamps; lane `created_at` and usage `closed_at` stay inside `.bee/`.
- **Stored scorecard at `bee close`:** not built (option B); the owner chose read-time derivation, which writes nothing.
- **Inferring fix-first cells from titles:** rejected as guesswork; explicit `fixes_feature` on the cell (outcome-scorecard D4) keeps the count exact.

## Pointers (implementation)

- Scorecard computation and formatting: `packages/bee-rs/crates/bee/src/verbs/report_outcomes.rs` (`read_inputs`, `scorecard`).
- Herded cap reaction carry: `packages/bee-rs/crates/bee/src/verbs/cells/finish_support.rs` (`report_from_job`).
- Gate audit row writer: `packages/bee-rs/crates/bee/src/verbs/state_group/set_gate.rs`.
- Job usage stamp: `packages/bee-rs/crates/bee/src/herding/run.rs`; close usage record: `packages/bee-rs/crates/bee/src/verbs/drivers/close.rs`.
