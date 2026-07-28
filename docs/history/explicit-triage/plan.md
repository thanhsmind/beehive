---
artifact_contract: bee-plan/v1
mode: standard
approved_gate2: 2026-07-28 (gate bypass TOTAL — audit decision logged)
---

# Explicit Triage — Plan

CONTEXT: `docs/history/explicit-triage/CONTEXT.md` (D1–D5).

## Mode-Gate Record

Flags: 1 (multi-domain: CLI code + triage law). Story-sized → standard.
Product files ≈ 7. (This plan's own route record is set via the new verb the
moment et-1 lands — the feature dogfoods itself at close.)

## Approach

One slice. **Wave 1 (parallel, wave-barrier): et-1 ∥ et-2**; **Wave 2: et-3**.

- **et-1 (D1+D2+D3, code):** `state route` verb (set/show) with typed enum
  refusals; `route` field on the active workflow record (workflow-store update
  path per fx-1 precedent); `status --json` route block; preamble `Route:` line
  when present (inject.mjs, ship_visibility precedent); `cells claim` stderr
  warning when the feature lacks a route (never refuses).
- **et-2 (D4, law):** hive triage section — count THEN record same turn
  (`state route --set`, format cited); re-lane updates the record in place;
  routing-and-contracts carries the full protocol; body edits net-neutral
  (9 bytes headroom), detail to references.
- **et-3 (test):** route verb validation (good set round-trips; bad class/lane/
  flag/count typed-refused; show renders), status+preamble surfacing, claim
  warning present/absent, re-lane update-in-place. Extend owning suites
  (test_bee_cli / test_state_projection style — locate first).
- **Barrier:** regen chain once at wave close, paid IMMEDIATELY after wave 1
  caps (foundation-fixes finding 4: self-hosted runtime changes sync fast).

## Risk Map

| Component | Risk | Proof |
|---|---|---|
| Route verb + store field | LOW | et-3 net; workflow-store suites |
| Preamble/status surfacing | LOW | et-3 |
| Claim warning | LOW | et-3 (soft, never refuses) |
| Law text within 9-byte body headroom | MEDIUM | skill fences green |

## Test Matrix (targeted)

Valid route round-trip · each enum violation typed-refused · absent route =
no preamble line, claim warns · present route = line rendered, no warn ·
re-lane demotion rewrites lane in place (one record) · fences green post-law.
