---
type: bee.area
title: "Workflow State — the workflow record, its rebuildable projections, and plan-revision-scoped gates"
description: "The durable per-feature workflow record that is now the real unit of pipeline state, the three-transaction creation that seeds live legacy work into it, the legacy state.json/lane files (and the legacy handoff file) as mechanically rebuildable projections that never outrank the record and whose writer set is enforced by a grep audit rather than convention, the plan-revision-scoped execution gate, and the per-workflow lock every state-mutation write now routes through instead of one blanket lock."
timestamp: 2026-07-25
bee:
  id: workflow-state-workflow-records-and-projections
  lifecycle: active
  areas: [workflow-state]
  required_context: [areas/workflow-state/overview.md, areas/workflow-state/sessions-lanes-and-identity.md, areas/workflow-state/holds-and-the-coordination-lock.md, areas/worktree-parallelism/control-plane-topology.md]
  decisions: ["multisession-native D1 (workflow-first state: the workflow record becomes the unit of state; state.json/lanes become read-only compatibility projections; startFeature's lock becomes workflow:<id>, ending cross-feature contention on the single state lock — docs/history/multisession-native/CONTEXT.md, decision e1ceca12)", "multisession-native D2 (control plane / data plane split: the workflow record and every store it seeds from — sessions, claims — resolve through controlRoot, i.e. main, from any linked worktree; docs/history/multisession-native/CONTEXT.md, decision e1ceca12)", "multisession-native D7 (gates scoped to plan revision: gate approval records approved_for_plan_rev; a plan_rev bump invalidates only that workflow's execution gate)", "multisession-native advisor-digest-slice2 conditions C1-C5/F5/F7/F8 (docs/history/multisession-native/reports/advisor-digest-slice2.md — idempotent seed before any rebuild treats state.json as derived, plan-rev-effective gate formula, startFeature worker-precondition self-exclusion, one global lock order with sessions and workflow:<id> never held together, the default-path residual seam scoped and later closed)", "multisession-native D5 amendment (msn-24, advisor-digest-slice5 condition E: the projection-writer discipline this concept states for state.json/lanes is generalized and enforced for the legacy handoff projection too — rebuildHandoffProjection is the sole sanctioned writer, a grep-audit test proves the exact production writer set rather than trusting a header comment; full detail in areas/workflow-state/handoff.md)"]
  sources: ["multisession-native cells multisession-native-5..10 (workflow-store.mjs, startFeature workflow creation, state-projection.mjs, activeWorkers, plan-rev gate scoping, default-path routing; traces .bee/cells/multisession-native-{5,6,7,8,9,10}.json, commits 1e7b538, f4fe163, 1c4d45d, c435add, 2dd834f, e7f365a, 2026-07-25)", "docs/history/multisession-native/CONTEXT.md (D1, D6, D7, D8 stage 2)", "docs/history/multisession-native/reports/advisor-digest-slice2.md (conditions C1-C5, findings F5/F7/F8)", "multisession-native cells multisession-native-18a/18b/18c (state.mjs's own workflow-record call sites, then bee.mjs's dispatcher, re-rooted onto controlRootFor(root); traces .bee/cells/multisession-native-{18a,18b,18c}.json, commits 5d0ec3c, a1431448, d69d81e, 2026-07-25; see areas/worktree-parallelism/control-plane-topology.md)", "multisession-native cell multisession-native-24 (rebuildHandoffProjection reclassified as sole sanctioned writer of the legacy handoff projection; grep-audit test in test_state.mjs; trace .bee/cells/multisession-native-24.json, commit cee2d5f, 2026-07-25; advisor digest docs/history/multisession-native/reports/advisor-digest-slice5.md condition E; full detail in areas/workflow-state/handoff.md)"]
  authoritative_for: "workflow-state: the workflow record schema and module, its creation at feature start, the rebuildable state.json/lane/handoff projections and their audited writer sets, plan-revision-scoped gates, and the per-workflow write lock"
---

