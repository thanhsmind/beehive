---
type: bee.pattern
title: "A drifted phase record is caught by the guard that counts things, never by the guard that reads state"
description: "A feature executed three cells, capped and merged them all while its phase record still said planning. Nothing that reads the phase objected — not the write guard, not dispatch, not the cells themselves. The close door caught it, because that door counts uncaptured behavior-change cells instead of asking the record what happened."
tags: [guards, state, phases, close, enforcement]
timestamp: 2026-08-14
bee:
  id: pattern-20260814-drifted-phase-caught-by-counting-guard
  lifecycle: active
  areas: [workflow-state]
  decisions: ["sweep-recovery-door promote proposal reviewed, nothing further to apply; recorded as a decision because state scribing-run is illegal from phase compounding, 2026-08-14"]
  sources: ["feature sweep-recovery-door: gate --merge approved, then three dispatches with no state set --phase swarming between them", "cells srd-1 9b02902f, srd-2 1f950a69, srd-3 8361df9a — all claimed, capped and merged under phase \"planning\"", "bee close refusal: 'Capture debt for sweep-recovery-door — close stops at the scribing-debt door: 2 behavior_change cell(s) uncaptured (srd-2, srd-3)'", "bee state scribing-run refusal: 'refused from phase planning — a scribing run records the spec sync for work that has been EXECUTED'"]
  polarity: pitfall
  critical: true
  evidence: observed
  evidence_ref: "the scribing-debt door counts cells whose trace carries behavior_change with no capture recorded, independent of the phase field"
  signature: phase-record-drifts-while-counting-guard-holds
---

# A drifted phase record is caught by the guard that counts things, never by the guard that reads state

## What happened

A feature's shape and execution gates were approved, and the orchestrator went
straight from recording that approval to dispatching workers. The step that
moves the phase record from planning to swarming was skipped.

Everything downstream worked anyway. Three cells were claimed, implemented,
tested, capped and merged. The write guard permitted every edit. Dispatch
prepared and claimed each cell without complaint. The declared test suite ran
at every cap and at the merge. Nothing in that chain consulted the phase to
decide whether the work was allowed to happen, so nothing noticed the record
was describing a different reality.

The drift surfaced twice, both times at a door that does arithmetic rather than
introspection. First the close refused, naming two behavior-change cells with
no capture recorded — a count of cell traces, not a reading of the phase. Then
the scribing stamp refused *because* of the phase, which is how the drift
finally got named: the fix for the first refusal was blocked by the second, and
the second quoted the phase out loud.

## Why the state-reading guards stayed quiet

They had nothing to object to. A phase field is a claim about where work
stands; a guard that reads it inherits whatever that claim says. When the claim
is wrong in the permissive direction — planning is *earlier* than swarming, and
the gates it gates were already approved — every check that consults it either
passes or is not reached at all. A record that lies in the direction of "less
has happened" is invisible to anything asking permission.

The counting door had no such dependency. It asks how many cells carry a
behavior change with no capture beside it. That number came from the cell
traces the work itself produced, so it stayed true while the phase field
drifted away from it.

## The rule

Where a workflow's integrity depends on a fact, derive that fact from the
artifacts the work produces, not from a field someone is supposed to update.
Phase, status and stage fields are useful for routing and for telling a human
where things stand; they are the wrong foundation for a door that must not be
walked around, because the failure mode is not someone forcing the door — it is
someone forgetting to write the field, after which the door opens politely.

Two practical consequences:

- A door that counts real things (cells, traces, uncaptured changes, unmerged
  commits) survives an orchestrator's bookkeeping mistakes. A door that reads a
  status field does not. When both are available, spend the implementation
  effort on the counting one.
- When a state field and an artifact disagree, the artifact is the evidence.
  Repair the field, and say plainly that it drifted — a silently corrected
  record teaches nobody, and the next session inherits the same gap.

## What it is not

This is not an argument against phase records. They routed this work correctly
for its whole life, and the refusal that finally named the drift was itself a
phase check doing its job. The point is narrower: a phase field is a good
narrator and a poor gatekeeper, and the difference matters exactly when someone
forgets to update it.
