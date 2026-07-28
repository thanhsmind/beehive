---
date: 2026-07-28
feature: foundation-fixes
categories: [workflow-state, verify-pipeline, orchestration]
severity: high
tags: [state-clobber, zombie-workflow, windows-ci, parallel-verification]
---

# foundation-fixes — feature close learnings

## What Happened

The user's verification run for the two new philosophies — parallel waves and
slice-tail test batching — over two foundation repairs. Wave 1 ran fx-1 ∥ fx-2
as a true parallel pair (scheduler-computed, wave-barrier acks); fx-3 was the
slice's single trailing test cell. Both repairs landed:
- **State-clobber killed at the root.** `STATUS_VALUES` contained `'closed'`
  with no writer — every past feature stayed `active` forever, and
  `bee-state-sync` (SubagentStop) idle-bootstrap picked the newest zombie,
  resurrecting closed features (4 incidents in one day, all during worker
  stops). Fix: `startFeature` closes outgoing workflows in the same guarded
  mutation; the picker excludes terminal-phase records (defense in depth).
  Red-first probe proved the zombie selection before, green after.
- **Windows CI timeout fixed by design, not by knob.** `test_worktree_store`
  (13 git-heavy tests, 2–4x slower on Windows, one 300→600s bump already
  spent) split at the topology/merge boundary: 4 store + 9 merge tests, both
  halves far under the ceiling, no timeout raised, no test lost.

## Findings

1. **An enum value with no writer is a latent bug, not dead code.** `'closed'`
   existed for the lifecycle the code never performed; every consumer filtered
   on it, so the gap was invisible until parallel SubagentStops multiplied the
   rebuild frequency. *Rule: for every state-machine enum value, name its
   writer; a value only ever read is a red flag to grep for.*
2. **Guards proved their worth twice in one wave.** test-economy D1 blocked
   fx-2's cap (refactor cell adding a test file — correct: planning had
   misclassified; a split IS class `test`), and the worker's block-and-escalate
   (advisor-consulted) was the right altitude call. The frozen judge then
   correctly flagged fx-3's owner-relocation deviation for review. *Rule: a
   guard refusal on green work usually means the metadata lied, not the work.*
3. **A declared verify home can be structurally wrong.** fx-3's planned suite
   (`test_workflow_store`) self-asserts it never imports `state.mjs` — the
   real owners were three other suites. *Rule: the test cell's first act is
   locating the owning suite, and extend-never-fork beats obeying a guessed
   filename.*
4. **Barrier timing matters for self-hosted fixes.** Drift #4 occurred after
   fx-1 capped but before the wave barrier synced `.bee/bin` — the live hooks
   still ran pre-fix code. *Rule: when a fix targets the vendored runtime the
   session itself executes, pay the barrier immediately after the fixing cell
   caps, not at leisure.*

## Verification of the two philosophies (the user's ask)

- Parallel: `Wave 1: fx-1, fx-2` computed by the scheduler, two live workers,
  disjoint reservations, zero conflicts; one [BLOCKED] resolved at orchestrator
  altitude without touching the sibling.
- Slice-tail batching: exactly one test-authoring cell for a 2-fix slice; the
  net (close-chain, zombie-never-picked, rebuild-preserves, registry mapping)
  landed as 3 suite extensions, 15+44+26+21 green; wave-close impacted run
  31/31 green at 16.5s under concurrency 12.
