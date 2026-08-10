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
  decisions: ["U6 (knowledge-usable, PBI p-355d4740): bee.critical earns a written bar — recurrence-prone AND cross-feature AND costly-when-missed; existing patterns re-graded to at most ~30 critical, re-grade recorded per-pattern in the commit, never a bulk strip)"]
  sources: ["docs/history/knowledge-usable/CONTEXT.md (U6 rationale: 85/101 critical = no filter; the ranker needs a selective pool)", "knowledge-usable cell ku-6 (the re-grade this concept authors; trace in `.bee/cells/`, 2026-08-10)"]
  authoritative_for: "okf-profile: the bee.critical bar and what earns a pattern that label"
---

# Bee OKF Profile — The `bee.critical` Bar and the Selective Pool It Grades

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

## Business Rules

- The bar applies only to `bee.pattern` concepts; no other concept type carries `bee.critical`.
- Failing one leg is sufficient to disqualify — the three legs are conjunctive (AND), never averaged.
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
- The re-grade this concept documents: `docs/history/knowledge-usable/CONTEXT.md` (U6), cell `ku-6`.
