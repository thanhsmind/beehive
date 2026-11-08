---
type: bee.delivery
title: agent-model-unpin — delivery
description: "Delivery record proposed by bee knowledge promote for work item agent-model-unpin: 2 capped cell(s), 3 recorded deviation(s)."
timestamp: 2026-08-26
bee:
  id: agent-model-unpin-delivery
  lifecycle: active
  required_context: [docs/history/agent-model-unpin/CONTEXT.md, docs/history/agent-model-unpin/plan.md]
  sources: [docs/history/agent-model-unpin/CONTEXT.md, docs/history/agent-model-unpin/plan.md, .bee/cells/archive/agent-model-unpin/amu-1.json, .bee/cells/archive/agent-model-unpin/amu-2.json]
---

# agent-model-unpin — Delivery

## What shipped

- **amu-1** — Claude agent files render without a model pin and survive herded/cli slot shapes; dispatch model param is the sole authority (7 file(s) changed)
- **amu-2** — Claude drift check flags a legacy model pin and accepts unpinned files under any slot shape; opencode verdicts untouched (2 file(s) changed)

## Verify

Each cell below was capped only against a recorded passing verify result — bee refuses a cap without one.

- **amu-1** — `cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee onboard — green over the changed plan/render logic; bee dev regen (render-skill-trees, onboard --apply, release-manifest --write) then bee dev release-manifest --check green`
- **amu-2** — `cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee status_full — green including the flipped drift cases`

## Deviations

- **amu-1** — Executed inline by the orchestrator session instead of a dispatched bee-build worker — the bee-build agent type was removed from this session by the herded-generation regen, the exact defect this cell fixes — hit an unforeseen obstacle
- **amu-2** — Executed inline by the orchestrator session instead of a dispatched bee-build worker — the bee-build agent type was removed from this session by the herded-generation regen, the defect this feature fixes — hit an unforeseen obstacle
- **amu-2** — The Claude agent-file-malformed arm was retired rather than kept — with no model line required there is nothing to fail to read, so a plain-text file is clean — the plan was wrong about a fact

## Provenance

Proposed by `bee knowledge promote --work agent-model-unpin` from 2 capped cell trace(s) in `.bee/cells/` and the anchor `docs/history/agent-model-unpin/CONTEXT.md`, `docs/history/agent-model-unpin/plan.md`. Every line above is copied from a trace or from the work item; nothing here is curated truth until a human or agent accepts it.

