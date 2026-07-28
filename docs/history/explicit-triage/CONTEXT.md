# Explicit Triage — Context

**Feature slug:** explicit-triage
**Date:** 2026-07-28
**Exploring session:** complete (user directive: "cho rõ ràng, không phải đoán nữa"; bypass TOTAL)
**Scope:** Standard
**Domain types:** CALL (CLI verb), ORGANIZE (triage law)

## Feature Boundary

Make lane triage leave a machine-readable trace instead of an in-head guess:
a validated **route record** — `class | lane | flags | product_files` — persisted
on the feature's workflow record at start, surfaced in `status --json` and the
session preamble, updated by the re-lane checkpoint. The delegation status-line
half of the original comparison is already closed (bee's `[DONE]/[BLOCKED]/
[HANDOFF]/[NOOP]` contract predates this feature); this feature closes the
routing half.

## Locked Decisions

| ID | Decision | Rationale |
|----|----------|-----------|
| D1 | New CLI verb `bee state route --set` / `--show`: persists `{class, lane, flags[], product_files, rationale}` on the ACTIVE feature's workflow record. Validated, typed refusals: `class` ∈ {feature, bugfix, docs, refactor, research, release, spike}; `lane` ∈ {docs, tiny, small, spike, standard, high-risk}; every `flags[]` entry from the canonical mode-gate list (auth, authorization, data-model, audit-security, external-systems, public-contracts, cross-platform, covered-contract-change, proof-weakening, multi-domain); `product_files` a non-negative integer. Free prose is refused — that is the whole point. | ak's classifier wins because the route is an explicit checkable line; an enum-validated record cannot be vibes. |
| D2 | Surfacing: `status --json` carries the `route` block; the session preamble renders one line — `Route: class=<c> | lane=<l> | flags=<n> [<names>] | files=<n>` — when a route exists for the active feature; nothing when absent. | The record must be seen every session start to be checkable; zero cost when absent (ship-visibility precedent). |
| D3 | Soft enforcement first: `cells claim` emits a one-line stderr warning when the claimed cell's feature has no route record — never a refusal in this pass. | Hard refusal risks blocking tiny-lane flow before the habit forms; warn now, tighten later if warnings recur (ratchet philosophy). |
| D4 | Triage law (hive): the mode gate's flag count is RECORDED same turn via `state route --set` — counting without recording is the "đoán" this feature kills; the re-lane checkpoint updates the same record (demotion = new lane + logged decision, never a second record). Mode-gate records in plan.md/cells stay and cite the route record. | One route per feature, updated in place, cited everywhere. |
| D5 | Parallel wave: et-1 (code) ∥ et-2 (law text), wave-barrier acks; et-3 slice-tail test cell. | Parallel-default doctrine; disjoint file sets. |

### Agent's Discretion

- Store shape details (field name on the workflow record, update path) —
  follow the workflow-store update pattern fx-1 used.
- Preamble wording within D2's format.

## Existing Code Context

- `packages/bee/lib/workflow-store.mjs` — workflow record + update path (fx-1's close transition is the fresh precedent).
- `packages/bee/lib/command-registry.mjs:866+` — `state.start-feature` entry; new verb registers beside it.
- `packages/bee/lib/inject.mjs` — preamble renderer (ship_visibility line is the one-conditional-line precedent).
- `packages/bee/bee.mjs` — status payload (ship_visibility field precedent) + claim handler for the D3 warning.
- `skills/bee-hive/SKILL.md` (budget 8183/8192 — 9 bytes headroom: net-neutral body edits, detail to references) + `references/routing-and-contracts.md` — triage/mode-gate law home.

## Outstanding Questions

### Deferred To Planning
- [ ] Whether `state route --set` also accepts `--feature` override for non-active features (default: active only).

## Handoff Note

Decision IDs stable; planning implements exactly.
