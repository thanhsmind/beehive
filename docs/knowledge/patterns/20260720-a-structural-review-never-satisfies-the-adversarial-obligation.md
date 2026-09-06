---
type: bee.pattern
title: A structural review never satisfies the adversarial obligation for an abuse-stopping rule
description: A structural review never satisfies the adversarial obligation for an abuse-stopping rule
tags: [process, validation, counting-rules, adversarial-review, budgets]
timestamp: 2026-07-22
bee:
  id: pattern-20260720-a-structural-review-never-satisfies-the-adversarial-obligation
  lifecycle: active
  areas: [advisor-protocol]
  sources: ["docs/history/learnings/critical-patterns.md#PAT40", "original feature: self-correcting-loop", "okf-switchover-f3 cell f3-2 judged NEEDS_REVISION 8 PASS / 2 FAIL, repaired by f3-3 (second recurrence — a free-text uniqueness gate; trace in `.bee/cells/`, 2026-07-22)"]
  polarity: pitfall
  critical: true
---

# A structural review never satisfies the adversarial obligation for an abuse-stopping rule

A claim-counting rule whose entire purpose was stopping a named abuse (a solo session
re-claiming the same cell in a loop) passed CONTEXT lock AND a thorough structural plan-check —
and still missed that exact abuse: counting "session transitions" can never see a
same-session re-claim. It was caught pre-code only by a fresh-context adversarial pass that ran
the abuse scenario against the rule (fix: distinct (claim_session, claimed_at) pairs).
**Rule:** when validating any counting/budget/limit invariant that exists to stop a named
pattern, the validating pass must include "run the exact abuse scenario against the rule as
written" as its own step — schema/consistency/freeze review does not substitute, regardless of
reviewer strength. Corollary: when two validators propose different fixes for one critical
path, CONTEXT records both and why one lost (the Δ2 inside-critical-section-vs-pre-acquire
record is the template) — an unrecorded resolution is a future re-litigation.

**Recurred 2026-07-22 (okf-switchover-f3, f3-2 → NEEDS_REVISION) — same obligation, new class: a
uniqueness gate over FREE TEXT.** The author verified an anti-fork ownership gate by trying case
and whitespace variants — the two transformations the normalizer already handled — and reported it
closed. An independent judge broke it four ways in one sitting: a trailing period, a Cyrillic
homoglyph, a non-string claim silently skipped by a `typeof !== 'string' → continue`, and an
empty subject that skipped the gate entirely and routed the write to the very default the gate
existed to prevent. Two further classes followed: pre-existing duplicate owners resolving by walk
order with no detection, and a topology no fixture had (all three were single-root, so a
divorced-product-root bug was invisible). A whole repair cell was the price.

**Rule, specialized for this class:** when a gate makes something unique over human-authored text,
the adversarial pass covers four axes, written BEFORE the implementation — (1) normalization and
confusables (NFKC, case, accents, cross-script look-alikes, trailing punctuation, whitespace);
(2) wrong shapes (non-string, array, boolean, null, empty, whitespace-only); (3) conflicts already
present in the data before the gate ran; (4) topologies the fixture set does not contain. Then
build in depth, because no string comparison can ever catch a genuine paraphrase.

**Corollary — a finding that only warns is not a backstop.** The control cited as the safety net
here already existed, as a *warning*, in a check the chain runs without `--strict`; it had never
blocked anything. Before naming any check as what catches what your gate misses, confirm it lands
in a bucket the chain actually fails on with no opt-in flag. Promote in the safe order: prove the
corpus is currently clean, then promote warning to error, then pin the clean measurement — so the
promotion never reds your own repo.
