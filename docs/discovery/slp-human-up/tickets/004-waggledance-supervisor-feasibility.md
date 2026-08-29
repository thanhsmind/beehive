---
type: research
status: open
claimed-by: none
blocked-by: none
---

## Question

Waggledance is read-only by design; the supervisor (2f4bf3b1) must observe
cross-project AND write its own records (observations, delegated decisions,
weekly report). Facts needed from the waggledance codebase: (a) what bee
state waggledance already reads and rolls up per registered project (the
"waiting on you" board's sources); (b) where an observer tick could run
(existing schedule/heartbeat machinery? a bee herding loop pointed at many
repos? a waggledance-side job?); (c) where supervisor records could live
without breaking read-only-by-design and without a cross-repo store —
per-target-repo `.bee/` writes, a waggledance-local db, or the cockpit's own
project; (d) what the MCP surface (`waggledance_ask_state`,
`waggledance_projects`, `waggledance_search`) already answers that the
observer would otherwise re-read from disk.

## Answer

(open)
