promote proposal for work item "herding-tier" (docs/history/herding-tier/CONTEXT.md + docs/history/herding-tier/plan.md) — 4 capped cell(s): ht-1, ht-2, ht-3, ht-4
anchor: history — docs/history/herding-tier/CONTEXT.md, docs/history/herding-tier/plan.md
PROPOSAL ONLY — nothing was written. Applying any section below is a human or agent decision.

(a) DELIVERY DRAFT — save as docs/knowledge/work/herding-tier/delivery.md

---
type: bee.delivery
title: herding-tier — delivery
description: "Delivery record proposed by bee knowledge promote for work item herding-tier: 4 capped cell(s), 0 recorded deviation(s)."
timestamp: 2026-08-20
bee:
  id: herding-tier-delivery
  lifecycle: active
  areas: [bee-herding]
  required_context: [docs/history/herding-tier/CONTEXT.md, docs/history/herding-tier/plan.md]
  sources: [docs/history/herding-tier/CONTEXT.md, docs/history/herding-tier/plan.md, .bee/cells/ht-1.json, .bee/cells/ht-2.json, .bee/cells/ht-3.json, .bee/cells/ht-4.json]
---

# herding-tier — Delivery

## What shipped

- **ht-1** — normalize_tier_value/resolve_tier/resolve_advisor route {kind:herding} per D1/D3 (2 file(s) changed)
- **ht-2** — bee herding run reads --task-file - from stdin; empty stdin refuses like an empty --task (2 file(s) changed)
- **ht-3** — prepare emits the herding-exec Bash payload; model guard denies Agent/Task on a herding tier (3 file(s) changed)
- **ht-4** — status/onboard display {kind:herding} as an executor (never a model name/crash); docs+samples document the config route (7 file(s) changed)

## Verify

Each cell below was capped only against a recorded passing verify result — bee refuses a cap without one.

- **ht-1** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml drivers`
- **ht-2** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml herding::run && PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml --test registry_dispatch`
- **ht-3** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml drivers && PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml model_guard`
- **ht-4** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml status_full && python3 -m json.tool .bee/config-sample.json > /dev/null && python3 -m json.tool .bee/config-sample-cli-executors.json > /dev/null`

## Deviations

None recorded in the capped cell traces.

## Provenance

Proposed by `bee knowledge promote --work herding-tier` from 4 capped cell trace(s) in `.bee/cells/` and the anchor `docs/history/herding-tier/CONTEXT.md`, `docs/history/herding-tier/plan.md`. Every line above is copied from a trace or from the work item; nothing here is curated truth until a human or agent accepts it.

(b) AREA UPDATES — candidate spec-sync bullets, each citing its cell

areas: from the scribing stamp for "herding-tier" — .bee/logs/scribing-runs.jsonl's most recent entry (2026-08-20T00:20:07.780Z), the work item declares no bee.areas.

area bee-herding:
  - [ht-1] normalize_tier_value/resolve_tier/resolve_advisor route {kind:herding} per D1/D3 — feature-wide sync per the scribing stamp, 2 file(s) changed (trace .bee/cells/ht-1.json)
  - [ht-2] bee herding run reads --task-file - from stdin; empty stdin refuses like an empty --task — feature-wide sync per the scribing stamp, 2 file(s) changed (trace .bee/cells/ht-2.json)
  - [ht-3] prepare emits the herding-exec Bash payload; model guard denies Agent/Task on a herding tier — feature-wide sync per the scribing stamp, 3 file(s) changed (trace .bee/cells/ht-3.json)

(c) PATTERN CANDIDATES — candidate bee.pattern concepts, bee.polarity pitfall

None: no capped cell trace carries a deviation or a failure signature.

knowledge promote: 4 capped cell(s) mined, 1 delivery draft, 3 area bullet(s), 0 pattern candidate(s), 0 file(s) written.