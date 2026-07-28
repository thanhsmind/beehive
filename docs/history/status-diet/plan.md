---
artifact_contract: bee-plan/v1
mode: standard
approved_gate2: 2026-07-28 (gate bypass TOTAL — audit decision logged)
---

# Status Diet — Plan

CONTEXT: `docs/history/status-diet/CONTEXT.md` (D1–D3).
Route: class=feature | lane=standard | flags=1 [multi-domain] | files=5 (recorded).

## Approach

One slice. Wave 1 parallel: st-1 (code) ∥ st-2 (contract text); st-3 test tail.

- **st-1 (D1):** `--brief` flag on the status verb: brief payload
  `{phase, feature, mode, gates, gate_bypass_level, ship_visibility, route}`
  from the state layer only — no cells/review/handoff/models work. Registry
  schema gains the flag. Human (non-json) brief prints one line.
- **st-2 (D2):** swarming-reference Worker Prompt Template — Startup step 2
  becomes `status --brief --json`; template's Identity/Inputs section gains an
  embedded `State:` line the orchestrator fills (phase, feature,
  gates.execution) at dispatch; align bee-executing references if they name
  full status. Fences green; body budgets hold.
- **st-3:** test — brief payload exact field set (nothing more), <50ms not
  asserted in CI (flaky) but payload byte ceiling asserted (<1KB), full status
  unchanged shape, flag registered (registry example executes).
- **Barrier** immediately after wave 1 (self-hosted runtime).

## Risk Map

| Component | Risk | Proof |
|---|---|---|
| --brief handler | LOW | st-3 + registry example test |
| Template change | LOW | fences; next dispatches use it live |

## Test Matrix (targeted)

Brief has exactly the 7 keys · route null when absent · full status byte-shape
untouched · unknown-flag refusal intact · brief under 1KB.
