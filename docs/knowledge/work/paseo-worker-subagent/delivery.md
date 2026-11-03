---
type: bee.delivery
title: paseo-worker-subagent — delivery
description: "Delivery record proposed by bee knowledge promote for work item paseo-worker-subagent: 1 capped cell(s), 1 recorded deviation(s)."
timestamp: 2026-10-15
bee:
  id: paseo-worker-subagent-delivery
  lifecycle: active
  required_context: [docs/history/paseo-worker-subagent/plan.md]
  sources: [docs/history/paseo-worker-subagent/plan.md, .bee/cells/pwsa-1.json]
---

# paseo-worker-subagent — Delivery

## What shipped

- **pwsa-1** — Paseo worker start now runs through call_as_caller (keeps PASEO_AGENT_ID/PASEO_AGENT_CWD) so the worker is the lead's subagent; every other paseo call stays scrubbed and --workspace is unchanged; knowledge doc and tests updated; commit ad71d1147 (3 file(s) changed)

## Verify

Each cell below was capped only against a recorded passing verify result — bee refuses a cap without one.

- **pwsa-1** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee --bin bee herding::` — leader re-ran herding:: 748/748; live run with the built binary: worker 329d1890 Cwd=/mnt/data/Projects/goglbe/beehive--wt--paseo-worker-subagent ParentAgentId=ed6469dc, evidence /mnt/data/Projects/g…

## Deviations

- **pwsa-1** — sync-ack: herding/paseo.rs and run.rs change; the-paseo-channel.md concept carries it and no bee-herding skill names the paseo env scrub

## Provenance

Proposed by `bee knowledge promote --work paseo-worker-subagent` from 1 capped cell trace(s) in `.bee/cells/` and the anchor `docs/history/paseo-worker-subagent/plan.md`. Every line above is copied from a trace or from the work item; nothing here is curated truth until a human or agent accepts it.
