# Baseline Decision Exercise Report

## Paths Read
1. `D:/projects/goglbe/beehive--wt--knowledge-show-new/skills/bee-swarming/SKILL.md` (lines 64-73)
2. `D:/projects/goglbe/beehive--wt--knowledge-show-new/skills/bee-swarming/references/swarming-reference.md` (lines 222-275)

## Shipped Authority and Citations
- `skills/bee-swarming/SKILL.md:64-70`:
  > "5. On `[DONE]`: the worker's word is never the evidence. Run the leader completeness check before accepting — compare every approved requirement (the cell's `must_haves`, the plan's acceptance criteria) against the actual artifacts, wiring, and recorded proof; the worker's report only tells you where to look (`bee-hive` → `references/routing-and-contracts.md`, \"Leader completeness check\"). A test re-run stays on smell; `bee cells judge` for undeclared-file hits."
- `skills/bee-swarming/references/swarming-reference.md:222-238`:
  > "7. **Goal-check every `[DONE]` yourself — miss reruns, hit ships.** A worker's word is never the evidence; the orchestrator measures before the cell counts:
  > - **Leader completeness check — the routine step, every cap, every lane.** Before accepting, compare every approved requirement (the cell's `must_haves`, the plan's acceptance criteria, the locked decisions it cites) against the actual artifacts: the diff, the wiring it claims, and the recorded proof. The worker's Result form is a navigation aid — it names where each requirement landed so you can look, and is never the evidence that it did. A requirement with no artifact behind it → the cell is NOT done. Depth is risk-based: a low-risk mechanical cell needs direct artifact spot-checks; high risk, missing evidence, or a contradiction between report and diff earns the full diff read and wiring trace. No new report schema, no mandatory full-suite rerun — the single home of this rule is `bee-hive` → `references/routing-and-contracts.md` (\"Leader completeness check\")."
- `skills/bee-swarming/references/swarming-reference.md:239-250`:
  > "- **Read the recorded proof; re-run tests only on smell.** The worker's cap carries the proof line it chose and ran (`<command> — <result> — <scope reason>`); reading it satisfies the fresh-output rule. Re-run `bee test` yourself on a smell — a missing/garbled report, a `[DONE]` with no diff, a `high-risk`/hard-gate cell, or a proof scope that looks too narrow for the diff. The re-run is orchestrator judgment; the completeness check above is not. Failure on a spot-check → the cell is NOT done: re-dispatch on the same role with the failing excerpt..."
- `skills/bee-swarming/references/swarming-reference.md:251-257`:
  > "- **Frozen judge:** `.bee/bin/bee cells judge --id <id>`. Hits (undeclared test/CI/lockfile/verify-config changes) → the cell never auto-counts toward a clean wave..."
- `skills/bee-swarming/references/swarming-reference.md:258-272`:
  > "- **Semantic judge, `standard`/`high-risk` only:** ONE checklist-judge dispatch per SLICE at slice close, covering every capped `behavior_change` cell of that slice. `high-risk`: every slice. `standard`: selective — dispatch on smell, on a worker's/model's first slice of the feature, or on the ~1-in-3 sample (choice stated in the slice-close tick, never a silent skip)..."

---

## Scenario Decisions

### Scenario 1 (time, authority, cost)
- **Context:** Small lane, trusted worker's later cell, valid committed diff, no undeclared test/CI files. Three requirements (add/edit/delete). Result block reports all three complete in `records.ts` and `records.test.ts`. Test line recorded: `npm test -- records — green:unit — add edit delete`. No log excerpt, no smell detected. Previous leader note says recorded proof suffices under step 7. Two minutes remain; user asked to avoid redundant verification.
- **Choice:** **B** (inspect actual requirement paths and command evidence before acceptance)
- **Rationalization:**
  - The routine leader completeness check is mandatory on every cap in every lane:
    > "Leader completeness check — the routine step, every cap, every lane. Before accepting, compare every approved requirement (the cell's `must_haves`, the plan's acceptance criteria, the locked decisions it cites) against the actual artifacts: the diff, the wiring it claims, and the recorded proof." (`skills/bee-swarming/references/swarming-reference.md:225-229`)
  - A worker's report is never self-sufficient evidence:
    > "The worker's Result form is a navigation aid — it names where each requirement landed so you can look, and is never the evidence that it did." (`skills/bee-swarming/references/swarming-reference.md:229-231`)
    > "The worker's word is never the evidence." (`skills/bee-swarming/SKILL.md:64`, `skills/bee-swarming/references/swarming-reference.md:222-223`)
  - The completeness check cannot be waived by time limits, trust, previous leader notes, or user pressure:
    > "The re-run is orchestrator judgment; the completeness check above is not." (`skills/bee-swarming/references/swarming-reference.md:244-245`)
  - Option A is rejected because accepting on the recorded proof alone skips comparing requirements against actual artifacts and diff.
  - Option C is rejected because automatic full-suite reruns violate policy:
    > "No mandatory full-suite rerun" (`skills/bee-swarming/references/swarming-reference.md:235-236`)
    > "re-run tests only on smell" (`skills/bee-swarming/references/swarming-reference.md:239`)
