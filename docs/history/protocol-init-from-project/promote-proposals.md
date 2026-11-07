promote proposal for work item "protocol-init-from-project" (docs/history/protocol-init-from-project/CONTEXT.md + docs/history/protocol-init-from-project/plan.md) — 2 capped cell(s): pi-1, pi-2
anchor: history — docs/history/protocol-init-from-project/CONTEXT.md, docs/history/protocol-init-from-project/plan.md
PROPOSAL ONLY — nothing was written. Applying any section below is a human or agent decision.

(a) DELIVERY DRAFT — save as docs/knowledge/work/protocol-init-from-project/delivery.md

---
type: bee.delivery
title: protocol-init-from-project — delivery
description: "Delivery record proposed by bee knowledge promote for work item protocol-init-from-project: 2 capped cell(s), 1 recorded deviation(s)."
timestamp: 2026-11-07
bee:
  id: protocol-init-from-project-delivery
  lifecycle: active
  areas: [workflow-state, onboarding]
  required_context: [docs/history/protocol-init-from-project/CONTEXT.md, docs/history/protocol-init-from-project/plan.md]
  sources: [docs/history/protocol-init-from-project/CONTEXT.md, docs/history/protocol-init-from-project/plan.md, .bee/cells/pi-1.json, .bee/cells/pi-2.json]
---

# protocol-init-from-project — Delivery

## What shipped

- **pi-1** — protocol init renders from git, uat_stop and the release entry with a Sources list, facts and review output, plus --print; commit 6b9d4ecaca2cd217e2f9c2a304c5fe9ae5952c88 (4 file(s) changed)
- **pi-2** — Added Workspace protocol section, SKILL.md pointer, concept update citing c5e4d24a, ran bee dev regen; commit 0662e4412 (10 file(s) changed)

## Verify

Each cell below was capped only against a recorded passing verify result — bee refuses a cap without one.

- **pi-1** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee protocol && PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee catalog && PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee --test registry_contracts` — protocol 33 passed, catalog and registry_contracts green, full suite 5375 passed; flags.rs gained print in FLAG_ALONE_BOOLEANS so a bare --print parses (D7 needs it)
- **pi-2** — `.bee/bin/bee dev release-manifest --check && rg -n 'Workspace protocol' skills/bee-hive/references/onboarding.md && rg -n 'c5e4d24a' docs/knowledge/areas/workflow-state/workspace-protocol.md` — docs-only change; manifest check and both content greps matched

## Deviations

- **pi-1** — sync-ack: the skill side of this change is pi-2's bee-hive onboarding reference; no workflow-state owned skill describes protocol init

## Provenance

Proposed by `bee knowledge promote --work protocol-init-from-project` from 2 capped cell trace(s) in `.bee/cells/` and the anchor `docs/history/protocol-init-from-project/CONTEXT.md`, `docs/history/protocol-init-from-project/plan.md`. Every line above is copied from a trace or from the work item; nothing here is curated truth until a human or agent accepts it.

(b) AREA UPDATES — candidate spec-sync bullets, each citing its cell

areas: from the scribing stamp for "protocol-init-from-project" — .bee/logs/scribing-runs.jsonl's most recent entry (2026-11-07T09:23:41.550Z), the work item declares no bee.areas.

area workflow-state:
  (no capped behavior_change cell exists for this feature)

area onboarding:
  (no capped behavior_change cell exists for this feature)

(c) PATTERN CANDIDATES — candidate bee.pattern concepts, bee.polarity pitfall

from cell pi-1 — save as docs/knowledge/patterns/protocol-init-from-project-pi-1-pitfall.md

---
type: bee.pattern
title: protocol-init-from-project cell pi-1 — pitfall candidate
description: "Pitfall candidate mined from cell pi-1's capped trace: sync-ack: the skill side of this change is pi-2's bee-hive onboarding reference; no workflow-state owned skill describes protocol init"
timestamp: 2026-11-07
bee:
  id: protocol-init-from-project-pi-1-pitfall
  lifecycle: draft
  areas: [workflow-state, onboarding]
  sources: [.bee/cells/pi-1.json]
  polarity: pitfall
---

# protocol-init-from-project cell pi-1 — pitfall candidate

## What the cell did

protocol init renders from git, uat_stop and the release entry with a Sources list, facts and review output, plus --print; commit 6b9d4ecaca2cd217e2f9c2a304c5fe9ae5952c88

## Recorded evidence (verbatim from .bee/cells/pi-1.json)

- **deviation** — sync-ack: the skill side of this change is pi-2's bee-hive onboarding reference; no workflow-state owned skill describes protocol init

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

knowledge promote: 2 capped cell(s) mined, 1 delivery draft, 0 area bullet(s), 1 pattern candidate(s), 0 file(s) written.