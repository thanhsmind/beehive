---
type: bee.delivery
title: paseo-cli-autodetect — delivery
description: "Delivery record proposed by bee knowledge promote for work item paseo-cli-autodetect: 1 capped cell(s), 2 recorded deviation(s)."
timestamp: 2026-10-15
bee:
  id: paseo-cli-autodetect-delivery
  lifecycle: active
  required_context: [docs/history/paseo-cli-autodetect/plan.md]
  sources: [docs/history/paseo-cli-autodetect/plan.md, .bee/cells/pcad-1.json]
---

# paseo-cli-autodetect — Delivery

## What shipped

- **pcad-1** — paseo_command now resolves config, then PASEO_CLI, then the running daemon's CLI from paseo.pid, else plain paseo; tests and the paseo-channel concept updated (commit 8f2bf79ed) (2 file(s) changed)

## Verify

Each cell below was capped only against a recorded passing verify result — bee refuses a cap without one.

- **pcad-1** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee --bin bee herding::paseo` — herding::paseo module holds paseo_command and its six resolution tests; 33 passed after the leader gated the exe-dir symlink test to unix

## Deviations

- **pcad-1** — leader gated paseo_command_falls_back_to_the_daemon_exe_dir with cfg(unix) and moved the symlink out of fake_proc so Windows builds compile
- **pcad-1** — sync-ack: skills/bee-herding never names the paseo command or its resolution; the owning concept the-paseo-channel.md carries the change

## Provenance

Proposed by `bee knowledge promote --work paseo-cli-autodetect` from 1 capped cell trace(s) in `.bee/cells/` and the anchor `docs/history/paseo-cli-autodetect/plan.md`. Every line above is copied from a trace or from the work item; nothing here is curated truth until a human or agent accepts it.
