---
type: bee.area
title: Workflow State — isolated linked worktrees and the transactional merge-back
description: "The opt-in dispatch mode that removes Git index contention without changing the ownership primitive: validated linked pointers, one main coordination store, canonical containment proved before every write, a concurrency-aware refusal for a shared nested checkout that fails closed on its own detection errors, and a merge-back that verifies committed main or preserves the worker recovery identity."
timestamp: 2026-07-26
bee:
  id: workflow-state-worktree-isolation
  lifecycle: active
  areas: [workflow-state]
  required_context: [areas/workflow-state/overview.md]
  decisions: [worktree-isolation D1-D4 (docs/history/worktree-isolation/CONTEXT.md; logged 58c56bb6/5de1fd36/8cc1bde1/b24a2efc), worktree-concurrency-guard D1(b)/D2/D3/D4/D5 (docs/history/worktree-concurrency-guard/CONTEXT.md; supersession 0ccc1cf3)]
  sources: ["worktree-isolation cells worktree-isolation-1..4 (capped traces and reports 2..4, 2026-07-16 — linked-root resolution, contained writes, dispatch attestation, transactional merge-back)", "docs/specs/workflow-state.md#B20", "docs/specs/workflow-state.md#R32", "docs/specs/workflow-state.md#R33", "docs/specs/workflow-state.md#R34", "docs/specs/workflow-state.md#R35", "docs/specs/workflow-state.md#E15", "docs/specs/workflow-state.md#E16", "docs/specs/workflow-state.md#E17", "docs/specs/workflow-state.md#E18", "docs/specs/workflow-state.md#P1", "worktree-concurrency-guard cells wcg-1/wcg-2 (capped traces and reports, 2026-07-24 — shared-nested-checkout detection primitive and write-guard wiring)", "worktree-concurrency-guard cell wcg-fix-2 (capped trace and report, 2026-07-26 — fail-closed-on-detection-error fix, review finding #2)"]
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

**B21 — A live write into a shared nested checkout is refused while another
session is concurrently active.** Trigger: a session attempts to write (Edit,
Write, or a Bash command) into a target that resolves inside a nested checkout
— a distinct git repository living inside the current one — while at least one
other session's heartbeat is live for this checkout. The write is denied,
hard and fail-closed, with no override, whenever the nested checkout is either
(a) a companion mount reached through a symlink that has never been verified
against a matching mount record, or (b) an ordinary nested checkout sitting
directly inside the current one's own directory tree (not reached through any
symlink) — unless that nested checkout is a registered git submodule of the
current checkout, which is never refused. A companion mount whose symlink has
been verified against a matching, unstale mount record is never refused by
this behavior regardless of concurrency — the verification itself is what
distinguishes a deliberately shared checkout from an unmanaged one. The
refusal names re-entering through a freshly created, dedicated worktree as the
fix; it is never worded as upgrading the current worktree in place, because
the current worktree cannot be converted to hold a verified mount after the
fact. When no other session is live, or the target is not inside any nested
checkout at all, nothing about this behavior changes today's write.

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
- R36 — A write into a nested checkout that another concurrently-live session
  can also reach is refused unless that nested checkout is either a verified
  companion mount or a registered git submodule (worktree-concurrency-guard
  D1(b)/D2). The refusal never depends on a configurable bypass switch — it is
  a permanent safety check, not an approval gate (worktree-concurrency-guard
  D5). There is no override flag; the only recovery is opening a fresh,
  properly mounted worktree (worktree-concurrency-guard D3/D4).

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
- An unrecognized symlink escaping the checkout's own tree — one that matches
  no verified companion mount and no known sibling/main checkout — is already
  denied by ordinary containment, regardless of whether any other session is
  live. The concurrency-aware refusal (B21/R36) only adds coverage for the two
  shapes containment alone does not catch: an unverified companion mount, and
  a nested checkout sitting in plain sight inside the current tree.
- A registered git submodule of the current checkout is never treated as a
  shared nested checkout, even while another session is live and even though
  it is structurally indistinguishable from an unmanaged nested checkout by
  "has its own git history" alone — the distinguishing signal is the
  submodule's own registration, not its shape.
- With no other session live, or with nothing sitting inside the checkout that
  qualifies as a nested checkout at all, every write behaves exactly as it did
  before this refusal existed — this is additive protection, not a new
  default posture.
- If the shared-checkout detection itself errors (an unreadable nested
  checkout, a broken symlink, a permission failure while resolving a path),
  the write is denied, not silently allowed — a detection failure is treated
  as evidence of risk, never as evidence of safety, since the error is most
  likely to happen during the exact race this refusal exists to stop.

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
- Concurrency-aware shared-checkout refusal (B21/R36): detection primitive
  `guards.mjs`'s `isSharedNestedCheckoutTarget` (point-check) and its
  companion-marker verification and submodule-registration exclusion helpers;
  wired into `hooks/bee-write-guard.mjs`'s dispatch, ahead of `checkWrite`.
  Evidence: capped cells `.bee/cells/wcg-1.json`, `.bee/cells/wcg-2.json`,
  reports `docs/history/worktree-concurrency-guard/reports/`, and the green
  `hooks/test_write_guard.mjs` suite (82 rows) on 2026-07-24.
