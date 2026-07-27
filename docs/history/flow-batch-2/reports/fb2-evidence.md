# flow-batch-2 evidence — specs #81 P4/P3 and #77 P2/P3/P4

Both dispatched workers died mid-edit on an API session limit, not on the work.
Their partial output was inspected, judged sound, and finished in-session rather
than discarded: the re-lane reference section and the `bee-validating` body
restructure were already complete and coherent.

## Byte budget — the constraint most likely to be quietly broken

Spec #78 exists because these bodies are injected whole on every invoke, so a
spec that improves flow by fattening them trades one cost for another. Every
line added here was paid for by a line removed.

| Skill body | Before | After | Δ |
|---|---|---|---|
| bee-hive | 30076 | 29808 | −268 |
| bee-swarming | 24965 | 24937 | −28 |
| bee-exploring | 16209 | 15915 | −294 |
| bee-validating | 17260 | 17243 | −17 |
| bee-planning | 23698 | 23681 | −17 |

Five bodies carrying three new rules, all five smaller than before. The
mechanics live in references, which are lazy-loaded.

## #81 P4 — progress ticks

Contract in `bee-hive/references/routing-and-contracts.md` ("Progress ticks"):
one line per event, never a question — cell cap, slice close, wave completion,
re-lane, draft PR, demo. Ticks are chat output the agent writes as it goes, not
an emitter subsystem: nothing to build, nothing to poll. Silence has exactly two
switches, `quiet: true` and `ship_visibility: "off"`.

Wired at the cap seam in `bee-swarming` and routed from `bee-hive`'s
silent-bookkeeping rule, which is the rule ticks could otherwise be read as
contradicting — hence the explicit "silent is not invisible" clause.

## #81 P3 — evidence-based lane demotion

One checkpoint per feature, after the first evidence pass: `bee-exploring` step
3 when exploring runs, `bee-planning` §2 step 7 only when it was skipped. All
three conditions required and measured — counted product-file touch set within
threshold, zero hard-gate flags, zero open gray areas. One step only, `standard`
→ `small`, never into `tiny`, never twice, never with a hard-gate flag.
Promotion stays available always. A demotion logs a one-line audit decision
naming the counts, and emits the re-lane tick.

The prohibition that mattered most: the checkpoint reads the scout's evidence
and never re-litigates it. Triage's existing rule already says that re-counting
flags to land under a threshold means you are already in `standard`; a
checkpoint that re-argues counts is that prohibition wearing a new name.

## #77 P2/P3/P4 — one wave, one reviewer, one pass

- **P2** — the merged reviewer and the orient worker dispatch together at stage
  start while the orchestrator runs the reality gate and matrix on the session
  model: the stage costs max(reviewer, matrix), not their sum. Sync point widened
  from the checker to the whole wave — Gate 3, or its bypass self-approval, never
  happens with any member outstanding.
- **P3** — one `bee-review` dispatch replaces the plan-checker/cell-reviewer
  pair, returning one report with two sections. **Both vocabularies survive**:
  BLOCKER/WARNING for structure, CRITICAL/MINOR for cells. Merging the dispatches
  never merges the finding classes — stated in the body, in the reference prompt,
  and as a red flag.
- **P4** — one shot, then at most one blocker-scoped pass. WARNING-level and
  mechanically fixable findings are orchestrator-applied directly to the cells,
  legal because cells are mutable before Gate 3. No third pass; escalation
  unchanged.

The two reviewer prompts in `validation-reference.md` were merged into one
two-mandate prompt with a single report template, so the body's pointer to "the
merged reviewer prompt" resolves. The high-risk persona panel now scales that
same merged dispatch.

Composes with `fs-2`'s delta rule rather than contradicting it: on slice 2+ the
wave's scope is the new/changed cells and stale rows, not the frozen plan.

## Verify

```
ledger_parity --check: .bee/bin/** matches the .bee/onboarding.json managed-hash ledger
PASS run_verify: 108 suite(s), concurrency=5, wall=72488ms
EXIT=0
```

No new test authored — behavior cells cap on `existing-targeted-green` since
`fs-1`, and the user asked for none.
