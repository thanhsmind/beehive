---
type: bee.delivery
title: pi-1-4-capability-audit — delivery
description: "Delivery record proposed by bee knowledge promote for work item pi-1-4-capability-audit: 1 capped cell(s), 2 recorded deviation(s)."
timestamp: 2026-10-27
bee:
  id: pi-1-4-capability-audit-delivery
  lifecycle: active
  required_context: [docs/history/pi-1-4-capability-audit/CONTEXT.md]
  sources: [docs/history/pi-1-4-capability-audit/CONTEXT.md, .bee/cells/p14-1.json]
---

# pi-1-4-capability-audit — Delivery

## What shipped

- **p14-1** — Support pi 1.4.0 in the bee Pi belt (6 file(s) changed)

## Verify

Each cell below was capped only against a recorded passing verify result — bee refuses a cap without one.

- **p14-1** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml --test pi_plugin_contracts && .bee/bin/bee dev release-manifest --check && pi -p "Reply with the single word ok"` — the cargo segment passed 123 of 123 and the manifest segment matched 568 files; the pi -p segment loaded the belt (bee prompt-context hook line printed) but the model call returned 402 Insufficient B…

## Deviations

- **p14-1** — herding run hit the 1800s ceiling while the worker ran the full crate suite after the narrow verify was green; the leader re-ran the narrow verify and the worker committed on a follow-up prompt
- **p14-1** — pi -p one-shot could not print ok: provider returned 402 Insufficient Balance (not a belt fault); the worker guard also refuses pi for a Paseo worker

## Provenance

Proposed by `bee knowledge promote --work pi-1-4-capability-audit` from 1 capped cell trace(s) in `.bee/cells/` and the anchor `docs/history/pi-1-4-capability-audit/CONTEXT.md`. Every line above is copied from a trace or from the work item; nothing here is curated truth until a human or agent accepts it.
