---
type: bee.delivery
title: codex-reliability-closeout — delivery
description: "Delivery record proposed by bee knowledge promote for work item codex-reliability-closeout: 5 capped cell(s), 11 recorded deviation(s)."
timestamp: 2026-09-13
bee:
  id: codex-reliability-closeout-delivery
  lifecycle: active
  areas: [hook-runtime]
  required_context: [docs/history/codex-reliability-closeout/CONTEXT.md, docs/history/codex-reliability-closeout/plan.md]
  sources: [docs/history/codex-reliability-closeout/CONTEXT.md, docs/history/codex-reliability-closeout/plan.md, .bee/cells/archive/codex-reliability-closeout/crc-1.json, .bee/cells/archive/codex-reliability-closeout/crc-2.json, .bee/cells/archive/codex-reliability-closeout/crc-3.json, .bee/cells/archive/codex-reliability-closeout/crc-4.json, .bee/cells/archive/codex-reliability-closeout/crc-5.json]
---

# codex-reliability-closeout — Delivery

## What shipped

- **crc-1** — Isolate Codex canary environment and direct executable routing (5 file(s) changed)
- **crc-2** — Retain truthful transcript regression history and direct token failure (9 file(s) changed)
- **crc-3** — Correct generated Codex onboarding guidance (5 file(s) changed)
- **crc-4** — Repaired gate preview flag parsing and verified approved_gates preservation (3 file(s) changed)
- **crc-5** — Repair Windows activity fixture first-call ordering. (4 file(s) changed)

## Verify

Each cell below was capped only against a recorded passing verify result — bee refuses a cap without one.

- **crc-1** — `bash scripts/codex-parity-canary-test.sh && bash -n scripts/codex-parity-canary.sh` — seven controls passed; installed live-canary.json records denied and allowed patch/shell writes
- **crc-2** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" TMPDIR=/var/tmp BEE_CODEX_PROBE_BIN=/bin/false cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee hooks::session_close::tests && git diff --check c4122d41 HEAD -- packages/bee-rs/crates/bee/src/hooks/session_close/tests.rs` — 43 transcript tests passed; committed whitespace check passed; full-suite 3929 passed
- **crc-3** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" TMPDIR=/var/tmp BEE_CODEX_PROBE_BIN=/bin/false cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee onboard` — onboard tests passed; onboarding-cli.log retains fresh and refresh CLI checks; dev regen refreshed repository metadata
- **crc-4** — `Run targeted gate preview CLI regression tests on base and head; corrected aliases return a packet and leave approval gates unchanged; bee dev release-manifest --check at integration` — targeted gate preview CLI regression tests and release-manifest check pass
- **crc-5** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" TMPDIR=/var/tmp BEE_CODEX_PROBE_BIN=/bin/false cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee hooks::activity::tests` — 55 activity tests passed; full suite 3930 passed, zero failed, 20 ignored.

## Deviations

- **crc-1** — Corrected verify field to an executable command — original mixed prose with shell and could not match proof parser; decision a8464d1c — the plan was wrong about a fact
- **crc-1** — sync-ack: Owner behavior checked; Codex map and hook-runtime knowledge synchronized; bee dev regen completed at integration.
- **crc-2** — Corrected verify field to an executable command — original mixed prose with shell and could not match proof parser; decision a8464d1c — the plan was wrong about a fact
- **crc-2** — Retained three historical failures and one pass with separately labeled synthetic mutation — saved 0690c1eb lacks Claude failure; decision 255ec53b — the plan was wrong about a fact
- **crc-2** — sync-ack: Owner behavior checked; Codex map and hook-runtime knowledge synchronized; bee dev regen completed at integration.
- **crc-3** — Corrected verify field to an executable command — original mixed prose with shell and could not match proof parser; decision a8464d1c — the plan was wrong about a fact
- **crc-3** — sync-ack: Owner behavior checked; Codex map and hook-runtime knowledge synchronized; bee dev regen completed at integration.
- **crc-4** — followed the plan
- **crc-4** — sync-ack: Parser repair adds preview to FLAG_ALONE_BOOLEANS without changing skill procedures
- **crc-5** — Leader completed stalled worker cell per decision aafa65ca-6f11-4c0c-a771-e6be102db070; actual Windows CI pending.
- **crc-5** — sync-ack: Test-only change; no skill or production contract changes. Existing hook-runtime knowledge owner records fallback regression and platform limitation.

## Provenance

Proposed by `bee knowledge promote --work codex-reliability-closeout` from 5 capped cell trace(s) in `.bee/cells/` and the anchor `docs/history/codex-reliability-closeout/CONTEXT.md`, `docs/history/codex-reliability-closeout/plan.md`. Every line above is copied from a trace or from the work item; nothing here is curated truth until a human or agent accepts it.
