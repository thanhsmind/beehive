# wc-6 advisor consult — fable

**Cell:** wc-6 (worker-conformance, lane `high-risk`) · **Advisor:** fable (configured `models.claude.advisor`)
**Scope of the ask:** the four-file bee-planning diff turning the trailing test cell's first
step into a coverage judgment (D4), scoping `edge-dimensions.md` to high-risk (D5), leaving the
D6 brakes alone, and recording the D13 doctrine-vs-machine gap.

## Ask 1 — is the `testCellDebt` claim accurate?

**Answer: true, but under-stated in a misleading way.** The advisor read the predicate
(`state.mjs:2570-2658`) and the unwaivable feature-debt set (`:2718-2797`). Confirmed: no lane is
read anywhere; the trigger is capped code-touching `behavior`/`api` cells; the test-cell door sits
in `FEATURE_DEBT_KINDS`, so no `gate_bypass` level — `total` included — lifts it. Two precision
gaps in my wording:

1. "no test cell at all" is only the *missing* kind. A test cell that exists but is
   open/claimed/blocked, capped red, or capped `trace.proof: "unrecorded"` refuses via *not-green*
   (`:2603-2616`). My sentence let a planner conclude that **creating** the cell satisfies the
   machine; it does not — the cell must cap green with recorded proof. A **dropped** test cell is
   skipped before the counter (`:2600`), so dropping the only one falls through to *missing*.
2. "code-touching" is conservative: an empty or missing recorded file list **counts as code**
   (`:2637-2638`, `:2647`).

**Adopted.** Both `SKILL.md` and the planning-reference D13 paragraph rewritten to name the two
refusal kinds, the capped-green-with-proof requirement, and the unrecorded-file-list case.

## Ask 2 — did the text weaken the unconditional floor?

**Answer: no.** "A code-touching slice with no test cell is a planning defect" survives in both
files; every "authors nothing" sentence is scoped to authoring, never to the cell's existence, and
each still requires capping by running the cited tests green. No change needed.

## Ask 3 — the wc-3 citation count

**Answer: drop the count.** `wc-3.md` is internally split — its outcome line says "four of the
five parts", its step-1 table grades eleven rows (ten covered, one partly). "Four of five already
pinned" also implies the gap was a fully-uncovered part, when the record grades it *partly
covered*. **Adopted:** `SKILL.md` now says "found all but one part of the story already pinned",
matching planning-reference.md's countless version.

## Ask 4 — rot and backfire

- **tiny/small rung:** no doctrinal hole — D5 and the new `edge-dimensions.md` scope note both say
  "standard **and below**", which subsumes the removed "2–3 dimensions that bite" rung. But the
  plan.md template block listed only two rungs. **Adopted:** changed to "standard and below".
- **Verdict table vs D6:** no contradiction. D6 bans a *numeric per-group cap*; the table
  constrains *necessity*, which is D4's own locked wording, and the "Unchanged by all of the above
  (D6)" paragraph restates the ratio ceiling as the sole volume brake. No change.
- **Unprompted, in my favor:** citing `state.mjs` without line numbers is the rot-resistant choice
  even though CONTEXT.md D13 pins `:2606`. Left as-is.

**Disposition: all four actionable points adopted, none rejected.** Advice is advisory only; no
gate was approved by it and no locked decision was overridden.
