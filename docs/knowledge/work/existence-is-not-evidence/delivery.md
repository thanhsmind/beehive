---
type: bee.delivery
title: existence-is-not-evidence — delivery
description: "Delivery record proposed by bee knowledge promote for work item existence-is-not-evidence: 2 capped cell(s), 3 recorded deviation(s)."
timestamp: 2026-08-30
bee:
  id: existence-is-not-evidence-delivery
  lifecycle: active
  required_context: [docs/history/existence-is-not-evidence/CONTEXT.md, docs/history/existence-is-not-evidence/plan.md]
  sources: [docs/history/existence-is-not-evidence/CONTEXT.md, docs/history/existence-is-not-evidence/plan.md, .bee/cells/eine-rust-claims-gate.json, .bee/cells/eine-skill-mandates.json]
---

# existence-is-not-evidence — Delivery

## What shipped

- **eine-rust-claims-gate** — Shape/merged gate approvals now refuse a plan.md whose load-bearing claims table is missing, malformed, or still guessed (3 file(s) changed)
- **eine-skill-mandates** — Landed the claims-table spec, Open Questions section, tiny/small inline evidence, the reality touch and pre-flight mandates, and the claims-audit home with a pointer-only hat row; regen chain green (6 file(s) changed)

## Verify

Each cell below was capped only against a recorded passing verify result — bee refuses a cap without one.

- **eine-rust-claims-gate** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --no-fail-fast --manifest-path packages/bee-rs/Cargo.toml`
- **eine-skill-mandates** — `bash -c '.bee/bin/bee dev regen && .bee/bin/bee onboard --repo-root . --json && .bee/bin/bee dev release-manifest --check && diff skills/bee-planning/references/planning-reference.md .claude/skills/bee-planning/references/planning-reference.md'`

## Deviations

- **eine-skill-mandates** — Wrote the Claims-table audit into expertise/review.md, not only the .bee/expertise/review.md the cell named — the cell named the rendered copy, and bee dev regen silently reverted my first edit to it; expertise/ at repo root is the source of truth for that file — the plan was wrong about a fact

## Provenance

Proposed by `bee knowledge promote --work existence-is-not-evidence` from 2 capped cell trace(s) in `.bee/cells/` and the anchor `docs/history/existence-is-not-evidence/CONTEXT.md`, `docs/history/existence-is-not-evidence/plan.md`. Every line above is copied from a trace or from the work item; nothing here is curated truth until a human or agent accepts it.
