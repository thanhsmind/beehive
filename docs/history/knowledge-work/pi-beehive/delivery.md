---
type: bee.delivery
title: pi-beehive — delivery
description: "Delivery record proposed by bee knowledge promote for work item pi-beehive: 6 capped cell(s), 9 recorded deviation(s)."
timestamp: 2026-09-02
bee:
  id: pi-beehive-delivery
  lifecycle: active
  areas: [hook-runtime]
  required_context: [docs/history/pi-beehive/CONTEXT.md, docs/history/pi-beehive/plan.md]
  sources: [docs/history/pi-beehive/CONTEXT.md, docs/history/pi-beehive/plan.md, .bee/cells/archive/pi-beehive/pib-1.json, .bee/cells/archive/pi-beehive/pib-2.json, .bee/cells/archive/pi-beehive/pib-3.json, .bee/cells/archive/pi-beehive/pib-4.json, .bee/cells/archive/pi-beehive/pib-5.json, .bee/cells/archive/pi-beehive/pib-6.json]
---

# pi-beehive — Delivery

## What shipped

- **pib-1** — activity now fires on the Pi session shutdown with a Claude-shaped exit reason, so a quitting Pi session is marked exited instead of staying alive in the record (2 file(s) changed)
- **pib-2** — Closed the Pi session record on session_shutdown for every reason that truly ends the session, and skipped reload, which keeps the same session alive (2 file(s) changed)
- **pib-3** — agent_settled now parses the session-close verdict and injects only a block reason through sendUserMessage; an ordinary advisory nudge never opens a turn (2 file(s) changed)
- **pib-4** — The advisory-gap gate now covers both belts and derives the Pi side from source; the two unwired rules carry their name and the exclusion marker on their own line (2 file(s) changed)
- **pib-5** — The config reference now states the Pi belt row set, its excluded rules and the Claude rows with no Pi carrier; the release manifest was regenerated with a binary matching the source (2 file(s) changed)
- **pib-6** — state-sync fires on the Pi turn end, the contract derivations ignore commented-out code, and every row of the Pi config reference is anchored to the belt (3 file(s) changed)

## Verify

Each cell below was capped only against a recorded passing verify result — bee refuses a cap without one.

- **pib-1** — `cargo test --release --no-fail-fast --manifest-path packages/bee-rs/Cargo.toml --test pi_plugin_contracts --test opencode_plugin_contracts`
- **pib-2** — `cargo test --release --no-fail-fast --manifest-path packages/bee-rs/Cargo.toml --test pi_plugin_contracts --test opencode_plugin_contracts`
- **pib-3** — `cargo test --release --no-fail-fast --manifest-path packages/bee-rs/Cargo.toml --test pi_plugin_contracts --test opencode_plugin_contracts`
- **pib-4** — `cargo test --release --no-fail-fast --manifest-path packages/bee-rs/Cargo.toml --test pi_plugin_contracts --test opencode_plugin_contracts`
- **pib-5** — `cargo test --release --no-fail-fast --manifest-path packages/bee-rs/Cargo.toml --test pi_plugin_contracts --test opencode_plugin_contracts && .bee/bin/bee dev release-manifest --check`
- **pib-6** — `cargo test --release --no-fail-fast --manifest-path packages/bee-rs/Cargo.toml --test pi_plugin_contracts --test opencode_plugin_contracts`

## Deviations

- **pib-1** — Pi reasons are normalised to a Claude-shaped exit word before activity sees them, rather than passed through, because activity.rs deliberately ignores the word resume, which means the opposite thing on Pi — the plan was wrong about a fact
- **pib-2** — followed the plan
- **pib-3** — A fourth probe was added beyond the three the cell asked for, driving the real bee binary under gate_bypass=full so the block verdict is proven end to end and not only against a stub — found a better route
- **pib-3** — The worker reused the belts existing turnStartPending latch instead of adding one, which is what the cell asked for; recorded because the cell named it as a prohibition rather than an instruction — followed the plan
- **pib-4** — followed the plan
- **pib-5** — The regen ran through /home/thanhsmind/.cache/cargo-target/release/bee instead of the vendored .bee/bin/bee, because the vendored copy is a symlink into the main checkout and is behind the source it would regenerate from; the write guard correctly refuses replacing it from this worktree — hit an unforeseen obstacle
- **pib-6** — The last three documentation rows were corrected by the orchestrator rather than a worker, after the cell missed the same class of defect on three dispatches — hit an unforeseen obstacle
- **pib-6** — A shared comment-stripping helper was written once and applied to three derivations, rather than patching only the one the finding named — found a better route
- **pib-6** — The cell was reopened and capped a second time because an earlier cap was recorded by accident with placeholder values — something else had to be fixed first

## Provenance

Proposed by `bee knowledge promote --work pi-beehive` from 6 capped cell trace(s) in `.bee/cells/` and the anchor `docs/history/pi-beehive/CONTEXT.md`, `docs/history/pi-beehive/plan.md`. Every line above is copied from a trace or from the work item; nothing here is curated truth until a human or agent accepts it.
