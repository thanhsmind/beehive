promote proposal for work item "herding-executor" (docs/history/herding-executor/CONTEXT.md + docs/history/herding-executor/plan.md) — 7 capped cell(s): hx-1, hx-2, hx-3, hx-4, hx-5, hx-6, hx-7
anchor: history — docs/history/herding-executor/CONTEXT.md, docs/history/herding-executor/plan.md
PROPOSAL ONLY — nothing was written. Applying any section below is a human or agent decision.

(a) DELIVERY DRAFT — save as docs/knowledge/work/herding-executor/delivery.md

---
type: bee.delivery
title: herding-executor — delivery
description: "Delivery record proposed by bee knowledge promote for work item herding-executor: 7 capped cell(s), 0 recorded deviation(s)."
timestamp: 2026-08-19
bee:
  id: herding-executor-delivery
  lifecycle: active
  areas: [bee-herding]
  required_context: [docs/history/herding-executor/CONTEXT.md, docs/history/herding-executor/plan.md]
  sources: [docs/history/herding-executor/CONTEXT.md, docs/history/herding-executor/plan.md, .bee/cells/hx-1.json, .bee/cells/hx-2.json, .bee/cells/hx-3.json, .bee/cells/hx-4.json, .bee/cells/hx-5.json, .bee/cells/hx-6.json, .bee/cells/hx-7.json]
---

# herding-executor — Delivery

## What shipped

- **hx-1** — Mailbox contract module: pure path layout, self-contained brief renderer, and typed-error result-N.json parsing (2 file(s) changed)
- **hx-2** — Removed bee's herdr agent-kind allow-list; token 0 passes through, herdr validates --kind itself (3 file(s) changed)
- **hx-3** — Exempted .bee/mailbox/ from the scratch-shape write-guard deny per D8 (2 file(s) changed)
- **hx-4** — Added bee herding run: pane split, agent start, native mailbox poll (idle-timeout/ceiling), pane lifecycle close, dispatch/wave-ledger bookkeeping, --dry-run, and the registry entry (5 file(s) changed)
- **hx-5** — Documented bee herding run in operational-invariants.md, synced the bee-herding knowledge overview, and named the herding execution branch beside cli in gates-and-delegation.md (3 file(s) changed)
- **hx-6** — bee herding run --continue reuses the job mailbox for follow-up rounds: sends the round N+1 brief via agent prompt (never agent start), waits on result-(N+1), and refuses typed on a missing job dir, prior result, or dead pane (2 file(s) changed)
- **hx-7** — bee-swarming wiring: the herding execution path and its orchestrator bookkeeping (1 file(s) changed)

## Verify

Each cell below was capped only against a recorded passing verify result — bee refuses a cap without one.

- **hx-1** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml mailbox`
- **hx-2** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml wave`
- **hx-3** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml write_guard`
- **hx-4** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml herding && PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml --test registry_dispatch`
- **hx-5** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo run --release --manifest-path packages/bee-rs/Cargo.toml -- knowledge index --check`
- **hx-6** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml herding && PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml --test registry_dispatch`
- **hx-7** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo run --release --manifest-path packages/bee-rs/Cargo.toml -- dev release-manifest --check`

## Deviations

None recorded in the capped cell traces.

## Provenance

Proposed by `bee knowledge promote --work herding-executor` from 7 capped cell trace(s) in `.bee/cells/` and the anchor `docs/history/herding-executor/CONTEXT.md`, `docs/history/herding-executor/plan.md`. Every line above is copied from a trace or from the work item; nothing here is curated truth until a human or agent accepts it.

(b) AREA UPDATES — candidate spec-sync bullets, each citing its cell

areas: from the scribing stamp for "herding-executor" — .bee/logs/scribing-runs.jsonl's most recent entry (2026-08-19T22:59:43.369Z), the work item declares no bee.areas.

area bee-herding:
  - [hx-1] Mailbox contract module: pure path layout, self-contained brief renderer, and typed-error result-N.json parsing — feature-wide sync per the scribing stamp, 2 file(s) changed (trace .bee/cells/hx-1.json)
  - [hx-2] Removed bee's herdr agent-kind allow-list; token 0 passes through, herdr validates --kind itself — feature-wide sync per the scribing stamp, 3 file(s) changed (trace .bee/cells/hx-2.json)
  - [hx-3] Exempted .bee/mailbox/ from the scratch-shape write-guard deny per D8 — feature-wide sync per the scribing stamp, 2 file(s) changed (trace .bee/cells/hx-3.json)
  - [hx-4] Added bee herding run: pane split, agent start, native mailbox poll (idle-timeout/ceiling), pane lifecycle close, dispatch/wave-ledger bookkeeping, --dry-run, and the registry entry — feature-wide sync per the scribing stamp, 5 file(s) changed (trace .bee/cells/hx-4.json)
  - [hx-6] bee herding run --continue reuses the job mailbox for follow-up rounds: sends the round N+1 brief via agent prompt (never agent start), waits on result-(N+1), and refuses typed on a missing job dir, prior result, or dead pane — feature-wide sync per the scribing stamp, 2 file(s) changed (trace .bee/cells/hx-6.json)

(c) PATTERN CANDIDATES — candidate bee.pattern concepts, bee.polarity pitfall

None: no capped cell trace carries a deviation or a failure signature.

knowledge promote: 7 capped cell(s) mined, 1 delivery draft, 5 area bullet(s), 0 pattern candidate(s), 0 file(s) written.