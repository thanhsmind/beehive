---
type: bee.pattern
title: Bound the fix ladder when a lexical rule stands in for a semantic claim
description: Bound the fix ladder when a lexical rule stands in for a semantic claim
tags: [guards, judging, claims, text-matching, revision-rounds]
timestamp: 2026-08-28
bee:
  id: pattern-20260828-bound-the-fix-ladder-when-a-lexical-rule-stands-in-for-a-semantic-claim
  lifecycle: active
  areas: [advisor-protocol, rust-runtime]
  sources: ["slp-blind-lanes cells bln-5, bln-6, bln-7 — three independent judge rounds over one sentence-boundary rule, each broken by a new written form; traces in .bee/cells/, 2026-08-28", "slp-blind-lanes decision 79b5437b (the citation check claims provenance, never faithfulness)"]
  polarity: pitfall
  critical: false
  evidence: prose
---

# Bound the Fix Ladder When a Lexical Rule Stands In for a Semantic Claim

A rule that matches text cannot decide meaning, so a promise that it does
invites one fix per written form that escapes it. Three rounds of independent
judging broke one such promise in a row: each round found a new spelling of the
same escape — an abbreviation's dot, a bracketed abbreviation, a list
enumerator — and each fix added a rung instead of ending the climb.

What stopped the spiral was not a better rule. It was writing the bound into
the fix unit itself: *this is the last rung, and if a judge breaks it again the
answer is to reduce the claim, not to add a fourth fix.* The over-claiming
comment was corrected with the same weight as the defects, because the false
promise is what kept inviting the next rung. The claim that shipped is the
narrow one the mechanism can actually keep.

Both judges found the holes the same way: they copied the functions out of the
module, compiled them on their own, and probed real strings. Neither found
anything by reading the tests — a guard and its tests are one model, so green
proves only that the model agrees with itself.

## The second half: a fold upstream kills every case rule downstream

The same round produced the concrete lesson about where such a rule may look.
The matching pipeline folds letter case before it scans, so every later rule
that reads case has nothing left to read. A prescribed marker shape of *one to
three letters, all lower case or all upper case* could never fire its
upper-case half, and its lower-case half swallowed ordinary short words at the
end of a sentence — a batch of existing tests went red, because the control
text ends in one of them.

The shipped shape is narrower on purpose: digits, a single letter, or a short
run of the numeral letters. When a prescribed shape cannot survive the suite it
inherits, the red is information about the prescription, not about the fix.
Record the narrowing loudly, and name what the narrow shape still misses.

## The rule

- Before writing a rule that stands in for a semantic claim, write down what
  ends the ladder — the last rung, and the reduced claim that replaces it.
- Fix an over-claiming comment as a defect of the same weight as the bug. The
  claim is what invites the next rung.
- Check what an earlier stage already destroyed before writing a rule that
  depends on it.
