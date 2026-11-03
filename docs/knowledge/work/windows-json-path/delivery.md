---
type: bee.delivery
title: windows-json-path — delivery
description: "Delivery record proposed by bee knowledge promote for work item windows-json-path: 1 capped cell(s), 2 recorded deviation(s)."
timestamp: 2026-09-22
bee:
  id: windows-json-path-delivery
  lifecycle: active
  required_context: [docs/history/windows-json-path/CONTEXT.md, docs/history/windows-json-path/plan.md]
  sources: [docs/history/windows-json-path/CONTEXT.md, docs/history/windows-json-path/plan.md, .bee/cells/wjp-1.json]
---

# windows-json-path — Delivery

## What shipped

- **wjp-1** — Built the herding ledger test's result JSON with serde_json::json! so Windows paths parse (1 file(s) changed)

## Verify

Each cell below was capped only against a recorded passing verify result — bee refuses a cap without one.

- **wjp-1** — `PATH="$HOME/.cargo/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee --bin bee ledger_outcome_blocked_with_report_fills_evidence` — the one rewritten test, run on Linux; the Windows path shape is proven only by the verify-windows CI job after the push

## Deviations

- **wjp-1** — The source edit landed a few seconds before the lane's merged gate was recorded in the same batch; recorded with bee mailbox reflect
- **wjp-1** — sync-ack: test-only fix of a JSON escape in one herding unit test; no herding behavior or skill text changes

## Provenance

Proposed by `bee knowledge promote --work windows-json-path` from 1 capped cell trace(s) in `.bee/cells/` and the anchor `docs/history/windows-json-path/CONTEXT.md`, `docs/history/windows-json-path/plan.md`. Every line above is copied from a trace or from the work item; nothing here is curated truth until a human or agent accepts it.
