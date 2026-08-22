---
type: bee.pattern
title: A door placed before an older one inherits every case the older one answered
description: A new refusal placed ahead of an existing one shadows it, so the new message must be true for every case the old door used to take — including the ones its own author never had in mind
tags: [failure, refusals, guards, messages]
timestamp: 2026-08-21
bee:
  id: pattern-20260821-a-door-placed-before-an-older-one-inherits-every-case-the-older-one-answered
  lifecycle: active
  areas: [workflow-state, rust-runtime]
  sources: [".bee/cells/archive/store-reach-gaps/srg-1.json", "original feature: store-reach-gaps", "commit 303ac98e"]
  polarity: pitfall
  critical: true
  evidence: wired
  evidence_ref: "packages/bee-rs/crates/bee/tests/route_lane_targeting.rs — a_featureless_default_record_is_refused_without_inventing_a_feature_to_protect asserts the ABSENCE of the shadowed door's false sentence, not merely the presence of the new one"
---

# A door placed before an older one inherits every case the older one answered

A new guard put ahead of an existing one does not merely add a case. It
**takes** cases — every input the older door used to answer now stops at the
new one, and hears the new one's explanation instead.

The instance: a refusal was added to stop an unbound session from blind-writing
a shared record while other lanes were live. It was deliberately placed before
the older "no active feature" refusal. In the ordinary case its message was
exactly right. But it now also caught the case the older door owned — a record
carrying no feature at all — and told that caller:

> the default record carries feature "none". Writing here would overwrite that
> feature's own triage.

There is no such feature and no triage to lose. The refusal was correct; its
stated reason was false. A correct guard explained by a false sentence teaches
the next reader the wrong model of the system, and sends the operator looking
for a feature that does not exist.

## The rule

- When you place a door ahead of another, enumerate what the older door used to
  answer. Every one of those inputs is now yours.
- Branch the message, not the ordering. Shadowing is often the right design —
  the fix is a sentence that is true per branch, not a reshuffle that gives the
  case back.
- Keep every exit visible in both branches. A caller who reaches the new door
  needs the same ways out whichever branch printed.
- Test the shadowed case by asserting the ABSENCE of the old door's wording,
  not just the presence of the new. An assertion that only checks the new
  sentence passes even when the false one is still emitted beside it.
