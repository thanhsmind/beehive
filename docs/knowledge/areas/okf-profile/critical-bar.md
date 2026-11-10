---
type: bee.area
title: Bee OKF Profile — the bee.critical bar and the selective pool it grades
description: "The three-leg bar a pattern must clear to carry bee.critical (recurrence-prone, cross-feature, costly-when-missed — failing any leg disqualifies it), why the label needed a bar at all, and how the pool is kept selective as new patterns are authored."
timestamp: 2026-08-10
bee:
  id: okf-profile-critical-bar
  lifecycle: active
  areas: [okf-profile]
  required_context: [areas/okf-profile/overview.md]
  decisions: ["U6 (knowledge-usable, PBI p-355d4740): bee.critical earns a written bar — recurrence-prone AND cross-feature AND costly-when-missed; existing patterns re-graded to at most ~30 critical, re-grade recorded per-pattern in the commit, never a bulk strip)", "knowledge-in-the-flow D8 (feature outcomes and pack pull tracking in knowledge report)", "knowledge-useful-rules CONTEXT.md D5 (knowledge report records concept use counts and loaded-never-used findings)"]
  sources: ["docs/history/knowledge-outcome-measure/plan.md (D1-D5: the outcomes measure)", "docs/history/knowledge-usable/CONTEXT.md (U6 rationale: 85/101 critical = no filter; the ranker needs a selective pool)", "knowledge-usable cell ku-6 (the re-grade this concept authors; trace in `.bee/cells/`, 2026-08-10)", "knowledge-in-the-flow cell kitf-3 (feature outcomes by feature arm and pack pull rate in knowledge report, commit 8794b2917, 2026-11-09)", "CONTEXT.md docs/history/knowledge-in-the-flow/CONTEXT.md", "docs/history/knowledge-useful-rules/CONTEXT.md"]
  authoritative_for: "okf-profile: the bee.critical bar and what earns a pattern that label"
  anchors: [packages/bee-rs/crates/bee/src/verbs/knowledge/report.rs, packages/bee-rs/crates/bee/tests/knowledge_outcomes.rs]
  verified_at: 2026-11-10
  verified_commit: 2f0495d24e30048da79fa3d5dcc287605cf6869c
---

# Bee OKF Profile — The `bee.critical` Bar and the Selective Pool It Grades

Read when: deciding whether a pattern earns bee.critical, or investigating why bee knowledge report lists concepts as loaded but never used.

This concept owns what `bee.critical: true` **means** on a `bee.pattern` concept, the three-leg bar
a pattern must clear to carry it, and why an unbarred label had already stopped working before this
concept existed.

## Behaviors & Operations

**B1 — The bar (U6).** A pattern earns `bee.critical: true` only if it clears all three legs. Failing
any single leg disqualifies it, however strong the other two read:

| Leg | Question | A pattern that fails this leg |
|---|---|---|
| **Recurrence-prone** | Has this defect class actually recurred — documented multiple times, cited by a later pattern as the same shape, or does its own history show it being relearned — or is it a plausible-but-unproven "could happen again"? | A one-off tooling quirk hit once, in one feature, with no echo anywhere else in the corpus. |
| **Cross-feature** | Does the failure mode reach code, process, or review work outside the single feature/tool/platform that produced it? | A defect tied to one platform's install path, one migration's own tooling, or one narrow instrumentation choice that will not recur in a different shape elsewhere. |
| **Costly-when-missed** | Does missing it cost a shipped defect, a security/attribution breach, a false-green that hides a real regression, a blocked release, or a burned session — not merely "extra effort" or "a less clean design"? | A practice that saves work when followed but whose absence is an inefficiency, not an incident. |

