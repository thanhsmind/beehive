---
name: bee-formal-upkeep
description: >-
  Reconcile a project-local formal verification skill with changed code: inspect commits since the sync commit, update the model or harness, rerun, and separate drift from bugs. Use when covered code changed without rechecking. Not for creating a new check (bee-formal-verifying).
disable-model-invocation: true
metadata:
  version: '0.1'
  ecosystem: bee
  dependencies:
    bee-cli:
      kind: command
      command: .bee/bin/bee
      missing_effect: degraded
      reason: Updates are made to the source under `.bee/verify/` and re-rendered into every runtime skill home by the vendored bee binary. Absent, the update lands in source but the runtime copies keep the stale wording.
---

# Formal upkeep — keep formal checks true to the code

A formal check only speaks for the model or harness it executes. When covered implementation files evolve while the model stays static, green runs describe stale behavior. Use this skill to reconcile project-local formal checks after merges, refactors, or edits that bypassed verification.

## Outcomes

Assign each check one outcome (a check can be both `updated` and `bug` when updating the model exposes a real code defect):

- **in sync** — commits since the sync commit leave the modeled behavior intact, and the check passes.
- **updated** — covered code shifted, the model or harness now reflects the new behavior, and the check passes while still catching a broken variant.
- **bug** — the model or harness faithfully reflects the implementation, and the code violates the invariant; treat this as a product defect, not model drift.
- **blocked** — reconciliation failed or the check could not execute; name the exact obstacle.

## Edit scope

Restrict edits to verification artifacts: models, harnesses, property configs, and the skill source under `.bee/verify/<name>/`. After changing any source file, run `bee onboard --verify-only --apply` to refresh every runtime skill home; never edit rendered copies directly.

Do not modify product code during upkeep unless explicitly instructed. Never relax an invariant or model constraint to turn a red check green; report the counterexample.

## Pass

1. **Locate existing checks.** Scan `.bee/verify/` for project-local skills that invoke a model checker, prover, or verifier on named source files (see the `formal-cell-lifecycle` verify skill in the bee repo itself as the worked example). Gather each skill's invariants, covered files and functions, bounds, command, and sync commit. If none exist, stop and direct the user to `bee-formal-verifying`. When a skill lacks a recorded sync commit, fall back to the most recent commit that touched its verification files and note that fallback in the report.

2. **Baseline run.** Execute each check as found before editing anything and log the outcome. A green baseline only confirms the prior model still holds; a red baseline may indicate either drift or a defect, which step 5 will separate.

3. **Audit covered-code changes.** Inspect the git diff of covered files from the sync commit to `HEAD` along with commit messages. Look for renamed, split, or relocated functions and for new call sites reading or writing the same guarded state. Classify each change:
   - **No effect:** formatting, comments, or paths outside the check's coverage (verify against the model or harness, not just the commit message).
   - **Verification drift:** an indivisible step split or merged, a branch changed, a guard moved, a new path reached guarded state, or value ranges changed.
   - **Intentional promise change:** evidence in tests, docs, or commits shows the code's promise changed on purpose.
   - **Possible broken promise:** code changed without evidence of an intentional promise change; treat any resulting failure as a bug candidate.

4. **Update from the code, not from the result.** Update models and harnesses to match the current code before rerunning, keeping one indivisible code step as one model step. Do not tune the model to force a pass. Change a property only when the promise changed on purpose, recording the old wording, new wording, and evidence. Keep existing bounds unless the code change invalidates them; if a bound must shrink, state why in the report.

5. **Rerun and judge.**
   - **Pass:** prove the updated check can still fail by testing a broken variant or temporary scratch break (for example, removing a guard or reordering a step), watching it fail, and dropping the break.
   - **Fail:** trace every counterexample step against the live implementation (using `bee-formal-verifying/references/statechart.md` for TLA+ traces). When a step is impossible in code, repair the model and log the discrepancy. When every step is reachable in code, file a `bug` citing the introducing commit, the plain-language trace, and a one-sentence harm statement (include a scratch test or live repro if inexpensive, without committing it to product code).

6. **Update the verify skill.** Update the coverage table, covered files/functions, commands, bounds, and assumptions in `.bee/verify/<name>/SKILL.md`, and advance the sync commit to the verified commit (never advance the sync commit on a `blocked` check). Run `bee onboard --verify-only --apply` and execute the verify skill once as a cold agent would.

## Report

For each check, report:
- the outcome (`in sync`, `updated`, `bug`, `blocked`, or `updated` + `bug`) and the commits reviewed;
- each verification change and its reason (code shape change or intentional promise change with evidence);
- the post-update run and the broken variant or scratch break that proved failure detection;
- for every bug, a plain-language trace, one-sentence harm, and the introducing commit;
- what remains uncovered, including new code that should be covered.

Commit verification changes separately from any product bug fix.
