# Worker Conformance And Evidence Diet — Context

**Feature slug:** worker-conformance
**Date:** 2026-07-29
**Exploring session:** complete
**Scope:** Standard
**Domain types:** CALL (the `bee.mjs cells cap` refusal contract) · READ (bee-executing / bee-planning instruction text)

## Feature Boundary

Stop asking a worker to *produce* evidence in order to cap a cell, keep the
build-emitted proof that already exists, and turn the slice's trailing test cell
from "author tests" into "judge whether more tests are needed, then author only
the gap." Ends at the instruction text in `skills/bee-executing` and
`skills/bee-planning` plus the matching refusal branches in
`packages/bee/lib/cells.mjs`. It does not touch feature-verify, the close-door
gate, `commands.test`/`commands.verify` scope, or the review pipeline.

## Locked Decisions

These are fixed. Planning must implement them exactly — cited, never reinterpreted.
Changing one requires the user, a new D-ID or an explicit supersession note, never
a silent edit.

| ID | Decision | Rationale (only if it changes implementation) |
|----|----------|-----------------------------------------------|
| D1 | Per-cell evidence stops being a cap precondition for every tier **except** red-first. Exactly two throws become non-blocking: the `behavior_change: true` door (`cells.mjs:1956-1960`) and decision 0004's "no recorded proof / an assertion is not evidence" door (`cells.mjs:2164-2168`). The cap succeeds, the absence is recorded on the cell, no refusal. **The non-empty `files_changed` door in the same block (`cells.mjs:2169-2173`) is NOT in scope and stays refusing** — it asks what the worker touched, not for authored proof. | The doors that made a worker overshoot are the ones asking it to *write* evidence to pass a gate. Producing evidence text is authoring work, and authoring work drifts. A file list is not authored proof. |
| D2 | The red-first tier is untouched: `security`/`migration` (all lanes) and lane `high-risk` (all classes) still refuse a cap without `red_failure_evidence` (≥80 chars, non-duplicate). Test-economy D2's scoping and validation-diet D9's definition both survive verbatim. | Red-first evidence is emitted by a real red run at the file's real ship path — it is build output, not authored prose, so it does not cause the drift D1 removes. Dropping it would remove the only red-before-green proof on the two riskiest classes. |
| D3 | Feature-boundary proof stays the sole blocking proof and is unchanged. main-verifies D3's close-door — leaving `swarming` or running `scribing-run` is refused while any capped cell carries a pending record and no fresh green feature-verify record exists, with **no `gate_bypass` level lifting it** — is load-bearing and must not be relaxed by this feature. `commands.test` stays impacted-only and `commands.verify` stays full and CI-owned (ci-owned-verify D1/D4). | D1 is only safe because this door still forces one real green run per feature. Removing per-cell evidence without it would leave nothing mandatory. |
| D4 | The trailing test cell per slice stays an unconditional planning artifact — "a code-touching slice with no test cell is a planning defect" survives — but its **first mandated step is a coverage judgment, not authoring**: cite the nearest existing tests by `file:line` (test-economy D5 read-first) and state whether they already cover the slice's acceptance criteria. Covered → the cell caps by running those tests green and recording "already covered, no new rows". Partly covered → it authors only the uncovered gap. "A test cell that authors no test" is explicitly **not** a defect. | Makes "do we need more tests?" the required thought instead of making "write tests" the required output. |
| D5 | Test shape at lane `standard` and below is the triad: **happy path, edge cases, error paths** — the smallest set that demonstrates each. `skills/bee-planning/references/edge-dimensions.md` (12 numbered prose sections, not a table) stops being the default checklist at `standard` and applies only at `high-risk` / hard-gate work. | The 12-dimension matrix read as a checklist to fill, which is a volume generator at standard lane. |
| D6 | No numeric per-group test cap is added. The existing brakes stay exactly as written: ratio ceiling (warn >3 tiny/small, refuse >4 standard/high-risk without an audited `ratio_waiver`), `new_suite_reason` ≥20 chars for a genuinely new suite file, and the unconditional refusal of a new test file on a `refactor`/`formatting` cell. | The triad is the shape guide; the ratio ceiling is the volume brake. Two brakes on the same axis would contradict. |
| D7 | Every loosening in D1 ships in the **same cell** as a table-driven test proving both halves: the loosened path now caps, **and** the red-first tier plus the `refactor`+new-test-file refusal still refuse. Non-negotiable, per test-economy D8 / knowledge R55-R57. | A guard loosened without a negative control is a guard nobody can prove still exists. |
| D8 | The three worker-conformance additions from the original routing ride along unchanged: (a) a pre-code conformance checklist at the worker — read routed docs, scout adjacent patterns, check existing helpers, verify interface contracts, cross-check the declared file inventory; (b) three cheap post-edit checks per file — compile/type, pattern match, import/cycle; (c) the coverage-first step, which is D4. | These are the cheap conformance work that replaces the deleted evidence work — the point is to move worker effort from proving to conforming, not to remove effort. |
| D10 | **A cap with no real recorded proof is auto-marked unproven.** *(Mechanism superseded by D12 — read D12 for the field; the intent below stands.)* Absence of proof routes the cell onto the pending path instead of passing silently — a `trace.verify_passed === true` with `output` empty or missing is treated as "no proof recorded", never as proof. This closes the hole D1 would otherwise open: `requiredProofTier` is consumed only at the two red-first sites (`cells.mjs:2103`, `:2135`), and `featureVerifyDebt` (`packages/bee/lib/state.mjs:2502-2518`) arms D3's close-door only when a capped cell carries `trace.feature_verify === "pending"`. Without D10 a worker could self-declare `verify --passed true` with no output (legal today — `recordVerify`, `cells.mjs:1747-1756`, validates only `command` and `passed`), cap off the pending path, and the feature would close with zero tests executed anywhere. `--feature-verify-pending` therefore becomes the *mechanical* default, not instruction prose. | D1 is only survivable because D3's door still forces one real green run per feature. That door has to arm itself; an instruction telling workers to prefer the pending flag would not. |
| D11 | `testCellDebt` (`packages/bee/lib/state.mjs:2546-2570`) inherits the same treatment: a capped `change_class: 'test'` cell whose pass was asserted with no output does not satisfy the test-cell door. Same hole, same fix, stated separately because it is a second call site. | — |
| D12 | **Supersedes D10's mechanism, keeps its intent.** The absence marker is a **new** cell trace field `trace.proof = "unrecorded"`, never a reuse of `trace.feature_verify = "pending"`. `featureVerifyDebt` and `testCellDebt` arm on either marker; the freshness comparison must run over the **union** of `pending` and `unrecorded` caps, so a green feature-verify record newer than the newest pending cap but older than a newer `unrecorded` cap still refuses the close-door. `"unrecorded"` is stamped after the whole refusal chain has run (post-`cells.mjs:2178`) and never by setting `feature_verify_pending` internally. Repos with `commands.verify === "none"` are exempt. | `pendingFeatureVerify` is not a neutral marker: it short-circuits six sites — `cells.mjs:1886-1899`, `:1912`, `:1999`, `:2032`, `:2135`, `:2164` — all keyed on the local flag, never on trace state. Reusing it would have voided D2's red-first tier and D6's brakes the moment the marker landed. A separate field is inert at every one of those sites. |
| D13 | This feature ships one trailing `change_class: 'test'` cell despite the `high-risk` per-cell red-first doctrine, because `testCellDebt` (`state.mjs:2606`) has no lane exemption and refuses a feature close when capped code-touching `behavior`/`api` cells exist and no `test` cell does. The doctrine-vs-machine gap — `skills/bee-planning/SKILL.md:76` permits what the machine refuses — is recorded for compounding, not fixed here. | Following the prose alone would leave the feature unable to leave `swarming`, with no bypass level lifting it. |
| D14 | A cap is `"unrecorded"` only when **neither** real verify output **nor** `verification_evidence` was supplied. A cell holding genuine evidence — e.g. a tiny-lane `security` cell whose 80-char `red_failure_evidence` already passed `cells.mjs:2135` — is not marked, even with empty `verify_output`. | Advisor consult found the predicate was defined against `verify_output` alone, which would arm the close-door for cells that in fact hold the strongest proof in the system. |
| D9 | All four decisions above were locked by the agent under `gate_bypass_level: total` after the user declined the question round. They are the recommended options, recorded as an audited autopilot choice, and remain the user's to reverse. | Traceability: no decision here carries a user quote beyond the originating instruction. |

