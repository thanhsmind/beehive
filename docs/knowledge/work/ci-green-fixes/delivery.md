---
type: bee.delivery
title: ci-green-fixes — delivery
description: "Delivery record proposed by bee knowledge promote for work item ci-green-fixes: 2 capped cell(s), 1 recorded deviation(s)."
timestamp: 2026-09-22
bee:
  id: ci-green-fixes-delivery
  lifecycle: active
  areas: [bee-herding]
  required_context: [docs/history/ci-green-fixes/CONTEXT.md, docs/history/ci-green-fixes/plan.md]
  sources: [docs/history/ci-green-fixes/CONTEXT.md, docs/history/ci-green-fixes/plan.md, .bee/cells/cgf-1.json, .bee/cells/cgf-2.json]
---

# ci-green-fixes — Delivery

## What shipped

- **cgf-1** — Windows delete-pending split lock reads as busy and is retried (1 file(s) changed)
- **cgf-2** — Pi marker fixture polls for its deferred switch instead of sleeping 50 ms (1 file(s) changed)

## Verify

Each cell below was capped only against a recorded passing verify result — bee refuses a cap without one.

- **cgf-1** — `PATH="$HOME/.cargo/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee --bin bee herding::split_lock` — the module's tests on Linux, where the new arm is compiled out; the Windows path is proven only by the verify-windows CI job after the push
- **cgf-2** — `PATH="$HOME/.cargo/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee --test pi_plugin_contracts shell_tool_result_captures_marker` — the one fixture under node v24.19.0; the other sleep_step(50) fixtures were not changed or re-run

## Deviations

- **cgf-1** — sync-ack: one cfg(windows) match arm in the split lock; no herding behavior on unix and no skill text changes

## Provenance

Proposed by `bee knowledge promote --work ci-green-fixes` from 2 capped cell trace(s) in `.bee/cells/` and the anchor `docs/history/ci-green-fixes/CONTEXT.md`, `docs/history/ci-green-fixes/plan.md`. Every line above is copied from a trace or from the work item; nothing here is curated truth until a human or agent accepts it.
