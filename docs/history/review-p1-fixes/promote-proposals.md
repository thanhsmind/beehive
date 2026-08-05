promote proposal for work item "review-p1-fixes" (.bee/logs/scribing-runs.jsonl) — 8 capped cell(s): p1-1, p1-2, p1-3, p1-4, p2-1, p2-2, p2-3, p2-4
anchor: ledger — .bee/logs/scribing-runs.jsonl
PROPOSAL ONLY — nothing was written. Applying any section below is a human or agent decision.

(a) DELIVERY DRAFT — save as docs/knowledge/work/review-p1-fixes/delivery.md

---
type: bee.delivery
title: review-p1-fixes — delivery
description: "Delivery record proposed by bee knowledge promote for work item review-p1-fixes: 8 capped cell(s), 4 recorded deviation(s)."
timestamp: 2026-07-28
bee:
  id: review-p1-fixes-delivery
  lifecycle: active
  areas: [workflow-state, verify-pipeline]
  required_context: [.bee/logs/scribing-runs.jsonl]
  sources: [.bee/logs/scribing-runs.jsonl, .bee/cells/p1-1.json, .bee/cells/p1-2.json, .bee/cells/p1-3.json, .bee/cells/p1-4.json, .bee/cells/p2-1.json, .bee/cells/p2-2.json, .bee/cells/p2-3.json, .bee/cells/p2-4.json]
---

# review-p1-fixes — Delivery

## What shipped

