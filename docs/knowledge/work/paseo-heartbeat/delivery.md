---
type: bee.delivery
title: paseo-heartbeat — delivery
description: "Delivery record proposed by bee knowledge promote for work item paseo-heartbeat: 4 capped cell(s), 0 recorded deviation(s)."
timestamp: 2026-10-05
bee:
  id: paseo-heartbeat-delivery
  lifecycle: active
  areas: [bee-herding]
  required_context: [docs/history/paseo-heartbeat/CONTEXT.md, docs/history/paseo-heartbeat/plan.md]
  sources: [docs/history/paseo-heartbeat/CONTEXT.md, docs/history/paseo-heartbeat/plan.md, .bee/cells/phb-1.json, .bee/cells/phb-2.json, .bee/cells/phb-3.json, .bee/cells/phb-4.json]
---

# paseo-heartbeat — Delivery

## What shipped

- **phb-1** — Paseo heartbeat helpers with node contract tests (2 file(s) changed)
- **phb-2** — A Pi leader in Paseo creates one bee-leader heartbeat and turns each firing into a short turn (2 file(s) changed)
- **phb-3** — The Paseo channel concept documents the leader heartbeat (1 file(s) changed)
- **phb-4** — A heartbeat marker with no schedule id older than 120 s is retried (2 file(s) changed)

## Verify

Each cell below was capped only against a recorded passing verify result — bee refuses a cap without one.

- **phb-1** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee --test pi_paseo_heartbeat_contracts` — 7 passed 0 failed under node, run by the leader
- **phb-2** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee --test pi_paseo_heartbeat_contracts --test pi_plugin_contracts` — 13 and 114 passed 0 failed; live Pi leader cb19f50e in Paseo 0.10.3 got schedule dd831d12, two firings at 23:45 and 23:50 became the ok turn and the leader was idle after each; evidence /home/thanhsm…
- **phb-3** — `.bee/bin/bee knowledge check --json` — errors 0, orphans 5 (none new)
- **phb-4** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee --test pi_paseo_heartbeat_contracts` — 17 passed 0 failed (pi_plugin_contracts 114 passed too), run by the leader

## Deviations

None recorded in the capped cell traces.

## Provenance

Proposed by `bee knowledge promote --work paseo-heartbeat` from 4 capped cell trace(s) in `.bee/cells/` and the anchor `docs/history/paseo-heartbeat/CONTEXT.md`, `docs/history/paseo-heartbeat/plan.md`. Every line above is copied from a trace or from the work item; nothing here is curated truth until a human or agent accepts it.
