promote proposal for work item "herding-liveness-signals" (docs/history/herding-liveness-signals/CONTEXT.md + docs/history/herding-liveness-signals/plan.md) — 2 capped cell(s): hls-1, hls-2
anchor: history — docs/history/herding-liveness-signals/CONTEXT.md, docs/history/herding-liveness-signals/plan.md
PROPOSAL ONLY — nothing was written. Applying any section below is a human or agent decision.

(a) DELIVERY DRAFT — save as docs/knowledge/work/herding-liveness-signals/delivery.md

---
type: bee.delivery
title: herding-liveness-signals — delivery
description: "Delivery record proposed by bee knowledge promote for work item herding-liveness-signals: 2 capped cell(s), 0 recorded deviation(s)."
timestamp: 2026-08-20
bee:
  id: herding-liveness-signals-delivery
  lifecycle: active
  areas: [bee-herding]
  required_context: [docs/history/herding-liveness-signals/CONTEXT.md, docs/history/herding-liveness-signals/plan.md]
  sources: [docs/history/herding-liveness-signals/CONTEXT.md, docs/history/herding-liveness-signals/plan.md, .bee/cells/archive/herding-liveness-signals/hls-1.json, .bee/cells/archive/herding-liveness-signals/hls-2.json]
---

# herding-liveness-signals — Delivery

## What shipped

- **hls-1** — Dead worker agent now reports a typed died outcome in about six seconds instead of after the 900s idle timeout (1 file(s) changed)
- **hls-2** — A stall now costs one pane read instead of roughly 4500 discarded subprocess spawns (1 file(s) changed)

## Verify

Each cell below was capped only against a recorded passing verify result — bee refuses a cap without one.

- **hls-1** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml herding`
- **hls-2** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml herding`

## Deviations

None recorded in the capped cell traces.

## Provenance

Proposed by `bee knowledge promote --work herding-liveness-signals` from 2 capped cell trace(s) in `.bee/cells/` and the anchor `docs/history/herding-liveness-signals/CONTEXT.md`, `docs/history/herding-liveness-signals/plan.md`. Every line above is copied from a trace or from the work item; nothing here is curated truth until a human or agent accepts it.

(b) AREA UPDATES — candidate spec-sync bullets, each citing its cell

areas: from the scribing stamp for "herding-liveness-signals" — .bee/logs/scribing-runs.jsonl's most recent entry (2026-08-20T13:38:41.540Z), the work item declares no bee.areas.

area bee-herding:
  - [hls-1] Dead worker agent now reports a typed died outcome in about six seconds instead of after the 900s idle timeout — feature-wide sync per the scribing stamp, 1 file(s) changed (trace .bee/cells/archive/herding-liveness-signals/hls-1.json)

(c) PATTERN CANDIDATES — candidate bee.pattern concepts, bee.polarity pitfall

None: no capped cell trace carries a deviation or a failure signature.

knowledge promote: 2 capped cell(s) mined, 1 delivery draft, 1 area bullet(s), 0 pattern candidate(s), 0 file(s) written.