promote proposal for work item "deploy-pane-authorization" (docs/history/deploy-pane-authorization/plan.md) — 1 capped cell(s): dpa-1
anchor: history — docs/history/deploy-pane-authorization/plan.md
PROPOSAL ONLY — nothing was written. Applying any section below is a human or agent decision.

(a) DELIVERY DRAFT — save as docs/knowledge/work/deploy-pane-authorization/delivery.md

---
type: bee.delivery
title: deploy-pane-authorization — delivery
description: "Delivery record proposed by bee knowledge promote for work item deploy-pane-authorization: 1 capped cell(s), 2 recorded deviation(s)."
timestamp: 2026-09-15
bee:
  id: deploy-pane-authorization-delivery
  lifecycle: active
  areas: [bee-herding, doctrine-layer]
  required_context: [docs/history/deploy-pane-authorization/plan.md]
  sources: [docs/history/deploy-pane-authorization/plan.md, .bee/cells/dpa-1.json]
---

# deploy-pane-authorization — Delivery

## What shipped

- **dpa-1** — deploy permit and issuer session reach the herding worker pane (3 file(s) changed)

## Verify

Each cell below was capped only against a recorded passing verify result — bee refuses a cap without one.

- **dpa-1** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee deploy_ && PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee herding::run::` — touched the deployment export line in prepare_dispatch_wire and the fresh-spawn pane env in herding run; deploy_ 8 passed, herding::run:: 214 passed incl. 2 new passthrough tests, red before fix per …

## Deviations

- **dpa-1** — dpa-1 escalated to the session model: the agy-flash worker wrote the red tests, then its herding run died with the leader session restart; the leader wrote the two fix edits
- **dpa-1** — sync-ack: skills/bee-herding/* never documents the pane export line, BEE_HERDING_WORKER, BEE_HERDING_JOB_ID or deploy permits (rg over skills/bee-herding finds none), so no owned skill text goes stale; the behavior is captured in docs/knowledge/areas/bee-herding/handing-a-foreign-agent-its-brief.md after merge

## Provenance

Proposed by `bee knowledge promote --work deploy-pane-authorization` from 1 capped cell trace(s) in `.bee/cells/` and the anchor `docs/history/deploy-pane-authorization/plan.md`. Every line above is copied from a trace or from the work item; nothing here is curated truth until a human or agent accepts it.

(b) AREA UPDATES — candidate spec-sync bullets, each citing its cell

areas: from the scribing stamp for "deploy-pane-authorization" — .bee/logs/scribing-runs.jsonl's most recent entry (2026-09-15T05:36:05.972Z), the work item declares no bee.areas.

area bee-herding:
  - [dpa-1] deploy permit and issuer session reach the herding worker pane — feature-wide sync per the scribing stamp, 3 file(s) changed (trace .bee/cells/dpa-1.json)

area doctrine-layer:
  - [dpa-1] deploy permit and issuer session reach the herding worker pane — feature-wide sync per the scribing stamp, 3 file(s) changed (trace .bee/cells/dpa-1.json)

(c) PATTERN CANDIDATES — candidate bee.pattern concepts, bee.polarity pitfall

from cell dpa-1 — save as docs/knowledge/patterns/deploy-pane-authorization-dpa-1-pitfall.md

---
type: bee.pattern
title: deploy-pane-authorization cell dpa-1 — pitfall candidate
description: "Pitfall candidate mined from cell dpa-1's capped trace: dpa-1 escalated to the session model: the agy-flash worker wrote the red tests, then its herding run died with the leader session restart; the leader wrote the…"
timestamp: 2026-09-15
bee:
  id: deploy-pane-authorization-dpa-1-pitfall
  lifecycle: draft
  areas: [bee-herding, doctrine-layer]
  sources: [.bee/cells/dpa-1.json]
  polarity: pitfall
---

# deploy-pane-authorization cell dpa-1 — pitfall candidate

## What the cell did

deploy permit and issuer session reach the herding worker pane

## Recorded evidence (verbatim from .bee/cells/dpa-1.json)

- **deviation** — dpa-1 escalated to the session model: the agy-flash worker wrote the red tests, then its herding run died with the leader session restart; the leader wrote the two fix edits
- **deviation** — sync-ack: skills/bee-herding/* never documents the pane export line, BEE_HERDING_WORKER, BEE_HERDING_JOB_ID or deploy permits (rg over skills/bee-herding finds none), so no owned skill text goes stale; the behavior is captured in docs/knowledge/areas/bee-herding/handing-a-foreign-agent-its-brief.md after merge

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

knowledge promote: 1 capped cell(s) mined, 1 delivery draft, 2 area bullet(s), 1 pattern candidate(s), 0 file(s) written.