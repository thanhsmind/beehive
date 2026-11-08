---
type: bee.delivery
title: anti-pattern-principles — delivery
description: "Delivery record proposed by bee knowledge promote for work item anti-pattern-principles: 1 capped cell(s), 2 recorded deviation(s)."
timestamp: 2026-11-04
bee:
  id: anti-pattern-principles-delivery
  lifecycle: active
  areas: [doctrine-layer]
  required_context: [docs/history/anti-pattern-principles/CONTEXT.md]
  sources: [docs/history/anti-pattern-principles/CONTEXT.md, cell app-1 (trace purged from .bee/cells/)]
---

# anti-pattern-principles — Delivery

## What shipped

- **app-1** — bee orient lists lock-the-contract and split-on-a-real-dependency for feature, refactor and perf routes, each skill pointing at its own heading in expertise/planning.md (5 file(s) changed)

## Verify

Each cell below was capped only against a recorded passing verify result — bee refuses a cap without one.

- **app-1** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee --test principle_index_parity --test skill_triggers --test pointer_integrity && .bee/bin/bee dev release-manifest --check` — principle_index_parity 5/5, pointer_integrity 10/10, skill_triggers 1 ok and 1 ignored (real-agent eval not run), release-manifest 599 files match; bee orient driven in a scratch repo with route clas…

## Deviations

- **app-1** — Ran the verify cargo step with the absolute path /home/thanhsmind/.cargo/bin/cargo instead of the CARGO_HOME PATH prefix — the worktree shell guard refused the runtime variable — hit an unforeseen obstacle
- **app-1** — Changed one Spike craft phrase from The section above to Build order versus discovery order — the new Split only on a real dependency heading made The section above point at the wrong section — something else had to be fixed first

## Provenance

Proposed by `bee knowledge promote --work anti-pattern-principles` from 1 capped cell trace(s) in `.bee/cells/` and the anchor `docs/history/anti-pattern-principles/CONTEXT.md`. Every line above is copied from a trace or from the work item; nothing here is curated truth until a human or agent accepts it.
