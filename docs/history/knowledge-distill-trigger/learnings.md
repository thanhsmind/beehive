# Learnings — knowledge-distill-trigger (2026-08-16)

6 capped cells, 1 recorded deviation, plan v1 rejected by plan-check.

- The plan-check pass earned its cost in one finding: v1's close door would
  have deadlocked every future close by construction, because
  `knowledge promote` generated `required_context` entries the checker
  resolved in a space they could never exist in (bundle-only). The fix was
  the generator/check contract, not the 13 symptom rows — repairing the
  rows alone would have re-broken on the next promote.
- Warn-only surfaces do rot silently: 58 warnings sat unread until they
  became a blocking door. The door armed clean only because the same
  feature repaired the in-scope debt first (D4's mechanism-then-backfill
  ordering) — arming a new guard over existing debt taxes bystanders.
- A required field on a shared params struct (`LogParams.relation`) fans
  out to every literal construction site at compile time — cell file
  lists are hypotheses (existing pattern 20260811); the deviation was
  named on the cap, not silently absorbed.
- The supersede citation-sweep proved itself the same day it gained a
  user: formally superseding the rust-rewrite rejection auto-queued 3
  stubs for docs still citing the dead decision, which were annotated
  (history records annotated, never rewritten) and the generated index
  re-rendered.
- Session craft: piping bee output through `tail -1` swallowed two typed
  refusals (phase set, scribing stamp) and cost a diagnosis loop — a
  refusal is output, never noise; keep refusal lines visible.

Promoted: nothing new — the feature's own specs were synced in-feature
(kdt-5, conformance-check.md, decision-memory/overview.md), and the one
pattern candidate duplicated 20260811-a-cells-declared-file-is-a-hypothesis.