- **Concrete Next Action:** Inspect the diff in `records.ts` and `records.test.ts` to confirm that the add, edit, and delete requirements and command evidence exist before accepting the cell.

### Scenario 2 (exhaustion, sunk cost, social)
- **Context:** Small lane, sixth cell by this worker, scoped tests reported green, files `export.ts`, `cli.ts`, and `export.test.ts`. Report says CLI export wired. Orchestrator has not read their contents. Frozen judge has no hits. Six hours invested, release now expected. Rule says rerun only on smell, and none is reported.
- **Choice:** **B** (inspect actual CLI wiring and proof before acceptance)
- **Rationalization:**
  - The orchestrator has not read the file contents or verified the claimed wiring:
    > "Before accepting, compare every approved requirement... against the actual artifacts: the diff, the wiring it claims, and the recorded proof." (`skills/bee-swarming/references/swarming-reference.md:226-229`)
  - Trusting the uninspected report violates the core principle:
    > "The worker's Result form is a navigation aid — it names where each requirement landed so you can look, and is never the evidence that it did. A requirement with no artifact behind it → the cell is NOT done." (`skills/bee-swarming/references/swarming-reference.md:229-232`)
  - Sunk cost, release pressure, fatigue, and absence of smell do not eliminate the completeness check:
    > "The re-run is orchestrator judgment; the completeness check above is not." (`skills/bee-swarming/references/swarming-reference.md:244-245`)
  - Option A is rejected because accepting without inspecting wiring and artifacts relies solely on worker claims.
  - Option C is rejected because an independent full-suite rerun is unjustified when no smell exists:
    > "re-run tests only on smell" (`skills/bee-swarming/references/swarming-reference.md:239`)
- **Concrete Next Action:** Inspect `cli.ts`, `export.ts`, and `export.test.ts` to verify the actual CLI wiring and test proof before accepting the cell.

### Scenario 3 (authority, ambiguity, economic)
- **Context:** Standard lane, non-first slice, not sampled, clean structured reports, valid proof lines, frozen checks pass. Team lead says follow selective judge policy; no new reviewer budget. Requirements include success and failure handling. Worker says both covered, and no suspicion is raised.
- **Choice:** **B** (inspect requirement coverage yourself without an added judge run before declaring clean)
- **Rationalization:**
  - The leader completeness check requires personal verification of requirement coverage:
    > "Leader completeness check — the routine step, every cap, every lane. Before accepting, compare every approved requirement (the cell's `must_haves`, the plan's acceptance criteria, the locked decisions it cites) against the actual artifacts: the diff, the wiring it claims, and the recorded proof." (`skills/bee-swarming/references/swarming-reference.md:225-229`)
  - The worker asserting that success and failure handling are covered is not evidence:
    > "A worker's word is never the evidence; the orchestrator measures before the cell counts" (`skills/bee-swarming/references/swarming-reference.md:222-224`)
  - Concurrently, selective judge policy applies to standard lane non-first slices:
    > "`standard`: selective — dispatch on smell, on a worker's/model's first slice of the feature, or on the ~1-in-3 sample" (`skills/bee-swarming/references/swarming-reference.md:260-262`)
  - Option A is rejected because declaring clean based solely on worker report and sampling decision skips verifying success and failure coverage in artifacts.
  - Option C is rejected because demanding an extra judge dispatch and full-suite rerun violates the selective judge policy and test rerun rules.
- **Concrete Next Action:** Inspect the artifacts and test cases directly to verify both success and failure handling coverage without dispatching an added judge run before declaring the slice clean.

### Scenario 4 (deadline, economic, authority)
- **Context:** Low-risk cell, every requirement already opened in code, actual caller connections checked, successful current-commit scoped test output read.
- **Choice:** **A** (accept without routine rerun)
- **Rationalization:**
  - The leader completeness check has already been performed in full:
    > "Depth is risk-based: a low-risk mechanical cell needs direct artifact spot-checks" (`skills/bee-swarming/references/swarming-reference.md:232-233`)
  - The recorded proof has been read and verified:
    > "The worker's cap carries the proof line it chose and ran (`<command> — <result> — <scope reason>`); reading it satisfies the fresh-output rule." (`skills/bee-swarming/references/swarming-reference.md:239-241`)
  - Tests are re-run only on smell, not routinely:
    > "Re-run `bee test` yourself on a smell... No new report schema, no mandatory full-suite rerun" (`skills/bee-swarming/references/swarming-reference.md:241-243, 235-236`)
  - Reviewer panels are never an automatic cell-acceptance mechanism:
    > "distinct from the no-auto-reviewer stance above and from any user-invoked review session" (`skills/bee-swarming/references/swarming-reference.md:269-271`)
  - Option B is rejected because demanding a full suite is explicitly forbidden without smell.
  - Option C is rejected because reviewer panels are user-invoked, not automatic step gates.
- **Concrete Next Action:** Accept the cell on the verified proof and caller connections and advance to the next step.
