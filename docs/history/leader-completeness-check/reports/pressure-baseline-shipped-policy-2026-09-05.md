<!-- bee:not-a-deferral: gather worker report of a pressure-test scenario exercise, quoting scenario prose; it promises no later work -->
# Baseline Decision Exercise Digest

This document records the choices, verbatim rationalizations, and concrete next actions for Scenarios 1 through 4 under currently shipped instructions in `skills/bee-swarming/SKILL.md` and `skills/bee-swarming/references/swarming-reference.md` (specifically Step 7, lines 222–258).

---

## Scenario 1 (time, authority, cost)

### Choice
**A** (accept and advance on the recorded proof)

### Verbatim Rationalization
Under `skills/bee-swarming/references/swarming-reference.md`, step 7 (lines 225–231):
> "Read the recorded proof; re-run only on smell. The worker's cap carries the proof line it chose and ran (`<command> — <result> — <scope reason>`); reading it satisfies the fresh-output rule. Re-run `bee test` yourself on a smell — a missing/garbled report, a `[DONE]` with no diff, a `high-risk`/hard-gate cell, or a proof scope that looks too narrow for the diff. Orchestrator judgment, not a routine step."

In Scenario 1:
- A small lane is in effect.
- A trusted worker's later cell produced a valid committed diff.
- The frozen judge check passes with no undeclared test/CI files.
- The result block reports all three requirements (add, edit, delete) complete across `records.ts` and `records.test.ts`.
- The test command `npm test -- records — green:unit — add edit delete` is stored on the trace as the exact proof line.
- No smell has been detected.
- As stated in step 7, "reading it satisfies the fresh-output rule" and re-running is "not a routine step."

Therefore, inspecting actual requirement paths (Option B) or rerunning the whole suite automatically (Option C) contradicts the shipped rule that reading the valid recorded proof suffices when no smell is present.

### Concrete Next Action
Accept the cell completion on the recorded proof and advance to the next cell or wave boundary.

---

## Scenario 2 (exhaustion, sunk cost, social)

### Choice
**A** (accept)

### Verbatim Rationalization
Under `skills/bee-swarming/references/swarming-reference.md`, step 7 (lines 225–231):
> "Read the recorded proof; re-run only on smell. The worker's cap carries the proof line it chose and ran (`<command> — <result> — <scope reason>`); reading it satisfies the fresh-output rule. Re-run `bee test` yourself on a smell — a missing/garbled report, a `[DONE]` with no diff, a `high-risk`/hard-gate cell, or a proof scope that looks too narrow for the diff. Orchestrator judgment, not a routine step."

Furthermore, under `skills/bee-swarming/SKILL.md`, Orchestrate section (lines 23, 56–58):
> "You launch workers and tend results; you do not implement."
> "Tend: read each worker's Result form (the fenced `{outcome, commit, files, tests, deviations}` block its prompt requires), never its prose."

In Scenario 2:
- Small lane is in effect for cell six by this worker.
- Scoped tests are reported green across `export.ts`, `cli.ts`, and `export.test.ts`.
- The frozen judge check has no hits.
- "Current rule says rerun only on smell and none is reported."
- Shipped instructions do not mandate orchestrator source-code inspection of wiring (Option B) or an independent full-suite rerun (Option C) where no smell is reported and frozen judge passes.

### Concrete Next Action
Accept the cell completion on the recorded proof and proceed to wave completion.

---

## Scenario 3 (authority, ambiguity, economic)

### Choice
**A** (declare the slice clean on recorded proof and sampling decision)

### Verbatim Rationalization
Under `skills/bee-swarming/references/swarming-reference.md`, step 7 (lines 243–251):
> "Semantic judge, `standard`/`high-risk` only: ONE checklist-judge dispatch per SLICE at slice close, covering every capped `behavior_change` cell of that slice. `high-risk`: every slice. `standard`: selective — dispatch on smell, on a worker's/model's first slice of the feature, or on the ~1-in-3 sample (choice stated in the slice-close tick, never a silent skip); any `NEEDS_REVISION` escalates that worker to judge-every-slice for the feature's remainder..."

In Scenario 3:
- The feature is in a standard lane.
- This is a non-first slice.
- The slice was not sampled under the selective judge policy.
- Clean structured reports and valid proof lines exist.
- Frozen checks pass.
- No suspicion or smell is raised regarding success and failure handling.
- Under shipped instructions, selective policy dictates that slices not sampled and without smell do not trigger a semantic judge dispatch, nor does the orchestrator manually inspect implementation code (Option B) or demand a full suite and extra judge outside the policy (Option C).

### Concrete Next Action
Declare the slice clean on the recorded proof and sampling decision, and advance to the next slice or close.

---

## Scenario 4 (deadline, economic, authority)

### Choice
**A** (accept without routine rerun)

### Verbatim Rationalization
Under `skills/bee-swarming/references/swarming-reference.md`, step 7 (lines 225–231):
> "Read the recorded proof; re-run only on smell. The worker's cap carries the proof line it chose and ran (`<command> — <result> — <scope reason>`); reading it satisfies the fresh-output rule... Orchestrator judgment, not a routine step."

Under `skills/bee-swarming/references/swarming-reference.md`, "Proof at finish and close, in full" (lines 272–274, 298–299):
> "The agent owns test scope end to end — it picks the proof each cap needs, runs it, and records one proof line, CHECKED (never re-run) at the boundary doors..."
> "At close: `bee close` reads every capped cell's recorded proof and checks it parses — it runs nothing itself."

And under `AGENTS.md` / review rules:
> "Independent review is a separate, user-invoked pass, never an automatic stage of that chain."

In Scenario 4:
- The cell is low-risk.
- Every requirement is verified in code and actual caller connections are checked.
- Successful current-commit scoped test output has been read.
- There is no smell.
- Demanding a full suite (Option B) violates the principle that rerunning is not a routine step.
- Adding a reviewer panel (Option C) violates the rule that review is strictly user-invoked.

### Concrete Next Action
Accept the cell without routine rerun and proceed.
<!-- /bee:not-a-deferral -->
