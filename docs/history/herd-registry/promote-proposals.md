promote proposal for work item "herd-registry" (docs/history/herd-registry/CONTEXT.md + docs/history/herd-registry/plan.md) — 3 capped cell(s): hr-1, hr-2, hr-3
anchor: history — docs/history/herd-registry/CONTEXT.md, docs/history/herd-registry/plan.md
PROPOSAL ONLY — nothing was written. Applying any section below is a human or agent decision.

(a) DELIVERY DRAFT — save as docs/knowledge/work/herd-registry/delivery.md

---
type: bee.delivery
title: herd-registry — delivery
description: "Delivery record proposed by bee knowledge promote for work item herd-registry: 3 capped cell(s), 0 recorded deviation(s)."
timestamp: 2026-08-20
bee:
  id: herd-registry-delivery
  lifecycle: active
  areas: [bee-herding]
  required_context: [docs/history/herd-registry/CONTEXT.md, docs/history/herd-registry/plan.md]
  sources: [docs/history/herd-registry/CONTEXT.md, docs/history/herd-registry/plan.md, .bee/cells/hr-1.json, .bee/cells/hr-2.json, .bee/cells/hr-3.json]
---

# herd-registry — Delivery

## What shipped

- **hr-1** — herding.agents registry + named resolution wired into resolve_agent_command and bee herding run --agent (3 file(s) changed)
- **hr-2** — kind:herding gains agent; prepare appends --agent when the slot names one, byte-identical otherwise (3 file(s) changed)
- **hr-3** — operational-invariants.md documents herding.agents (map shape, three reference spellings, unknown-name refusal, herd=pane rule); config-reference.md and both samples point at it, samples stay copy-paste-safe (4 file(s) changed)

## Verify

Each cell below was capped only against a recorded passing verify result — bee refuses a cap without one.

- **hr-1** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml herding && PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml --test registry_dispatch`
- **hr-2** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml drivers && PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml model_guard`
- **hr-3** — `python3 -m json.tool .bee/config-sample.json > /dev/null && python3 -m json.tool .bee/config-sample-cli-executors.json > /dev/null && echo parse-ok`

## Deviations

None recorded in the capped cell traces.

## Provenance

Proposed by `bee knowledge promote --work herd-registry` from 3 capped cell trace(s) in `.bee/cells/` and the anchor `docs/history/herd-registry/CONTEXT.md`, `docs/history/herd-registry/plan.md`. Every line above is copied from a trace or from the work item; nothing here is curated truth until a human or agent accepts it.

(b) AREA UPDATES — candidate spec-sync bullets, each citing its cell

areas: from the scribing stamp for "herd-registry" — .bee/logs/scribing-runs.jsonl's most recent entry (2026-08-20T04:45:13.764Z), the work item declares no bee.areas.

area bee-herding:
  - [hr-1] herding.agents registry + named resolution wired into resolve_agent_command and bee herding run --agent — feature-wide sync per the scribing stamp, 3 file(s) changed (trace .bee/cells/hr-1.json)
  - [hr-2] kind:herding gains agent; prepare appends --agent when the slot names one, byte-identical otherwise — feature-wide sync per the scribing stamp, 3 file(s) changed (trace .bee/cells/hr-2.json)

(c) PATTERN CANDIDATES — candidate bee.pattern concepts, bee.polarity pitfall

None: no capped cell trace carries a deviation or a failure signature.

knowledge promote: 3 capped cell(s) mined, 1 delivery draft, 2 area bullet(s), 0 pattern candidate(s), 0 file(s) written.