---
type: bee.pattern
title: Defaults merged into a read erase the difference between absent and refused
description: Defaults merged into a read erase the difference between absent and refused
tags: [failure, state, defaults, precedence, planning]
timestamp: 2026-08-18
bee:
  id: pattern-20260818-defaults-merged-into-a-read-erase-the-difference-between-absent-and-refused
  lifecycle: active
  areas: [workflow-state]
  sources: ["uat-approval-reaches-the-door cell uad-1 and decision 8ca2378f, 2026-08-18"]
  polarity: pitfall
  critical: true
  evidence: wired
  evidence_ref: "the gate defaults are merged over every record on read, so a record that never mentioned a gate returns the same value as one that refused it — precedence by presence cannot be written against that shape"
---

# Defaults merged into a read erase the difference between absent and refused

A plan specified a precedence for finding an approval: prefer the record that
has an opinion, fall back only when it has none. It could not be built. Every
read in this store merges the record it finds over a shared set of defaults, and
those defaults stamp "not approved" onto anything silent — so a record that
never mentioned the gate and a record that explicitly refused it come back as
the same bytes. Precedence by presence would have read every silent record as a
veto, and it would have taken a passing test down with it.

What shipped instead treats the two fallback sources as equals: approved if
either says so. That is weaker than intended, and the difference is worth
naming — an explicit refusal on one source can no longer veto a stale approval
on the other.

**The rule for planning:** before writing a rule that turns on whether a field
is *present*, check what the read path does to absent fields. If it merges
defaults, presence is not observable at the decision point and the rule is
unimplementable as written, however natural it reads. Either resolve the
question against a raw read that preserves absence, give the field a third state
that means "unset", or design the rule to need only the values themselves.

**The rule for reviewing:** when an implementation quietly swaps a cascade for
an OR, or a precedence for a union, that is not a shortcut — it is usually the
data model refusing the specification. Ask which read erased the distinction
before asking the worker to try harder. And record the swap where the miners
look, with the cost it accepts stated plainly: a frozen plan is superseded by a
decision, never edited to match what shipped.
