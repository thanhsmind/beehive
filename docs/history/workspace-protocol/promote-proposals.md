promote proposal for work item "workspace-protocol" (docs/history/workspace-protocol/CONTEXT.md + docs/history/workspace-protocol/plan.md) — 4 capped cell(s): wsp-1, wsp-2, wsp-3, wsp-4
anchor: history — docs/history/workspace-protocol/CONTEXT.md, docs/history/workspace-protocol/plan.md
PROPOSAL ONLY — nothing was written. Applying any section below is a human or agent decision.

(a) DELIVERY DRAFT — save as docs/knowledge/work/workspace-protocol/delivery.md

---
type: bee.delivery
title: workspace-protocol — delivery
description: "Delivery record proposed by bee knowledge promote for work item workspace-protocol: 4 capped cell(s), 1 recorded deviation(s)."
timestamp: 2026-11-07
bee:
  id: workspace-protocol-delivery
  lifecycle: active
  areas: [workflow-state, doctrine-layer]
  required_context: [docs/history/workspace-protocol/CONTEXT.md, docs/history/workspace-protocol/plan.md]
  sources: [docs/history/workspace-protocol/CONTEXT.md, docs/history/workspace-protocol/plan.md, .bee/cells/wsp-1.json, .bee/cells/wsp-2.json, .bee/cells/wsp-3.json, .bee/cells/wsp-4.json]
---

# workspace-protocol — Delivery

## What shipped

- **wsp-1** — Added verbs/protocol.rs (parse, state, sha256, template, check and init verbs), the protocol router arm, registry entries and the advisory doctor row; commit 29bf129473fb4a7409ab2651761b09cd4e4d9d47 (5 file(s) changed)
- **wsp-2** — Raised PINNED_FLAG_COUNT to 240 and COMMAND_SURFACE_BUDGET_CHARS to 2750 per aa45854c; the full workspace suite is green; commit f90d0646c064b239ce1b43cdd60f8b5e2539aaef (2 file(s) changed)
- **wsp-3** — Added the no-menu worker line, the advisor no-verdict line, the D6 leader rule and the brief-discipline paragraph; regen done; two catalog tests fail from other cells' verbs; commit 4b2dd2684facee6ccacb07b5a33fa19501b2cb0a (21 file(s) changed)
- **wsp-4** — Wrote the workspace-protocol concept and committed it as d99373034; the docs/knowledge/index.md entry was not added because that file is generated and never lists area concepts — run bee knowledge index. (1 file(s) changed)

## Verify

Each cell below was capped only against a recorded passing verify result — bee refuses a cap without one.

- **wsp-1** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee protocol && PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee doctor && PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee --test registry_contracts` — full workspace run after the change; protocol 22, doctor 72, registry_contracts 12 passed, zero failed result lines
- **wsp-2** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee prepare && PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee dispatch && PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee --test registry_contracts` — the cell's proof command over the whole workspace, exit 0 (4776 passed, 0 failed in the bee unit binary; no failures in any other binary)
- **wsp-3** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee prompt && .bee/bin/bee dev release-manifest --check` — prompt render tests 110 passed and release-manifest --check matches 605 files; full-suite reds (flag pin, preamble budget) come from the wsp-1/wsp-2 verbs and are fixed in wsp-2's reopen
- **wsp-4** — `rg -n 'WORKSPACE_PROTOCOL.md|prompt_sha256|dispatch verify|purpose_leaning' docs/knowledge/areas/workflow-state/workspace-protocol.md && rg -n 'workspace-protocol.md' docs/knowledge/index.md` — docs-only change; the index half (rg workspace-protocol.md docs/knowledge/index.md) exits 1 by design of the generated file; cargo not run

## Deviations

- **wsp-3** — sync-ack: the LEADER.md addition is a new paragraph between rule blocks in Start a session; agents-capture-line-at-close text is unchanged, so its applied_at homes need no edit

## Provenance

Proposed by `bee knowledge promote --work workspace-protocol` from 4 capped cell trace(s) in `.bee/cells/` and the anchor `docs/history/workspace-protocol/CONTEXT.md`, `docs/history/workspace-protocol/plan.md`. Every line above is copied from a trace or from the work item; nothing here is curated truth until a human or agent accepts it.

(b) AREA UPDATES — candidate spec-sync bullets, each citing its cell

areas: from the scribing stamp for "workspace-protocol" — .bee/logs/scribing-runs.jsonl's most recent entry (2026-11-07T08:13:54.957Z), the work item declares no bee.areas.

area workflow-state:
  (no capped behavior_change cell exists for this feature)

area doctrine-layer:
  (no capped behavior_change cell exists for this feature)

(c) PATTERN CANDIDATES — candidate bee.pattern concepts, bee.polarity pitfall

from cell wsp-3 — save as docs/knowledge/patterns/workspace-protocol-wsp-3-pitfall.md

---
type: bee.pattern
title: workspace-protocol cell wsp-3 — pitfall candidate
description: "Pitfall candidate mined from cell wsp-3's capped trace: sync-ack: the LEADER.md addition is a new paragraph between rule blocks in Start a session; agents-capture-line-at-close text is unchanged, so its applied_at h…"
timestamp: 2026-11-07
bee:
  id: workspace-protocol-wsp-3-pitfall
  lifecycle: draft
  areas: [workflow-state, doctrine-layer]
  sources: [.bee/cells/wsp-3.json]
  polarity: pitfall
---

# workspace-protocol cell wsp-3 — pitfall candidate

## What the cell did

Added the no-menu worker line, the advisor no-verdict line, the D6 leader rule and the brief-discipline paragraph; regen done; two catalog tests fail from other cells' verbs; commit 4b2dd2684facee6ccacb07b5a33fa19501b2cb0a

## Recorded evidence (verbatim from .bee/cells/wsp-3.json)

- **deviation** — sync-ack: the LEADER.md addition is a new paragraph between rule blocks in Start a session; agents-capture-line-at-close text is unchanged, so its applied_at homes need no edit

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

knowledge promote: 4 capped cell(s) mined, 1 delivery draft, 0 area bullet(s), 1 pattern candidate(s), 0 file(s) written.