---
type: bee.delivery
title: herding-agent-brief — delivery
description: "Delivery record proposed by bee knowledge promote for work item herding-agent-brief: 2 capped cell(s), 2 recorded deviation(s)."
timestamp: 2026-09-08
bee:
  id: herding-agent-brief-delivery
  lifecycle: active
  areas: [bee-herding]
  required_context: [.bee/logs/scribing-runs.jsonl, .bee/lanes/herding-agent-brief.json, docs/history/herding-agent-brief/promote-proposals.md]
  sources: [.bee/logs/scribing-runs.jsonl, .bee/lanes/herding-agent-brief.json, docs/history/herding-agent-brief/promote-proposals.md, .bee/cells/archive/herding-agent-brief/hab-1.json, .bee/cells/archive/herding-agent-brief/hab-2.json]
---

# herding-agent-brief — Delivery

## What shipped

- **hab-1** — Embedded the four bee agent bodies in prompt.rs and moved split_frontmatter to textutil.rs as the single parser (3 file(s) changed)
- **hab-2** — Every non-Agent dispatch payload now carries the related bee agent body above its kind brief (2 file(s) changed)

## Verify

Each cell below was capped only against a recorded passing verify result — bee refuses a cap without one.

- **hab-1** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml drivers::prompt onboard::agents`
- **hab-2** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml drivers`

## Deviations

- **hab-1** — followed the plan
- **hab-2** — followed the plan

## Provenance

Proposed by `bee knowledge promote --work herding-agent-brief` from 2 capped cell trace(s) in `.bee/cells/` and the anchor `.bee/logs/scribing-runs.jsonl`, `.bee/lanes/herding-agent-brief.json`, `docs/history/herding-agent-brief/promote-proposals.md`. Every line above is copied from a trace or from the work item; nothing here is curated truth until a human or agent accepts it.
