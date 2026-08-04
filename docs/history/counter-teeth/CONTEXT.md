# counter-teeth — CONTEXT

Locked product decisions for Batch A of the prose-rule-audit: four advisory
counters gain refusal teeth. Parent decision: c2a7bd4f (scope approved by
user, 2026-08-04). Deviation record: 3baa41f6 (no route record — route verb
broken under live worktree grants, P1 finding filed).

## Locked decisions

- **D1 — close refuses uncaptured behavior_change cells.** `bee close`
  refuses when any behavior_change cell of the closing feature has no
  capture recorded, unless a logged decision tagged `capture-deferral`
  names the feature (precedent: decision c8e25271). The refusal names the
  uncaptured cell ids and the two remedies (run bee-capturing, or log a
  deferral decision). Cite: c2a7bd4f item 1.
- **D2 — capture-queue blocker thresholds.** `bee orient` escalates the
  capture-queue line from offer to blocker when the queue holds ≥ 10
  pending stubs OR the oldest pending stub is older than 7 days. Constants
  in code for this batch; config keys are future work, not this scope.
  Cite: c2a7bd4f item 2.
- **D3 — ceiling tier refusal over budget.** `bee cells tier` with tier
  `ceiling` refuses when the resulting ceiling share of tiered cells would
  exceed 40% (decision 0012's threshold), unless `--reason <text>` is
  given; the reason is stored on the cell's tier record. The refusal names
  the current share and the threshold. Cite: c2a7bd4f item 3.
- **D4 — route-record warn escalates to deny.** The first `cells claim` in
  a feature with no route record keeps today's stderr warning; the second
  and later claims refuse, naming `bee route --set` as the remedy.
  Cite: c2a7bd4f item 4.
- **D5 — D4 is blocked on the route fix (fix-first).** The remedy command
  `bee route --set` is currently broken for code-touching lanes whenever
  any worktree grant exists (unported Node arm, Err2::Ex → misleading
  refusal; P1 finding on backlog, feature counter-teeth). Per the
  sequencing law in c2a7bd4f, a check flips to blocking only after its
  counter and its remedy are verified correct — so the route granted-arm
  fix is a prerequisite cell inside this feature, landing before D4's
  deny.
- **D6 — sequencing law.** For every one of D1-D4: first a test proving
  the counter/condition is computed correctly, then the flip to refusal.
  No flip ships with a known false positive (pattern 20260729).

## Open questions

- None blocking. Config-key surfacing of D2/D3 thresholds is deferred
  future work (backlog, not this feature).

## Out of scope

- Any other audit batch (B: new hooks; C: doctrine diet).
- Config keys for thresholds; changing decision 0012's 40% value.
