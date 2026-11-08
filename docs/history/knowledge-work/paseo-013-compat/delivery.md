---
type: bee.delivery
title: paseo-013-compat — delivery
description: "Delivery record proposed by bee knowledge promote for work item paseo-013-compat: 2 capped cell(s), 4 recorded deviation(s)."
timestamp: 2026-10-27
bee:
  id: paseo-013-compat-delivery
  lifecycle: active
  areas: [bee-herding]
  required_context: [docs/history/paseo-013-compat/CONTEXT.md, docs/history/paseo-013-compat/plan.md]
  sources: [docs/history/paseo-013-compat/CONTEXT.md, docs/history/paseo-013-compat/plan.md, .bee/cells/archive/paseo-013-compat/p13-1.json, .bee/cells/archive/paseo-013-compat/p13-2.json]
---

# paseo-013-compat — Delivery

## What shipped

- **p13-1** — Paseo workers spawn with --no-wait, every floor is 0.12.0, steer goes through paseo send --behavior steer for every provider; Node helper deleted (8 file(s) changed)
- **p13-2** — Channel doc names paseo run --no-wait, the 0.12.0 floor at doctor and pre-spawn, and the CLI steer with -- for every provider (1 file(s) changed)

## Verify

Each cell below was capped only against a recorded passing verify result — bee refuses a cap without one.

- **p13-1** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee herding:: && PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee doctor && ! rg -n 'paseo_steer::|mod paseo_steer|paseo_steer\.(rs|mjs)|0\.10\.3' packages/bee-rs/crates/bee/src/herding/job_verbs.rs packages/bee-rs/crates/bee/src/herding/run.rs packages/bee-rs/crates/bee/src/herding.rs packages/bee-rs/crates/bee/src/doctor.rs packages/bee-rs/crates/bee/src/doctor/tests.rs` — leader re-ran all three parts in the worktree: herding 776 passed, doctor 67 passed, rg gate prints nothing; full suite not run here, CI runs it on push
- **p13-2** — `! rg -n '0\.10\.3|paseo run -d|daemon helper|Node\.js|activeTurnBehavior|has no steer flag|WebSocket interface' docs/knowledge/areas/bee-herding/the-paseo-channel.md && rg -q 'paseo-013-compat' docs/knowledge/areas/bee-herding/the-paseo-channel.md` — docs-only cell, the cell verify parity check over the one edited file

## Deviations

- **p13-1** — herding/tests.rs listed in files but not edited — the non-Paseo steer test calls steer_job whose signature did not move — the plan was wrong about a fact
- **p13-1** — the spec binding in the Paseo branch became a match that keeps only the two paseo_steer_unsupported refusals — an unused let would be a dead variable — found a better route
- **p13-1** — sync-ack: skills/bee-herding describe the cockpit roles, not the paseo argv or version floor; the owning concept the-paseo-channel.md was updated in p13-2
- **p13-2** — spawn refusal block quotes the real run.rs text (paseo version 0.12.0 or newer required, FIX: install or update paseo (npm @getpaseo/cli 0.12.0+)) instead of the old 'FIX: upgrade paseo to ... or newer' line — the old doc line never matched the code — the plan was wrong about a fact

## Provenance

Proposed by `bee knowledge promote --work paseo-013-compat` from 2 capped cell trace(s) in `.bee/cells/` and the anchor `docs/history/paseo-013-compat/CONTEXT.md`, `docs/history/paseo-013-compat/plan.md`. Every line above is copied from a trace or from the work item; nothing here is curated truth until a human or agent accepts it.
