---
name: bee-formal-upkeep
description: >-
  Bring a project-local formal verification skill back in line after covered code changes: diff since the sync commit, update the model or harness, rerun, and separate drift from bugs. Use when covered code changed without rechecking. Not for creating a new check (bee-formal-verifying).
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

A formal check only proves a property of the model or harness it runs against. When covered code changes and the verification does not, a passing check can describe code that no longer exists. This skill brings project-local formal checks back in line after merges, refactors, or edits that bypassed the verify skill.

## Outcomes

Assign each check one outcome (a check can be both `updated` and `bug` when updating the model exposes a real code defect):

- **in sync** — no change since the sync commit alters what the check describes, and the check passes.
- **updated** — covered code changed, the verification now describes the new code, and the check passes (and still fails on a broken variant).
- **bug** — the verification accurately describes the current code, and the code breaks the property. This is a code finding, not drift.
- **blocked** — the check could not be brought back in line or could not run. State the exact blocker.

## Edit scope

Edit only the verification: models, harnesses, property files, and the verify skill source under `.bee/verify/<name>/`. After any source edit, run `bee onboard --verify-only --apply` so every runtime skill home receives the update; never hand-edit rendered copies.

Never edit product code during an upkeep run unless the user explicitly asks. Never weaken a property or model to make a failing check pass; report the failure.

## Pass

1. **Find the checks.** Inspect `.bee/verify/` for project-local skills that run a model checker, prover, or verifier against named code (see `.bee/verify/formal-cell-lifecycle` as the worked example). For each check, collect the properties, covered files and functions, bounds and assumptions, run command, and sync commit. If no formal verify skill exists, stop and point to `bee-formal-verifying`. If no sync commit is recorded, use the last commit that modified the verification files and state that in the report.

2. **Run as-is.** Run each check once before changing anything and record the result. A pass here only means the old description still holds; a failure may be verification drift or a code bug. Do not classify until step 5.

3. **Read every change to covered code.** Diff the covered files from the sync commit to `HEAD` and read each commit message. Also check for covered functions that were renamed, split, or moved to other files, and for new code that writes or reads the same guarded state. Classify each change:
   - **No effect:** formatting, comments, or paths outside the check's coverage (verify against the model or harness, not just the commit message).
   - **Verification drift:** an indivisible step split or merged, a branch changed, a guard moved, a new path reached guarded state, or value ranges changed.
   - **Intentional promise change:** evidence in tests, docs, or commits shows the code's promise changed on purpose.
   - **Possible broken promise:** code changed without evidence of an intentional promise change; treat any resulting failure as a bug candidate.

4. **Update from the code, not from the result.** Update models and harnesses to match the current code before rerunning, keeping one indivisible code step as one model step. Do not tune the model to force a pass. Change a property only when the promise changed on purpose, recording the old wording, new wording, and evidence. Keep existing bounds unless the code change invalidates them; if a bound must shrink, state why in the report.

5. **Rerun and judge.**
   - **Pass:** prove the updated check can still fail by testing a broken variant or temporary scratch break (for example, removing a guard or reordering a step), watching it fail, and dropping the break.
   - **Fail:** replay the counterexample step by step against the real code (using `skills/bee-formal-verifying/references/statechart.md` for TLA+ traces). If any step cannot happen in code, fix the verification and record what was wrong. If every step can happen, report it as a `bug` with the introducing commit, the plain-word step sequence, and the harm in one sentence (attach a scratch test or live repro as evidence if cheap, without committing it to product code).

6. **Update the verify skill.** Update the coverage table, covered files/functions, commands, bounds, and assumptions in `.bee/verify/<name>/SKILL.md`, and advance the sync commit to the verified commit (never advance the sync commit on a `blocked` check). Run `bee onboard --verify-only --apply` and execute the verify skill once as a cold agent would.

## Report

For each check, report:
- the outcome (`in sync`, `updated`, `bug`, `blocked`, or `updated` + `bug`) and the commits reviewed;
- each verification change and its reason (code shape change or intentional promise change with evidence);
- the rerun result and the broken variant or scratch break that proved the check can still fail;
- each bug in plain words: counterexample steps, one-sentence harm, and introducing commit;
- what remains uncovered, including new code that should be covered.

Keep verification updates in their own change, separate from any product fix.
