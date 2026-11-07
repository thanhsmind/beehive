promote proposal for work item "knowledge-outcome-measure" (docs/history/knowledge-outcome-measure/CONTEXT.md + docs/history/knowledge-outcome-measure/plan.md) — 3 capped cell(s): kom-1, kom-2, kom-3
anchor: history — docs/history/knowledge-outcome-measure/CONTEXT.md, docs/history/knowledge-outcome-measure/plan.md
PROPOSAL ONLY — nothing was written. Applying any section below is a human or agent decision.

(a) DELIVERY DRAFT — save as docs/knowledge/work/knowledge-outcome-measure/delivery.md

---
type: bee.delivery
title: knowledge-outcome-measure — delivery
description: "Delivery record proposed by bee knowledge promote for work item knowledge-outcome-measure: 3 capped cell(s), 0 recorded deviation(s)."
timestamp: 2026-11-07
bee:
  id: knowledge-outcome-measure-delivery
  lifecycle: active
  areas: [workflow-state, okf-profile]
  required_context: [docs/history/knowledge-outcome-measure/CONTEXT.md, docs/history/knowledge-outcome-measure/plan.md]
  sources: [docs/history/knowledge-outcome-measure/CONTEXT.md, docs/history/knowledge-outcome-measure/plan.md, .bee/cells/kom-1.json, .bee/cells/kom-2.json, .bee/cells/kom-3.json]
---

# knowledge-outcome-measure — Delivery

## What shipped

- **kom-1** — Cell dispatches get a deterministic holdout arm that drops the Learned context block; the cell prepare record carries knowledge_arm and learned_context. Commit 79560ca607deb2ceaad7f5aac41ed832e9e72798 (2 file(s) changed)
- **kom-2** — bee knowledge report gains an outcomes section (per-arm counts, rates, unrecorded, comparable at 30/arm) with 8 new tests; commit 8729d72340d52927b46b4b1f3df48144f19a60b0 (2 file(s) changed)
- **kom-3** — Help text for dispatch prepare and knowledge report plus R21 in dispatch.md and B6 in critical-bar.md; commit c07748e62 (3 file(s) changed)

## Verify

Each cell below was capped only against a recorded passing verify result — bee refuses a cap without one.

- **kom-1** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee verbs::drivers` — whole workspace: 5320 passed, 0 failed; drivers subset 502 passed
- **kom-2** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee --test knowledge_outcomes` — full suite exit 0, includes knowledge_outcomes 8 passed (7 seen red first)
- **kom-3** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee --test registry_contracts && .bee/bin/bee knowledge check` — docs and help-text JSON only, registry contract tests passed (12) and knowledge check reported 0 errors

## Deviations

None recorded in the capped cell traces.

## Provenance

Proposed by `bee knowledge promote --work knowledge-outcome-measure` from 3 capped cell trace(s) in `.bee/cells/` and the anchor `docs/history/knowledge-outcome-measure/CONTEXT.md`, `docs/history/knowledge-outcome-measure/plan.md`. Every line above is copied from a trace or from the work item; nothing here is curated truth until a human or agent accepts it.

(b) AREA UPDATES — candidate spec-sync bullets, each citing its cell

areas: from the scribing stamp for "knowledge-outcome-measure" — .bee/logs/scribing-runs.jsonl's most recent entry (2026-11-07T05:59:47.133Z), the work item declares no bee.areas.

area workflow-state:
  - [kom-1] Cell dispatches get a deterministic holdout arm that drops the Learned context block; the cell prepare record carries knowledge_arm and learned_context. Commit 79560ca607deb2ceaad7f5aac41ed832e9e72798 — feature-wide sync per the scribing stamp, 2 file(s) changed (trace .bee/cells/kom-1.json)
  - [kom-2] bee knowledge report gains an outcomes section (per-arm counts, rates, unrecorded, comparable at 30/arm) with 8 new tests; commit 8729d72340d52927b46b4b1f3df48144f19a60b0 — feature-wide sync per the scribing stamp, 2 file(s) changed (trace .bee/cells/kom-2.json)

area okf-profile:
  - [kom-1] Cell dispatches get a deterministic holdout arm that drops the Learned context block; the cell prepare record carries knowledge_arm and learned_context. Commit 79560ca607deb2ceaad7f5aac41ed832e9e72798 — feature-wide sync per the scribing stamp, 2 file(s) changed (trace .bee/cells/kom-1.json)
  - [kom-2] bee knowledge report gains an outcomes section (per-arm counts, rates, unrecorded, comparable at 30/arm) with 8 new tests; commit 8729d72340d52927b46b4b1f3df48144f19a60b0 — feature-wide sync per the scribing stamp, 2 file(s) changed (trace .bee/cells/kom-2.json)

(c) PATTERN CANDIDATES — candidate bee.pattern concepts, bee.polarity pitfall

None: no capped cell trace carries a deviation or a failure signature.

knowledge promote: 3 capped cell(s) mined, 1 delivery draft, 4 area bullet(s), 0 pattern candidate(s), 0 file(s) written.