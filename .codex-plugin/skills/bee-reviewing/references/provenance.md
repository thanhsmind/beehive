# Provenance — bee-reviewing body rules

The reviewing body states its rules bare (provenance exile, skill-token-diet D8). This table maps each
body rule to the decision(s) that authorize it and the rationale in one line. Long-form records:
`docs/decisions/`, `.bee/decisions.jsonl` (via `bee.mjs decisions search`), and
`docs/history/<feature>/CONTEXT.md` for the feature that locked each ID.

| Body rule | Decision IDs / labels | Rationale |
|---|---|---|
| Review is not an automatic stage; it runs only on explicit user request | decision `565e68d0-327f-404e-b49e-d1c61ba81bfd` | Execution always closes through scribing/compounding unreviewed; review is an add-on the human chooses, not a tax on every close |
| Trigger phrasings; nothing else dispatches a session | R1 | The five phrasings are the complete match set — no inference, no vibes |
| "merge"/"ship"/"release" alone is not a trigger; report count+risk, ask exactly one question, silence = no | spec §7.4 / A9 | A ship request should surface unreviewed risk without forcing a review the user didn't ask for |
| Gate bypass never creates or auto-approves a session | R8 | Bypass covers gate approval speed, not the decision to spend reviewer tokens at all |
| User owns the scope boundary; five resolvable scope types | R4 | The reviewer's job is to review the boundary the user drew, not to guess a bigger or smaller one |
| Scope frozen before any reviewer dispatch (build JSON → `reviews create` → preview → record manifest) | R5 | An unfrozen scope lets the diff move under the reviewers' feet mid-session |
| Batch scope (types 3/5) resolves to one cumulative diff with a per-region feature/cell mapping | spec §7.3 | Reviewers need to see interaction bugs between changes made together — the whole point of batching |
| In-progress (`open`/`claimed`) cells are excluded from scope, never waited on or assumed done | A6 | Reviewing a moving target produces findings against code that hasn't settled |
| Active workflow state is preserved before entering review and restored exactly after | spec §7.5 | A review session must never overwrite active work or drop a handoff |
| `reviews create` preflights verification evidence and fails closed on any gap | A10 | A session must not exist over unproven work; missing evidence stops session creation itself |
| No lane auto-runs a reviewer at feature close | goal 1 | Zero reviewer tokens spent without an explicit request |
| `tiny`'s done-report (single-execution-worker dispatch inside bee-swarming) is verification, never independent review | AO14 | Verification and review are different guarantees; one never silently substitutes for the other |
| Once requested, panel depth scales to the SESSION's scope risk, never reduced by the originating feature's lane or by bypass | goal 5 | A `tiny` feature swept into a `standard`/high-risk batch still gets the batch's full panel |
| Reviewers spawn as the runtime's default/general subagent type with an inline persona, never another plugin's registered agent type | spec (spawn contract) | A same-named foreign agent silently carries a different finding format, severity scale, and report path |
| Reviewers resolve the dedicated `review` tier (independent reviewer > self-review) | P16, decision 0021 | The model that reviews should not be the model that implemented |
| A cli-shaped review slot resolves via the purpose-scoped 4-arg form; a bare 3-arg resolve refuses | AO12/B1, plan 2A-ii | An external adversarial reviewer must be dispatched through the Delegation contract's cli gather branch, never as a bare resolve |
| Conditional reviewers spawn on mechanical diff triggers, cap the wave at 6 | spec (conditional reviewer table) | Triggers are grepped, not vibed; the cap tracks the roster (4 core + 2 conditional) |
| Synthesis happens only after every reviewer returns, and only on the orchestrator | spec §2 | The synthesis agent used to run on the orchestrator's own model anyway — dispatching it added a hop, not a mind |
| Verification-evidence gate is a backstop, not the primary catch; an assertion-capped cell reaching review is a double bypass and a P1 | decision 0009 | Cap-time and `reviews create`-time enforcement already block bad evidence from reaching a session; a real gap is a P1, not a backfill-document loop |
| Frozen-judge flags are reviewed assuming the judge was moved, not passed; a weakened judge is always P1 | P12, decision 0018 | A softened judge invalidates the whole wave's evidence, not just one file |
| Delta re-review: re-review the fix delta AND sweep the whole scope for the finding's defect class | fix protocol R9/A12; critical pattern 20260711 (grill deltas) | The same bug class often recurs elsewhere in scope, not just at the line that changed |
| A localized, boundary-respecting P1 fix needs only its delta + defect-class sweep re-reviewed, never a full-panel re-run | A12 | Forcing a full re-run on content that never changed wastes the panel without adding evidence |
| A session stays blocked until every open P1's delta re-review passes | A11 | Partial resolution is not resolution; the gate holds until the whole wave clears |
| Bypass may auto-approve the merge question only when P1 = 0 and every UAT item passed; secret reads always need human approval | R8, decision 0010 boundary | UAT and P1 stops are the two guarantees bypass may never silently skip |
| No re-dispatch for an unchanged range already reported `reviewed (covered by <id>)` | R6/A7 | Re-reviewing settled, unchanged work spends tokens without adding evidence |
| Session closeout closes the review, never a feature's own phase; every feature already closed independently through execution → scribing → compounding | spec §11.1 | A review session is an overlay on already-closed work, not a workflow stage a feature must pass through |
