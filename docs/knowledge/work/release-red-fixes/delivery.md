---
type: bee.delivery
title: release-red-fixes — delivery
description: "Delivery record proposed by bee knowledge promote for work item release-red-fixes: 1 capped cell(s), 3 recorded deviation(s)."
timestamp: 2026-09-15
bee:
  id: release-red-fixes-delivery
  lifecycle: active
  areas: [onboarding]
  required_context: [docs/history/release-red-fixes/plan.md]
  sources: [docs/history/release-red-fixes/plan.md, .bee/cells/archive/release-red-fixes/rrf-1.json]
---

# release-red-fixes — Delivery

## What shipped

- **rrf-1** — the three release-gate reds are green without changing release behavior (5 file(s) changed)

## Verify

Each cell below was capped only against a recorded passing verify result — bee refuses a cap without one.

- **rrf-1** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee --bin bee no_shipped_command_spelling && BEE_SESSION_ID=pane-issuer-session PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee --test work_verbs && git check-ignore -q .bee/authorizations/x.json && PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee onboard::` — each red reproduced on main first (spelling test FAILED on the dis-1 span; work_verbs left Null with BEE_SESSION_ID set; regen then check-ignore exit=1); after the fix: spelling test 1 passed, work_v…

## Deviations

- **rrf-1** — rrf-1 escalated to the session model: the dispatched agy-flash worker never started because DNS failed for every host, so the leader made the edits inline
- **rrf-1** — the cli_shape change grew from one pinned exception entry to that entry plus counting distinct pinned exceptions hit, because release-red-fixes plan.md quotes the pinned span again inside its fenced cells JSON and occurrence counting read that as a dead exception; still test-only code, the recorded judge_obligation_ack holds
- **rrf-1** — sync-ack: skills owned by the touched areas never document the CLI spelling test's exception list, the gitignore template entries, or the work_verbs test env; rg over the owning skills finds none, so no owned skill text goes stale

## Provenance

Proposed by `bee knowledge promote --work release-red-fixes` from 1 capped cell trace(s) in `.bee/cells/` and the anchor `docs/history/release-red-fixes/plan.md`. Every line above is copied from a trace or from the work item; nothing here is curated truth until a human or agent accepts it.
