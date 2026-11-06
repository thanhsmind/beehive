promote proposal for work item "bee-config-verbs" (.bee/logs/scribing-runs.jsonl + .bee/lanes/bee-config-verbs.json) — 1 capped cell(s): cfg-1
anchor: ledger — .bee/logs/scribing-runs.jsonl, .bee/lanes/bee-config-verbs.json
PROPOSAL ONLY — nothing was written. Applying any section below is a human or agent decision.

(a) DELIVERY DRAFT — save as docs/knowledge/work/bee-config-verbs/delivery.md

---
type: bee.delivery
title: bee-config-verbs — delivery
description: "Delivery record proposed by bee knowledge promote for work item bee-config-verbs: 1 capped cell(s), 5 recorded deviation(s)."
timestamp: 2026-11-06
bee:
  id: bee-config-verbs-delivery
  lifecycle: active
  areas: [rust-runtime]
  required_context: [.bee/logs/scribing-runs.jsonl, .bee/lanes/bee-config-verbs.json]
  sources: [.bee/logs/scribing-runs.jsonl, .bee/lanes/bee-config-verbs.json, .bee/cells/cfg-1.json]
---

# bee-config-verbs — Delivery

## What shipped

- **cfg-1** — bee config get, set, unset, explain and validate run natively; governed keys refused via the write guard check (7 file(s) changed)

## Verify

Each cell below was capped only against a recorded passing verify result — bee refuses a cap without one.

- **cfg-1** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml --test config_verbs --test registry_contracts` — 7 + 12 passed over the touched config verbs and registry payload; also ran registry_dispatch (9), bin router:: (20) and help (32), comment-baseline and release-manifest checks, all clean; cargo was a…

## Deviations

- **cfg-1** — scope: edited tests/registry_dispatch.rs (not in cell files) — its command_unavailable fixture used config get, which this cell's own action makes runnable; moved it to herding enable, one line
- **cfg-1** — governed-key refusal reuses the guard's config_governed_change_deny over the merged tracked+local config in both directions (forward gives the guard's own deny text; reverse catches adds the guard lets through), so no list is copied; a field the guard's view ignores (for example a role description) stays settable
- **cfg-1** — no registry generator exists in the repo (scripts/export_registry_payload.mjs is gone); registry_payload.json was edited by a JSON round-trip that keeps its compact byte format
- **cfg-1** — config get without --local reads the tracked file only, as Node did; the old forced-local routing for guards.* and hooks.* was not carried over (hooks is now governed and refused)
- **cfg-1** — set and unset results carry local and warning beside the contract keys

## Provenance

Proposed by `bee knowledge promote --work bee-config-verbs` from 1 capped cell trace(s) in `.bee/cells/` and the anchor `.bee/logs/scribing-runs.jsonl`, `.bee/lanes/bee-config-verbs.json`. Every line above is copied from a trace or from the work item; nothing here is curated truth until a human or agent accepts it.

(b) AREA UPDATES — candidate spec-sync bullets, each citing its cell

areas: from the scribing stamp for "bee-config-verbs" — .bee/logs/scribing-runs.jsonl's most recent entry (2026-11-06T03:41:27.412Z), the work item declares no bee.areas.

area rust-runtime:
  (no capped behavior_change cell exists for this feature)

(c) PATTERN CANDIDATES — candidate bee.pattern concepts, bee.polarity pitfall

from cell cfg-1 — save as docs/knowledge/patterns/bee-config-verbs-cfg-1-pitfall.md

---
type: bee.pattern
title: bee-config-verbs cell cfg-1 — pitfall candidate
description: "Pitfall candidate mined from cell cfg-1's capped trace: scope: edited tests/registry_dispatch.rs (not in cell files) — its command_unavailable fixture used config get, which this cell's own action makes runnable; mo…"
timestamp: 2026-11-06
bee:
  id: bee-config-verbs-cfg-1-pitfall
  lifecycle: draft
  areas: [rust-runtime]
  sources: [.bee/cells/cfg-1.json]
  polarity: pitfall
---

# bee-config-verbs cell cfg-1 — pitfall candidate

## What the cell did

bee config get, set, unset, explain and validate run natively; governed keys refused via the write guard check

## Recorded evidence (verbatim from .bee/cells/cfg-1.json)

- **deviation** — scope: edited tests/registry_dispatch.rs (not in cell files) — its command_unavailable fixture used config get, which this cell's own action makes runnable; moved it to herding enable, one line
- **deviation** — governed-key refusal reuses the guard's config_governed_change_deny over the merged tracked+local config in both directions (forward gives the guard's own deny text; reverse catches adds the guard lets through), so no list is copied; a field the guard's view ignores (for example a role description) stays settable
- **deviation** — no registry generator exists in the repo (scripts/export_registry_payload.mjs is gone); registry_payload.json was edited by a JSON round-trip that keeps its compact byte format
- **deviation** — config get without --local reads the tracked file only, as Node did; the old forced-local routing for guards.* and hooks.* was not carried over (hooks is now governed and refused)
- **deviation** — set and unset results carry local and warning beside the contract keys

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

knowledge promote: 1 capped cell(s) mined, 1 delivery draft, 0 area bullet(s), 1 pattern candidate(s), 0 file(s) written.