---
date: 2026-07-27
feature: compounding-gate
categories: [state, gates, process]
severity: P2
tags: [compounding, scribing, close, census]
---

# compounding-gate close — learnings

## What Happened

The close gate gained its missing fourth wall: `state compounding-run` records
learning-capture evidence, and `compounding-complete` now refuses without a
stamp fresh against the scribing run — waivable only through a decision-logged
`--waive-compounding`. A six-check suite proved the gate load-bearing by
mutation. Along the way the slice tripped over two census reds the sd2-1 skill
diet had left behind (the review-candidate close step and the worktree
three-check gate trimmed out of skill bodies) — restored as fix-first cg-3.

## Root Cause

- The phase name `compounding-complete` asserted history the machine never
  checked (same defect class chain-integrity fixed for scribing in D1-REVISED
  — the compounding half was simply never built).
- The sd2-1 diet moved census-pinned anchors into references without running
  the census; sequential asserts inside one census check masked the full depth
  of the breakage (first failure hides the rest).

## Recommendation

- When a phase name asserts that a step ran, make the transition demand the
  step's recorded evidence — a name is not a proof; mirror the
  scribing-debt/waiver shape (fail-closed, one audited door).
- When dieting skill bodies, run the census suite (`test_misc.mjs`) in the same
  cell — it is the contract for what must stay in bodies; a diet that only runs
  `skill_lint` is checking style, not contract.
- When a census check with sequential asserts fails, expect masked depth:
  after fixing the first assert, rerun before scoping the fix — the real scope
  is unknown until the check walks further.