**B2 — Why a bar, not a filter on `polarity`.** Before this bar, 85 of 101 patterns (84%) carried
`critical: true` — functionally no filter at all, because a ranker that surfaces "critical" patterns
first (`context-and-promote.md`'s relevance ranking) cannot rank against a pool that is nearly the
whole corpus. The label had drifted to mean "was written down," which every pattern already satisfies
by existing. The bar restores the label's only useful meaning: **this is the subset worth loading
under budget pressure even when everything else gets cut.**

**B3 — Grading, not deleting.** Re-grading a pattern's `bee.critical` field is a frontmatter edit, not
a content judgment — a de-flagged pattern is not wrong, less true, or less worth having written; it
stays `active`, stays findable via `bee knowledge list`, and stays fully in the bundle. Only its
priority under budget pressure changes. A pattern is never deleted to shrink this pool (see
Prohibitions below); the checker's `dangling_supersedes`/`dangling_required_context` findings and every
existing citation stay valid regardless of a pattern's `critical` value.

**B4 — Same-shape patterns keep one carrier.** When several patterns in the corpus name the same
underlying defect class from different incidents (a hand-maintained list that rots, a scan scope
narrower than the corpus it must cover, a guard that tests one state of a set), the bar is applied to
the class, not to each instance independently: the instance with the strongest documented recurrence,
the widest cross-reference from later patterns, or the clearest mechanized fix carries `critical`, and
sibling instances that add no leg the carrier lacks stay uncritical without being wrong — the
recurrence is what the carrier's own body already documents.

**B5 — Applying the bar to a new pattern.** An author proposing `bee.critical: true` on a newly
authored pattern states, in the pattern's own body or its `sources`, which incident(s) establish
recurrence (or names the generalizing mechanism if this is the first incident of a class already
proven to recur elsewhere), which other feature/surface the failure mode reaches, and what the missed
cost was. A pattern that cannot state that in one sentence per leg has not cleared the bar yet, whatever
its narrative interest.

**B6 — Knowledge usefulness is measured by cell outcomes, not signature counts (knowledge-outcome-measure
plan D1).** The `bee.signature` recurrence count is not the usefulness measure: no signature string is
ever written into `.bee/decisions.jsonl` or `.bee/capture-queue.jsonl`, so that count reads 0 for every
pattern whatever the bundle does. `bee knowledge report` therefore also prints an `outcomes` section
that compares closed cells by the arm their brief was dispatched in (`knowledge`, or `holdout` with no
Learned context block; `areas/workflow-state/dispatch.md`, R21, plan D2-D4). The population is every
`capped` or `dropped` cell under `.bee/cells/` (archive included). A cell's arm is `knowledge_arm` on the
earliest `.bee/logs/dispatch.jsonl` row with `source: "prepare"`, `kind: "cell"` and that cell id; a cell
with no such row is `unrecorded` and is never assigned an arm after the fact (plan D5). Per arm the
section counts `cells` and four trace signals — `blocked` (`blocked_reason` a non-empty string),
`reopened` (`reopened_for_rework` or `reopened_at` present), `red_rounds` (`attempts` non-empty),
`with_mistakes` (`mistakes` non-empty) — with each as a 3-decimal rate of the arm's cells. `comparable`
is true only when both arms hold at least `min_per_arm` (30) cells; until then the text output names how
many more cells each arm needs, and the rates are not a finding.

**Feature-level outcomes and pack pull tracking (knowledge-in-the-flow D8).** In addition to cell
outcomes, `bee knowledge report` outputs a `feature_outcomes` section comparing features by their
assigned feature arm (`knowledge` or `holdout`). A feature's arm is read from the earliest
`.bee/logs/dispatch.jsonl` row with `source: "pack"` or `kind: "advisor"`; closed features with no
such row are tallied as unrecorded. For each arm, the report counts features, cells, and the four
friction signals (blocked, reopened, red rounds, with mistakes). In the Pull rate table,
`knowledge pack` is counted alongside `search`, `context`, and `show` as an active knowledge pull
contributing to feature-level and total pull rates.

**The use measure — which concepts answered, and which were loaded and never used (knowledge-useful-rules
D5).** The outcome signals above say whether a brief helped; this section says which concept did. It reads
two existing sources and writes nothing. From every `capped` or `dropped` cell's own trace: an entry in
`trace.knowledge_used` (`[{path, changed}]`, written by the cap on `--knowledge-used` or the report's
`knowledge_used` array) counts the cell as `used` and credits `uses` to each path it names; a
`trace.no_knowledge_used: true` counts the cell as `none`; a trace carrying neither counts the cell as
`silent`. `answered` is `used + none`, so silence stays visible beside the two answers rather than
folding into either — the same distinction the cap itself keeps. From the `learned_context` lists of
`.bee/logs/dispatch.jsonl` rows with `source: "prepare"` and `kind: "cell"`: every entry starting
`docs/knowledge/` counts one load, and a concept loaded five or more times with zero recorded uses is a
`never used` finding. The text output prints one `knowledge used: <a> cell(s) answered (<u> used a
concept, <n> none), <s> silent` line, then up to ten `  <uses>  <path>` lines ordered by uses
descending then path, then one `loaded, never used: <path> — loaded <n> time(s)` line per finding,
ordered by loads descending then path; the JSON carries `knowledge_used {answered, used, none, silent,
concepts: [{path, uses}], never_used: [{path, loads}]}` and keeps every concept in `concepts` while the
text truncates at ten. Two limits are deliberate: a use count only ever comes from a cap's own answer,
because nothing else records it, so a concept that helped without being named reads as unused; and
`uses` counts the concept per cell, not per decision, so one cell naming a concept twice credits it
once. Both directions under-count, which is the safe way to read a metric whose absence is silence.

## Business Rules

- The bar applies only to `bee.pattern` concepts; no other concept type carries `bee.critical`.
- Failing one leg is sufficient to disqualify — the three legs are conjunctive (AND), never averaged.
- The outcomes section measures the bundle as a whole by arm; it grades no single pattern, and a
  low signature count (or a zero) says nothing about whether the bundle helps (plan D1).
- Re-grading is a frontmatter-only operation: `lifecycle`, body content, and `bee.id` never change as
  a side effect of a `critical` re-grade.

## Edge Cases Settled

- A pattern with `polarity: practice` (not `pitfall`) can still clear the bar — the bar grades
  recurrence/reach/cost, not whether the pattern describes a failure or a discipline that prevents one.
- A pattern that is the *origin* of a locked decision elsewhere in the bundle (its lesson is cited as
  the rationale for a `docs/history/<feature>/CONTEXT.md` decision) has independent, external proof of
  cross-feature cost and clears that leg by citation rather than by re-argument.
- The ~30-pattern target (U6) is a calibration, not a hard ceiling enforced by `bee knowledge check` —
  the bar is what is graded; the count is evidence the bar was applied with the intended selectivity,
  re-measured whenever the pool is re-graded.

## Pointers (implementation)

- The field itself: `bee.critical` on any `bee.pattern` concept under `docs/knowledge/patterns/`.
- Where the pool is consumed: `docs/knowledge/areas/okf-profile/context-and-promote.md` (the
  relevance ranking that "cuts critical patterns without losing one" under budget), and
  `docs/knowledge/index.md`'s generated "Critical patterns" section (`bee knowledge index`).
- The outcomes measure (B6): `build_outcomes`, `build_feature_outcomes`, and `build_pull_rate` in
  `packages/bee-rs/crates/bee/src/verbs/knowledge/report.rs`, tested in
  `packages/bee-rs/crates/bee/tests/knowledge_outcomes.rs` and
  `packages/bee-rs/crates/bee/tests/knowledge_in_the_flow_e2e.rs`; plan
  `docs/history/knowledge-outcome-measure/plan.md` (D1-D5),
  `docs/history/knowledge-in-the-flow/plan.md` (D8).
- The use measure: `build_knowledge_used` in the same `report.rs`, reading `trace.knowledge_used` and
  `trace.no_knowledge_used` off closed cells and `learned_context` off cell prepare rows
  (`knowledge-useful-rules` CONTEXT.md D5; the trace shape is written in
  `packages/bee-rs/crates/bee/src/verbs/cells/handlers_close.rs`).
- The re-grade this concept documents: `docs/history/knowledge-usable/CONTEXT.md` (U6), cell `ku-6`.
