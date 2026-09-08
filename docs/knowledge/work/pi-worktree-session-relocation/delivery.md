---
type: bee.delivery
title: pi-worktree-session-relocation — delivery
description: "Delivery record proposed by bee knowledge promote for work item pi-worktree-session-relocation: 3 capped cell(s), 1 recorded deviation(s)."
timestamp: 2026-09-07
bee:
  id: pi-worktree-session-relocation-delivery
  lifecycle: active
  areas: [worktree-parallelism, hook-runtime]
  required_context: [docs/history/pi-worktree-session-relocation/CONTEXT.md, docs/history/pi-worktree-session-relocation/plan.md]
  sources: [docs/history/pi-worktree-session-relocation/CONTEXT.md, docs/history/pi-worktree-session-relocation/plan.md, .bee/cells/archive/pi-worktree-session-relocation/pwsr-1.json, .bee/cells/archive/pi-worktree-session-relocation/pwsr-2.json, .bee/cells/archive/pi-worktree-session-relocation/pwsr-3.json]
---

# pi-worktree-session-relocation — Delivery

## What shipped

- **pwsr-1** — Emit verified worktree session-transition intent from the CLI (3 file(s) changed)
- **pwsr-2** — Relocate live Pi sessions across verified worktree boundaries (2 file(s) changed)
- **pwsr-3** — Document and verify Pi worktree session relocation end to end (5 file(s) changed)

## Verify

Each cell below was capped only against a recorded passing verify result — bee refuses a cap without one.

- **pwsr-1** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml verbs::worktree && PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml --test registry_contracts --test registry_dispatch`
- **pwsr-2** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml --test pi_plugin_contracts`
- **pwsr-3** — `/home/thanhsmind/.cache/cargo-target/release/bee dev release-manifest --check && PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --no-fail-fast --manifest-path packages/bee-rs/Cargo.toml`

## Deviations

- **pwsr-3** — release-manifest regeneration moved into pwsr-3 — pwsr-2 used the approved wave-barrier acknowledgment for its shipped Pi extension change — something else had to be fixed first

## Provenance

Proposed by `bee knowledge promote --work pi-worktree-session-relocation` from 3 capped cell trace(s) in `.bee/cells/` and the anchor `docs/history/pi-worktree-session-relocation/CONTEXT.md`, `docs/history/pi-worktree-session-relocation/plan.md`. Every line above is copied from a trace or from the work item; nothing here is curated truth until a human or agent accepts it.
