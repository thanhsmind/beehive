---
type: bee.pattern
title: A hit list is only as complete as the search that produced it
description: "A cell scoped by a guessed grep inherits the guess: naming the list complete makes the worker stop looking, so the worker is right by its instructions while the result is wrong. Measured on herding-orchestration cell ho-14: a four-file grep left nine live references standing — seven in the very document that describes the permission posture and the runtime adapter seam the feature's riskiest decision protects — while the full-tree sweep in ho-15 found 43 files, most of them mirrors and several of them history that must not be rewritten."
timestamp: 2026-08-19
bee:
  id: pattern-20260819-hit-list-inherits-its-search
  lifecycle: active
  areas: [doctrine-layer, workflow-state]
  sources: ["capture stub 5d2a503a (herding-orchestration, captured in its worktree)", skills/bee-herding/references/operational-invariants.md, herding-orchestration cells ho-14 and ho-15]
---

The hit list is only as complete as the search that produced it, and
naming it complete makes the worker stop looking. The grep behind
cell ho-14 covered four files chosen by guess — the bootstrap script,
two docs, and the knowledge area — and the cell inherited exactly
that scope. Nine live references survived in files nobody searched,
seven of them in
skills/bee-herding/references/operational-invariants.md, the very
document that describes the permission posture and the runtime
adapter seam the feature's riskiest decision protects. The worker was
right by its instructions and the result was still wrong.

Two corrections follow, and the second is the one a bigger grep would
not give. First, search the whole tree and then subtract, rather than
searching a guessed subset and calling the result exhaustive: the
full sweep in ho-15 found 43 files, of which most were mirrors and
several were history. Second, separate live instructions from
records. Files under docs/history, docs/discovery, and plans mention
the retired script as a true statement about the past, and rewriting
them to match the present would corrupt the record. A reference is
stale only when it tells a reader to do something that no longer
works, never merely because its subject moved.

**The rule:** sweep the whole tree, then subtract — never enumerate a
guessed subset and call it exhaustive. And a reference is stale only
when it instructs a reader to do something that no longer works,
never merely because its subject moved — records are not rewritten.
