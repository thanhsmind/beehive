---
type: bee.delivery
title: pi-parity-review-fixes — delivery
description: "Delivery record proposed by bee knowledge promote for work item pi-parity-review-fixes: 4 capped cell(s), 7 recorded deviation(s)."
timestamp: 2026-09-14
bee:
  id: pi-parity-review-fixes-delivery
  lifecycle: active
  areas: [hook-runtime, workflow-state]
  required_context: [docs/history/pi-parity-review-fixes/CONTEXT.md, docs/history/pi-parity-review-fixes/plan.md]
  sources: [docs/history/pi-parity-review-fixes/CONTEXT.md, docs/history/pi-parity-review-fixes/plan.md, .bee/cells/archive/pi-parity-review-fixes/pprf-1.json, .bee/cells/archive/pi-parity-review-fixes/pprf-2.json, .bee/cells/archive/pi-parity-review-fixes/pprf-3.json, .bee/cells/archive/pi-parity-review-fixes/pprf-4.json]
---

# pi-parity-review-fixes — Delivery

## What shipped

- **pprf-1** — Pi session initialization now injects runtime-correct dispatch guidance on normal and compact paths. (6 file(s) changed)
- **pprf-2** — Pi now records pre-tool and nested user-wait activity without weakening write protection. (5 file(s) changed)
- **pprf-3** — Linearize workflow close and mailbox writes under workflow then handoff locks (8 file(s) changed)
- **pprf-4** — Pi doctor now reports fail-closed runtime health and has an accurate installed verification recipe. (8 file(s) changed)

## Verify

Each cell below was capped only against a recorded passing verify result — bee refuses a cap without one.

- **pprf-1** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml session_init && PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml --test pi_plugin_contracts injected` — runtime normalization, both injected branches, extracted command execution, and Pi herding-only dispatch passed
- **pprf-2** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml --test pi_plugin_contracts activity && PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml --test opencode_plugin_contracts pi_belt && .bee/bin/bee dev release-manifest --check` — four Pi activity tests, full six-test OpenCode parity suite, and 376 release-manifest entries passed
- **pprf-3** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml handoff && PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml --test workflow_verbs` — verified handoff and workflow_verbs suites
- **pprf-4** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml doctor::tests` — 32 doctor tests cover ready and all required Pi refusal classes

## Deviations

- **pprf-1** — followed the plan
- **pprf-2** — followed the plan
- **pprf-2** — sync-ack: Pi lifecycle behavior and its existing hook-runtime knowledge home changed together; no skill workflow changed.
- **pprf-3** — followed the plan
- **pprf-3** — sync-ack: Internal lock linearization does not alter skill workflows
- **pprf-4** — Retried the verification recipe after the user authorized release — two earlier worker results missed its complete JSON contract — hit an unforeseen obstacle
- **pprf-4** — sync-ack: Doctor behavior and its existing hook-runtime knowledge home changed together; no skill workflow changed.

## Provenance

Proposed by `bee knowledge promote --work pi-parity-review-fixes` from 4 capped cell trace(s) in `.bee/cells/` and the anchor `docs/history/pi-parity-review-fixes/CONTEXT.md`, `docs/history/pi-parity-review-fixes/plan.md`. Every line above is copied from a trace or from the work item; nothing here is curated truth until a human or agent accepts it.
