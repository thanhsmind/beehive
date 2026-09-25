# refactor

1. Record existing behavior FIRST — a characterization test, a snapshot, or an
   equivalence script. A type check or a lint is not a record.
2. Prove that record green on the UNCHANGED tree.
3. Change structure in small steps.
4. The record stays green at every step.
5. A behavior change is not a refactor — it is a separate cell.
6. Subtract first: an early cell deletes dead code and one-caller wrappers.
7. An API move migrates every caller and deletes the old API in the same
   slice; then search strings and docs for the old name.
8. Never edit the record, the harness, or the baseline to make a step green.
9. Keep the diff only if it lowers what a reader must hold in mind.

Proof line: the record, green before the change and green after it.
