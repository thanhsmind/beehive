---
type: bee.delivery
title: deploy-pane-authorization — delivery
description: "Delivery record proposed by bee knowledge promote for work item deploy-pane-authorization: 1 capped cell(s), 2 recorded deviation(s)."
timestamp: 2026-09-15
bee:
  id: deploy-pane-authorization-delivery
  lifecycle: active
  areas: [bee-herding, doctrine-layer]
  required_context: [docs/history/deploy-pane-authorization/plan.md]
  sources: [docs/history/deploy-pane-authorization/plan.md, .bee/cells/archive/deploy-pane-authorization/dpa-1.json]
---

# deploy-pane-authorization — Delivery

## What shipped

- **dpa-1** — deploy permit and issuer session reach the herding worker pane (3 file(s) changed)

## Verify

Each cell below was capped only against a recorded passing verify result — bee refuses a cap without one.

- **dpa-1** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee deploy_ && PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee herding::run::` — touched the deployment export line in prepare_dispatch_wire and the fresh-spawn pane env in herding run; deploy_ 8 passed, herding::run:: 214 passed incl. 2 new passthrough tests, red before fix per …

## Deviations

- **dpa-1** — dpa-1 escalated to the session model: the agy-flash worker wrote the red tests, then its herding run died with the leader session restart; the leader wrote the two fix edits
- **dpa-1** — sync-ack: skills/bee-herding/* never documents the pane export line, BEE_HERDING_WORKER, BEE_HERDING_JOB_ID or deploy permits (rg over skills/bee-herding finds none), so no owned skill text goes stale; the behavior is captured in docs/knowledge/areas/bee-herding/handing-a-foreign-agent-its-brief.md after merge

## Provenance

Proposed by `bee knowledge promote --work deploy-pane-authorization` from 1 capped cell trace(s) in `.bee/cells/` and the anchor `docs/history/deploy-pane-authorization/plan.md`. Every line above is copied from a trace or from the work item; nothing here is curated truth until a human or agent accepts it.
