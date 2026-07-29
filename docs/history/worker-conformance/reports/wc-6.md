# wc-6 — Make asking whether tests are needed the required step, not writing them

**Status:** [DONE]
**Outcome:** The trailing test cell stays unconditional; what changed is its first mandated step —
a coverage judgment with `file:line` citations, not authoring. A test cell that authors no test is
now stated plainly to be no defect. The triad is the shape at `standard` and below and
`edge-dimensions.md` is scoped to `high-risk`/hard-gate at both its call sites and at the top of
the file itself. The D13 doctrine-vs-machine gap is recorded where a planner meets it, unfixed.

**Files touched:** `skills/bee-planning/SKILL.md`,
`skills/bee-planning/references/planning-reference.md`,
`skills/bee-planning/references/edge-dimensions.md`,
`skills/bee-planning/references/provenance.md`
**Full trace / evidence:** `.bee/cells/wc-6.json`

## What changed, by decision

- **D4 — `SKILL.md:76` and `planning-reference.md` ("Slice-tail test batching in full").** The
  floor survives verbatim ("a code-touching slice with no test cell is a planning defect") and the
  cell is called unconditional in as many words. Its first step is now the coverage judgment: cite
  the nearest existing tests by `file:line`, grade each acceptance criterion, then act on the
  verdict — covered caps by running the cited tests green and recording "already covered, no new
  rows"; partly covered authors only the gap. Both files say outright that a test cell authoring no
  test is not a defect, and name duplicate rows as the waste the rule exists to stop. The worked
  instance `docs/history/worker-conformance/reports/wc-3.md` is linked from both, with its step-1
  table named as the shape to copy.
- **D5 — the triad, and the demotion of the 12 dimensions.** Stated at `standard` and below in
  `SKILL.md`, in the expanded reference section, in the plan.md `## Test matrix` template, and in
  the reference-map row. `edge-dimensions.md` opens with a scope note naming `high-risk`/hard-gate
  as its only readers, with the one-line reason: read as a checklist to fill, twelve dimensions
  generate volume.
- **D6 — untouched, and said so.** A dedicated paragraph restates that the ratio ceiling,
  `new_suite_reason`, and the `refactor`+new-test-file refusal are unchanged and that no numeric
  per-group cap is added — triad is the shape guide, ratio ceiling is the volume brake.
- **D13 — the gap, recorded not fixed.** Both `SKILL.md` (immediately under the "never batched"
  line that permits what the machine refuses) and the expanded reference section carry it. No
  predicate was touched.

## Deviation

None from the cell. One correction to my own first draft, from the advisor: the D13 paragraphs
originally said the machine refuses when "no test cell exists at all", which would let a planner
conclude that *creating* the cell satisfies the door. It does not — `testCellDebt` refuses on two
kinds, *missing* and *not-green*, so the cell must cap green with recorded proof. Corrected in both
files before commit.

## Verification

Instruction text only; no source touched (`packages/bee/lib` and every other source path untouched
— the diff is four `.md` files under `skills/bee-planning/`). The cell's `verify`
(`node packages/bee/tests/test_misc.mjs`) is MAIN's at the feature boundary and was deliberately
not run here; capped `--feature-verify-pending`. Regen scripts were **not** run — the cell carries
`regen_obligation_ack: "wave-barrier"` and the orchestrator owes the full regen chain at wave
close.

## Consults

1 consult — advisor **fable** (`docs/history/worker-conformance/reports/wc-6-consult.md`; recorded
via `state advisor-ref record`).

- **Ask:** is my `testCellDebt` claim precisely true; did I weaken D4's unconditional floor; is the
  wc-3 citation count defensible; does scoping `edge-dimensions.md` leave a tiny/small hole or does
  the verdict table contradict D6?
- **Answer:** the machine claim was true but under-stated — it named only the *missing* refusal
  kind and missed that an unrecorded file list counts as code-touching; the floor was **not**
  weakened; the "four of five" count should be dropped because wc-3's own record is split between
  its outcome line and its eleven-row table; the tiny/small rung is covered by "standard and below"
  everywhere except the plan.md template, which listed only two rungs; the verdict table does not
  contradict D6, which bans a numeric cap, not a necessity test. All four actionable points
  adopted, none rejected.
