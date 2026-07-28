---
artifact_contract: bee-plan/v1
mode: standard
approved_gate2: 2026-07-28 (gate bypass TOTAL — audit decision logged)
---

# Main Verifies — Plan

CONTEXT: `docs/history/main-verifies/CONTEXT.md` (D1–D5).
Route: class=feature | lane=standard | flags=2 [multi-domain, proof-weakening] | files=8 (recorded; proof-weakening deliberate — relocation with a new gate).

## Approach

One slice. Wave 1 parallel: mv-1 (code) ∥ mv-2 (doctrine); mv-3 test tail.
This feature itself still closes under the OLD law (its cells carry classic
verify) — the new law applies to features started after it ships.

- **mv-1 (D1+D2+D3):** cap `--feature-verify-pending` flag + trace marker
  (classic path untouched); `state feature-verify record` verb (command,
  output sha, result, at → active workflow record; red storable, never
  satisfying); close-door guard at both doors (state set out of swarming +
  scribing-run), typed refusal naming the pending cells, no bypass lifts it —
  `guardTestCellDebt` mirror.
- **mv-2 (D4):** bee-executing loop text (implement→commit→report; cap via
  pending path; no suite runs); bee-swarming (drop routine re-runs + wave
  impacted; ONE feature verify at final-slice close → record → close);
  routing-and-contracts lane-table verify columns + red-path (D5);
  authoring law: bugfix repro red is main-produced pre-dispatch. Body budgets:
  executing 10225 zero headroom, swarming ~12B — net-zero bodies, detail to
  references.
- **mv-3:** owning-suite extensions: pending cap records marker without
  evidence; record verb round-trip + red never satisfies; both doors refuse
  with pending cells and no green record, pass after green record; classic
  cap path unchanged; guard immune to gate_bypass total.
- **Barrier** immediately after wave 1.

## Risk Map

| Component | Risk | Proof |
|---|---|---|
| Cap pending branch | MEDIUM | mv-3 + existing cap suites stay green |
| Close-door guard | MEDIUM | mv-3 both-doors cases; bypass immunity |
| Doctrine flip | LOW | fences; next feature runs the new law live |

## Test Matrix (targeted)

Pending cap: no evidence demanded, marker recorded · classic cap byte-identical ·
record verb: green/red round-trip, output sha stored · door: refuses (pending +
no record / red record / stale record), passes (green record), unaffected when
no pending cells · bypass total does not lift the door.
