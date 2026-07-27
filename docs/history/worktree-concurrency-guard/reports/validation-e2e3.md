# Validation Report — worktree-concurrency-guard, Batch 2 (cells wcg-2, wcg-3)

**Feature:** worktree-concurrency-guard
**Mode:** high-risk
**Slice:** Epics 2 and 3 (write-guard wiring, worktree-new wiring)
**Date:** 2026-07-24

## Reality Gate

| Dimension | Verdict | Evidence |
|---|---|---|
| MODE FIT | PASS | Same feature, same locked hard-gate flag; no re-triage needed. |
| REPO FIT | PASS | `hooks/bee-write-guard.mjs:791,804,818` and `handleWorktreeNew` at `skills/bee-hive/templates/bee.mjs:3845` confirmed live at cited line numbers post-wcg-1 commit (plan-checker re-verified after wcg-1 landed). `guards.isSharedNestedCheckoutTarget` confirmed at `guards.mjs:795`. |
| ASSUMPTIONS | PASS (after fix) | The one real assumption gap — whether the E1 primitive's single-target-check signature fits worktree-new's directory-scan need — was flagged by the plan-checker as "near-certain, not maybe." Resolved by declaring the expected new `guards.mjs` variant explicitly in `wcg-3`'s scope (files + action), not left as worker discretion. |
| SMALLER PATH | PASS | Batch is exactly the next epic-map slice (E2, E3); E4 (regression suite extension) correctly deferred to the batch after. |
| PROOF SURFACE | PASS | Both verify commands confirmed real and currently green on the pre-change tree (baseline for each cell's required red-first proof): `node --test hooks/test_write_guard.mjs` (1 pass), `node --test scripts/test_worktree_companion.mjs` (10/10 pass). |

## Feasibility Matrix

| Assumption | Risk | Proof Required | Evidence | Result |
|---|---|---|---|---|
| wcg-2/wcg-3 schedule safely as declared | High | `bee cells schedule` + regen-artifact overlap analysis | Plan-checker found BOTH cells' `regen_obligation_ack` commits them to `release_manifest.mjs --write` (full-file rewrite) and `.bee/onboarding.json` regen, but only `wcg-3.files` originally declared those paths — a real lost-update race in the scheduled parallel wave, invisible to `cells schedule` because they were undeclared side-effects, not declared `files`. | **BLOCKER, resolved**: `wcg-2.files` now also declares `release-manifest.json` + `.bee/onboarding.json`; `cells schedule` now returns two serialized waves (`[wcg-2]`, `[wcg-3]`), confirmed by direct run. |
| E1's primitive signature fits `handleWorktreeNew`'s need | Medium-High | Code-level check of `isSharedNestedCheckoutTarget`'s actual walk direction | Plan-checker confirmed `findNestedCheckoutDir` walks UP from a concrete target — worktree-new has no such target, it needs a directory-scan (walk DOWN) variant. Originally left as worker discretion ("if needed"); this understated the certainty. | **WARNING, resolved**: `wcg-3`'s action and `files` now explicitly require a new directory-scan `guards.mjs` export (all 6 copies), reusing the existing companion-marker and submodule-registration logic rather than duplicating it — no longer left ambiguous. |
| Cited line numbers (`bee-write-guard.mjs:791/804/818`, `bee.mjs:3845`) are still accurate after wcg-1 landed | Required | Direct re-read | Plan-checker re-read the current tree post-wcg-1 commit and confirmed all citations exact. | PASS |
| Both cells' verify commands are real and pass on the current (pre-change) tree | Required | Direct run | Confirmed by both the orchestrator and the plan-checker, independently: `hooks/test_write_guard.mjs` 1/1, `scripts/test_worktree_companion.mjs` 10/10. | PASS |

## Plan-Checker Findings and Resolution

Independently dispatched `bee-review` plan-checker returned **1 BLOCKER, 1 WARNING** (both above), **0** on requirement coverage, cell completeness/line-numbers, and dependency correctness. Both findings fixed via `cells update` (see feasibility matrix). Re-ran `cells schedule` myself to confirm the fix, rather than trusting the update alone.

## Advisor Consult (AO2b/AO3, mandatory for high-risk before Gate 3)

Configured advisor (fable) independently re-verified the serialization fix (re-ran `cells schedule`, confirmed wcg-1's reservations all show `released_at`) and the wcg-3 scope call, and approved with two P3 flags: `scripts/test_worktree_companion.mjs` was missing from `wcg-3.files` despite being its verify target, and `wcg-3` lacked wcg-2's red-first-proof requirement. Both applied via `cells update` before recording this consult. `advisor_ref` recorded and non-stale.

## Decision

**READY WITH CONSTRAINTS** — ready to execute `wcg-2` then `wcg-3`, strictly serialized per the corrected schedule (never dispatch both as a parallel wave — the whole point of this fix). Epic 4 (regression-suite extension) remains out of scope for this Gate 3 approval.