### Agent's Discretion

Planning chooses whether the non-blocking D1 branches emit a warning line, a
recorded cell field, or both — and where the D8(a)/(b) checklists physically live
(`SKILL.md` vs `references/worker-details.md`). Constraint: no new required
output artifact from the worker, or D1 is undone by the back door.

## Terms

| Term | Meaning in this feature |
|------|-------------------------|
| Evidence | Output the build already emits — a red test run, a verify run, `git diff`/`git show`. Authored prose asserting a cell works has never been evidence and after D1 is no longer requested at all. |
| Coverage judgment | The trailing test cell's first step (D4): a `file:line` citation of the nearest existing test plus one line on whether it already covers the slice's criteria. |
| Just-enough tests | The triad of D5 at its smallest demonstrating size, bounded by D6's existing ratio ceiling. |

## Existing Code Context

### Integration Points

- `packages/bee/lib/cells.mjs:1956-1960` — the `behavior_change` evidence refusal (D1 target).
- `packages/bee/lib/cells.mjs:2164-2168` — decision 0004's small+ "assertion is not evidence" refusal (D1 target). `:2169-2173`, the non-empty `files_changed` refusal in the same block, is deliberately out of scope.
- `packages/bee/lib/cells.mjs:1747-1756` — `recordVerify`; `--passed true` with no `--output` is legal today (D10 target).
- `packages/bee/lib/state.mjs:2502-2518` — `featureVerifyDebt`, what arms D3's close-door (D10).
- `packages/bee/lib/state.mjs:2546-2570` — `testCellDebt` (D11).
- `packages/bee/lib/cells.mjs:2103-2156` — red-first `red_failure_evidence` branches (D2: untouched).
- `packages/bee/lib/cells.mjs:1975-2014` — new-test-file / `new_suite_reason` refusals (D6: untouched, D7 negative control).
- `packages/bee/lib/cells.mjs:2023-2059` — ratio ceiling (D6: untouched).
- `packages/bee/lib/cells.mjs:162-185` — `requiredProofTier`, the class×lane matrix D1 edits one column of.
- `skills/bee-executing/references/worker-details.md:13,59,164-192` — the worker-side statements of every rule above.
- `skills/bee-planning/SKILL.md:76` and `references/planning-reference.md:295-297` — the trailing test cell floor (D4 target).
- `skills/bee-planning/references/edge-dimensions.md` — the 12-dimension matrix D5 demotes to high-risk only.

