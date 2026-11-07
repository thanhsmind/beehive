---
type: bee.delivery
title: tmux-herding-transport — delivery
description: "Delivery record for work item tmux-herding-transport: 6 capped cell(s), the contract-changing deviations, and the verify each cell capped against."
timestamp: 2026-08-22
bee:
  id: tmux-herding-transport-delivery
  lifecycle: active
  areas: [bee-herding]
  required_context: [docs/history/tmux-herding-transport/CONTEXT.md, docs/history/tmux-herding-transport/plan.md]
  sources: [docs/history/tmux-herding-transport/CONTEXT.md, docs/history/tmux-herding-transport/plan.md, .bee/cells/archive/tmux-herding-transport/tht-1.json, .bee/cells/archive/tmux-herding-transport/tht-2.json, .bee/cells/archive/tmux-herding-transport/tht-3.json, .bee/cells/archive/tmux-herding-transport/tht-4.json, .bee/cells/archive/tmux-herding-transport/tht-5.json, .bee/cells/archive/tmux-herding-transport/tht-6.json]
---

# tmux-herding-transport — Delivery

## What shipped

- **tht-1** — herding.transport selects the probe; both probes gained a tmux arm and transport.kind (3 file(s) changed)
- **tht-2** — run's private Herdr trait is now pub(crate) PaneTransport; Liveness and PaneGeom pub(crate); behavior unchanged (1 file(s) changed)
- **tht-3** — RealTmux implements PaneTransport over tmux verbs with a screen classifier and stub-tmux tests (2 file(s) changed)
- **tht-4** — bee herding run selects RealHerdr or RealTmux from herding.transport, refuses an illegal value before any side effect, and names the transport in dry-run JSON (2 file(s) changed)
- **tht-5** — Record the tmux pane id in the activity record (2 file(s) changed)
- **tht-6** — tmux transport documented: herding.transport plus every herding.tmux.* knob in the invariants reference and the annotated sample, a D1-D4 Transport section with the D5 source on the run-verb page, and the herdr-cli dependency reason split by transport (6 file(s) changed)

## Verify

Each cell below was capped only against a recorded passing verify result — bee refuses a cap without one.

- **tht-1** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee transport_`
- **tht-2** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee herding::run`
- **tht-3** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee herding::tmux`
- **tht-4** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee herding::run`
- **tht-5** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee activity`
- **tht-6** — `.bee/bin/bee knowledge check && .bee/bin/bee knowledge index --check && .bee/bin/bee dev release-manifest --check`

## Deviations

- **tht-1** — Pre-existing red at base, untouched by this cell: crates/bee/tests/opencode_plugin_contracts.rs two failures (activity PreToolUse matcher, opencode invalid-tool anchor) — confirmed failing with my changes stashed.
- **tht-2** — PaneGeom fields also made pub(crate): a pub(crate) type a sibling module must construct needs constructible fields
- **tht-2** — Commit ba3cd40 is empty: a concurrent worker (cell tht-5) git-added this file into commit c9f063bd before this cell committed; c9f063bd carries the exact rename diff (verified line by line), ba3cd40 records the cell trailer
- **tht-3** — process_info format gained a leading #{pane_id} field: list-panes -t <pane> resolves to the pane WINDOW and lists every pane in it, so the cell's three-field format could not tell the target row from a sibling worker's
- **tht-3** — agent_prompt preflights the pane and refuses a blocked screen before typing — the cell did not name it, D3's prohibition (no key into a dialog) requires it
- **tht-3** — pane_split falls open to tmux's default even split when the parent geometry cannot be read, rather than failing the spawn
- **tht-4** — Extracted read_main_config from execute_new so run() and execute_new share one config read instead of two copies
- **tht-4** — emit_result gained a transport param (one caller); the dry-run transport key is asserted through select_transport().name() rather than by capturing stdout, which emit_result writes directly
- **tht-6** — bee dev regen rewrote 15 paths outside the cell files list (the five rendered skill trees plus .bee/onboarding.json timestamp); reserved each under w-tht-6 before committing them with the cell

## Provenance

Proposed by `bee knowledge promote --work tmux-herding-transport` from 6 capped cell trace(s) in `.bee/cells/` and the anchor `docs/history/tmux-herding-transport/CONTEXT.md`, `docs/history/tmux-herding-transport/plan.md`; reviewed and accepted at the 2026-08-23 compounding pass. The proposal's second area tag, hook-runtime, was dropped: pane transport is herding work, and the one hook-runtime fact (tht-5) already lives in that area's activity-record concept. Sync-ack and in-cell bookkeeping rows were trimmed.
