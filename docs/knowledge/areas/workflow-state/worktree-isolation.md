---
type: bee.area
title: Workflow State — isolated linked worktrees and the transactional merge-back
description: "The opt-in dispatch mode that removes Git index contention without changing the ownership primitive: validated linked pointers, one main coordination store, canonical containment proved before every write, and a merge-back that verifies committed main or preserves the worker's recovery identity."
timestamp: 2026-07-22
bee:
  id: workflow-state-worktree-isolation
  lifecycle: active
  areas: [workflow-state]
  required_context: [areas/workflow-state/overview.md]
  decisions: [worktree-isolation D1-D4 (docs/history/worktree-isolation/CONTEXT.md; logged 58c56bb6/5de1fd36/8cc1bde1/b24a2efc)]
  sources: ["worktree-isolation cells worktree-isolation-1..4 (capped traces and reports 2..4, 2026-07-16 — linked-root resolution, contained writes, dispatch attestation, transactional merge-back)", "docs/specs/workflow-state.md#B20", "docs/specs/workflow-state.md#R32", "docs/specs/workflow-state.md#R33", "docs/specs/workflow-state.md#R34", "docs/specs/workflow-state.md#R35", "docs/specs/workflow-state.md#E15", "docs/specs/workflow-state.md#E16", "docs/specs/workflow-state.md#E17", "docs/specs/workflow-state.md#E18", "docs/specs/workflow-state.md#P1"]
  authoritative_for: "workflow-state: isolated linked-worktree dispatch, containment, and transactional merge-back"
---

# Workflow State — isolated linked worktrees and the transactional merge-back

Isolation buys exactly one thing — workers stop fighting over a single Git index
— and it is careful to buy nothing else. Reservations remain the ownership
primitive, one validated main checkout remains the only coordination store, and
every step that could turn a convenience into a data-loss path (invalid linked
metadata, an escaping path, a red verification after the merge commit) fails
closed or preserves the worker's state rather than guessing.

## Behaviors & Operations

**B20 — Eligible workers may execute in isolated linked worktrees.** Trigger: an
enabled Claude multi-worker wave has at least two workers and the orchestrator
opts into isolation; shared checkout remains the default, with one explicitly
defined single-worker validation run as the sole acceptance exception. Before
dispatch, the orchestrator validates both Git pointers and captures the
control-plane attestation. A valid link keeps the physical work root and routes
all state, claims, and reservations to one main store. Missing or inconsistent
attestation makes the mode ineligible; linked-shaped invalid metadata produces
a typed refusal and never falls back to a local coordination store.

Every write-capable operation proves canonical physical containment before
logical normalization and reservation lookup. Existing targets use their
canonical location; new targets use the nearest existing ancestor. Traversal,
outside absolute paths, symlink escapes, and unresolvable targets are refused
before mutation, while owner and foreign-reservation semantics stay unchanged.

On completion, the orchestrator independently rechecks identity, symbolic ref,
common Git location, base ancestry, and that every changed path is reserved.
Only then may it begin merge-back without finalizing a commit. A conflict or
targeted red check aborts with main history unchanged. Targeted green permits
the merge commit, followed by exact full verification on committed main with
working directory, pre/post revisions, ancestry, command, and output recorded.
Unexpected post-commit red creates a non-destructive revert before any later
work and preserves worker state. Automatic cleanup requires a clean worker
checkout, green committed-main verification, and a worker revision reachable
from main; removal and branch deletion are non-force. Every other outcome
preserves recovery identity. Destructive disposal requires explicit operator
authorization plus captured status, diff, revision, reachability, and a
recovery reference or patch.

## Business Rules

- R32 — Worktree isolation removes Git index contention only; reservations
  remain the ownership primitive. Isolation is opt-in for enabled Claude
  multi-worker waves, never a new default (worktree-isolation D1).
- R33 — All linked worktrees share exactly one validated main coordination
  store. Onboarding markers are neither consent nor proof, and invalid linked
  metadata always fails closed (worktree-isolation D2).
- R34 — Same-user workers are cooperative and fallible, not security
  principals. Git metadata is consistency evidence; authoritative attestation,
  integration, verification, and cleanup remain orchestrator-owned goal checks
  (worktree-isolation D3).
- R35 — Canonical physical containment always precedes logical path
  normalization and authorization. When safe resolution is impossible,
  worktree mode is refused rather than run unguarded (worktree-isolation D4).

## Edge Cases Settled

- Linked pointers may be absolute or relative and may use supported Windows
  path forms. Corrupt, forged, outside-namespace, missing-reverse, or backlink-
  mismatched metadata fails closed. Ordinary repositories, submodules, and
  legitimate separate-Git-directory layouts keep ordinary behavior.
- Traversal, outside-main absolute paths, symlink escapes, and separator/case
  escapes are denied across every write-capable operation. A new contained
  target is authorized only through a contained existing ancestor.
- Detached/ref/identity/common-location mismatch, non-descendant revisions,
  and out-of-reservation diffs halt integration. Conflict or pre-commit red
  aborts; post-commit red reverts. Blocked, handed-off, abandoned, mismatched,
  conflicted, or red worktree state is preserved and never auto-cleaned.
- Transaction behavior is proven in deterministic temporary Git repositories
  because the live checkout's Git metadata is read-only; no live-checkout
  commit is claimed by that acceptance evidence.

## Pointers (implementation)

- Worktree isolation (B20/R32-R35): root resolution in
  `packages/bee/lib/state.mjs`; hook transport and containment in
  `packages/bee/hooks/adapter.mjs` and `packages/bee/hooks/bee-write-guard.mjs`; dispatch, attestation,
  merge-back, recovery, and disposal contracts in `skills/bee-swarming/SKILL.md`,
  `skills/bee-swarming/references/swarming-reference.md`, and
  `skills/bee-executing/references/worker-details.md`. Evidence: capped cells
  `.bee/cells/worktree-isolation-{1..4}.json`, reports
  `docs/history/worktree-isolation/reports/`, 333 passing library checks, and
  the green configured repository verify on 2026-07-16.
