promote proposal for work item "sweep-recovery-door" (docs/history/sweep-recovery-door/CONTEXT.md + docs/history/sweep-recovery-door/plan.md) — 3 capped cell(s): srd-1, srd-2, srd-3
anchor: history — docs/history/sweep-recovery-door/CONTEXT.md, docs/history/sweep-recovery-door/plan.md
PROPOSAL ONLY — nothing was written. Applying any section below is a human or agent decision.

(a) DELIVERY DRAFT — save as docs/knowledge/work/sweep-recovery-door/delivery.md

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
  sources: [docs/history/sweep-recovery-door/CONTEXT.md, docs/history/sweep-recovery-door/plan.md, .bee/cells/srd-1.json, .bee/cells/srd-2.json, .bee/cells/srd-3.json]
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

Proposed by `bee knowledge promote --work sweep-recovery-door` from 3 capped cell trace(s) in `.bee/cells/` and the anchor `docs/history/sweep-recovery-door/CONTEXT.md`, `docs/history/sweep-recovery-door/plan.md`. Every line above is copied from a trace or from the work item; nothing here is curated truth until a human or agent accepts it.

(b) AREA UPDATES — candidate spec-sync bullets, each citing its cell

areas: from the scribing stamp for "sweep-recovery-door" — .bee/logs/scribing-runs.jsonl's most recent entry (2026-08-13T17:38:32.305Z), the work item declares no bee.areas.

area workflow-state:
  - [srd-2] recovery scan serves as the releasing door: three independent passes (release/mark/report), R98 decline, sessions-lock race guard; recovery window still refuses — feature-wide sync per the scribing stamp, 6 file(s) changed (trace .bee/cells/srd-2.json)
  - [srd-3] Both heartbeat writers clear the dead mark under their existing sessions lock and stamp revived_at, each proved by its own test; orient's D6 decline text and its two stale comments now name bee recovery scan — feature-wide sync per the scribing stamp, 4 file(s) changed (trace .bee/cells/srd-3.json)

(c) PATTERN CANDIDATES — candidate bee.pattern concepts, bee.polarity pitfall

None: no capped cell trace carries a deviation or a failure signature.

knowledge promote: 3 capped cell(s) mined, 1 delivery draft, 2 area bullet(s), 0 pattern candidate(s), 0 file(s) written.