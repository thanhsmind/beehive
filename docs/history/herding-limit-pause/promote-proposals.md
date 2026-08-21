promote proposal for work item "herding-limit-pause" (docs/history/herding-limit-pause/CONTEXT.md) — 3 capped cell(s): hlp-0, hlp-1, hlp-2
anchor: history — docs/history/herding-limit-pause/CONTEXT.md
PROPOSAL ONLY — nothing was written. Applying any section below is a human or agent decision.

(a) DELIVERY DRAFT — save as docs/knowledge/work/herding-limit-pause/delivery.md

---
type: bee.delivery
title: herding-limit-pause — delivery
description: "Delivery record proposed by bee knowledge promote for work item herding-limit-pause: 3 capped cell(s), 0 recorded deviation(s)."
timestamp: 2026-08-20
bee:
  id: herding-limit-pause-delivery
  lifecycle: active
  areas: [bee-herding]
  required_context: [docs/history/herding-limit-pause/CONTEXT.md]
  sources: [docs/history/herding-limit-pause/CONTEXT.md, .bee/cells/hlp-0.json, .bee/cells/hlp-1.json, .bee/cells/hlp-2.json]
---

# herding-limit-pause — Delivery

## What shipped

- **hlp-0** — pane_run reads raw exit status; JSON-envelope parse removed (1 file(s) changed)
- **hlp-1** — Usage-limit stop is a typed paused_limit outcome; pane kept; job.json stamped (1 file(s) changed)
- **hlp-2** — Same-round resume for limit-paused jobs via --continue; control loop holds the paused slot (2 file(s) changed)

## Verify

Each cell below was capped only against a recorded passing verify result — bee refuses a cap without one.

- **hlp-0** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml herding`
- **hlp-1** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml herding`
- **hlp-2** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml herding`

## Deviations

None recorded in the capped cell traces.

## Provenance

Proposed by `bee knowledge promote --work herding-limit-pause` from 3 capped cell trace(s) in `.bee/cells/` and the anchor `docs/history/herding-limit-pause/CONTEXT.md`. Every line above is copied from a trace or from the work item; nothing here is curated truth until a human or agent accepts it.

(b) AREA UPDATES — candidate spec-sync bullets, each citing its cell

areas: from the scribing stamp for "herding-limit-pause" — .bee/logs/scribing-runs.jsonl's most recent entry (2026-08-20T11:50:03.541Z), the work item declares no bee.areas.

area bee-herding:
  - [hlp-0] pane_run reads raw exit status; JSON-envelope parse removed — feature-wide sync per the scribing stamp, 1 file(s) changed (trace .bee/cells/hlp-0.json)
  - [hlp-1] Usage-limit stop is a typed paused_limit outcome; pane kept; job.json stamped — feature-wide sync per the scribing stamp, 1 file(s) changed (trace .bee/cells/hlp-1.json)
  - [hlp-2] Same-round resume for limit-paused jobs via --continue; control loop holds the paused slot — feature-wide sync per the scribing stamp, 2 file(s) changed (trace .bee/cells/hlp-2.json)

(c) PATTERN CANDIDATES — candidate bee.pattern concepts, bee.polarity pitfall

None: no capped cell trace carries a deviation or a failure signature.

knowledge promote: 3 capped cell(s) mined, 1 delivery draft, 3 area bullet(s), 0 pattern candidate(s), 0 file(s) written.