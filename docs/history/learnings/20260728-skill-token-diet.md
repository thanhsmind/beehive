---
date: 2026-07-28
feature: skill-token-diet
categories: [instruction-surfaces, verify-pipeline, process]
severity: medium
tags: [skills, performance, fence, ratchet, migration]
---

# skill-token-diet — feature close learnings

## What Happened

A byte-budget fence (`scripts/skill_budget_fence.mjs`, blocking, selftest+bare pair
in the verify chain) and the thin-body doctrine were shipped over bee's six
heaviest skill bodies: hive 30,078→8,183 · swarming 24,676→8,187 · scribing
24,472→8,151 · planning 23,027→8,139 · reviewing 22,303→8,190 · validating
18,273→8,049 — six bodies 142,829→48,899 bytes (−65.8%). Exiled content landed
meaning-intact in each skill's `references/` (incl. a per-skill `provenance.md`);
the regrowth law (bundle/references by default, body only for load-bearing
invariants, one-in-one-out) now lives in `bee-writing-skills` and `bee-evolving`.
Source spec: `AI/ak/plans/reports/spec-260727-1619-bee-skill-token-diet.md`.

## Root Cause (of the original problem)

Bodies are injected whole on every invoke; `bee-evolving`/`bee-compounding`
wrote learnings into skill prose, so bodies grew monotonically, and dense
142-char law-lines let byte weight triple while line counts passed review.

## Findings

1. **A spec's measurements are claims, not facts.** Two of the spec's numbers
   were stale/wrong (bee-herding "56K body" was 14.3K — the 56K counted
   `references/`; bee-executing "19.5K" was already 10.2K). Re-measuring at
   exploring changed the scope decision (herding needed no owner special-case).
   *Rule: when a spec cites measurements, re-run the measurement before locking
   scope — a `wc -c`/`rg -c` pass costs seconds and beats a frozen false premise.*
2. **Status is a declared list, never a proxy-metric inference.** The plan froze
   "migrated = budget ≤ 8,192"; validation found an unmigrated 6,454-byte skill
   with 11 legitimate citations that would have redded the chain. Fixed to an
   explicit `migrated: []` array. *Rule: classify by explicit membership, not by
   an inferred threshold; validate every inference rule against live data before
   Gate 3.* (Already mechanized: the fence reads only the list.)
3. **Ratchets need a documented introduction move.** `--update-baseline` rightly
   refuses raises, but real pre-fence drift (4 skills over baseline) needed a
   one-time direct re-seed, done inside the fence cell and recorded as a
   validation decision. *Rule: introducing a ratchet over drifted reality
   requires one explicit, recorded re-seed — never a loosened tool.*
4. **A worker's [DONE] can omit its commit.** diet-8's worker reported done with
   verify green but never committed; only the orchestrator's git-log check
   caught it, and a resume message produced the commit. *Rule: goal-check every
   [DONE] for a commit containing the cell id, same as re-running verify.*
   (Filed as friction — candidate mechanization: `cells cap` or the goal-check
   asserting a commit referencing the cell id exists.)
5. **Exemplar-report-driven serial migration scales.** One ceiling-tier worker
   produced the first migration + report (diet-3); five generation-tier workers
   replicated it faithfully by reading that report first. Side-by-side
   before/after excerpts per migration made meaning-drift visible and cheap to
   review. *Rule: for N symmetric risky edits, buy one high-tier exemplar, then
   fan the rest to cheaper tiers with the exemplar as required reading.*

## Recommendation (promotion candidates weighed)

Findings 2, 3, 5 are already mechanized or process-encoded (fence list, recorded
re-seed decision, exemplar pattern in this file). Promoted as prose-critical:
**finding 1** (re-measure spec claims before locking scope) — multi-feature,
prevents frozen false premises, generalizable. Finding 4 filed as P2 friction
for mechanization rather than prose.
