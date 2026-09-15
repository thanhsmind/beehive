---
type: bee.delivery
title: codex-parity-completion — delivery
description: "Delivery record proposed by bee knowledge promote for work item codex-parity-completion: 4 capped cell(s), 7 recorded deviation(s)."
timestamp: 2026-09-13
bee:
  id: codex-parity-completion-delivery
  lifecycle: active
  areas: [hook-runtime]
  required_context: [docs/history/codex-parity-completion/CONTEXT.md, docs/history/codex-parity-completion/plan.md]
  sources: [docs/history/codex-parity-completion/CONTEXT.md, docs/history/codex-parity-completion/plan.md, .bee/cells/archive/codex-parity-completion/cpc-1.json, .bee/cells/archive/codex-parity-completion/cpc-2.json, .bee/cells/archive/codex-parity-completion/cpc-3.json, .bee/cells/archive/codex-parity-completion/cpc-4.json]
---

# codex-parity-completion — Delivery

## What shipped

- **cpc-1** — Installed Codex hooks protect observed native tools and retain real runtime evidence. (10 file(s) changed)
- **cpc-2** — Enforce configured Codex dispatch settings, bounded hermetic probes, exact observed native refusal, and actual prepared read-only filesystem boundary. (6 file(s) changed)
- **cpc-3** — Normalize native Codex transcript usage, identity and final text with Claude compatibility. (5 file(s) changed)
- **cpc-4** — Publish verified Codex runtime limits, dispatch instructions and executable sandbox recipe. (12 file(s) changed)

## Verify

Each cell below was capped only against a recorded passing verify result — bee refuses a cap without one.

- **cpc-1** — `cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee --test hook_contracts && cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee --bin bee hook_manifests && cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee --bin bee hooks_wiring` — 13/7/23 passed with TMPDIR=/var/tmp in docs/history/codex-parity-completion/cpc-1-checks.log; additional real installed canary exited 0 at /var/tmp/bee-codex-canary-ZPuEAI/evidence: patch/shell deny …
- **cpc-2** — `cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee --bin bee model_guard && cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee --bin bee verbs::drivers` — Leader ran exact groups under TMPDIR=/var/tmp BEE_CODEX_PROBE_BIN=/bin/false: 70 and 382 passed, 8 ignored. Final amendment comment-only. Additional green:live actual prepared command/stdin in cpc-2-…
- **cpc-3** — `cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee --bin bee session_close && cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee --bin bee hooks::activity` — 50 session_close and 54 activity tests; leader rerun exited 0; real-binary fixture wait subject and session checked at /tmp/bee-verify/run/20260912-215549-1244378/repo/.bee/state.json
- **cpc-4** — `git diff --check` — final instruction scenarios passed in job-1789272594951-3409518-1; actual prepared a3600d96-c439-433d-90dc-4186da0994fd executed through documented bash-c helper: raw read-only denials both status1, …

## Deviations

- **cpc-1** — Use private authentication and deny opaque native dispatch — live input exposed inaccessible role metadata — hit an unforeseen obstacle
- **cpc-1** — Consolidate recovery commits at wave close — sibling commits intervene — found a better route
- **cpc-2** — Existing windows-sys Pipes feature added under decision 7741c278; no new crate. Separate recovery commit retained until wave-close consolidation because sibling commits intervene.
- **cpc-2** — Observed Codex 0.154.0 opaque native messages require named refusal; explicit configured external routes retained; no full native parity claim.
- **cpc-3** — Used path-scoped revision commits pending wave-close consolidation — a sibling commit separated the original cell commit from revisions — hit an unforeseen obstacle
- **cpc-3** — Classified the direct session-close fixture separately from live Codex execution — installed native hook proof belongs to the running cpc-1 canary — found a better route
- **cpc-4** — Leader owns wave-barrier regeneration and integration proof; worker amended cpc-4 through user-approved bounded revisions.

## Provenance

Proposed by `bee knowledge promote --work codex-parity-completion` from 4 capped cell trace(s) in `.bee/cells/` and the anchor `docs/history/codex-parity-completion/CONTEXT.md`, `docs/history/codex-parity-completion/plan.md`. Every line above is copied from a trace or from the work item; nothing here is curated truth until a human or agent accepts it.
