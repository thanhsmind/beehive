---
type: bee.delivery
title: herding-cockpit-completeness — delivery
description: "Delivery record proposed by bee knowledge promote for work item herding-cockpit-completeness: 10 capped cell(s), 11 recorded deviation(s)."
timestamp: 2026-09-06
bee:
  id: herding-cockpit-completeness-delivery
  lifecycle: active
  areas: [bee-herding]
  required_context: [docs/history/herding-cockpit-completeness/CONTEXT.md, docs/history/herding-cockpit-completeness/plan.md]
  sources: [docs/history/herding-cockpit-completeness/CONTEXT.md, docs/history/herding-cockpit-completeness/plan.md, .bee/cells/archive/herding-cockpit-completeness/hcc-1.json, .bee/cells/archive/herding-cockpit-completeness/hcc-2.json, .bee/cells/archive/herding-cockpit-completeness/hcc-3.json, .bee/cells/archive/herding-cockpit-completeness/hcc-4.json, .bee/cells/archive/herding-cockpit-completeness/hcc-5.json, .bee/cells/archive/herding-cockpit-completeness/hcc-6.json, .bee/cells/archive/herding-cockpit-completeness/hcc-7.json, .bee/cells/archive/herding-cockpit-completeness/hcc-8.json, .bee/cells/archive/herding-cockpit-completeness/hcc-9.json, .bee/cells/archive/herding-cockpit-completeness/hcc-10.json]
---

# herding-cockpit-completeness — Delivery

## What shipped

- **hcc-1** — Mark enum plus read/write/clear mark helpers on job.json (1 file(s) changed)
- **hcc-2** — Add pane_send_key to PaneTransport for herdr, tmux, and test doubles (3 file(s) changed)
- **hcc-3** — Interrupted and cancelled run outcomes from the job mark, continue refusal with FIX line, retryable envelope bit (1 file(s) changed)
- **hcc-4** — bee herding interrupt and cancel verbs with registry entries (4 file(s) changed)
- **hcc-5** — Add mark_orphans and transition_status helpers with unit tests (1 file(s) changed)
- **hcc-6** — Project stalled and recovered in run poll tick with progress lines (1 file(s) changed)
- **hcc-7** — Add jobs array to status and orphan sweep to status and occupancy (2 file(s) changed)
- **hcc-8** — Add git handoff block to done and blocked run envelopes (1 file(s) changed)
- **hcc-9** — Carry retryable bit on wave bucket rows and ledger worker rows (4 file(s) changed)
- **hcc-10** — Synced herding knowledge docs and config reference with cockpit signals (5 file(s) changed)

## Verify

Each cell below was capped only against a recorded passing verify result — bee refuses a cap without one.

- **hcc-1** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --no-fail-fast --manifest-path packages/bee-rs/Cargo.toml -p bee herding::mailbox`
- **hcc-2** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --no-fail-fast --manifest-path packages/bee-rs/Cargo.toml -p bee herding::`
- **hcc-3** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --no-fail-fast --manifest-path packages/bee-rs/Cargo.toml -p bee herding::run`
- **hcc-4** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --no-fail-fast --manifest-path packages/bee-rs/Cargo.toml -p bee herding::job_verbs && PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee --test registry_contracts --test registry_dispatch`
- **hcc-5** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --no-fail-fast --manifest-path packages/bee-rs/Cargo.toml -p bee herding::mailbox`
- **hcc-6** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --no-fail-fast --manifest-path packages/bee-rs/Cargo.toml -p bee herding::run`
- **hcc-7** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --no-fail-fast --manifest-path packages/bee-rs/Cargo.toml -p bee herding::`
- **hcc-8** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --no-fail-fast --manifest-path packages/bee-rs/Cargo.toml -p bee herding::run`
- **hcc-9** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --no-fail-fast --manifest-path packages/bee-rs/Cargo.toml -p bee herding::wave`
- **hcc-10** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --no-fail-fast --manifest-path packages/bee-rs/Cargo.toml -p bee --test registry_contracts`

## Deviations

- **hcc-1** — sync-ack: internal mailbox.rs helpers with no surface change; the bee-herding skill sync is hcc-10's docs cell after hcc-3..9 land
- **hcc-2** — sync-ack: skill docs update scheduled for slice 3 per plan.md
- **hcc-3** — sync-ack: run.rs behaviour change; the bee-herding skill and knowledge sync is hcc-10's docs cell after hcc-3..9 land
- **hcc-4** — sync-ack: new herding verbs; the bee-herding skill and knowledge sync is hcc-10's docs cell after hcc-3..9 land
- **hcc-5** — sync-ack: internal herding change; skill sync is hcc-10
- **hcc-6** — sync-ack: internal herding change; skill sync is hcc-10
- **hcc-7** — sync-ack: internal herding change; skill sync is hcc-10
- **hcc-8** — sync-ack: internal herding change; skill sync is hcc-10
- **hcc-9** — WorkerRow struct initialization in run.rs and control_loop.rs required retryable: None — field addition broke compilation — something else had to be fixed first
- **hcc-9** — sync-ack: internal herding change; skill sync is hcc-10
- **hcc-10** — sync-ack: this cell is the docs sync

## Provenance

Proposed by `bee knowledge promote --work herding-cockpit-completeness` from 10 capped cell trace(s) in `.bee/cells/` and the anchor `docs/history/herding-cockpit-completeness/CONTEXT.md`, `docs/history/herding-cockpit-completeness/plan.md`. Every line above is copied from a trace or from the work item; nothing here is curated truth until a human or agent accepts it.
