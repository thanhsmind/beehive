---
type: bee.delivery
title: sweep-recovery-door — delivery
description: "Delivery record proposed by bee knowledge promote for work item sweep-recovery-door: 3 capped cell(s), 0 recorded deviation(s)."
timestamp: 2026-08-13
bee:
  id: sweep-recovery-door-delivery
  lifecycle: active
  areas: [workflow-state]
  required_context: [docs/history/sweep-recovery-door/CONTEXT.md, docs/history/sweep-recovery-door/plan.md]
  sources: [docs/history/sweep-recovery-door/CONTEXT.md, docs/history/sweep-recovery-door/plan.md, .bee/cells/archive/sweep-recovery-door/srd-1.json, .bee/cells/archive/sweep-recovery-door/srd-2.json, .bee/cells/archive/sweep-recovery-door/srd-3.json]
---

# sweep-recovery-door — Delivery

## What shipped

- **srd-1** — sweep_expired_claims returns SweepSummary{released,parked,unreachable}; all 15 call sites compile unchanged; new test pins the sets against the decision rows (2 file(s) changed)
- **srd-2** — recovery scan serves as the releasing door: three independent passes (release/mark/report), R98 decline, sessions-lock race guard; recovery window still refuses (6 file(s) changed)
- **srd-3** — Both heartbeat writers clear the dead mark under their existing sessions lock and stamp revived_at, each proved by its own test; orient's D6 decline text and its two stale comments now name bee recovery scan (4 file(s) changed)

## Verify

Each cell below was capped only against a recorded passing verify result — bee refuses a cap without one.

- **srd-1** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml`
- **srd-2** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml`
- **srd-3** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml`

## Deviations

None recorded in the capped cell traces.

## Provenance

Proposed by `bee knowledge promote --work sweep-recovery-door` from 3 capped cell trace(s) in `.bee/cells/` and the anchor `docs/history/sweep-recovery-door/CONTEXT.md`, `docs/history/sweep-recovery-door/plan.md`. Every line above is copied from a trace or from the work item; Applied 2026-08-16 from docs/history/sweep-recovery-door/promote-proposals.md; area bullets declined (feature-wide scribing sync already stamped), no pattern candidates survived review.
