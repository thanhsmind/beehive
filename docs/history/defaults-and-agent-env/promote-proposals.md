promote proposal for work item "defaults-and-agent-env" (docs/history/defaults-and-agent-env/CONTEXT.md) — 2 capped cell(s): dae-1, dae-2
anchor: history — docs/history/defaults-and-agent-env/CONTEXT.md
PROPOSAL ONLY — nothing was written. Applying any section below is a human or agent decision.

(a) DELIVERY DRAFT — save as docs/knowledge/work/defaults-and-agent-env/delivery.md

---
type: bee.delivery
title: defaults-and-agent-env — delivery
description: "Delivery record proposed by bee knowledge promote for work item defaults-and-agent-env: 2 capped cell(s), 0 recorded deviation(s)."
timestamp: 2026-08-20
bee:
  id: defaults-and-agent-env-delivery
  lifecycle: active
  required_context: [docs/history/defaults-and-agent-env/CONTEXT.md]
  sources: [docs/history/defaults-and-agent-env/CONTEXT.md, .bee/cells/dae-1.json, .bee/cells/dae-2.json]
---

# defaults-and-agent-env — Delivery

## What shipped

- **dae-1** — Flipped absent-key defaults: uat_stop reads as close, staging_before_merge reads as false; repinned every ripple fixture across uat.rs, staging/mod.rs, worktree/tests.rs, drivers/close.rs, drivers/tests.rs, and tests/cells_archive_sweep.rs (8 file(s) changed)
- **dae-2** — Seeded built-in herd registry defaults (claude-sonnet, agy-flash) overridable by config, and added per-agent env carried through resolve_agent_command into a pane export line sent before agent start (3 file(s) changed)

## Verify

Each cell below was capped only against a recorded passing verify result — bee refuses a cap without one.

- **dae-1** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml uat staging`
- **dae-2** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml herding`

## Deviations

None recorded in the capped cell traces.

## Provenance

Proposed by `bee knowledge promote --work defaults-and-agent-env` from 2 capped cell trace(s) in `.bee/cells/` and the anchor `docs/history/defaults-and-agent-env/CONTEXT.md`. Every line above is copied from a trace or from the work item; nothing here is curated truth until a human or agent accepts it.

(b) AREA UPDATES — candidate spec-sync bullets, each citing its cell

None: the work item declares no bee.areas, so there is no area to sync (D19).

(c) PATTERN CANDIDATES — candidate bee.pattern concepts, bee.polarity pitfall

None: no capped cell trace carries a deviation or a failure signature.

knowledge promote: 2 capped cell(s) mined, 1 delivery draft, 0 area bullet(s), 0 pattern candidate(s), 0 file(s) written.