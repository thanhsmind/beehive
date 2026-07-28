# Status Diet — Context

**Feature slug:** status-diet
**Date:** 2026-07-28
**Exploring session:** complete (measured evidence; bypass TOTAL)
**Scope:** Standard
**Domain types:** CALL (CLI), ORGANIZE (worker contract)

## Feature Boundary

Every worker startup pays `status --json`: measured **372ms wall + 15,171B
payload** (353 cell files scanned; models/tier_mix/handoff/review blocks a
worker never reads). Raw `state.json` parse costs 22ms incl. node startup.
Fix both sides: a `--brief` fast path, and a worker contract that stops
calling the full verb.

## Locked Decisions

| ID | Decision | Rationale |
|----|----------|-----------|
| D1 | `bee status --brief` (with `--json`): reads ONLY the state layer needed for orientation — `phase, feature, mode, gates, gate_bypass_level, ship_visibility, route` — no cells scan, no review scan, no handoff resolution, no models/tier_mix. Target <50ms wall, <400B JSON. Full `status` unchanged for humans/orchestrator routing. | The hot path is worker startup × every dispatch; 15KB × N workers is pure token waste (user's call). |
| D2 | Worker contract: the dispatch prompt EMBEDS the state line (the orchestrator just read it); worker startup step 2 becomes `status --brief --json` (cheap live check), and `cells show` stays the claim authority. Full `status --json` is no longer part of the worker template. | The orchestrator already holds fresh state at dispatch; live re-validation only needs the brief surface. |
| D3 | Parallel wave: st-1 (code) ∥ st-2 (template/law), wave-barrier acks; st-3 slice-tail test. | Doctrine. |

### Agent's Discretion

- Brief payload field order/shape; flag parsing details (mirror existing status flags).
- Template wording within D2.

## Existing Code Context

- `packages/bee/bee.mjs` — status handler (payload build; route/ship_visibility blocks fresh precedent); `packages/bee/lib/command-registry.mjs` status entry.
- `skills/bee-swarming/references/swarming-reference.md` — Worker Prompt Template, Startup step 2 (`node .bee/bin/bee.mjs status --json`).
- `skills/bee-executing/SKILL.md` / references — if the executing loop also names full status, align (verify at implementation).
- Measurement (this session): full 372ms/15,171B; state.json parse 22ms; 353 cells.

## Handoff Note

Decision IDs stable; planning implements exactly.
