---
type: bee.delivery
title: pi-extension-split — delivery
description: "Delivery record proposed by bee knowledge promote for work item pi-extension-split: 4 capped cell(s), 6 recorded deviation(s)."
timestamp: 2026-10-02
bee:
  id: pi-extension-split-delivery
  lifecycle: active
  areas: [onboarding, hook-runtime]
  required_context: [docs/history/pi-extension-split/CONTEXT.md, docs/history/pi-extension-split/plan.md]
  sources: [docs/history/pi-extension-split/CONTEXT.md, docs/history/pi-extension-split/plan.md, .bee/cells/pes-1.json, .bee/cells/pes-2.json, .bee/cells/pes-3.json, .bee/cells/pes-4.json]
---

# pi-extension-split — Delivery

## What shipped

- **pes-1** — Restore verbatim comments and align Pi guard contract path (12 file(s) changed)
- **pes-2** — bee doctor --runtime pi compares every .ts file of the build.rs-generated bee-guard list byte for byte, flags missing and unshipped modules with a count and bee onboard --apply, and keeps the legacy-file failure. (3 file(s) changed)
- **pes-3** — bee onboard vendors every .ts file of .pi/extensions/bee-guard/, plans remove_pi_extension for the legacy single file before any copy, prunes unshipped .ts files, and the remove arm refuses paths outside its scope. (3 file(s) changed)
- **pes-4** — Live docs and the four verify-app feature copies name the .pi/extensions/bee-guard/ folder; the hook-runtime audit concept carries the module map citing pi-extension-split D1-D6. (19 file(s) changed)

## Verify

Each cell below was capped only against a recorded passing verify result — bee refuses a cap without one.

- **pes-1** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --no-fail-fast --manifest-path packages/bee-rs/Cargo.toml -p bee --test pi_plugin_contracts --test opencode_plugin_contracts && PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee --bin bee doctor:: && .bee/bin/bee dev release-manifest --check` — cell verify passed
- **pes-2** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee --bin bee doctor::` — doctor:: filter covers the wiring row cases; the full declared suite also ran green on 16e5836e8 (main bin 3909 passed, pi_plugin_contracts 104, opencode_plugin_contracts 8, no failures)
- **pes-3** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee --bin bee onboard::` — 193 passed, 1 ignored, onboard:: filter covers the plan and apply tests for the Pi folder; the Pi contract suite still has one red owned by pes-1 (its installed-path check)
- **pes-4** — `! rg -n 'extensions/bee-guard\.ts' docs/knowledge/areas docs/config-reference.md docs/01-distillation.md docs/02-architecture.md docs/06-runtime-integration.md docs/product-description .bee/verify .claude/skills/verify-app .agents/skills/verify-app .opencode/skills/verify-app .bee/config-sample.json | rg -v -i 'legacy'` — docs-only cell: no live doc names the old single file except as the legacy file onboard removes; md5 of the four pi-runtime.md copies is identical

## Deviations

- **pes-1** — followed the plan
- **pes-2** — build.rs lost its module doc comment; the same rationale already lives in docs/knowledge/areas/onboarding/release-identity-and-version-parity.md, so the no-comment rule keeps one home
- **pes-2** — the worker run returned exit 1 after committing; the leader verified the tree and capped
- **pes-3** — apply_plan split into apply_plan plus apply_computed_plan so a test can apply a hand-built plan
- **pes-4** — docs/01-distillation.md, docs/02-architecture.md and docs/06-runtime-integration.md needed no edit: their bee-guard mentions name the OpenCode plugin
- **pes-4** — sync-ack: governed-paths-and-the-intake-gate.md changed only the Pi guard path in one line; the hook-runtime-docs-lane-allowlist rule text is unchanged, so routing-and-contracts.md owes no edit

## Provenance

Proposed by `bee knowledge promote --work pi-extension-split` from 4 capped cell trace(s) in `.bee/cells/` and the anchor `docs/history/pi-extension-split/CONTEXT.md`, `docs/history/pi-extension-split/plan.md`. Every line above is copied from a trace or from the work item; nothing here is curated truth until a human or agent accepts it.
