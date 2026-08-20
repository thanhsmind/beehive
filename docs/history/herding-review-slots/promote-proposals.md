promote proposal for work item "herding-review-slots" (docs/history/herding-review-slots/CONTEXT.md) — 3 capped cell(s): hrv-1, hrv-2, hrv-3
anchor: history — docs/history/herding-review-slots/CONTEXT.md
PROPOSAL ONLY — nothing was written. Applying any section below is a human or agent decision.

(a) DELIVERY DRAFT — save as docs/knowledge/work/herding-review-slots/delivery.md

---
type: bee.delivery
title: herding-review-slots — delivery
description: "Delivery record proposed by bee knowledge promote for work item herding-review-slots: 3 capped cell(s), 0 recorded deviation(s)."
timestamp: 2026-08-20
bee:
  id: herding-review-slots-delivery
  lifecycle: active
  areas: [bee-herding]
  required_context: [docs/history/herding-review-slots/CONTEXT.md]
  sources: [docs/history/herding-review-slots/CONTEXT.md, .bee/cells/archive/herding-review-slots/hrv-1.json, .bee/cells/archive/herding-review-slots/hrv-2.json, .bee/cells/archive/herding-review-slots/hrv-3.json]
---

# herding-review-slots — Delivery

## What shipped

- **hrv-1** — reviewer/advisor purposes on a herding slot now resolve to the herding-exec Bash payload; gather stays on the default model; cli behavior unchanged (4 file(s) changed)
- **hrv-2** — Docs+samples now describe the full-purpose herding tier route (cell/gather/reviewer/advisor/extraction) and the optional fallback:default degradation, with copy-paste-safe sample text (4 file(s) changed)
- **hrv-3** — Widened herding routing to every purpose (D1) and added the fallback:default payload field (D3) (3 file(s) changed)

## Verify

Each cell below was capped only against a recorded passing verify result — bee refuses a cap without one.

- **hrv-1** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml drivers && PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml model_guard`
- **hrv-2** — `python3 -m json.tool .bee/config-sample.json > /dev/null && python3 -m json.tool .bee/config-sample-cli-executors.json > /dev/null && echo parse-ok`
- **hrv-3** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml drivers`

## Deviations

None recorded in the capped cell traces.

## Provenance

Proposed by `bee knowledge promote --work herding-review-slots` from 3 capped cell trace(s) in `.bee/cells/` and the anchor `docs/history/herding-review-slots/CONTEXT.md`. Every line above is copied from a trace or from the work item; nothing here is curated truth until a human or agent accepts it.

(b) AREA UPDATES — candidate spec-sync bullets, each citing its cell

areas: from the scribing stamp for "herding-review-slots" — .bee/logs/scribing-runs.jsonl's most recent entry (2026-08-20T06:45:21.482Z), the work item declares no bee.areas.

area bee-herding:
  - [hrv-1] reviewer/advisor purposes on a herding slot now resolve to the herding-exec Bash payload; gather stays on the default model; cli behavior unchanged — feature-wide sync per the scribing stamp, 4 file(s) changed (trace .bee/cells/archive/herding-review-slots/hrv-1.json)
  - [hrv-3] Widened herding routing to every purpose (D1) and added the fallback:default payload field (D3) — feature-wide sync per the scribing stamp, 3 file(s) changed (trace .bee/cells/archive/herding-review-slots/hrv-3.json)

(c) PATTERN CANDIDATES — candidate bee.pattern concepts, bee.polarity pitfall

None: no capped cell trace carries a deviation or a failure signature.

knowledge promote: 3 capped cell(s) mined, 1 delivery draft, 2 area bullet(s), 0 pattern candidate(s), 0 file(s) written.