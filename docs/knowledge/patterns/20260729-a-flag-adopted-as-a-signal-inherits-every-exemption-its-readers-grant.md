---
type: bee.pattern
title: A flag adopted as a signal inherits every exemption its readers already grant
description: A flag adopted as a signal inherits every exemption its readers already grant
tags: [failure, guards, state-fields, validation-removal, plan-review]
timestamp: 2026-07-29
bee:
  id: pattern-20260729-a-flag-adopted-as-a-signal-inherits-every-exemption-its-readers-grant
  lifecycle: active
  sources: ["original feature: worker-conformance", docs/history/learnings/20260729-worker-conformance.md]
  polarity: pitfall
  critical: false
---

# A flag adopted as a signal inherits every exemption its readers already grant

You need a new signal — "this unit closed without proof", "this record is provisional". An existing
field already means something adjacent, and the door you need to arm already reads it. Reusing it
looks free.

It is not free. Every *other* reader of that field also fires, and those readers are usually
exemptions: skip this check, downgrade that refusal to a warning, defer this door. The new signal
silently grants all of them.

`worker-conformance` locked exactly this as its mechanism — mark a proofless close by setting the
existing "pending" marker — and a Gate 2 review wave found the flag short-circuits **six** refusal
sites, including the red-first tier and the ratio ceiling. Setting it as the new default would have
disarmed both in the first slice, before the negative controls that would have noticed existed. The
fix was a new field whose only power is arming the one door.

**Before adopting any existing field as a signal, grep every read site of it.** A field with N
readers grants N behaviours you did not ask for. If more than one reader exists, the answer is
almost always a new inert field — a marker that does one thing is auditable; a marker that
piggybacks is not.

The tell in a plan is a decision that names the field by the door it needs to arm, with no inventory
of what else reads it.

**Full entry:** docs/history/learnings/20260729-worker-conformance.md
