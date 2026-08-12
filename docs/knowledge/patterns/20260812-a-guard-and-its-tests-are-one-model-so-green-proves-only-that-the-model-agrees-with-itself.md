---
type: bee.pattern
title: "A guard and its tests are one model, so green proves only that the model agrees with itself"
description: "Three consecutive fixes to two guards each shipped with a full green suite and each was wrong, because the author's fixture encoded the same assumption as the guard; every one was found by an independent read against the live store's real shape distribution, never by the suite."
tags: [guards, enforcement, fixtures, review, sampling, fail-open]
timestamp: 2026-08-12
bee:
  id: pattern-20260812-a-guard-and-its-tests-are-one-model
  lifecycle: active
  areas: [workflow-state]
  decisions: ["harness-p1-fixes: the judge-debt door reads the route only when route.feature owns the closing feature, and falls back to a lane record's mode only when that mode names a lane class, 2026-08-12", "harness-p1-fixes: bee dispatch wave resolves exactly one feature and refuses by type when none resolves, 2026-08-12"]
  sources: ["review session-harness-lessons-20260811 (.bee/reviews/session-harness-lessons-20260811.json): five independent reviewers, 3 P1 / 21 P2-P3", "cells wfl-3, wfl-5, hpf-1, hpf-2, hpf-3 (.bee/cells/archive/)", "measured 2026-08-11 over .bee/lanes/: 54 lane records — 27 standard, 14 small, 6 high-risk, 1 tiny, 2 feature; 40 carry no route at all", "suite green at every wrong step: 1569, 1573, 1617 passed"]
  polarity: pitfall
  critical: true
  evidence: wired
  evidence_ref: "packages/bee-rs/crates/bee/src/verbs/cells/validate.rs (validate_new_cell calls assert_judge_obligation beside assert_regen_obligation; exercised end-to-end through the real `cells add` CLI door in packages/bee-rs/crates/bee/src/verbs/cells/tests.rs, jo-2)"
  signature: guard-fixture-shares-the-guards-own-assumption
---

# A guard and its tests are one model, so green proves only that the model agrees with itself

A guard is written to enforce a rule. Its tests are written minutes later, by
the same author, from the same picture of what the data looks like. When the
picture is wrong, the guard is wrong and the tests agree with it — and a green
suite reports that agreement as correctness.

Three rounds of this, on two different guards, in one day.

A close-time door keyed off a lane record's `mode`. Its tests wrote a fixture
carrying `mode: "feature"`, so the door never fired in them the way it fired in
the store. The fix changed the key to `route.lane` — chosen by reading **one**
record: the one the fixing session happened to own. That record was 2 of 54.
Forty records carry no `route` at all, so the new key silently read a route
belonging to whatever feature the session was on: a small feature was refused a
close it should have had, and a standard feature lost the door entirely. The
suite was green across the defect, the fix, and the fix's own tests, because
the fixture was rebuilt from the same single record.

The second guard was the fix to the first defect of a batch verb. Its unwind
treated one error type as one state, and force-released a claim on the path
where no claim had been taken — writing through the very ownership guard the
codebase exists to hold. Green again: the author's test drove the path the
author had in mind.

What found all three: an independent read that went to the live store and
counted. Not a smarter reviewer — a reader with no stake in the model, checking
the guard against the distribution of shapes that actually exist.

## The rule

- A guard is not proven by tests written beside it. Before shipping one, go
  **count the live store**: how many records carry the field you key on, what
  the other shapes are, which branch each real record takes. A distribution is
  evidence; one record is an anecdote, and the record your own session owns is
  the least representative one you could have picked.
- Build the fixture **from the measured distribution**, not from the example in
  front of you. If 40 of 54 records lack the field, a fixture that always has it
  tests a store that does not exist.
- A guard that can be absent has two failure directions. Test both: it must fire
  where it should, and stay silent where it must not. A test suite that only
  proves the quiet direction is how a fail-open ships green.
- Treat an error type that unions two states as a state you have not decided.
  Split it at the source, or the recovery path will pick the wrong one exactly
  when it matters.
- The fix to a guard deserves the same suspicion as the guard. Three of three
  attempts here were wrong; two of them were fixes. Route every guard change
  through an independent read before it counts as done — the author cannot
  review their own model, whatever the suite says.
