---
type: bee.pattern
title: "A refusal from one verb of a family does not speak for the verb the contract rests on"
description: "Four sibling verbs refused from a granted worktree, so the fifth was assumed to refuse too — and on that assumption a user-authored decision was superseded, a grant was dropped, and a defect was filed for a problem the dropped grant created. The one verb the worker contract actually depends on had been widened nine days earlier, and testing it would have cost one command."
tags: [guards, doors, worktrees, decisions, assumption, verification]
timestamp: 2026-08-13
bee:
  id: pattern-20260813-a-refusal-from-one-verb-does-not-speak-for-the-contract-verb
  lifecycle: active
  areas: [worktree-parallelism, workflow-state]
  decisions: ["sweep-at-every-door: a feature worktree that runs the bee chain stays GRANTED; the split is by role, not by store — orchestration verbs run from main, the worker edits and caps in the worktree, 2026-08-13"]
  sources: [".bee/decisions.jsonl events 884f4350, 3ccbfe9a, d7b83394 (the wrong claim, the supersession built on it, and the correction)", "docs/knowledge/areas/worktree-parallelism/control-plane-topology.md:100-117 (the finish door was widened 2026-08-04, stating 'declared tests with cwd in the worktree — the changed code is the evidence')", "backlog finding: cells finish commit-trailer check scans main HEAD instead of the current worktree (filed against a symptom only the dropped grant produced)", "cells sad-1, sad-2 (.bee/cells/archive/sweep-at-every-door/)"]
  polarity: pitfall
  critical: true
  evidence: observed
  evidence_ref: "packages/bee-rs/crates/bee/src/verbs/cells/handlers_close.rs (cells finish runs from a granted worktree: record and claim at the main store, declared tests with cwd in the worktree, holds released through hold_topology)"
  signature: sibling-refusals-generalized-onto-the-untested-contract-verb
---

# A refusal from one verb of a family does not speak for the verb the contract rests on

## What happened

Dispatching a slice inside a granted feature worktree, four verbs refused with
one identical message naming the same cause — the command reads the shared
control plane, run it from the main checkout. From four refusals sharing a
cause, the fifth verb was assumed to share it too.

That fifth verb was the one the whole worker contract rests on: the verb that
caps a unit of work and runs the declared tests. The reasoning that followed
was sound given the premise and wrong because of it — if the worker cannot cap
where it worked, then capping from the main checkout would run the tests
against the wrong tree, so the isolation must be given up to keep the proof
honest. The grant was dropped. A user-authored decision was superseded. A
defect was filed against a check that only misbehaves once the grant is gone.

The premise was false. That one verb had been widened nine days earlier,
deliberately and for exactly this reason: its record and claim resolve to the
main store while its declared tests run with the working directory in the
worktree, because the changed code is the evidence. One command would have
shown it.

## Why the reasoning felt safe

The four refusals were not noise. They shared a message, a cause, and a
remedy, and the cause named a property — reading the shared control plane —
that the fifth verb plainly has too. The generalization was not lazy; it was
supported by every observation available. What it missed is that a door's
width is a policy choice per verb, not a property derived from what the verb
touches. Sharing a cause predicts nothing about sharing a verdict when the
verdict is somebody's decision.

The failure compounded because the conclusion was acted on in the direction of
apparent caution. Dropping the grant looked like the careful move — trading
isolation for honest proof. Caution spent on a false premise is not caution;
it bought a superseded decision, a filed defect, and a worse configuration
than the one it replaced.

## The rule

Before generalizing a refusal across a family of commands, test the specific
one your contract depends on. The cost is one invocation. The tell is a
sentence of the form "these all refuse, so that one must refuse" carrying any
weight in a decision — especially a decision that supersedes someone else's,
changes a workflow's shape, or trades away a property the project chose on
purpose.

Two supporting habits, both cheaper than the recovery:

- When a conclusion is about to override a recorded decision, read the state
  layer for the area first. The widening was documented in the bundle with its
  date and its reasoning. The knowledge layer already held the refutation while
  the wrong decision was being written.
- When the corrective action is "give up an isolation property to preserve a
  proof property", treat the trade itself as the alarm. A project that built
  both usually reconciled them somewhere, and the reconciliation is worth
  finding before the trade is made.

## What it is not

This is not an argument for testing every command before every claim. The
scope is narrow and the trigger is specific: a verb whose behavior a decision
or a contract will rest on, when the evidence for its behavior is inference
from siblings rather than observation of it.
