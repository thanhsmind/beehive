promote proposal for work item "cli-surface-in-context" (docs/history/cli-surface-in-context/plan.md) — 4 capped cell(s): csc-1, csc-2, csc-3, csc-4
anchor: history — docs/history/cli-surface-in-context/plan.md
PROPOSAL ONLY — nothing was written. Applying any section below is a human or agent decision.

(a) DELIVERY DRAFT — save as docs/knowledge/work/cli-surface-in-context/delivery.md

---
type: bee.delivery
title: cli-surface-in-context — delivery
description: "Delivery record proposed by bee knowledge promote for work item cli-surface-in-context: 4 capped cell(s), 0 recorded deviation(s)."
timestamp: 2026-08-06
bee:
  id: cli-surface-in-context-delivery
  lifecycle: active
  areas: [hook-runtime]
  required_context: [docs/history/cli-surface-in-context/plan.md]
  sources: [docs/history/cli-surface-in-context/plan.md, .bee/cells/csc-1.json, .bee/cells/csc-2.json, .bee/cells/csc-3.json, .bee/cells/csc-4.json]
---

# cli-surface-in-context — Delivery

## What shipped

- **csc-1** — Render the whole registry into a new Command surface preamble section, path-sorted, --json omitted per-line and noted once in the header (2 file(s) changed)
- **csc-2** — unsupported_argument_shape now names every unknown flag with a nearest-spelling suggestion; unchanged when all flags are declared (1 file(s) changed)
- **csc-3** — Pinned the distinct flag-name count (143) as a vocabulary ratchet, naming the worker/agent divergence; no renames (1 file(s) changed)
- **csc-4** — Made catalog::distance pub(crate); router.rs's nearest_flag already called it directly via a tangled sibling commit, so only catalog.rs needed a change (2 file(s) changed)

## Verify

Each cell below was capped only against a recorded passing verify result — bee refuses a cap without one.

- **csc-1** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml session_preamble`
- **csc-2** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml router`
- **csc-3** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml catalog`
- **csc-4** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml router catalog`

## Deviations

None recorded in the capped cell traces.

## Provenance

Proposed by `bee knowledge promote --work cli-surface-in-context` from 4 capped cell trace(s) in `.bee/cells/` and the anchor `docs/history/cli-surface-in-context/plan.md`. Every line above is copied from a trace or from the work item; nothing here is curated truth until a human or agent accepts it.

(b) AREA UPDATES — candidate spec-sync bullets, each citing its cell

areas: from the scribing stamp for "cli-surface-in-context" — .bee/logs/scribing-runs.jsonl's most recent entry (2026-08-06T12:46:59.733Z), the work item declares no bee.areas.

area hook-runtime:
  - [csc-1] Render the whole registry into a new Command surface preamble section, path-sorted, --json omitted per-line and noted once in the header — feature-wide sync per the scribing stamp, 2 file(s) changed (trace .bee/cells/csc-1.json)
  - [csc-2] unsupported_argument_shape now names every unknown flag with a nearest-spelling suggestion; unchanged when all flags are declared — feature-wide sync per the scribing stamp, 1 file(s) changed (trace .bee/cells/csc-2.json)

(c) PATTERN CANDIDATES — candidate bee.pattern concepts, bee.polarity pitfall

None: no capped cell trace carries a deviation or a failure signature.

knowledge promote: 4 capped cell(s) mined, 1 delivery draft, 2 area bullet(s), 0 pattern candidate(s), 0 file(s) written.