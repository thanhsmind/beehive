---
type: bee.delivery
title: paseo-system-prompts — delivery
description: "Delivery record proposed by bee knowledge promote for work item paseo-system-prompts: 1 capped cell(s), 1 recorded deviation(s)."
timestamp: 2026-10-15
bee:
  id: paseo-system-prompts-delivery
  lifecycle: active
  required_context: [docs/history/paseo-system-prompts/plan.md]
  sources: [docs/history/paseo-system-prompts/plan.md, .bee/cells/psp-1.json]
---

# paseo-system-prompts — Delivery

## What shipped

- **psp-1** — A <paseo-system> prompt now keeps a live waiting_on mark (the stale reap still runs on every prompt), and pi_plugin_contracts strips PASEO_AGENT_ID/PASEO_AGENT_CWD/PASEO_CLI from every extension-loading spawn — commit 47450f79b (2 file(s) changed)

## Verify

Each cell below was capped only against a recorded passing verify result — bee refuses a cap without one.

- **psp-1** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee --bin bee prompt_context && PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee --test pi_plugin_contracts` — leader re-ran prompt_context 39/39 and pi_plugin_contracts 122/122 inside a Paseo agent; active Paseo schedules after the run: 0

## Deviations

- **psp-1** — sync-ack: hooks/prompt_context.rs change is a one-predicate guard; decision f247ae6c carries the rule and no skill names the clear path

## Provenance

Proposed by `bee knowledge promote --work paseo-system-prompts` from 1 capped cell trace(s) in `.bee/cells/` and the anchor `docs/history/paseo-system-prompts/plan.md`. Every line above is copied from a trace or from the work item; nothing here is curated truth until a human or agent accepts it.
