promote proposal for work item "test-doctrine" (docs/history/test-doctrine/CONTEXT.md + docs/history/test-doctrine/plan.md) — 5 capped cell(s): td-1, td-2, td-3, td-4, td-5
anchor: history — docs/history/test-doctrine/CONTEXT.md, docs/history/test-doctrine/plan.md
PROPOSAL ONLY — nothing was written. Applying any section below is a human or agent decision.

(a) DELIVERY DRAFT — save as docs/knowledge/work/test-doctrine/delivery.md

---
type: bee.delivery
title: test-doctrine — delivery
description: "Delivery record proposed by bee knowledge promote for work item test-doctrine: 5 capped cell(s), 0 recorded deviation(s)."
timestamp: 2026-08-18
bee:
  id: test-doctrine-delivery
  lifecycle: active
  required_context: [docs/history/test-doctrine/CONTEXT.md, docs/history/test-doctrine/plan.md]
  sources: [docs/history/test-doctrine/CONTEXT.md, docs/history/test-doctrine/plan.md, .bee/cells/td-1.json, .bee/cells/td-2.json, .bee/cells/td-3.json, .bee/cells/td-4.json, .bee/cells/td-5.json]
---

# test-doctrine — Delivery

## What shipped

- **td-1** — Replaced the cap tests enum with the D8 proof-string contract; --report required on every cap path (11 file(s) changed)
- **td-2** — Retired the close test door in favor of a D8 proof-line check reading trace.report (9 file(s) changed)
- **td-3** — Reordered worktree-merge help to proof-check-first and re-capped with a D8 proof line (1 file(s) changed)
- **td-4** — Rewrote the no-test-sentinel preamble line to the D7/D8 proof doctrine: every cap still owes a proof line (command segment none, reason naming the parity/docs proof), and recording a real commands.test re-enables CI's full-run net; repointed the pinned test byte-for-byte. (2 file(s) changed)
- **td-5** — Rewrote merge-prompt.md's red-verify clause to the landed MERGE_CONFLICT/WORKTREE_MERGE_PROOF_DEBT stops and repointed README.md's SKILL.md pointer from the nonexistent 'Permission posture' to 'Safety boundaries'; regenerated trees and manifest (19 file(s) changed)

## Verify

Each cell below was capped only against a recorded passing verify result — bee refuses a cap without one.

- **td-1** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee && .bee/bin/bee dev release-manifest --check && bee onboard --repo-root . --json && diff packages/bee/prompts/worker-cell.md .bee/bin/prompts/worker-cell.md`
- **td-2** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee`
- **td-3** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee`
- **td-4** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee`
- **td-5** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee && .bee/bin/bee dev release-manifest --check && .bee/bin/bee onboard --repo-root . --json`

## Deviations

None recorded in the capped cell traces.

## Provenance

Proposed by `bee knowledge promote --work test-doctrine` from 5 capped cell trace(s) in `.bee/cells/` and the anchor `docs/history/test-doctrine/CONTEXT.md`, `docs/history/test-doctrine/plan.md`. Every line above is copied from a trace or from the work item; nothing here is curated truth until a human or agent accepts it.

(b) AREA UPDATES — candidate spec-sync bullets, each citing its cell

None: the work item declares no bee.areas, so there is no area to sync (D19).

(c) PATTERN CANDIDATES — candidate bee.pattern concepts, bee.polarity pitfall

None: no capped cell trace carries a deviation or a failure signature.

knowledge promote: 5 capped cell(s) mined, 1 delivery draft, 0 area bullet(s), 0 pattern candidate(s), 0 file(s) written.