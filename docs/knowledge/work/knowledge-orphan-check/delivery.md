---
type: bee.delivery
title: knowledge-orphan-check — delivery
description: "Delivery record proposed by bee knowledge promote for work item knowledge-orphan-check: 1 capped cell(s), 1 recorded deviation(s)."
timestamp: 2026-09-06
bee:
  id: knowledge-orphan-check-delivery
  lifecycle: active
  required_context: [.bee/lanes/knowledge-orphan-check.json]
  sources: [.bee/lanes/knowledge-orphan-check.json, .bee/cells/archive/knowledge-orphan-check/koc-1.json]
---

# knowledge-orphan-check — Delivery

## What shipped

- **koc-1** — orphan list in knowledge check (6 file(s) changed)

## Verify

Each cell below was capped only against a recorded passing verify result — bee refuses a cap without one.

- **koc-1** — `cargo test --release --no-fail-fast --manifest-path packages/bee-rs/Cargo.toml author:: and knowledge; .bee/bin/bee knowledge check on the repo bundle stays OK; bee dev release-manifest --check`

## Deviations

- **koc-1** — orphans are a note and a JSON list, not a warning — 207 of 351 concepts are unlinked, so a warning would turn every check --strict red and broke 7 existing tests — the plan was wrong about a fact

## Provenance

Proposed by `bee knowledge promote --work knowledge-orphan-check` from 1 capped cell trace(s) in `.bee/cells/` and the anchor `.bee/lanes/knowledge-orphan-check.json`. Every line above is copied from a trace or from the work item; nothing here is curated truth until a human or agent accepts it.
