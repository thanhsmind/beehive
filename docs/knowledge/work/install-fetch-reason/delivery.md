---
type: bee.delivery
title: install-fetch-reason — delivery
description: "Delivery record proposed by bee knowledge promote for work item install-fetch-reason: 1 capped cell(s), 2 recorded deviation(s)."
timestamp: 2026-09-07
bee:
  id: install-fetch-reason-delivery
  lifecycle: active
  required_context: [docs/history/install-fetch-reason/CONTEXT.md, docs/history/install-fetch-reason/plan.md]
  sources: [docs/history/install-fetch-reason/CONTEXT.md, docs/history/install-fetch-reason/plan.md, .bee/cells/archive/install-fetch-reason/ifr-1.json]
---

# install-fetch-reason — Delivery

## What shipped

- **ifr-1** — install.sh now names the transport error on both published-binary fallback lines and retries three times with a connect deadline; the contract test asserts both installers (4 file(s) changed)

## Verify

Each cell below was capped only against a recorded passing verify result — bee refuses a cap without one.

- **ifr-1** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --manifest-path packages/bee-rs/Cargo.toml --test installer_contracts --test installer_invocations --test instruction_laws && .bee/bin/bee dev release-manifest --check`

## Deviations

- **ifr-1** — Added a second test and a third assertion beyond plan.md — curl's multi-line stderr split the log verdict from its reason on the first driven run, so the one-line collapse is pinned as its own contract
- **ifr-1** — Added a third assertion and a second test beyond the plan: curl writes multi-line diagnostics, and the first driven run stranded the closing paren and the building-from-source verdict on their own lines — the reason had to be collapsed to one line to be readable, so the collapse is pinned too

## Provenance

Proposed by `bee knowledge promote --work install-fetch-reason` from 1 capped cell trace(s) in `.bee/cells/` and the anchor `docs/history/install-fetch-reason/CONTEXT.md`, `docs/history/install-fetch-reason/plan.md`. Every line above is copied from a trace or from the work item; nothing here is curated truth until a human or agent accepts it.
