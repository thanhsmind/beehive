promote proposal for work item "opencode-2x-tool-ids" (.bee/logs/scribing-runs.jsonl + .bee/lanes/opencode-2x-tool-ids.json) — 1 capped cell(s): oc2x-1
anchor: ledger — .bee/logs/scribing-runs.jsonl, .bee/lanes/opencode-2x-tool-ids.json
PROPOSAL ONLY — nothing was written. Applying any section below is a human or agent decision.

(a) DELIVERY DRAFT — save as docs/knowledge/work/opencode-2x-tool-ids/delivery.md

---
type: bee.delivery
title: opencode-2x-tool-ids — delivery
description: "Delivery record proposed by bee knowledge promote for work item opencode-2x-tool-ids: 1 capped cell(s), 3 recorded deviation(s)."
timestamp: 2026-11-06
bee:
  id: opencode-2x-tool-ids-delivery
  lifecycle: active
  areas: [hook-runtime]
  required_context: [.bee/logs/scribing-runs.jsonl, .bee/lanes/opencode-2x-tool-ids.json]
  sources: [.bee/logs/scribing-runs.jsonl, .bee/lanes/opencode-2x-tool-ids.json, .bee/cells/oc2x-1.json]
---

# opencode-2x-tool-ids — Delivery

## What shipped

- **oc2x-1** — Registry gate derives from both opencode 1.18.x and 2.x Set literals; 2.x ids named as gaps; 2.x does not load the 1.x bee-guard.ts plugin (2 file(s) changed)

## Verify

Each cell below was capped only against a recorded passing verify result — bee refuses a cap without one.

- **oc2x-1** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml --test opencode_plugin_contracts && .bee/bin/bee dev release-manifest --check` — 10 passed against installed @opencode/cli 2.2.5 (cargo from ~/.cargo/bin on the default PATH); the 1.18.x shape is pinned by a new synthetic-text unit test because no 1.18 binary is on this machine; …

## Deviations

- **oc2x-1** — found a better route: step 1 showed 2.x hands new ids (shell/patch/subagent) to execute.before hooks, but its loader accepts only default {id, effect|setup}; bee-guard.ts default-exports a 1.x hook function, so it never loads on 2.x. A case shell/patch alias in mapToolCall would be false coverage, so bee-guard.ts is untouched (decision 31ba5052 allows the named-gap path) and no regen/manifest change was needed
- **oc2x-1** — 2.x also renames write/edit/read args filePath->path and moves hook args to input; porting the belt to the 2.x {id, setup} API is proposed as follow-up work
- **oc2x-1** — 2.x registry adds additional anchors artifact_create, artifact_update, schedule_create (experimental flag), named as gaps; webfetch/websearch/todowrite/skill named as exclusions since 2.x tools are never classified safe by a filePath scan

## Provenance

Proposed by `bee knowledge promote --work opencode-2x-tool-ids` from 1 capped cell trace(s) in `.bee/cells/` and the anchor `.bee/logs/scribing-runs.jsonl`, `.bee/lanes/opencode-2x-tool-ids.json`. Every line above is copied from a trace or from the work item; nothing here is curated truth until a human or agent accepts it.

(b) AREA UPDATES — candidate spec-sync bullets, each citing its cell

areas: from the scribing stamp for "opencode-2x-tool-ids" — .bee/logs/scribing-runs.jsonl's most recent entry (2026-11-06T03:40:38.281Z), the work item declares no bee.areas.

area hook-runtime:
  (no capped behavior_change cell exists for this feature)

(c) PATTERN CANDIDATES — candidate bee.pattern concepts, bee.polarity pitfall

from cell oc2x-1 — save as docs/knowledge/patterns/opencode-2x-tool-ids-oc2x-1-pitfall.md

---
type: bee.pattern
title: opencode-2x-tool-ids cell oc2x-1 — pitfall candidate
description: "Pitfall candidate mined from cell oc2x-1's capped trace: found a better route: step 1 showed 2.x hands new ids (shell/patch/subagent) to execute.before hooks, but its loader accepts only default {id, effect|setup}; b…"
timestamp: 2026-11-06
bee:
  id: opencode-2x-tool-ids-oc2x-1-pitfall
  lifecycle: draft
  areas: [hook-runtime]
  sources: [.bee/cells/oc2x-1.json]
  polarity: pitfall
---

# opencode-2x-tool-ids cell oc2x-1 — pitfall candidate

## What the cell did

Registry gate derives from both opencode 1.18.x and 2.x Set literals; 2.x ids named as gaps; 2.x does not load the 1.x bee-guard.ts plugin

## Recorded evidence (verbatim from .bee/cells/oc2x-1.json)

- **deviation** — found a better route: step 1 showed 2.x hands new ids (shell/patch/subagent) to execute.before hooks, but its loader accepts only default {id, effect|setup}; bee-guard.ts default-exports a 1.x hook function, so it never loads on 2.x. A case shell/patch alias in mapToolCall would be false coverage, so bee-guard.ts is untouched (decision 31ba5052 allows the named-gap path) and no regen/manifest change was needed
- **deviation** — 2.x also renames write/edit/read args filePath->path and moves hook args to input; porting the belt to the 2.x {id, setup} API is proposed as follow-up work
- **deviation** — 2.x registry adds additional anchors artifact_create, artifact_update, schedule_create (experimental flag), named as gaps; webfetch/websearch/todowrite/skill named as exclusions since 2.x tools are never classified safe by a filePath scan

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

knowledge promote: 1 capped cell(s) mined, 1 delivery draft, 0 area bullet(s), 1 pattern candidate(s), 0 file(s) written.