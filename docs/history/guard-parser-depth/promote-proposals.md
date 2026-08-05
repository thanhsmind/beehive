promote proposal for work item "guard-parser-depth" (docs/history/guard-parser-depth/plan.md) — 1 capped cell(s): gpd-1
anchor: history — docs/history/guard-parser-depth/plan.md
PROPOSAL ONLY — nothing was written. Applying any section below is a human or agent decision.

(a) DELIVERY DRAFT — save as docs/knowledge/work/guard-parser-depth/delivery.md

---
type: bee.delivery
title: guard-parser-depth — delivery
description: "Delivery record proposed by bee knowledge promote for work item guard-parser-depth: 1 capped cell(s), 0 recorded deviation(s)."
timestamp: 2026-08-05
bee:
  id: guard-parser-depth-delivery
  lifecycle: active
  areas: [hook-runtime]
  required_context: [docs/history/guard-parser-depth/plan.md]
  sources: [docs/history/guard-parser-depth/plan.md, .bee/cells/gpd-1.json]
---

# guard-parser-depth — Delivery

## What shipped

- **gpd-1** — Shipped in commit 98888896; tokenize_deep + find_git_invocations wire all three write-guard consumers, pinned by the gpd-1 tests in write_guard/tests.rs. (4 file(s) changed)

## Verify

Each cell below was capped only against a recorded passing verify result — bee refuses a cap without one.

- **gpd-1** — `cargo test --release --manifest-path packages/bee-rs/Cargo.toml, plus a live probe with the freshly built binary: `git status && git stash` must now refuse, `sh -c 'echo x > .bee/state.json'` must now refuse, and `echo "git stash"` must still pass.`

## Deviations

None recorded in the capped cell traces.

## Provenance

Proposed by `bee knowledge promote --work guard-parser-depth` from 1 capped cell trace(s) in `.bee/cells/` and the anchor `docs/history/guard-parser-depth/plan.md`. Every line above is copied from a trace or from the work item; nothing here is curated truth until a human or agent accepts it.

(b) AREA UPDATES — candidate spec-sync bullets, each citing its cell

areas: from the scribing stamp for "guard-parser-depth" — .bee/logs/scribing-runs.jsonl's most recent entry (2026-08-05T13:32:32.795Z), the work item declares no bee.areas.

area hook-runtime:
  - [gpd-1] Shipped in commit 98888896; tokenize_deep + find_git_invocations wire all three write-guard consumers, pinned by the gpd-1 tests in write_guard/tests.rs. — feature-wide sync per the scribing stamp, 4 file(s) changed (trace .bee/cells/gpd-1.json)

(c) PATTERN CANDIDATES — candidate bee.pattern concepts, bee.polarity pitfall

None: no capped cell trace carries a deviation or a failure signature.

knowledge promote: 1 capped cell(s) mined, 1 delivery draft, 1 area bullet(s), 0 pattern candidate(s), 0 file(s) written.