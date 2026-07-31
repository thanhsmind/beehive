# Provenance — bee-scribing body rules

The scribing body states its rules bare (provenance exile, skill-token-diet D8). This table maps each
body rule to the decision(s) that authorize it and the rationale in one line. Long-form records:
`docs/decisions/`, `.bee/decisions.jsonl` (via `bee.mjs decisions search`), and
`docs/history/<feature>/CONTEXT.md` for the feature that locked each ID.

| Body rule | Decision IDs / labels | Rationale |
|---|---|---|
| The spec is the meaning, must survive a full rewrite on a different stack | decision 0002 | Code is the implementation; the BA-grade spec is what outlives it |
| §1/§2/§3/harvest/reading-map steps delegate as extraction/generation-tier I/O workers | Delegation contract D2/D3 (`bee-hive/references/routing-and-contracts.md`) | Mechanical gather/render steps dispatch down-tier; decide-altitude never delegates |
| A spoken user settlement ("chốt"/"final"/"ok ship it") makes capture mandatory in that same turn | decision 0003 | Explicit settlement signals are the loud case; capture cannot be deferred once said out loud |
| Capture cost is lane-scaled: high-risk merges now, every other lane queues a stub, merge happens at flush | decision 0017 | Durability now, elaboration at flush — the flow is never held hostage to spec elaboration |
| Bootstrap is offered, never auto-run, and creates only the missing map file(s) | D2 of harness10 | Bootstrap is inventory, not meaning; only user approval starts the pass |
| Sync mode runs after goal-check, never instead of it | D4 (goal-check judge tier, table in `bee-hive/references/routing-and-contracts.md`) | The semantic checklist judge already verifies `standard`/`high-risk` `behavior_change` cells before scribing sees them |
| Scribing debt (uncleared `behavior_change` caps) backstops self-detection, never replaces it | decision 0011 | Self-detection is the first duty; the debt count catches only what watching missed |
| Detection is the scribe's own duty, unprompted — announce and capture, never ask "should I document this?" | decision 0007 | A user having to say "ghi lại" means detection already failed once |
| A deferred user request becomes a `proposed` backlog row in the same turn, never a silent drop | D8 | The missed-capture failure applies to deferred/backlog items exactly as it applies to settled rules |
| Sync fires once at feature close (merging every capped `behavior_change` cell's deltas together), never per execution round or per cap | decision 7346e9d7 (feature-close philosophy), aligned with main-verifies D1-D5 (R82) | A feature = many slices = many cells; one executing pass is a small part — the spec merge runs once the feature's work fully completes |
