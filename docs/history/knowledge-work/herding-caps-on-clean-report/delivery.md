---
type: bee.delivery
title: herding-caps-on-clean-report — delivery
description: "Delivery record proposed by bee knowledge promote for work item herding-caps-on-clean-report: 2 capped cell(s), 2 recorded deviation(s)."
timestamp: 2026-09-19
bee:
  id: herding-caps-on-clean-report-delivery
  lifecycle: active
  areas: [dispatch-door, herding, workflow-state]
  required_context: [docs/history/herding-caps-on-clean-report/CONTEXT.md, docs/history/herding-caps-on-clean-report/plan.md]
  sources: [docs/history/herding-caps-on-clean-report/CONTEXT.md, docs/history/herding-caps-on-clean-report/plan.md, .bee/cells/archive/herding-caps-on-clean-report/wci-1.json, .bee/cells/archive/herding-caps-on-clean-report/wci-2.json]
---

# herding-caps-on-clean-report — Delivery

## What shipped

- **wci-1** — Made finish instruction runnable, single, and terminal at the end of worker-cell prompt (5 file(s) changed)
- **wci-2** — Dispatched real execution worker in control-bee sandbox and verified self-cap (1 file(s) changed)

## Verify

Each cell below was capped only against a recorded passing verify result — bee refuses a cap without one.

- **wci-1** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --no-fail-fast --manifest-path packages/bee-rs/Cargo.toml -p bee onboard && .bee/bin/bee dev release-manifest --check` — cell verify passed onboard unit tests and release manifest check
- **wci-2** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --no-fail-fast --manifest-path packages/bee-rs/Cargo.toml -p bee onboard` — live drive in sandbox verified worker self-cap

## Deviations

- **wci-1** — checked worktree root for prompt skew in prepare.rs — dispatch prepare in a granted worktree would otherwise compare against main — hit an unforeseen obstacle
- **wci-2** — followed the plan

## Provenance

Proposed by `bee knowledge promote --work herding-caps-on-clean-report` from 2 capped cell trace(s) in `.bee/cells/` and the anchor `docs/history/herding-caps-on-clean-report/CONTEXT.md`, `docs/history/herding-caps-on-clean-report/plan.md`. Every line above is copied from a trace or from the work item; nothing here is curated truth until a human or agent accepts it.