# Workflow State — the workflow record, its rebuildable projections, and plan-revision-scoped gates

Through `multisession-native-9`, `.bee/state.json` and `.bee/lanes/*.json` were
themselves the source of truth, serialized behind one blanket `state` lock
that every feature's mutation shared. Slice 2 of `multisession-native`
(issue #56 stage 2, D8) inverts that: a durable **workflow record** per
feature attempt is now the real unit of pipeline state, `state.json` and the
lane files become views mechanically rebuilt from it, and a mutation locks
only the one workflow it targets. This concept owns that record, its
creation, its projections, and the plan-revision scoping riding on top of it;
`sessions-lanes-and-identity.md` keeps ownership of session/lane identity
itself, and `holds-and-the-coordination-lock.md` keeps ownership of the
coordination-lock primitive this record's lock is an instance of.

## Behaviors & Operations

**The workflow record is the durable unit of pipeline state, structurally
session-free (multisession-native D1).** Trigger: any read or write of where
a feature's pipeline stands. What happens: the record lives at
`.bee/runtime/workflows/<workflow-id>/state.json` with schema `{id, feature,
phase, mode, plan_rev, gates: {context|shape|execution|review: {approved,
approved_for_plan_rev}}, summary, next_action, status, created_at}`. The
`workflow_id` is a generated `wf-<hex>` id — enforced, not just incidental,
to never equal the feature slug, because a feature can reopen or run
competing attempts across its lifetime and each needs its own identity. Reads
(`readWorkflow`, `listWorkflows`) are lock-free; creates and updates run
under that one workflow's own lock (`withWorkflowLock`, a named wrapper over
the same per-id store-lock primitive `cells.mjs`'s `cells:<id>` locks
already use). A read of a missing or unparseable record throws a typed
`WorkflowStoreError` (`WORKFLOW_MISSING`/`WORKFLOW_CORRUPT`) — mirroring
`state.mjs`'s existing `LANE_MISSING`/`LANE_CORRUPT` discipline — never a
silent default over a record that might carry real gate state; `listWorkflows`
stays fail-open for display, skipping and reporting an unreadable entry
rather than aborting the whole listing. The module that owns this record
imports only Node builtins plus the filesystem/lock utilities — never
`claims.mjs` or `state.mjs` — so it can never read a session record and can
never acquire the `sessions` lock; this is a structural guarantee proven by a
static source-scan test, not a convention to remember, and it is exactly what
makes the lock-order rule in `holds-and-the-coordination-lock.md` (a
`sessions` lock and a `workflow:<id>` lock never held together) impossible to
violate by accident from this side. What each actor observes: every reader
that once trusted `state.json`/a lane file directly now, transitively, trusts
whichever workflow record backs it; a workspace with zero workflow records
(every pre-slice-2 repo) has nothing here to read yet.

**Starting a feature creates its own workflow record, seeded so nothing
already live is erased (multisession-native D1, advisor conditions
C1/F5).** Trigger: `start-feature`, on the default path or `--as-lane`. What
happens: three separate lock transactions, none nested inside another. First,
an idempotent seed (`seedLegacyWorkflows`) runs once, before the first-ever
workflow record lands in a repo: it materializes any live legacy pipeline — a
non-idle default record, or a non-terminal lane — into its own workflow
record, mapping the legacy `approved_gates` booleans onto `{approved,
approved_for_plan_rev: null}`. It runs *before* this same call's own legacy
write, so it never re-materializes the very feature or lane this call is
about to write (the C1 advisor condition: guard against a rebuild treating
`state.json` as derived before every live pipeline has a record backing it).
Second, the unchanged legacy `state.json`/lane write happens exactly as
before slice 2. Third, the new workflow record is created under its own
`workflow:<id>` lock. Preconditions widen to match: the nonterminal-cell and
worker checks (`checkNoLiveWorkflowForFeature`/`checkNoSameFeatureClaimedCells`)
scope to live workflow records plus same-feature cells, additive to every
existing legacy check, shared by both the default and lane paths; the global
HANDOFF precondition is now scoped per-feature on the default path exactly
like the lane path already was (F5) — a different feature's pause snapshot
never blocks this start. What each actor observes: a feature start behaves
exactly as B1 (`gates.md`) already describes — atomic, all-or-nothing,
side-effect-free on refusal — with a workflow record now standing behind it
from the first moment the feature exists.

