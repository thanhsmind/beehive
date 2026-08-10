promote proposal for work item "worktree-manifest-native" (.bee/logs/scribing-runs.jsonl) — 1 capped cell(s): wmn-1
anchor: ledger — .bee/logs/scribing-runs.jsonl
PROPOSAL ONLY — nothing was written. Applying any section below is a human or agent decision.

(a) DELIVERY DRAFT — save as docs/knowledge/work/worktree-manifest-native/delivery.md

---
type: bee.delivery
title: worktree-manifest-native — delivery
description: "Delivery record proposed by bee knowledge promote for work item worktree-manifest-native: 1 capped cell(s), 0 recorded deviation(s)."
timestamp: 2026-08-10
bee:
  id: worktree-manifest-native-delivery
  lifecycle: active
  areas: [okf-profile]
  required_context: [.bee/logs/scribing-runs.jsonl]
  sources: [.bee/logs/scribing-runs.jsonl, .bee/cells/wmn-1.json]
---

# worktree-manifest-native — Delivery

## What shipped

- **wmn-1** — real bug fixed: dispatch prepare's learned-context bundle now reads from the cell's granted worktree (control-root fallback when none); named deviation: knowledge context from a worktree already resolves via the wide door (proven empirically), NeedsNode kept as legitimate non-worktree refusal paths (2 file(s) changed)

## Verify

Each cell below was capped only against a recorded passing verify result — bee refuses a cap without one.

- **wmn-1** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml`

## Deviations

None recorded in the capped cell traces.

## Provenance

Proposed by `bee knowledge promote --work worktree-manifest-native` from 1 capped cell trace(s) in `.bee/cells/` and the anchor `.bee/logs/scribing-runs.jsonl`. Every line above is copied from a trace or from the work item; nothing here is curated truth until a human or agent accepts it.

(b) AREA UPDATES — candidate spec-sync bullets, each citing its cell

areas: from the scribing stamp for "worktree-manifest-native" — .bee/logs/scribing-runs.jsonl's most recent entry (2026-08-10T16:13:56.437Z), the work item declares no bee.areas.

area okf-profile:
  - [wmn-1] real bug fixed: dispatch prepare's learned-context bundle now reads from the cell's granted worktree (control-root fallback when none); named deviation: knowledge context from a worktree already resolves via the wide door (proven empirically), NeedsNode kept as legitimate non-worktree refusal paths — feature-wide sync per the scribing stamp, 2 file(s) changed (trace .bee/cells/wmn-1.json)

(c) PATTERN CANDIDATES — candidate bee.pattern concepts, bee.polarity pitfall

None: no capped cell trace carries a deviation or a failure signature.

knowledge promote: 1 capped cell(s) mined, 1 delivery draft, 1 area bullet(s), 0 pattern candidate(s), 0 file(s) written.