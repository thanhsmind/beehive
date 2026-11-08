---
type: bee.delivery
title: release-clean-tree — delivery
description: "Delivery record proposed by bee knowledge promote for work item release-clean-tree: 1 capped cell(s), 2 recorded deviation(s)."
timestamp: 2026-09-15
bee:
  id: release-clean-tree-delivery
  lifecycle: active
  areas: [onboarding]
  required_context: [docs/history/release-clean-tree/plan.md]
  sources: [docs/history/release-clean-tree/plan.md, .bee/cells/archive/release-clean-tree/rct-1.json]
---

# release-clean-tree — Delivery

## What shipped

- **rct-1** — dispatched release passes the clean-tree check without widening it (5 file(s) changed)

## Verify

Each cell below was capped only against a recorded passing verify result — bee refuses a cap without one.

- **rct-1** — `bash -n scripts/release.sh && bash -n scripts/release-dirt.sh && PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee release_ && PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee deploy_` — touched release.sh dirt check/snapshot/restore, the new release-dirt.sh filter, .gitignore and deployment_prompt; syntax ok, release_ 40 passed incl. dirt-filter and ignore tests, deploy_ 8 passed in…

## Deviations

- **rct-1** — red-before-green not observed by the worker report; red holds by base state instead: scripts/release-dirt.sh absent at base 4a162ef1, git check-ignore on main returned exit=1 for .bee/authorizations/, deployment_prompt at base lacks the sentence
- **rct-1** — the abort-path truth (restore never resets the two bee paths) is covered by code review, not by a test

## Provenance

Proposed by `bee knowledge promote --work release-clean-tree` from 1 capped cell trace(s) in `.bee/cells/` and the anchor `docs/history/release-clean-tree/plan.md`. Every line above is copied from a trace or from the work item; nothing here is curated truth until a human or agent accepts it.
