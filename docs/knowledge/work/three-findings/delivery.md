---
type: bee.delivery
title: three-findings — delivery
description: "Delivery record proposed by bee knowledge promote for work item three-findings: 3 capped cell(s), 3 recorded deviation(s)."
timestamp: 2026-09-18
bee:
  id: three-findings-delivery
  lifecycle: active
  areas: [bee-herding, hook-runtime]
  required_context: [docs/history/three-findings/CONTEXT.md, docs/history/three-findings/plan.md]
  sources: [docs/history/three-findings/CONTEXT.md, docs/history/three-findings/plan.md, .bee/cells/archive/three-findings/thf-1.json, .bee/cells/archive/three-findings/thf-2.json, .bee/cells/archive/three-findings/thf-3.json]
---

# three-findings — Delivery

## What shipped

- **thf-1** — Delete the dead paths block from the dispatch prompts (9 file(s) changed)
- **thf-2** — Point the codex probe concept at the mechanism that exists (1 file(s) changed)
- **thf-3** — Publish the two herding flags in the command registry (2 file(s) changed)

## Verify

Each cell below was capped only against a recorded passing verify result — bee refuses a cap without one.

- **thf-1** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --no-fail-fast --manifest-path packages/bee-rs/Cargo.toml && .bee/bin/bee dev release-manifest --check && .bee/bin/bee onboard --repo-root . --json` — leader re-ran the exact compound: exit 0, 0 failures, manifest 376 files matched, onboard up_to_date; cmp also confirms each template matches its .bee/bin twin
- **thf-2** — `.bee/bin/bee knowledge check --json && .bee/bin/bee knowledge index --check --json` — leader re-ran both checks after the worker: errors 0, profile_errors 0, index drift false
- **thf-3** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --no-fail-fast --manifest-path packages/bee-rs/Cargo.toml` — leader re-ran the declared suite on the merged tree: exit 0, 36 result blocks, 0 failures, 4068 passed, including the catalog pinned-count test at 210

## Deviations

- **thf-1** — followed the plan
- **thf-2** — followed the plan
- **thf-3** — The worker proved with a filtered subset (cargo test ... -p bee -- catalog) instead of the cell's approved verify, so the leader re-ran the full declared command before this cap — the worker narrowed the proof, the plan did not — something else had to be fixed first

## Provenance

Proposed by `bee knowledge promote --work three-findings` from 3 capped cell trace(s) in `.bee/cells/` and the anchor `docs/history/three-findings/CONTEXT.md`, `docs/history/three-findings/plan.md`. Every line above is copied from a trace or from the work item; nothing here is curated truth until a human or agent accepts it.
