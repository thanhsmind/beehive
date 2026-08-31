---
type: bee.delivery
title: pi-result-mailbox — delivery
description: "Delivery record for work item pi-result-mailbox: 5 capped cell(s), 11 recorded deviation(s)."
timestamp: 2026-08-29
bee:
  id: pi-result-mailbox-delivery
  lifecycle: active
  required_context: [docs/history/pi-result-mailbox/CONTEXT.md, docs/history/pi-result-mailbox/plan.md]
  sources: [docs/history/pi-result-mailbox/CONTEXT.md, docs/history/pi-result-mailbox/plan.md, .bee/cells/prm-1.json, .bee/cells/prm-2.json, .bee/cells/prm-3.json, .bee/cells/prm-4.json, .bee/cells/prm-5.json]
---

# pi-result-mailbox — Delivery

## What shipped

- **prm-1** — The report rides the mailbox as a path-only additive envelope key, and `--inbox-session` makes the detached fact a flag with a pre-spawn marker (2 file(s) changed)
- **prm-2** — The Pi belt drains its result inbox and injects a header-only result envelope with pi-peer claim discipline (1 file(s) changed)
- **prm-3** — Drain fixtures prove at-least-once delivery, header-only fences and a never-throw drain; the Pi harness is now timeout-bounded and records injections (1 file(s) changed)
- **prm-4** — Lifted the pi not-production caveat at every site with text naming the delivered result contract and its at-least-once limits (7 file(s) changed)
- **prm-5** — `result-inbox` joins the generated ignore block; the pi herding payload carries a `detached_delivery` instruction naming `--inbox-session` (2 file(s) changed)

## Verify

- **prm-1** — `cargo test --release --no-fail-fast --manifest-path packages/bee-rs/Cargo.toml --bin bee`
- **prm-2** — `node --check .pi/extensions/bee-guard.ts`
- **prm-3** — `cargo test --release --no-fail-fast --manifest-path packages/bee-rs/Cargo.toml --test pi_plugin_contracts`
- **prm-4** — `rg -c "pi-result-mailbox" docs/config-reference.md && rg -n "report_path" skills/bee-swarming/references/swarming-reference.md docs/knowledge/areas/bee-herding/the-run-verb-and-worker-outcomes.md`
- **prm-5** — `cargo test --release --no-fail-fast --manifest-path packages/bee-rs/Cargo.toml --bin bee`

All green.

## Deviations

- **prm-1** — Named the `.bee/result-inbox/` `.gitignore` gap instead of fixing it (generated from `onboard/templates.rs`, outside cell scope).
- **prm-1** — Wrote the pre-spawn marker in `execute()` rather than `execute_new()` alone, so a detached `--continue` round also leaves one.
- **prm-1** — Added `MailboxResult.report_note` beside `report_path` — the plan named it as an envelope key with no carrier yet.
- **prm-2** — Added a status row to the fenced header beyond the plan's five, since a blocked result read as done is exactly the corruption class this feature exists to stop.
- **prm-2** — Parked the drain interval in a `globalThis` slot as well as a module variable, since Pi `/reload` can hand the file a fresh module scope with the old interval still armed.
- **prm-3** — Lifted the never-throw hand list into `never_throw_event_rows()` gated by `every_advisory_event_the_belt_registers_has_a_never_throw_row`, so the next `pi.on` event added cannot skip its row.
- **prm-4** — Renamed the config-reference heading (dropped the `(preview)` suffix) and cleared two more preview mentions the cell action did not enumerate.
- **prm-4** — Committed with an explicit `--` pathspec through the shared index guard, excluding a sibling cell's in-flight test file.
- **prm-5** — Added a local `#[cfg(test)] mod detached_delivery_tests` inside `prepare.rs` to pin the `--inbox-session` payload claim.

## Provenance

Mined from 5 capped cell traces in `.bee/cells/` and `docs/history/pi-result-mailbox/CONTEXT.md`, `docs/history/pi-result-mailbox/plan.md`. No `bee.areas` declared — no area sync performed.