**`state.json` and the lane files are rebuildable projections, never a
second source of truth (multisession-native D1, advisor conditions
C1/F8).** Trigger: any projection rebuild — the `bee-state-sync` hook's
opportunistic refresh, or the explicit `bee state rebuild-projections` verb.
What happens: `rebuildStateProjection`/`rebuildLaneProjection`/
`rebuildAllProjections` turn a workflow record back into the legacy shape
every existing reader still expects. Zero workflow records anywhere is a pure
no-op (C1 fallback): the legacy files stay exactly as hand-written,
byte-identical, until a workflow record exists to project from — a repo that
has never touched slice 2 sees no behavior change at all. Lane projections
are fully authoritative: every lane mutation now routes through
`updateWorkflow` + `rebuildLaneProjection` rather than writing the lane file
directly — "record wins" self-heal, proven by a drift test (write a workflow
record, corrupt or delete its lane projection, rebuild, and the projection
comes back matching the record, not the other way around). The default
`state.json` projection carries one extra safety gate beyond "project the
newest active workflow": it adopts a workflow record only while the default
pipeline is *itself* idle, or (after `multisession-native-10`) while the
workflow record's own `feature` field matches the default record's current
feature — a feature-matched branch that is authoritative even while
non-idle, taking precedence over the "newest active" bootstrap heuristic —
because an unconditional rebuild would otherwise silently clobber or
misattribute a live default feature's real state with a stale or foreign
(e.g. an active lane's) record. `bee-state-sync` (the hook) performs this as
a full idempotent rebuild — cells/last_activity refresh and the D1 pipeline
fields land in one write, under the same gating — under its existing
try-once discipline (never a partial read-modify-write), and it never calls
`createWorkflow`/`updateWorkflow` itself, so the hook can never write a
workflow record, only project from one. The core invariant — delete a
projection, rebuild it, get back the same bytes — holds unconditionally for
lane projections and for the idle-`state.json` case (F8). What each actor
observes: a projection file can always be deleted and regenerated with no
information lost, because it never held information the workflow record did
not already have.

**The "record wins, projection never a second source of truth" discipline is
enforced structurally for the legacy handoff projection too, not merely
documented (multisession-native D5 amendment, msn-24).** `state.json` and the
lane files are rebuildable *because their writers are constrained* — this
concept's own module-import guarantee (R63) is one such constraint. The legacy
`.bee/HANDOFF.json` (a projection this area's own `handoff.md` concept owns in
full) generalizes the same idea one step further: rather than trusting a
header comment that `rebuildHandoffProjection` is the only writer,
`test_state.mjs` grep-audits every `.mjs` file under `lib/` plus `bee.mjs` for
the file's two mutation primitives and asserts the production writer set is
exactly `{rebuildHandoffProjection, writeHandoff C1 fallback, adoptHandoff C1
fallback}` — a fourth writer added anywhere fails the audit by name. See
`areas/workflow-state/handoff.md` for the full mailbox/legacy-file story;
this concept notes the pattern because it is the same "projection is
derived, never authoritative" contract R65 states for `state.json`/lanes,
proven here by a structural test rather than asserted in prose.

**A granted gate's effective approval is scoped to the plan revision it was
granted under (multisession-native D7, advisor condition C2).** Trigger: any
read of a projected gate boolean, or a `bee state plan-rev bump` against a
workflow. What happens: `workflowGatesToApprovedGates(gates, planRev)`
renders the boolean a reader actually sees as `approved && (approved_for_plan_rev
== null || approved_for_plan_rev === planRev)` — never the bare stored
`approved` flag. `null` (every legacy/seeded gate, and every gate D7 never
rev-scopes) stays always-effective by construction. Approving the execution
gate stamps *only* that gate's `approved_for_plan_rev` to the target
workflow's *current* `plan_rev` at the moment of approval; every other write
path that round-trips `approved_gates` through the same projection function
(`state set`, `scribing-run`) leaves `approved_for_plan_rev` on every gate it
was not asked to stamp untouched. `bee state plan-rev bump --lane <feature>`
bumps a single workflow's `plan_rev` by exactly 1 and immediately rebuilds
that lane's projection, so a `cells claim` against that lane's cells refuses
right away, citing the execution gate. The verb is lane-scoped by
construction: it refuses outright when resolution would land on the default
(non-lane) record, or when the named lane has no live workflow record. What
each actor observes: bumping workflow W1's `plan_rev` flips W1's projected
execution boolean to false and a subsequent claim against W1's cells refuses
— while a completely different workflow W2's `plan_rev`, gates, and claims
are untouched; context, shape, and review gates on either workflow are
untouched by any bump, on either workflow, ever.

**Every state-mutation write locks and routes through its own feature's
workflow record — default path included (multisession-native D1, closing
advisor condition C5's residual seam).** Trigger: `state set`, `state gate`,
`state scribing-run`, `state advisor-ref record` — on the default (non-lane)
target, exactly as the lane target already did since the record/projection
split landed. What happens: a new `writeStateRecordThroughProjection` (the
default-record sibling of the lane path's
`writeLaneRecordThroughProjection`) routes a default-target mutation through
its own live workflow record whenever one exists. The lock every one of
these verbs used to hold for its whole body — the single blanket `state`
lock, through `multisession-native-9` — is replaced by `withMutationLock`,
which locks per-workflow (`workflow:<id>`) instead. A new
`updateWorkflowAssumingLock` in the workflow-record module (a thin refactor
of the ordinary `updateWorkflow`'s body) lets these verbs' own
already-held `workflow:<id>` lock cover their whole read-validate-write body
without a same-lock nested-acquire deadlock; `updateWorkflow` itself is
unchanged for every other caller. Two narrow carve-outs stay byte-identical
to pre-slice-2 behavior on purpose: a `--feature` swap on the default record
writes `state.json` directly, because a workflow record's `feature` is
immutable identity and a swap is not a mutation *of* one workflow; and the C1
no-workflow legacy-repo fallback (a repo with zero workflow records) is
untouched. Session→workflow binding needed no schema change — it still rides
the existing `lane` field alias on the session record — and
`resolvePipeline`'s own read path needed no code change either, because it
already reflected the workflow record indirectly, through the kept-in-sync
projection file. What each actor observes: a default-record mutation and a
lane mutation for two *different* features now provably never contend on one
lock (two deterministic seam tests prove zero cross-workflow `LOCK_BUSY`, and
decoupling from both the blanket `state` lock and a different feature's own
workflow lock); a missing or corrupt target workflow record refuses loudly
(the same typed refusal as any other read), never a silent fallback to an
unlocked write.

**A workflow record and every store it seeds from resolve through main's
control plane, not the writing checkout (multisession-native D2, msn-18a-c).**
Trigger: any read or write of a workflow record, session record, or claim
made from inside a linked worktree. What happens: `state.mjs`'s own
`createWorkflow`/`readWorkflow`/`updateWorkflow`/`listWorkflows` call sites,
its `readClaim`/`adoptClaim` calls inside handoff handling, and its session/
lane reads inside `resolvePipeline` all route through `controlRootFor(root)`
instead of the writing checkout's own root — so a workflow record created
from a granted linked worktree lands at the same path a call from main
would use, and a later read from either checkout sees the identical record.
`bee.mjs`'s own dispatcher-level call sites against `claims.mjs` and
`workflow-store.mjs` (session reads, `withMutationLock`'s target resolution,
the handoff-mailbox write/adopt/show handlers, `state-projection.mjs`'s
`rebuildAllProjections`) were swept as their own standalone cell precisely so
that re-rooting `bee.mjs`'s writes alone could never desync a write-then-
rebuild pair from a still-bare-root projection read. What each actor
observes: this concept's own claims — the record's schema, its per-workflow
lock, the projection rebuild invariant — are unchanged; only *where on disk*
the record for a linked worktree lives has moved, and it now agrees with
where every other coordination store for that same checkout lives. See
`areas/worktree-parallelism/control-plane-topology.md` for the resolver
itself and the full declared plane split (which stores are control-plane vs.
workspace-local).

## Business Rules

- R62 — A workflow record's `id` is a generated `wf-<hex>` value, never the
  feature slug; `readWorkflow`/`listWorkflows` are lock-free, `createWorkflow`/
  `updateWorkflow` run under that workflow's own `workflow:<id>` lock; a
  missing or corrupt record throws typed `WORKFLOW_MISSING`/`WORKFLOW_CORRUPT`
  rather than defaulting silently (multisession-native D1).
- R63 — The workflow-record module never imports `claims.mjs` or `state.mjs`
  and can therefore never acquire the `sessions` or legacy `state` locks — a
  structural guarantee proven by a static source-scan test, not a convention
  (multisession-native D1, advisor condition C4).
- R64 — Starting a feature performs three separate, non-nested lock
  transactions — an idempotent legacy-pipeline seed, the unchanged legacy
  write, and the new workflow-record creation — and its preconditions (the
  nonterminal-cell check, the worker check, and the HANDOFF check) scope to
  live workflow records plus same-feature cells on both the default and lane
  paths (multisession-native D1, advisor conditions C1/F5).
- R65 — `state.json` and every lane file are mechanically rebuildable from
  workflow records: deleting one and rebuilding reproduces the same bytes: a
  zero-workflow-record repo is an unconditional no-op leaving legacy files
  byte-identical; where a workflow record and its projection diverge, the
  record wins (multisession-native D1, advisor condition F8).
- R66 — A gate's effective (projected) boolean is `approved && (approved_for_plan_rev
  == null || approved_for_plan_rev === plan_rev)`; only the execution gate is
  ever stamped with a `plan_rev` on approval; `bee state plan-rev bump` bumps
  and rebuilds exactly one named lane's workflow, touching no other workflow's
  `plan_rev`, gates, or claims (multisession-native D7, advisor condition C2).
- R67 — Every state-mutation write path — default and lane alike — locks and
  routes through its own feature's workflow record via `withMutationLock`,
  never the blanket `state` lock; the sole standing exceptions are a
  `--feature` swap on the default record (writes `state.json` directly,
  because identity is immutable on a workflow record) and a repo with zero
  workflow records (multisession-native D1, advisor condition C5, closed by
  `multisession-native-10`).
- R74 — Every workflow-record and session/claim call site `state.mjs` and
  `bee.mjs` make resolves through `controlRootFor(root)`, never the writing
  checkout's own root; a linked worktree's workflow record therefore always
  lands at the same path a call from main would use (multisession-native D2,
  msn-18a-c).

## Edge Cases Settled

- A repo with zero workflow records anywhere behaves exactly as it did before
  slice 2 landed, on every path: `state.json`/lane reads and writes are
  untouched, and the first `start-feature` call is what creates the first
  workflow record (multisession-native D1, C1 fallback).
- A `--feature` swap on the default record is not treated as "mutating a
  workflow" — a workflow record's `feature` field is immutable identity —
  so a swap keeps writing `state.json` directly rather than attempting to
  retarget a workflow record mid-flight (multisession-native-10).
- `bee-state-sync` (the hook) only ever reads workflow records to rebuild a
  projection; it never calls `createWorkflow`/`updateWorkflow`, so an
  opportunistic, try-once, lock-losing checkpoint can never be the write path
  that creates or corrupts a workflow record (multisession-native D1/F8).
- `bee state plan-rev bump` targeting the default (non-lane) record, or a
  named lane with no live workflow record, refuses outright rather than
  silently bumping something adjacent (multisession-native D7).

## Pointers (implementation)

- Workflow record store: `createWorkflow`/`readWorkflow`/`updateWorkflow`/
  `listWorkflows`/`withWorkflowLock`/`updateWorkflowAssumingLock` in
  `packages/bee/lib/workflow-store.mjs` (byte-mirrored to
  `.bee/bin/lib/`). 13 tests in `test_workflow_store.mjs`, red-first proven
  (constant lock name broke cross-workflow isolation; silent-default
  `readWorkflow` broke the typed-refusal tests). Evidence: trace
  `.bee/cells/multisession-native-5.json`, commit 1e7b538.
- Feature-start workflow creation: `seedLegacyWorkflows`,
  `checkNoLiveWorkflowForFeature`, `checkNoSameFeatureClaimedCells` in
  `packages/bee/lib/state.mjs`. New msn-6 concurrency test proves
  two different features' workflow-record creation never share a lock; new
  msn-6 C1 test proves a mid-flight legacy `state.json` survives as a
  workflow record after an unrelated feature's lane start and is never
  duplicated on a later start. Evidence: trace
  `.bee/cells/multisession-native-6.json`, commit f4fe163.
- Projections: `rebuildStateProjection`/`rebuildLaneProjection`/
  `rebuildAllProjections` in `packages/bee/lib/state-projection.mjs`;
  `bee-state-sync.mjs` (hook) rewritten onto a full idempotent rebuild; new
  verb `bee state rebuild-projections`. Invariant proof (delete → rebuild →
  byte-identical) in `test_state_projection.mjs`. Evidence: trace
  `.bee/cells/multisession-native-7.json`, commit 1c4d45d.
- Plan-rev-scoped gates: `workflowGatesToApprovedGates` in
  `state-projection.mjs`; `handleStateGate`'s execution-gate stamping and
  `writeLaneRecordThroughProjection`'s optional `gateStamp` param, and the new
  `state plan-rev bump` verb, in `packages/bee/bee.mjs` +
  `lib/command-registry.mjs`. Proved red-first (a `plan_rev` bump flips only
  the targeted workflow's projected execution boolean; a sibling workflow is
  untouched). Evidence: trace `.bee/cells/multisession-native-9.json`, commit
  2dd834f.
- Default-path routing through the workflow record: `writeStateRecordThroughProjection`,
  the extended `rebuildStateProjection` feature-matched idle-gate, and
  `withMutationLock` in `state.mjs`/`state-projection.mjs`; `updateWorkflowAssumingLock`
  in `workflow-store.mjs`. Two deterministic seam tests proving zero
  cross-workflow `LOCK_BUSY` and decoupling from both the `sessions` lock and
  a different feature's own workflow lock. Evidence: trace
  `.bee/cells/multisession-native-10.json`, commit e7f365a; decision recorded
  in `.bee/decisions.jsonl` at feature close (ce89afc5).
- Advisor consult record for this whole slice: `docs/history/multisession-native/reports/advisor-digest-slice2.md`
  (conditions C1-C5, findings F5/F7/F8, verdict proceed-with-conditions).
- Control-plane re-root (R74): `controlRootFor(root)` in `state.mjs`, called
  from every workflow/session/claim site this file and `bee.mjs` own; see
  `areas/worktree-parallelism/control-plane-topology.md` for the resolver,
  the declared plane split, and the msn-18 honest-block re-slice that swept
  it in. Evidence: traces `.bee/cells/multisession-native-{18a,18b,18c}.json`,
  commits 5d0ec3c, a1431448, d69d81e.
- Legacy handoff projection's audited writer set (msn-24): see
  `areas/workflow-state/handoff.md` (R77) for the full behavior; the
  grep-audit test itself lives in
  `packages/bee/tests/test_state.mjs`. Evidence: trace
  `.bee/cells/multisession-native-24.json`, commit cee2d5f.
