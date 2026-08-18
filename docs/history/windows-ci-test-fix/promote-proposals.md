promote proposal for work item "windows-ci-test-fix" (.bee/lanes/windows-ci-test-fix.json) — 1 capped cell(s): windows-ci-test-fix-1
anchor: ledger — .bee/lanes/windows-ci-test-fix.json
PROPOSAL ONLY — nothing was written. Applying any section below is a human or agent decision.

(a) DELIVERY DRAFT — save as docs/knowledge/work/windows-ci-test-fix/delivery.md

---
type: bee.delivery
title: windows-ci-test-fix — delivery
description: "Delivery record proposed by bee knowledge promote for work item windows-ci-test-fix: 1 capped cell(s), 0 recorded deviation(s)."
timestamp: 2026-08-17
bee:
  id: windows-ci-test-fix-delivery
  lifecycle: active
  required_context: [.bee/lanes/windows-ci-test-fix.json]
  sources: [.bee/lanes/windows-ci-test-fix.json, .bee/cells/windows-ci-test-fix-1.json]
---

# windows-ci-test-fix — Delivery

## What shipped

- **windows-ci-test-fix-1** — Make staging and worktree tests hermetic on Windows (autocrlf, 8.3 short paths) (0 file(s) changed)

## Verify

Each cell below was capped only against a recorded passing verify result — bee refuses a cap without one.

- **windows-ci-test-fix-1** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml --bin bee -- one_conflicting_feature_never_blocks_the_others_on_rebuild merge_with_no_flags_keeps_the_worktree_by_default_and_queues_one_entry`

## Deviations

None recorded in the capped cell traces.

## Provenance

Proposed by `bee knowledge promote --work windows-ci-test-fix` from 1 capped cell trace(s) in `.bee/cells/` and the anchor `.bee/lanes/windows-ci-test-fix.json`. Every line above is copied from a trace or from the work item; nothing here is curated truth until a human or agent accepts it.

(b) AREA UPDATES — candidate spec-sync bullets, each citing its cell

None: the work item declares no bee.areas, so there is no area to sync (D19).

(c) PATTERN CANDIDATES — candidate bee.pattern concepts, bee.polarity pitfall

None: no capped cell trace carries a deviation or a failure signature.

knowledge promote: 1 capped cell(s) mined, 1 delivery draft, 0 area bullet(s), 0 pattern candidate(s), 0 file(s) written.