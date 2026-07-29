---
type: bee.pattern
title: A gate keyed on an annotation misses exactly what it exists to catch
description: "The annotation is applied by the same author the gate exists to catch, so its miss-rate and the defect rate are the same number — deriving the trigger from the artifact's own wording fails toward a red build instead of toward silence."
tags: [verification, gates, derived-ground-truth, fail-closed, doctrine, tick-contract-inline]
timestamp: 2026-07-29
bee:
  id: pattern-20260729-a-gate-keyed-on-an-annotation-misses-exactly-what-it-exists-to-catch
  lifecycle: active
  sources: ["tick-contract-inline cell tci-3 (wording-derived every-turn detection, scripts/tests/test_always_loaded_rules.mjs, commit 69de4c1d, 2026-07-29)", docs/history/tick-contract-inline/CONTEXT.md T5, docs/history/learnings/20260729-tick-contract-inline.md N3, "derived-check-hardening cell dch-6 (the honest-boundary case for hardcoded seeds, commit 60f16da7)"]
  polarity: pitfall
  critical: true
---

# A gate keyed on an annotation misses exactly what it exists to catch

A new gate had to know which doctrine rules claim to apply *every turn*, so it could
fail the build when such a rule is reachable only from an on-demand reference. Two
designs were available:

| Design | Fails toward |
|---|---|
| An explicit `every-turn` marker in the doctrine source | **Silence** |
| Deriving from the rule's own wording | **A red build** |

The marker looks cleaner — unambiguous, cheap to check, no regex. It is the wrong
choice, and the reason generalises past this case:

**the annotation is applied by the same author the gate exists to catch.** An author who
correctly marks a rule as every-turn is an author who was already thinking about its
scope — and would probably have filed it correctly. The author who misfiles it is
precisely the one who forgets the marker. So the marker's miss-rate and the defect rate
are not independent; they are approximately the same number. The gate is present,
green, and blind in exactly the cases it was built for.

Wording-derivation inverts the failure direction. Its mistakes are false positives: a
red build a human clears in one step by moving the rule or rewording it. That is
recoverable. A gate that fails toward silence is indistinguishable from a passing build,
which is the one outcome you cannot audit.

The implementation stayed honest about its irreducible seed: six regexes for how English
states per-turn scope — a domain fact about wording, not a location and not a roster of
rules. Bare `every step` / `per step` were tested and **rejected** because they produced
four false positives on cross-references and procedural lists. Naming the seed and the
rejected candidates is what "derive, don't hardcode" looks like when a fully seedless
derivation is not honestly available.

**Rule.** Before building a gate, ask which way it fails and who supplies its trigger. If
the trigger is an annotation, ask who applies it — and if the answer is "the same person
whose mistake the gate catches," the gate does not work, however clean its code is.
Prefer a trigger derived from the artifact itself, accept that it will over-flag, and
choose over-flagging deliberately: one human step to clear a false positive is a real
cost, and it is the cheaper side of the trade every time. Where a seed is genuinely
unavoidable, make it a fact about the *domain* rather than about *locations*, name it in
the check's own header, and record which candidate seeds you rejected and why — a seed
that quietly grew is the same defect one level up.

See also
[[pattern-20260728-one-membership-hand-copied-six-times-has-no-owner-and-no-alarm]]
(a hand-maintained list drifting with nothing to announce it) and
[[pattern-20260729-an-advisory-check-that-is-wrong-survives-because-it-cannot-cost-anything]]
(the other way a present check delivers no coverage).
