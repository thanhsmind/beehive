---
type: bee.pattern
title: "A test can assert, as intended behavior, the very escape a fix is dispatched to remove"
description: "A guard scoped to one departure path had a test asserting it never fires on any other path — the escape itself, written down as a promise — so closing the hole meant inverting that test's assertion, and a worker that treats a green test as a specification would have concluded the fix was wrong."
tags: [tests, guards, refactoring, assumptions, review]
timestamp: 2026-08-06
bee:
  id: pattern-20260806-a-test-can-assert-the-very-escape-a-fix-is-dispatched-to-remove
  lifecycle: active
  areas: [verify-pipeline, workflow-state]
  decisions: [review-p1-fixes p2-1 (invert the debt guards — the origin-phase allowlist becomes a derived guarded set and one shared debt core serves all four doors)]
  sources: ["review-p1-fixes cell p2-1, whose trace names the inverted test and quotes its original assertion: the guard is scoped to the exit from swarming, and never fires on a feature that is not in swarming", docs/history/review-p1-fixes/promote-proposals.md]
  polarity: pitfall
  critical: false
---

# A test can assert, as intended behavior, the very escape a fix is dispatched to remove

A test suite is usually read as the specification: if a change turns a green
test red, the change is suspect. That reading fails in one specific place — when
the behavior being fixed is a *hole*, and some earlier test wrote the hole down
as a feature.

The instance: a debt guard fired only on departures from one particular phase. A
test asserted exactly that, in its own words — the guard "never fires on a
feature that is not in swarming." That sentence is the escape stated as a
promise. When the guard was widened to fire on every guarded departure, that
test went red, and it went red *because the fix worked*. Closing the hole meant
rewriting the assertion to demand the refusal it had previously forbidden.

The trap is the reflex it triggers. A worker that has been told never to weaken
a test, and never to build on red, sees a red test its change caused and
concludes it has broken something. The correct move — invert the assertion — is
indistinguishable, from inside the diff, from the forbidden move: weakening a
test to make your change pass.

## The rule

- Before fixing a hole, grep the suite for tests that name the hole's shape. If
  one exists, the cell's scope includes rewriting it, and the cell text should
  say so up front — otherwise the worker meets a red it was told never to walk
  past, with no sanctioned way forward.
- The distinction to hold onto: **weakening** a test drops an assertion or
  loosens it so the change can pass. **Inverting** one replaces a wrong
  assertion with the opposite, equally strict, assertion. The first is
  forbidden; the second is the work. Say which one you did, in the trace, in the
  same words.
- A test that asserts a *negative scope* — "never fires when X" — is a
  scope decision written in test form. Treat every such assertion as a decision
  record with no decision id: it will outlive the reasoning that produced it,
  and nothing will re-derive that reasoning for you.
- When a guard's scope is an allowlist of the places it applies, prefer deriving
  the guarded set from the property that makes a place dangerous. An allowlist
  encodes today's map; a derived set survives the next place appearing.
