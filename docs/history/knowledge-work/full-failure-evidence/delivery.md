---
type: bee.delivery
title: full-failure-evidence — delivery
description: "Delivery record proposed by bee knowledge promote for work item full-failure-evidence: 2 capped cell(s), 0 recorded deviation(s)."
timestamp: 2026-08-14
bee:
  id: full-failure-evidence-delivery
  lifecycle: active
  required_context: [docs/history/full-failure-evidence/CONTEXT.md, docs/history/full-failure-evidence/plan.md]
  sources: [docs/history/full-failure-evidence/CONTEXT.md, docs/history/full-failure-evidence/plan.md, .bee/cells/archive/full-failure-evidence/ffe-1.json, .bee/cells/archive/full-failure-evidence/ffe-2.json]
---

# full-failure-evidence — Delivery

## What shipped

- **ffe-1** — Extracted the three failure-excerpt blocks into `crate::fsutil::failure_excerpt`; every existing test passes with only the three `FAILURE_EXCERPT_MAX` references retargeted (4 file(s) changed)
- **ffe-2** — Wired the test-runner runner and the refusal-text log line; `register.md` and tests updated (8 file(s) changed)

## Verify

Each cell below was capped only against a recorded passing verify result — bee refuses a cap without one.

- **ffe-1** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml`
- **ffe-2** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml`

## Deviations

None recorded in the capped cell traces.

## Provenance

Proposed by `bee knowledge promote --work full-failure-evidence` from 2 capped cell trace(s) in `.bee/cells/` and the anchor `docs/history/full-failure-evidence/CONTEXT.md`, `docs/history/full-failure-evidence/plan.md`. Applied during the traceable-runs capture pass (2026-08-14), verified against shipped source first: `crate::fsutil::failure_excerpt` (`packages/bee-rs/crates/bee/src/fsutil.rs:193`) and the `failure_log` key in `docs/handbook/register.md:231-234` both still exist as described. The area-update half of this proposal (part b) had already been merged into `docs/knowledge/areas/verify-pipeline/suite-result-cache.md` (four bullets citing "full-failure-evidence D1/D2/D3") before this delivery record was written; only this delivery draft (part a) was still outstanding. Part (c), pattern candidates, was empty in the original proposal and stays empty — no capped cell trace carried a deviation or a failure signature.
