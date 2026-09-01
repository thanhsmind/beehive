# plan-conflicts derive scope — Context

**Feature slug:** plan-conflicts-scope
**Date:** 2026-09-01
**Shaping session:** complete
**Scope:** Quick
**Domain types:** RUN

## Feature Boundary

`bee state plan-conflicts derive` returns a candidate for every active decision
scoring >= 2 term hits against the plan's term set. In a repo with a large
decision store that is hundreds of candidates, each needing its own CLI call to
verdict, and `bee gate --merge` refuses until every one carries one. This
feature narrows the term set so the candidate list is proportionate. It ends
there: the scoring rule, the verdict vocabulary, the gate precondition, and the
rule-home half of the derivation are all untouched.

## What was asked, found, and will be done

**Asked.** `pstack-adoption` could not pass Gate 2: 753 unverdicted candidates
for a 9-file standard feature. The user chose to fix this before resuming that
work.

**Found**, measured on this repo's live store (2589 active decisions, the 31
terms `pstack-adoption`'s four cells produce):

| Rule | Candidates |
|---|---|
| today — hits >= 2 | 694 |
| hits >= 3 | 314 |
| hits >= 4 | 132 |
| drop terms with document frequency > 10%, hits >= 2 | 442 |
| drop terms with document frequency > 5%, hits >= 2 | 252 |
| **drop terms with document frequency > 3%, hits >= 2** | **36** |
| drop terms with document frequency > 2%, hits >= 2 | 20 |

No single term dominates — the worst is `name` at 15.9%. The defect is that a
**fixed >= 2 threshold meets an unbounded term set**: 31 moderately common terms
produce hundreds of meaningless two-hit coincidences. It scales the wrong way —
a bigger plan gets more noise, not more precision.

The trigger is narrow and worth recording: derive is clean *before* a feature
has cells (`plan_cells` returns none, so the term set is empty and no decision
candidate is produced). It explodes on any **re-derive after cells exist** —
which is exactly what a mid-flight plan revision forces.

`plan_conflicts.rs:28-35` already states the intended seam: the existing
length-and-stopword filter is "a property of BUILDING the term set, not of the
scoring rule, which is untouched", and it exists because without it common words
"reach `count_term_hits`'s >= 2 threshold against nearly every decision ever
logged, which makes '0 conflicts' unreachable and the whole check noise." That
is precisely the failure being fixed; the hand-written stop list simply covers
generic English and not bee's own domain vocabulary.

**Will be done.** Extend that same filter with a corpus-derived document
frequency cut, plus a hard cap as a safety rail.

## Locked Decisions

| ID | Decision | Rationale |
|----|----------|-----------|
| D1 | The fix lands in the TERM SET, not the scoring rule. `count_term_hits`, `conflict_candidates`, the >= 2 threshold, and the rule-home half of `derive_candidates` are all unchanged. | The module header names this exact seam as the design intent (`plan_conflicts.rs:28-35`). Changing the scorer would change `decisions log`'s hints too — a far wider blast radius for the same result. |
| D2 | A term whose document frequency across the active decision store exceeds **3%** is dropped from the term set. | Measured above: 3% is the knee — 694 → 36 candidates, while 5% only reaches 252 and 2% starts discarding genuinely specific terms. Corpus-derived, so it self-tunes per repo instead of needing a hand-maintained word list that goes stale. |
| D3 | The document-frequency filter applies ONLY when the active decision store holds at least **200** entries. Below that, behavior is byte-identical to today. | With a handful of decisions, document frequency is noise: one hit in a two-decision fixture is 50%. Existing tests use exactly such fixtures, and a freshly onboarded host repo has almost no decisions — neither may change behavior. |
| D4 | After filtering and scoring, the candidate list is ranked by hit count and capped at **50**, and a truncated list says so in its own output. | 3% happens to yield 36 here, but nothing bounds it in a repo ten times this size. The cap is the rail; being loud about truncation is what keeps it from hiding a real conflict silently. |
| D5 | No change to the verdict vocabulary, to `build_conflict_review`, or to the gate precondition that reads it. | Those are a separate cell's scope by explicit boundary (`plan_conflicts.rs:8-11`). This feature makes the list sane; it does not renegotiate what a verdict means or who checks it. |

## Terms

| Term | Meaning in this feature |
|------|-------------------------|
| Document frequency | The share of active decisions whose searchable text contains a given term. `name` is 15.9% in this repo; `perf` is 2.0%. |
| Term set | The lowercased, de-duplicated words derived from a feature's open and capped cell titles and path stems — what `plan_terms` returns. |

## Existing Code Context

### Integration Points

- `packages/bee-rs/crates/bee/src/verbs/state_group/plan_conflicts.rs:171` —
  `plan_terms`, where the filter is applied today and where D2/D3 land.
- `packages/bee-rs/crates/bee/src/verbs/state_group/plan_conflicts.rs:245` —
  `derive_candidates`, which needs the active-decision set for the frequency
  count. It already loads it (`active_decisions(root, false)`), so no new read.
- `packages/bee-rs/crates/bee/src/verbs/state_group/plan_conflicts.rs:81-86` —
  `TERM_STOPWORDS`, the existing hand-written filter D2 complements rather than
  replaces.

### Established Patterns

- Filter at term-set build time, never in the scorer (`plan_conflicts.rs:28-35`).
- A documented constant with its measured justification beside it, as
  `PLAN_CELL_STATUSES` and `TERM_STOPWORDS` already are.

## Outstanding Questions

### Resolve Before Planning

None.

### Resolve During Execution

- Whether `active_decisions` is already in hand at the point `plan_terms` is
  called, or whether the signature must change to receive it. Read
  `derive_candidates` at the call site and thread it rather than loading twice.