- **p1-1** — F1: test_verify_cache fixture no longer leaks the parent's cache-disabling env into spawned run_verify children (CI=true went 0-passed/8-failed -> 12 passed). F2: BEE_CHECK_ONLY now disables the cache outright (never read, never written, cache=disabled printed in the summary); suites can declare extra non-import input paths/globs hashed into their cache key (seeded for skill_budget_fence, okf_instructions_fence, test_misc); broad repo scanners are explicitly opted out; and any suite naming a live repo surface it has not declared is auto-opted-out, making opt-out the default when in doubt. Suite grew 8 -> 12 cases, none weakened. (3 file(s) changed)
- **p1-2** — Added scripts/tests/test_check_filter.mjs pinning BEE_CHECK_ONLY's 5 safety properties via a throwaway fixture suite run as a child process (unfiltered byte-identical, substring/regex filtering with body-execution sentinel, regex flags, zero-match refusal); also fixed the unguarded new RegExp() at module load in checkOnlyPredicate (scripts/lib/test-fixture.mjs) to be a typed refusal instead of an uncaught SyntaxError crash. Suite green: 6 passed, 0 failed. (2 file(s) changed)
- **p1-3** — F3: added featureSwapGuardFeatureVerifyDebt (the feature-swap door for feature-verify debt, no waiver, no bypass) and widened guardFeatureVerifyDebt's phase condition from the literal 'swarming' to SCRIBING_RUN_FROM (swarming/reviewing/scribing); debt computation extracted into a shared featureVerifyDebtState so both doors read one rule. F5: resolveActiveFeatureForWorkflowsClose now returns {ok,feature|reason} instead of swallowing every failure into null — --all-but-active and --feature refuse typed and close nothing on an unresolved active feature, --id keeps its explicit exception. 9 new tests in test_bee_cli.mjs. Both owning suites green: test_bee_cli.mjs 354 passed / 0 failed, test_state.mjs 44 passed / 0 failed. (2 file(s) changed)
- **p1-4** — Restated AGENTS.md critical rule 2 (root + template) from the retired per-cell verify law to the shipped R82 feature-verify-pending default with close-door gate; both suites named in the cell verified green; no census check pinned the old wording. (2 file(s) changed)
- **p2-1** — Inverted the debt guards: the origin-phase allowlist is replaced by a derived guarded set (isDebtGuardedDeparture — every phase change asks), one shared debt core in lib/state.mjs serves all four doors, startFeature refuses over the OUTGOING feature's feature-verify and test-cell debt, and an unreadable cell store is a refusal naming the store instead of a silent 'no debt'. Reviewer's repro reproduced red before the fix (state set --phase compounding-complete exit 0 with alpha-1 still pending, then state start-feature exit 0) and refused after (exit 1 at the first door, naming alpha-1 and the runnable FIX). Findings F2 (scribing-run from reviewing/scribing over an open/red test cell) and F3-class (unreadable store) likewise flipped from exit 0 to exit 1. No bypass level or waiver lifts any of it; every existing refusal and FIX text preserved. Both allowed suites green: test_bee_cli.mjs 377 passed/0 failed, test_state.mjs 44 passed/0 failed. (4 file(s) changed)
- **p2-2** — Verify-cache caching inverted to opt-IN by declaration: a suite is cacheable only when its entry file is declared in the new scripts/verify-cache-inputs.json (import closure + declared runtime inputs); every undeclared suite is never read from or written to the cache. Removed the source-token heuristic (UNDECLARED_INPUT_TOKENS/mentionsUndeclaredInput) and the CACHE_OPT_OUT set. The declaration table is read fail-CLOSED (missing/corrupt/wrong-shape => nothing cacheable). Run summary now prints 'cacheable=<n>/<total> declared (<n> uncached)'; live split is 6/115 suite entries. Four new cases in test_verify_cache (16 total, all green): an undeclared suite is never cached; a declared suite is invalidated by editing an input it loads only at runtime; the summary reports the split; the table fails closed. (4 file(s) changed)
- **p2-3** — Re-anchored the owner-declaration census in test_cli_state.mjs (not test_misc.mjs — the cell's file/verify fields pointed at the wrong file; the check and its byte-identical repro live in test_cli_state.mjs, confirmed before editing). Replaced the hardcoded 4-file skill list + node-prefix-required regex with a dynamic scan of skills/*/SKILL.md + skills/*/references/*.md, node-prefix optional, requiring --owner in the same match. Excludes hand-write teaching examples and placeholder-owner syntax lines. Expected count updated 5->4 (actual shipped calls: bee-exploring, bee-compounding, bee-validating body, bee-validating reference). Mutation-proved: removed --owner from bee-compounding's call, count assertion went red (4->3), reverted byte-identical. (1 file(s) changed)
- **p2-4** — Added six exports (FEATURE_VERIFY_FIX_TAIL, featureVerifyDebt, isDebtGuardedDeparture, readFeatureCellsStrict, testCellDebt, testCellDebtFixTail) to EXPECTED_STATE_EXPORTS allowlist in alphabetical order. Test now passes: 118 passed, 0 failed (1 file(s) changed)

## Verify

Each cell below was capped only against a recorded passing verify result — bee refuses a cap without one.

- **p1-1** — `CI=true node scripts/tests/test_verify_cache.mjs && node scripts/tests/test_verify_cache.mjs && node scripts/tests/test_run_verify_impacted.mjs`
- **p1-2** — `node scripts/tests/test_check_filter.mjs`
- **p1-3** — `node packages/bee/tests/test_bee_cli.mjs && node packages/bee/tests/test_state.mjs`
- **p1-4** — `node scripts/tests/test_agents_budget.mjs && node packages/bee/tests/test_misc.mjs`
- **p2-1** — `node packages/bee/tests/test_bee_cli.mjs && node packages/bee/tests/test_state.mjs`
- **p2-2** — `CI=true node scripts/tests/test_verify_cache.mjs && node scripts/tests/test_verify_cache.mjs && node scripts/tests/test_check_filter.mjs && node scripts/tests/test_run_verify_impacted.mjs`
- **p2-3** — `node packages/bee/tests/test_misc.mjs`
- **p2-4** — `node packages/bee/tests/test_misc.mjs`

## Deviations

- **p2-1** — Inverted packages/bee/tests/test_cli_state.mjs:3362 ("slice-tail P4: the guard is scoped to the exit from swarming — it never fires on a feature that is not in swarming"). That check asserted, as intended behavior, the exact escape this cell was dispatched to remove: an open test cell letting a planning->validating departure through. Under the derived guarded set it must refuse. Rewritten to assert the refusal (naming the open cell, phase untouched) plus the one surviving exemption, a literal no-op re-set. The file was reserved before editing; it is outside the cell's `files` list, so it is recorded here rather than assumed. Scenario verified by hand through the CLI (refused exit 1 naming p4-test-elsewhere; no-op re-set exit 0); the suite itself was NOT run, per the cell's "do not run other suites" prohibition — MAIN's feature-level verify covers it.
- **p2-2** — Added a third source file, scripts/verify-cache-inputs.json (cell listed two): the declaration table has to be repo DATA, not a constant inside run_verify.mjs, so the fixture mini-repos in test_verify_cache can declare their own fake suites without test paths leaking into production code.
- **p2-2** — Left scripts/tests/test_workflow_step_paths.mjs and packages/bee/tests/test_herding.mjs UNDECLARED (=> uncached) rather than declaring them. test_workflow_step_paths asserts the EXISTENCE of every script path named in .github/workflows/*.yml, so its true input set includes deletions anywhere in the repo and cannot be honestly declared. test_herding lives under packages/bee/** which the concurrent sibling cell owns, so it was neither read nor declared. The must-have permits "declared with their real inputs OR uncached".
- **p2-2** — Declared scripts/tests/test_check_filter.mjs with ["scripts/lib/**", "packages/bee/lib/**"]: it dynamic-loads scripts/lib/test-fixture.mjs at runtime, which in turn imports packages/bee/lib/fsutil.mjs. Deliberately coarse — over-declaring only costs extra re-runs, under-declaring costs a false green.

## Provenance

Proposed by `bee knowledge promote --work review-p1-fixes` from 8 capped cell trace(s) in `.bee/cells/` and the anchor `.bee/logs/scribing-runs.jsonl`. Every line above is copied from a trace or from the work item; nothing here is curated truth until a human or agent accepts it.

(b) AREA UPDATES — candidate spec-sync bullets, each citing its cell

areas: from the scribing stamp for "review-p1-fixes" — .bee/logs/scribing-runs.jsonl's most recent entry (2026-07-28T10:58:34.736Z), the work item declares no bee.areas.

area workflow-state:
  - [p1-1] F1: test_verify_cache fixture no longer leaks the parent's cache-disabling env into spawned run_verify children (CI=true went 0-passed/8-failed -> 12 passed). F2: BEE_CHECK_ONLY now disables the cache outright (never read, never written, cache=disabled printed in the summary); suites can declare extra non-import input paths/globs hashed into their cache key (seeded for skill_budget_fence, okf_instructions_fence, test_misc); broad repo scanners are explicitly opted out; and any suite naming a live repo surface it has not declared is auto-opted-out, making opt-out the default when in doubt. Suite grew 8 -> 12 cases, none weakened. — feature-wide sync per the scribing stamp, 3 file(s) changed (trace .bee/cells/p1-1.json)
  - [p1-3] F3: added featureSwapGuardFeatureVerifyDebt (the feature-swap door for feature-verify debt, no waiver, no bypass) and widened guardFeatureVerifyDebt's phase condition from the literal 'swarming' to SCRIBING_RUN_FROM (swarming/reviewing/scribing); debt computation extracted into a shared featureVerifyDebtState so both doors read one rule. F5: resolveActiveFeatureForWorkflowsClose now returns {ok,feature|reason} instead of swallowing every failure into null — --all-but-active and --feature refuse typed and close nothing on an unresolved active feature, --id keeps its explicit exception. 9 new tests in test_bee_cli.mjs. Both owning suites green: test_bee_cli.mjs 354 passed / 0 failed, test_state.mjs 44 passed / 0 failed. — feature-wide sync per the scribing stamp, 2 file(s) changed (trace .bee/cells/p1-3.json)
  - [p2-1] Inverted the debt guards: the origin-phase allowlist is replaced by a derived guarded set (isDebtGuardedDeparture — every phase change asks), one shared debt core in lib/state.mjs serves all four doors, startFeature refuses over the OUTGOING feature's feature-verify and test-cell debt, and an unreadable cell store is a refusal naming the store instead of a silent 'no debt'. Reviewer's repro reproduced red before the fix (state set --phase compounding-complete exit 0 with alpha-1 still pending, then state start-feature exit 0) and refused after (exit 1 at the first door, naming alpha-1 and the runnable FIX). Findings F2 (scribing-run from reviewing/scribing over an open/red test cell) and F3-class (unreadable store) likewise flipped from exit 0 to exit 1. No bypass level or waiver lifts any of it; every existing refusal and FIX text preserved. Both allowed suites green: test_bee_cli.mjs 377 passed/0 failed, test_state.mjs 44 passed/0 failed. — feature-wide sync per the scribing stamp, 4 file(s) changed (trace .bee/cells/p2-1.json)
  - [p2-2] Verify-cache caching inverted to opt-IN by declaration: a suite is cacheable only when its entry file is declared in the new scripts/verify-cache-inputs.json (import closure + declared runtime inputs); every undeclared suite is never read from or written to the cache. Removed the source-token heuristic (UNDECLARED_INPUT_TOKENS/mentionsUndeclaredInput) and the CACHE_OPT_OUT set. The declaration table is read fail-CLOSED (missing/corrupt/wrong-shape => nothing cacheable). Run summary now prints 'cacheable=<n>/<total> declared (<n> uncached)'; live split is 6/115 suite entries. Four new cases in test_verify_cache (16 total, all green): an undeclared suite is never cached; a declared suite is invalidated by editing an input it loads only at runtime; the summary reports the split; the table fails closed. — feature-wide sync per the scribing stamp, 4 file(s) changed (trace .bee/cells/p2-2.json)

area verify-pipeline:
  - [p1-1] F1: test_verify_cache fixture no longer leaks the parent's cache-disabling env into spawned run_verify children (CI=true went 0-passed/8-failed -> 12 passed). F2: BEE_CHECK_ONLY now disables the cache outright (never read, never written, cache=disabled printed in the summary); suites can declare extra non-import input paths/globs hashed into their cache key (seeded for skill_budget_fence, okf_instructions_fence, test_misc); broad repo scanners are explicitly opted out; and any suite naming a live repo surface it has not declared is auto-opted-out, making opt-out the default when in doubt. Suite grew 8 -> 12 cases, none weakened. — feature-wide sync per the scribing stamp, 3 file(s) changed (trace .bee/cells/p1-1.json)
  - [p1-3] F3: added featureSwapGuardFeatureVerifyDebt (the feature-swap door for feature-verify debt, no waiver, no bypass) and widened guardFeatureVerifyDebt's phase condition from the literal 'swarming' to SCRIBING_RUN_FROM (swarming/reviewing/scribing); debt computation extracted into a shared featureVerifyDebtState so both doors read one rule. F5: resolveActiveFeatureForWorkflowsClose now returns {ok,feature|reason} instead of swallowing every failure into null — --all-but-active and --feature refuse typed and close nothing on an unresolved active feature, --id keeps its explicit exception. 9 new tests in test_bee_cli.mjs. Both owning suites green: test_bee_cli.mjs 354 passed / 0 failed, test_state.mjs 44 passed / 0 failed. — feature-wide sync per the scribing stamp, 2 file(s) changed (trace .bee/cells/p1-3.json)
  - [p2-1] Inverted the debt guards: the origin-phase allowlist is replaced by a derived guarded set (isDebtGuardedDeparture — every phase change asks), one shared debt core in lib/state.mjs serves all four doors, startFeature refuses over the OUTGOING feature's feature-verify and test-cell debt, and an unreadable cell store is a refusal naming the store instead of a silent 'no debt'. Reviewer's repro reproduced red before the fix (state set --phase compounding-complete exit 0 with alpha-1 still pending, then state start-feature exit 0) and refused after (exit 1 at the first door, naming alpha-1 and the runnable FIX). Findings F2 (scribing-run from reviewing/scribing over an open/red test cell) and F3-class (unreadable store) likewise flipped from exit 0 to exit 1. No bypass level or waiver lifts any of it; every existing refusal and FIX text preserved. Both allowed suites green: test_bee_cli.mjs 377 passed/0 failed, test_state.mjs 44 passed/0 failed. — feature-wide sync per the scribing stamp, 4 file(s) changed (trace .bee/cells/p2-1.json)
  - [p2-2] Verify-cache caching inverted to opt-IN by declaration: a suite is cacheable only when its entry file is declared in the new scripts/verify-cache-inputs.json (import closure + declared runtime inputs); every undeclared suite is never read from or written to the cache. Removed the source-token heuristic (UNDECLARED_INPUT_TOKENS/mentionsUndeclaredInput) and the CACHE_OPT_OUT set. The declaration table is read fail-CLOSED (missing/corrupt/wrong-shape => nothing cacheable). Run summary now prints 'cacheable=<n>/<total> declared (<n> uncached)'; live split is 6/115 suite entries. Four new cases in test_verify_cache (16 total, all green): an undeclared suite is never cached; a declared suite is invalidated by editing an input it loads only at runtime; the summary reports the split; the table fails closed. — feature-wide sync per the scribing stamp, 4 file(s) changed (trace .bee/cells/p2-2.json)

(c) PATTERN CANDIDATES — candidate bee.pattern concepts, bee.polarity pitfall

from cell p2-1 — save as docs/knowledge/patterns/review-p1-fixes-p2-1-pitfall.md

---
type: bee.pattern
title: review-p1-fixes cell p2-1 — pitfall candidate
description: "Pitfall candidate mined from cell p2-1's capped trace: Inverted packages/bee/tests/test_cli_state.mjs:3362 (\"slice-tail P4: the guard is scoped to the exit from swarming — it never fires on a feature that is not in…"
timestamp: 2026-07-28
bee:
  id: review-p1-fixes-p2-1-pitfall
  lifecycle: draft
  areas: [workflow-state, verify-pipeline]
  sources: [.bee/cells/p2-1.json]
  polarity: pitfall
---

# review-p1-fixes cell p2-1 — pitfall candidate

## What the cell did

Inverted the debt guards: the origin-phase allowlist is replaced by a derived guarded set (isDebtGuardedDeparture — every phase change asks), one shared debt core in lib/state.mjs serves all four doors, startFeature refuses over the OUTGOING feature's feature-verify and test-cell debt, and an unreadable cell store is a refusal naming the store instead of a silent 'no debt'. Reviewer's repro reproduced red before the fix (state set --phase compounding-complete exit 0 with alpha-1 still pending, then state start-feature exit 0) and refused after (exit 1 at the first door, naming alpha-1 and the runnable FIX). Findings F2 (scribing-run from reviewing/scribing over an open/red test cell) and F3-class (unreadable store) likewise flipped from exit 0 to exit 1. No bypass level or waiver lifts any of it; every existing refusal and FIX text preserved. Both allowed suites green: test_bee_cli.mjs 377 passed/0 failed, test_state.mjs 44 passed/0 failed.

## Recorded evidence (verbatim from .bee/cells/p2-1.json)

- **deviation** — Inverted packages/bee/tests/test_cli_state.mjs:3362 ("slice-tail P4: the guard is scoped to the exit from swarming — it never fires on a feature that is not in swarming"). That check asserted, as intended behavior, the exact escape this cell was dispatched to remove: an open test cell letting a planning->validating departure through. Under the derived guarded set it must refuse. Rewritten to assert the refusal (naming the open cell, phase untouched) plus the one surviving exemption, a literal no-op re-set. The file was reserved before editing; it is outside the cell's `files` list, so it is recorded here rather than assumed. Scenario verified by hand through the CLI (refused exit 1 naming p4-test-elsewhere; no-op re-set exit 0); the suite itself was NOT run, per the cell's "do not run other suites" prohibition — MAIN's feature-level verify covers it.

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

from cell p2-2 — save as docs/knowledge/patterns/review-p1-fixes-p2-2-pitfall.md

---
type: bee.pattern
title: review-p1-fixes cell p2-2 — pitfall candidate
description: "Pitfall candidate mined from cell p2-2's capped trace: Added a third source file, scripts/verify-cache-inputs.json (cell listed two): the declaration table has to be repo DATA, not a constant inside run_verify.mjs,…"
timestamp: 2026-07-28
bee:
  id: review-p1-fixes-p2-2-pitfall
  lifecycle: draft
  areas: [workflow-state, verify-pipeline]
  sources: [.bee/cells/p2-2.json]
  polarity: pitfall
---

# review-p1-fixes cell p2-2 — pitfall candidate

## What the cell did

Verify-cache caching inverted to opt-IN by declaration: a suite is cacheable only when its entry file is declared in the new scripts/verify-cache-inputs.json (import closure + declared runtime inputs); every undeclared suite is never read from or written to the cache. Removed the source-token heuristic (UNDECLARED_INPUT_TOKENS/mentionsUndeclaredInput) and the CACHE_OPT_OUT set. The declaration table is read fail-CLOSED (missing/corrupt/wrong-shape => nothing cacheable). Run summary now prints 'cacheable=<n>/<total> declared (<n> uncached)'; live split is 6/115 suite entries. Four new cases in test_verify_cache (16 total, all green): an undeclared suite is never cached; a declared suite is invalidated by editing an input it loads only at runtime; the summary reports the split; the table fails closed.

## Recorded evidence (verbatim from .bee/cells/p2-2.json)

- **deviation** — Added a third source file, scripts/verify-cache-inputs.json (cell listed two): the declaration table has to be repo DATA, not a constant inside run_verify.mjs, so the fixture mini-repos in test_verify_cache can declare their own fake suites without test paths leaking into production code.
- **deviation** — Left scripts/tests/test_workflow_step_paths.mjs and packages/bee/tests/test_herding.mjs UNDECLARED (=> uncached) rather than declaring them. test_workflow_step_paths asserts the EXISTENCE of every script path named in .github/workflows/*.yml, so its true input set includes deletions anywhere in the repo and cannot be honestly declared. test_herding lives under packages/bee/** which the concurrent sibling cell owns, so it was neither read nor declared. The must-have permits "declared with their real inputs OR uncached".
- **deviation** — Declared scripts/tests/test_check_filter.mjs with ["scripts/lib/**", "packages/bee/lib/**"]: it dynamic-loads scripts/lib/test-fixture.mjs at runtime, which in turn imports packages/bee/lib/fsutil.mjs. Deliberately coarse — over-declaring only costs extra re-runs, under-declaring costs a false green.

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

knowledge promote: 8 capped cell(s) mined, 1 delivery draft, 8 area bullet(s), 2 pattern candidate(s), 0 file(s) written.