### Established Patterns

- Audited exemption over silent one (`new_suite_reason`, `ratio_waiver`) — D1's non-blocking branches should record, not vanish.
- Negative-control pairing on every guard loosening (knowledge R55 edge case) — D7.

## Canonical References

- `docs/history/test-economy/CONTEXT.md:21-28` — D1/D2 proof-tier matrix, D3 test shape, D5 read-first, D8 negative control.
- `docs/history/validation-diet/CONTEXT.md:46` — D9, the definition of evidence this feature keeps.
- `docs/history/main-verifies/CONTEXT.md:25-29` — the feature-boundary proof and close-door this feature leans on.
- `docs/history/ci-owned-verify/CONTEXT.md:12,36-53` — impacted-only `commands.test`, CI-owned full verify.
- `docs/knowledge/areas/workflow-state/cells-completion-judge-and-archive.md:245-289` — R55/R57, the shipped encoding of all of it.

## Outstanding Questions

### Resolve Before Planning

- [ ] None blocking. D9 records that the user declined the question round; any of D1-D6 can be reversed on request.

### Deferred To Planning

- [ ] Whether the impact-registry mismatch warning and the judge `NEEDS_REVISION` block belong in D1's scope — both refuse at cap but neither asks the worker to author evidence, so the current reading is no.
- [ ] Whether `no-test repo` sentinel handling (`commands.verify: "none"`, decision 55b951e1) needs any change once D1 lands — likely dead code paths overlap.

## Deferred Ideas

- Replacing the `change_class` self-declaration with something the worker cannot set — the routing rationale named it as "a gate keyed on an annotation applied by the party it exists to catch." Real, but a separate feature: D1 reduces what that annotation buys, which lowers the urgency.

## Handoff Note

CONTEXT.md is the source of truth. Decision IDs are stable. Planning reads locked
decisions, code context, canonical references, and deferred-to-planning questions.
Planning's Gate 2 shape stage and reviewing use locked decisions for coverage and UAT.
