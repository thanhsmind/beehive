promote proposal for work item "workflow-verb-tests" (.bee/logs/scribing-runs.jsonl) — 1 capped cell(s): wvt-1
anchor: ledger — .bee/logs/scribing-runs.jsonl
PROPOSAL ONLY — nothing was written. Applying any section below is a human or agent decision.

(a) DELIVERY DRAFT — save as docs/knowledge/work/workflow-verb-tests/delivery.md

---
type: bee.delivery
title: workflow-verb-tests — delivery
description: "Delivery record proposed by bee knowledge promote for work item workflow-verb-tests: 1 capped cell(s), 0 recorded deviation(s)."
timestamp: 2026-08-06
bee:
  id: workflow-verb-tests-delivery
  lifecycle: active
  areas: [workflow-state]
  required_context: [.bee/logs/scribing-runs.jsonl]
  sources: [.bee/logs/scribing-runs.jsonl, .bee/cells/wvt-1.json]
---

# workflow-verb-tests — Delivery

## What shipped

- **wvt-1** — Added packages/bee-rs/crates/bee/tests/workflow_verbs.rs: 16 integration tests driving run_start_feature, run_workflows_list and run_workflows_close through the built binary (no run_x_body split exists for this trio, so an in-process unit test would assert on nothing) — clean start-feature + listing, all five start-feature guarded refusals with zero-mutation proof, workflows list mixed status/phase, close by --feature/--id/--all-but-active each leaving other records untouched, and both close modes refusing rather than degrading to "all" when the active feature is unresolvable. state_group/tests.rs untouched (test-only, no production change; no defect found). (1 file(s) changed)

## Verify

Each cell below was capped only against a recorded passing verify result — bee refuses a cap without one.

- **wvt-1** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml`

## Deviations

None recorded in the capped cell traces.

## Provenance

Proposed by `bee knowledge promote --work workflow-verb-tests` from 1 capped cell trace(s) in `.bee/cells/` and the anchor `.bee/logs/scribing-runs.jsonl`. Every line above is copied from a trace or from the work item; nothing here is curated truth until a human or agent accepts it.

(b) AREA UPDATES — candidate spec-sync bullets, each citing its cell

areas: from the scribing stamp for "workflow-verb-tests" — .bee/logs/scribing-runs.jsonl's most recent entry (2026-08-06T03:45:18.254Z), the work item declares no bee.areas.

area workflow-state:
  (no capped behavior_change cell exists for this feature)

(c) PATTERN CANDIDATES — candidate bee.pattern concepts, bee.polarity pitfall

None: no capped cell trace carries a deviation or a failure signature.

knowledge promote: 1 capped cell(s) mined, 1 delivery draft, 0 area bullet(s), 0 pattern candidate(s), 0 file(s) written.