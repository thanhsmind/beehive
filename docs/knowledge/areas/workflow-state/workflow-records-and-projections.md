---
type: bee.area
title: "Workflow State — the workflow record, its rebuildable projections, and plan-revision-scoped gates"
description: "The durable per-feature workflow record that is now the real unit of pipeline state, the three-transaction creation that seeds live legacy work into it, the legacy state.json/lane files (and the legacy handoff file) as mechanically rebuildable projections that never outrank the record and whose writer set is enforced by a grep audit rather than convention, the plan-revision-scoped shape and execution gates (widened from execution alone once the two approve together), the per-record projection lock — one lock per record actually written (workflow:<id>, 'state' for state.json, lane:<feature> for a lane projection), under a single global acyclic acquisition order, that every writer of a projection record now shares instead of a lock scoped to whichever code path happens to be nearby, the persisted per-gate approval record (state/actor/at/reason/bypass_level) that replaced the bare boolean, and the workflow's own persisted run_state exposed alongside it in bee status --json, and the waiting_on mark beside run_state naming what the agent asked the human and is still waiting on, ending three live ways, session-scoped so it exists even with no feature active, and readable through bee status --json/orient/the text renderer/the session preamble/the post-compaction capsule plus a CLI setter/clearer (bee status --brief deliberately excluded, a frozen minimal surface)."
timestamp: 2026-08-14
bee:
  id: workflow-state-workflow-records-and-projections
  lifecycle: active
  areas: [workflow-state]
  required_context: [areas/workflow-state/overview.md, areas/workflow-state/sessions-lanes-and-identity.md, areas/workflow-state/holds-and-the-coordination-lock.md, areas/worktree-parallelism/control-plane-topology.md]
  decisions: ["multisession-native D1 (workflow-first state: the workflow record becomes the unit of state; state.json/lanes become read-only compatibility projections; startFeature's lock becomes workflow:<id>, ending cross-feature contention on the single state lock — docs/history/multisession-native/CONTEXT.md, decision e1ceca12)", "multisession-native D2 (control plane / data plane split: the workflow record and every store it seeds from — sessions, claims — resolve through controlRoot, i.e. main, from any linked worktree; docs/history/multisession-native/CONTEXT.md, decision e1ceca12)", "multisession-native D7 (gates scoped to plan revision: gate approval records approved_for_plan_rev; a plan_rev bump invalidates only that workflow's execution gate — scope later widened by validation-diet D15)", "validation-diet D2/D14/D15 (the standalone execution gate folds into Gate 2 as one merged approval via `bee state gate --merge`, flipping `shape` and `execution` together; the merge inherits the high-risk advisor-consult precondition D14 previously guarding execution alone; the merge also stamps `approved_for_plan_rev` on both fields together so a later bump can never leave the merged approval half-revoked — docs/history/validation-diet/CONTEXT.md, cell vd-3, 2026-07-28)", "multisession-native advisor-digest-slice2 conditions C1-C5/F5/F7/F8 (docs/history/multisession-native/reports/advisor-digest-slice2.md — idempotent seed before any rebuild treats state.json as derived, plan-rev-effective gate formula, startFeature worker-precondition self-exclusion, one global lock order with sessions and workflow:<id> never held together, the default-path residual seam scoped and later closed)", "multisession-native D5 amendment (msn-24, advisor-digest-slice5 condition E: the projection-writer discipline this concept states for state.json/lanes is generalized and enforced for the legacy handoff projection too — rebuildHandoffProjection is the sole sanctioned writer, a grep-audit test proves the exact production writer set rather than trusting a header comment; full detail in areas/workflow-state/handoff.md)", "state-phase-lock-race D1-D4/D9-D13 (GH #70: the lost-update race between the state-sync hook's 'state' hold and the CLI's workflow:<id>-only hold is closed by making every writer of a projection record share the lock scoped to that exact record, under one global acyclic order workflow:<id> -> 'state' -> lane:<feature>; D13 supersedes D1/D2's blanket 'state' wrap of the whole workflow branch, which mis-scoped lane mutations onto a lock they never needed and turned a live invariant red — docs/history/state-phase-lock-race/CONTEXT.md, decision 61e21a42-39b2-4f8a-bcb8-2a4d99f00154)", "state-phase-lock-race D9 (advisor-found pre-existing inversion: handleStatePlanRevBump held 'state' then called the self-locking updateWorkflow, violating the canonical order; repaired to resolve the workflow first and use updateWorkflowAssumingLock inside the ordered locks — decision cde492e3-800e-4a29-b574-b65cb06aabd7)", "workflow-lifecycle D1/D2 (docs/history/workflow-lifecycle/promote-proposals.md — one creation seam owns workflow-record creation, and zombie records are retirable through a verb instead of by hand)", "traceable-runs D3 (docs/history/traceable-runs/CONTEXT.md, 2026-08-14 — a gate stops being a bare boolean and becomes a record: state/actor/at/reason/bypass_level, persisted not derived, additive over the existing approved boolean)", "traceable-runs D2 (gate_bypass decides whether a run halts, never whether the approval record exists; an auto-approval writes actor:auto plus the bypass level and reason)", "traceable-runs D4 (cell and feature/workflow each get a real, persisted status vocabulary including an explicit waiting state, stored not computed at read time — this concept implements the workflow-record half as run_state; cell status keeps its five existing values)", "traceable-runs D7 (the dashboard itself is out of scope; this feature delivers the persisted data and the CLI/JSON surface a dashboard would read)", "awaiting-human D1 (origin: traceable-runs D8, 8cb30970 — awaiting-approval covers every moment the agent is waiting on the human, not only a pending gate; one waiting_on mark beside run_state, session-scoped, closed on kind, non-empty subject/session; docs/history/awaiting-human/CONTEXT.md)", "awaiting-human D2 (c6b52383 — the waiting mark ends three live ways: the UserPromptSubmit hook clears it best-effort on any human message, the agent may clear it explicitly, and stale expiry reuses the dual-condition rule cells/claims.rs already applies)", "awaiting-human D3 (93f164b1 — a question asked with no active feature still records the wait on the default state record; session-scoped first, feature-scoped only when a feature is live)", "awaiting-human D4 (stale expiry never clears on age alone — the owning session heartbeat must also be stale, reusing cells/claims.rs's proven dual-condition rule rather than a second staleness semantics)", "awaiting-human D5 (no dashboard, UI, or web surface — same boundary as traceable-runs D7; this feature delivers the persisted waiting_on state and the CLI/JSON that exposes it)", "merge-closes-the-lane D3 (500fa2f9, 2026-08-18 — bee state waiting-on clear accepts a feature whose workflow record is already closed by falling back to the newest record regardless of status, and clears the LANE PROJECTION's waiting_on/run_state pair, run_state only when it reads awaiting-approval; only a feature with no workflow record at all still refuses; bee state waiting-on set never widens, its refusal stays byte-identical; extends waiting-on-pair-clear f9fd9d46, which had fixed the same pair rule only for the default .bee/state.json record; commit a4d5ca90)"]
  sources: ["multisession-native cells multisession-native-5..10 (workflow-store.mjs, startFeature workflow creation, state-projection.mjs, activeWorkers, plan-rev gate scoping, default-path routing; traces .bee/cells/multisession-native-{5,6,7,8,9,10}.json, commits 1e7b538, f4fe163, 1c4d45d, c435add, 2dd834f, e7f365a, 2026-07-25)", "docs/history/multisession-native/CONTEXT.md (D1, D6, D7, D8 stage 2)", "docs/history/multisession-native/reports/advisor-digest-slice2.md (conditions C1-C5, findings F5/F7/F8)", "multisession-native cells multisession-native-18a/18b/18c (state.mjs's own workflow-record call sites, then bee's dispatcher, re-rooted onto controlRootFor(root); traces .bee/cells/multisession-native-{18a,18b,18c}.json, commits 5d0ec3c, a1431448, d69d81e, 2026-07-25; see areas/worktree-parallelism/control-plane-topology.md)", "multisession-native cell multisession-native-24 (rebuildHandoffProjection reclassified as sole sanctioned writer of the legacy handoff projection; grep-audit test in test_state.mjs; trace .bee/cells/multisession-native-24.json, commit cee2d5f, 2026-07-25; advisor digest docs/history/multisession-native/reports/advisor-digest-slice5.md condition E; full detail in areas/workflow-state/handoff.md)", "state-phase-lock-race cells splr-1/splr-2/splr-3 (per-record lock scoping, the handleStatePlanRevBump order repair, re-vendoring into .bee/bin/, and the multi-process proof in test_state_projection_race.mjs; commits e787819a, 73eadc5f, ebc68f04, 2026-07-27; full suite verified green independently by the orchestrator, 109 suites)", "docs/history/state-phase-lock-race/CONTEXT.md (root cause: the state-sync hook holds 'state', CLI mutation verbs with a live workflow record hold workflow:<id> ONLY, and two more writers held nothing at all)", "docs/history/state-phase-lock-race/reports/advisor-consult.md (fable, 2026-07-27, PROCEED WITH CHANGES: refuted the plan's own claim that no 'state' -> workflow:<id> edge existed)", "workflow-lifecycle cells wl-1/wl-2 (one idempotent creation seam per feature, plus the list/close verb with its three exclusive modes; docs/history/workflow-lifecycle/promote-proposals.md, 2026-08-05)", "traceable-runs cells trun-1/trun-2/trun-3/trun-7 (traces .bee/cells/trun-{1,2,3,7}.json, capped 2026-08-14 — persisted gate record, bee state gate's actor/bypass/reason flags, bee status --json gate_records exposure, and run_state; docs/history/traceable-runs/CONTEXT.md, plan.md)", "awaiting-human cells ah-1/ah-2/ah-3/ah-4 (traces .bee/cells/ah-{1,2,3,4}.json, capped 2026-08-14, commits b306201f/39facee4/fe0b8885/00c67cb6 — the waiting_on mark on the workflow record and the default state record, the widened run_state derivation, the three clearing paths, the five reporting surfaces, and the bee state waiting-on set/clear CLI verbs; docs/history/awaiting-human/CONTEXT.md, plan.md)", "awaiting-human ah-2 rework (commit b7b422f9, 2026-08-14 — the human-message clearing path proved through the real UserPromptSubmit hook entry point (pub fn run), not only the inner store function: one test sets a live mark, runs the hook end to end, and asserts the mark is gone; a second makes the clear's own write fail and asserts the hook still resolves natively and the turn is never taken down)", "awaiting-human ah-3 rework (commit c5dc3d70, 2026-08-14 — full repo-wide census of run_state/waiting_on/phase-and-gate reporting surfaces, written down in docs/history/awaiting-human/reports/ah-3-rework.md: a sixth candidate, the post-compaction capsule (hooks/compaction.rs build_compact_capsule), was found already reading waiting_on but not yet naming it, and is wired in this rework reusing session_preamble::waiting_on_note; a seventh candidate, bee status --brief, is deliberately left unwired as a recorded deviation — status-diet D1/D2's frozen 7-key minimal surface)"]
  authoritative_for: "workflow-state: the workflow record schema and module, its creation at feature start, the rebuildable state.json/lane/handoff projections and their audited writer sets, plan-revision-scoped gates, and the per-record write lock order"
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
`test_state.mjs` grep-audits every `.mjs` file under `lib/` plus `bee` for
the file's two mutation primitives and asserts the production writer set is
exactly `{rebuildHandoffProjection, writeHandoff C1 fallback, adoptHandoff C1
fallback}` — a fourth writer added anywhere fails the audit by name. See
`areas/workflow-state/handoff.md` for the full mailbox/legacy-file story;
this concept notes the pattern because it is the same "projection is
derived, never authoritative" contract R65 states for `state.json`/lanes,
proven here by a structural test rather than asserted in prose.

**A granted gate's effective approval is scoped to the plan revision it was
granted under, and the merged approval extends that scope to both its fields
together (multisession-native D7, advisor condition C2; scope widened by
validation-diet D2/D15, cell vd-3).** Trigger: any read of a projected gate
boolean, or a `bee state plan-rev bump` against a workflow. What happens:
`workflowGatesToApprovedGates(gates, planRev)` renders the boolean a reader
actually sees as `approved && (approved_for_plan_rev == null ||
approved_for_plan_rev === planRev)` — never the bare stored `approved` flag —
applied generically to every gate, with no per-gate exception in the
function itself. `null` (every legacy/seeded gate, and every gate no
approval path ever rev-scopes) stays always-effective by construction.
Context and review are never rev-scoped by any approval path that exists
today, so they stay always-effective once granted. Shape and execution are
the two fields a plan revision can scope, and which of them actually gets
stamped depends on which door approved them: the standalone `--name <gate>`
path — still the only way to approve context or review, and still available
for shape or execution individually — stamps *only* the one gate it names,
exactly as it did before the merge existed. The merged `--merge` path
(validation-diet D2) — the way `planning`'s Gate 2 is now approved as one
question — stamps `approved_for_plan_rev` on BOTH `shape` and `execution`
together at the moment of approval (validation-diet D15), so the two fields
one merged question covers can never end up half-revoked: a later bump
always finds them either both still current or both stale, never one of
each. Every other write path that round-trips `approved_gates` through the
same projection function (`state set`, `scribing-run`) leaves
`approved_for_plan_rev` on every gate it was not asked to stamp untouched.
`bee state plan-rev bump --lane <feature>` bumps a single workflow's
`plan_rev` by exactly 1 and immediately rebuilds that lane's projection, so a
`cells claim` against that lane's cells refuses right away, citing whichever
of shape/execution the bump just invalidated. The verb is lane-scoped by
construction: it refuses outright when resolution would land on the default
(non-lane) record, or when the named lane has no live workflow record. What
each actor observes: bumping workflow W1's `plan_rev` flips every gate on W1
that was stamped for the pre-bump revision to projected-ungranted — both
shape and execution together when W1's Gate 2 was approved via `--merge`,
execution alone when it was approved through the standalone path — and a
subsequent claim against W1's cells refuses either way; a completely
different workflow W2's `plan_rev`, gates, and claims are untouched; context
and review on either workflow are untouched by any bump, on either workflow,
ever.

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

**The projection lock is scoped per record, not per code path, under one
global acyclic order (state-phase-lock-race D1-D4/D9-D13, GH #70).** Trigger:
any write of `.bee/state.json` or of a lane projection
(`.bee/lanes/<feature>.json`), from any of the writers that can reach one:
the `bee-state-sync` hook's opportunistic rebuild, every CLI mutation verb
routed through `withMutationLock`, `start-feature`'s post-`startFeature`
projection rebuild, `state rebuild-projections`
(`rebuildAllProjections`), and `state plan-rev bump`. What happens: `'state'`
guards `.bee/state.json` and nothing else; `lane:<feature>` guards
`.bee/lanes/<feature>.json` and nothing else — each writer of a record
acquires the lock for the record it actually writes, never a lock scoped to
"the branch this write happens to sit inside." The single global order is
`workflow:<id>` → `'state'` → `lane:<feature>`; a given call takes only a
prefix of that chain (e.g. the hook takes `'state'` alone with no workflow
lock at all), and no production path anywhere takes any two of these names in
the reverse order. This closes GH #70's root cause — the hook holding
`'state'` while a live-workflow CLI mutation held `workflow:<id>` only, so
neither writer's read-modify-write of `.bee/state.json` excluded the other —
and it closes the two writers the original audit found holding nothing at
all (`start-feature`'s post-write rebuild, `rebuild-projections`). It
supersedes `splr-1`'s first attempt, which wrapped the *entire* `if (wf)`
branch of `withMutationLock` — default-record **and** lane mutations alike —
in `'state'`: a lane mutation never writes `.bee/state.json`
(`state.mjs:1704` writes only `lanePath`), so that wrap protected nothing for
lanes while it forced every lane mutation to queue behind unrelated
default-record work, turning `test_cli_state.mjs`'s live invariant — a lane
mutation with a live workflow record never needs the shared `'state'`
lock — from near-instant to the ~5 s lock timeout (D12/D13; see
`pattern-20260727-a-lock-scoped-to-the-wrong-record-buys-nothing`).
`rebuildStateProjection`/`rebuildLaneProjection` themselves stay
caller-serialized and acquire nothing internally — unchanged from D3's
original rule — because `lock.mjs` is non-reentrant and the hook already
holds `'state'` when it calls `rebuildStateProjection`. What each actor
observes: a session mutating `.bee/state.json` is excluded from every other
writer of `.bee/state.json`, including the hook, with no lost update
possible; a session mutating lane F's projection is excluded from every
other writer of lane F's projection, and is never blocked by, nor blocks,
an unrelated default-record write; two sessions on two different lanes still
never contend on either lock at all.

**A pre-existing lock-order inversion in `handleStatePlanRevBump` is repaired
to the same canonical order (state-phase-lock-race D9, advisor-found).**
Trigger: `state plan-rev bump` against a lane with a live workflow record.
What happens: this handler previously held `withStoreLock(root, 'state', …)`
and, inside that closure, called the self-locking `updateWorkflow` — which
itself acquires `workflow:<id>` — producing the one live `'state'` →
`workflow:<id>` edge the plan's own feasibility claim had said did not exist
anywhere in the repo (refuted by the `fable` advisor consult, verified
independently against `bee:2915-2950` and
`workflow-store.mjs:385-390`). Left unrepaired, adding this feature's new
`workflow:<id>` → `'state'` edge elsewhere would have produced a genuine
lock-order inversion the moment `plan-rev bump` on lane F raced a `state
set`/`state gate` on the same lane F: session A holds `workflow:F`, waits on
`'state'`; session B holds `'state'`, waits on `workflow:F` — not a
permanent hang (`lock.mjs` is timeout-bounded and neither holder is
stale-eligible) but a deterministic ~5 s dual `LOCK_BUSY` refusal on both
sides. The handler now resolves the target lane's workflow record *first*,
outside any lock, then acquires `withWorkflowLock(ctrlRoot, wf.id, () =>
withStoreLock(root, 'state', body))` and uses `updateWorkflowAssumingLock`
inside that nested hold instead of the self-locking `updateWorkflow` — the
same canonical `workflow:<id>` → `'state'` order every other writer now
uses, with no self-deadlock because the workflow lock is acquired exactly
once. What each actor observes: `plan-rev bump` behaves identically from the
caller's side (same refusals, same bumped `plan_rev`, same rebuilt
projection); the inversion that would have reintroduced GH #70-shaped
contention under this feature's own fix is gone before the fix ships.

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
`bee`'s own dispatcher-level call sites against `claims.mjs` and
`workflow-store.mjs` (session reads, `withMutationLock`'s target resolution,
the handoff-mailbox write/adopt/show handlers, `state-projection.mjs`'s
`rebuildAllProjections`) were swept as their own standalone cell precisely so
that re-rooting `bee`'s writes alone could never desync a write-then-
rebuild pair from a still-bare-root projection read. What each actor
observes: this concept's own claims — the record's schema, its per-workflow
lock, the projection rebuild invariant — are unchanged; only *where on disk*
the record for a linked worktree lives has moved, and it now agrees with
where every other coordination store for that same checkout lives. See
`areas/worktree-parallelism/control-plane-topology.md` for the resolver
itself and the full declared plane split (which stores are control-plane vs.
workspace-local).

**A record is created through one seam, and closed through one verb
(workflow-lifecycle, cells wl-1/wl-2, 2026-08-05).** Trigger: starting a
feature, and later retiring records that outlived their work. What happens on
creation: exactly one seam answers "make sure this feature has a record," and it
is idempotent by feature — asked twice, it leaves the existing live record
untouched rather than minting a second one, so a feature can never end up with
two live records competing to describe it, and a restart is never a duplicate.
An empty feature name is refused outright rather than creating an anonymous
record. What happens on retirement: a listing verb shows every record — its id,
feature, status, phase, and creation time, newest first — and a closing verb
retires them, under exactly one of three modes, never two at once: by feature,
by id, or everything except the caller's own active feature. The all-but-active
mode refuses when no active feature can be resolved, because with nothing to
keep it would silently degrade into *close everything*, including live work.
What each actor observes: zombie records left by interrupted sessions are
visible and retirable without hand-editing the store, and the record for the
work actually in flight is protected from every sweep — only naming its id
explicitly can close it, which is the deliberate escape.

**A gate entry stops being a bare boolean and becomes a persisted record —
`state`, `actor`, `at`, `reason`, `bypass_level` — kept equal to the
existing `approved` boolean by construction, never a second source of truth
(traceable-runs D3, trun-1, 2026-08-14).** Trigger: any read or write of a
workflow record's per-gate entry. What happens: each gate entry on
`{context, shape, execution, review}` gains five fields: `state` (one of
`pending`/`approved`/`rejected`, persisted rather than derived — the whole
point being that "awaiting approval" now survives a restart, where before it
was indistinguishable from "gate never asked"), `actor` (`user`/`auto`, or
`null` for a gate never explicitly stamped by this mechanism), `at` (ISO
timestamp or `null`), `reason` (free text or `null`), and `bypass_level`
(`off`/`normal`/`full`/`total`, or `null`). The existing `approved: bool`
stays and is kept equal to `state == "approved"` by every one of the FOUR
paths that author a gate entry: `default_gate_entry` (a brand-new record —
every gate reads `pending` for free, with no seeding code needed at
`start-feature`, see below), `merge_gates` (the patch overlay a `--merge`
approval spreads over the base), `gates_patch_from_record` (the real author
behind `bee state gate` — it builds each touched entry from an empty map,
so a stale `state` field would otherwise survive while `approved` flips),
and `legacy_gates_to_workflow_gates` (the C1 seed-from-legacy-boolean
backfill). An entry written in the OLD shape — no `state` field at all,
every pre-trun-1 record — reads back deriving `state` from its existing
`approved` boolean, so an existing repo never mis-gates on upgrade. An
unknown `state` or `actor` value refuses with a typed error and writes
nothing, mirroring the existing `WORKFLOW_MISSING`/`WORKFLOW_CORRUPT`
discipline. **One divergence is intended and stays intended, now pinned by
a test:** the RECORD entry's `approved` stays `true` after a `plan-rev
bump` invalidates it — only the *projected* boolean
(`workflowGatesToApprovedGates`, R66 above) reads `false`, because
plan-rev staleness is a projection-layer fact, not a record-layer one; the
equivalence invariant above is scoped to the record entry, never to the
projection.

**`bee state gate` now records WHO approved a gate, under WHAT bypass
level, and WHY (traceable-runs D2, trun-2, 2026-08-14).** Trigger: any
`bee state gate` call, standalone or `--merge`. What happens: three new
optional flags — `--actor` (`user`/`auto`, default `user`), `--bypass-level`
(`off`/`normal`/`full`/`total`), `--reason` (free text) — write onto the
touched gate entry/entries (both `shape` and `execution` under `--merge`)
via a second, still-lock-held `update_workflow_assuming_lock_with` patch,
issued right after `write_through_projection` lands `approved`/`state` on
the legacy `approved_gates` projection (which never carried `actor`/
`reason` and gets nothing new). Per D2, `--actor auto` is REQUIRED to carry
both `--bypass-level` and `--reason` — a call that omits either refuses
outright, zero mutation, so a bypassed run can never end up LESS traceable
than a stopped one; a plain `--approved false` keeps stamping
`gate_revoked_at` exactly as before and additionally sets `state:
"rejected"`. What each actor observes: `gate_bypass` no longer decides
whether an approval record exists, only whether the run halts at it — the
record is identical either way, differing only in `actor`/`bypass_level`/
`reason`.

**A brand-new workflow record already seeds every gate `pending`, with no
change needed to `start-feature` itself (traceable-runs trun-1, correcting
the plan's own assumption).** `default_gates()` in `state_group/policy.rs`
(the seam the plan pointed at) resets only the PROJECTED
`.bee/state.json` `approved_gates` booleans, never the D3 record's new
fields. The real seam is `default_gate_entry` in
`workflow_store/record.rs`: `ensure_workflow_record_for_feature`'s primary
path (`run_start_feature`) passes `gates: None`, so a brand-new record's
every gate already reads `state: "pending"` for free, the moment the
record is created — starting a feature is what puts the run into
`awaiting-approval` (D1), without a caller ever typing a state.

**The workflow record gains a persisted `run_state` a reader can display
without re-deriving it (traceable-runs D4, trun-7, 2026-08-14).** Trigger:
any create or update of a workflow record. What happens: `run_state` — a
closed vocabulary, `shaping`/`awaiting-approval`/`running`/`blocked`/
`done` — is derived fresh (`derive_run_state`, never taken from a caller's
patch — `check_patch_run_state` accepts only a value already matching what
the derivation would produce) from the record's own `status`, its D3 gate
entries, and a live cell-count scan of the same feature, and WRITTEN on
every `create_workflow`/`update_workflow_assuming_lock_with` call —
persisted, not computed at read time, so it survives a restart the same
way `state` does on a gate entry. `awaiting-approval` is the value whenever
a gate entry reads `state: "pending"` and no later gate has been approved
— the state the whole feature exists to make visible. An unknown value
refuses with a typed error, the same discipline as an unknown gate `state`.
**CRITICAL, and the reason this is stated explicitly:** `run_state` reaches
`.bee/state.json` only because `apply_workflow_d1_fields`
(`workflow_store/projections.rs`) was extended to carry it — that function
otherwise copies only `phase`, `feature`, `mode`, `approved_gates`,
`summary`, `next_action`, so a field added to the record and NOT added
there projects as permanently absent, and the byte-identical-rebuild proof
(R65) would pass vacuously while the field never actually reached a
reader. No new cell `status` value was introduced: the five existing
values (`open`/`claimed`/`capped`/`blocked`/`dropped`) still cover the
cell side of waiting/running/blocked/done, and a reader tells the two
vocabularies apart without re-deriving either.

**`bee status --json` exposes the full per-gate record beside the
projected booleans it already showed, and now `run_state` too
(traceable-runs D7 boundary, trun-3, 2026-08-14).** Trigger: any `bee
status` call. What happens: `build_gate_records` (`status_full/build.rs`)
reads each gate's `state`/`actor`/`at`/`reason`/`bypass_level` from the
LIVE WORKFLOW RECORD — never the projection — under the key
`gate_records`, alongside the pre-existing `gates` key (the projected
booleans, unchanged in name, type, and meaning: this is additive-only,
because the JSON shape is a published contract, `docs/07-contracts.md`).
A gate never approved now reads as `gate_records.<name>.state ==
"pending"`, distinguishable from a rejected gate, where before both were
the same indistinguishable `false`. `run_state` rides beside it at the top
level. The text renderer prints one short human line per non-approved
gate. This feature's own D7 scope line holds exactly here: the data and
the read surface are delivered; no dashboard is built on top of it.

**A `waiting_on` mark rides beside `run_state`, on the workflow record and on
the default `.bee/state.json` record alike (awaiting-human D1/D3, ah-1,
2026-08-14; kind widened to three values by auto-wait-mark D3, 2026-08-18).**
Trigger: the agent asks the human something — a gate or a free-form
question — and has not yet received an answer, or (the third kind) an
ordinary turn ends and control returns to the human with nothing owed. What
happens: `build_waiting_on(kind, subject, session)` validates a closed
three-value `kind` (`gate`/`question`/`turn-end`), a non-empty `subject`
(the gate name, the question text, or — for `turn-end` — the turn's last
line), and a non-empty `session` (the owning session id, which is
what D4's stale-expiry heartbeat check reclaims against), stamps `asked_at`,
and refuses with a typed error — writing nothing — on an unknown kind or an
empty subject/session, the same `WORKFLOW_MISSING`/`WORKFLOW_CORRUPT`
discipline `run_state` itself already uses. Per D3 the setter works with NO
active feature: `set_default_state_waiting_on` writes the mark straight onto
the default record for exactly the case the user named (a question asked
before any feature exists), and `set_workflow_waiting_on` writes it onto a
live feature's workflow record. `run_state` (R109, below) widens to read
`awaiting-approval` whenever a live mark exists OR the pre-existing
pending-gate condition holds — one state, two sources, never a conflict when
both hold at once. What each actor observes: a mark set with no feature
active is immediately readable, and setting one makes `run_state` read
`awaiting-approval` with no gate pending at all.

**The mark ends three independent, live ways — never only the agent's own
hand (awaiting-human D2/D4, ah-2, 2026-08-14).** Trigger: the human sends any
message, the agent clears the mark explicitly after acting on the answer, or
the mark's age crosses the stale threshold. What happens: the
`UserPromptSubmit` hook (`hooks/prompt_context.rs`) clears both the default
record's mark and the current feature's workflow-record mark, best-effort,
inside its own failure boundary — a clearing error is logged and never fails
the hook or the turn (the same try-once discipline B20 already documents for
this hook's other opportunistic side jobs). `clear_workflow_waiting_on`/
`clear_default_state_waiting_on` are no-ops, not errors, when no mark is
live. Stale expiry (`reap_stale_waiting_on`) reuses `cells/claims.rs`'s
proven dual-condition rule rather than inventing a second staleness
semantics: a mark expires only when its age is past the threshold AND its
owning session's heartbeat is also stale — age alone never clears it, so a
human who simply takes a long time to answer never has their live wait
silently erased. What each actor observes: sending any message clears a live
wait even if the agent forgets to; a session that dies mid-question still
self-clears once its heartbeat goes stale; a session that is merely slow
never loses its mark.

**Clearing a recordless mark clears the pair, not half of it
(waiting-on-pair-clear, wpc-1, 2026-08-16).** Trigger: the explicit clear
runs on the default record (no workflow record exists). What happens: the
setter on that layer wrote TWO fields — the mark and
`run_state: "awaiting-approval"` — so the clear nulls both; before this
rule the clear nulled only the mark and `run_state` read
`awaiting-approval` forever, which a dashboard displayed as a stuck
approval badge after the wait had ended. The `run_state` reset is guarded
on that exact value: a `run_state` any other path derived (a workflow
record's projection) is never touched by this clear. What each actor
observes: after a set-then-clear with no feature active, a reader of the
default record sees no mark and no waiting state.

**Clearing a feature-scoped mark now also reaches the LANE PROJECTION's own
pair, and works even after the feature's workflow record has already closed
(merge-closes-the-lane D3, 500fa2f9, extends waiting-on-pair-clear's
f9fd9d46).** Trigger: `bee state waiting-on clear` resolves to a feature whose
workflow record already reads `closed`. What happens: resolution now falls
back to the newest record for that feature regardless of its status — the
narrower resolver used to refuse a feature whose only record was closed — and
the clear then nulls the LANE PROJECTION's `waiting_on` and `run_state`
fields together, `run_state` only when it currently reads
`awaiting-approval`; a `run_state` of `done`, `shaping`, or any other
record-derived value is left untouched, the same guard the pair-clear rule
above already applies. This closes a gap that rule left open: it fixed the
pair on the default `.bee/state.json` record only, but the stuck
`run_state: awaiting-approval` actually lived on the LANE PROJECTION file,
which stops syncing once the workflow itself is no longer live. Only a
feature with no workflow record at all still refuses; the narrower
"status !== closed" clause is dropped from that refusal's wording.
`bee state waiting-on set` never widens its own resolution the same way —
its refusal stays byte-identical, since arming a new wait on an
already-closed feature is not a stuck-state repair. What each actor
observes: `bee state waiting-on clear --lane <feature>` against a closed
feature now leaves that lane's `run_state` at `null` where it used to stay
stuck on `awaiting-approval`.

**The human-message clearing path is proven through the real hook entry
point, not only its inner store function (awaiting-human D2, ah-2 rework,
2026-08-14).** Two tests drive `prompt_context::run` — the exact `pub fn` the
`bee hook prompt-context` dispatch calls — end to end rather than calling
`clear_and_reap_waiting_on_best_effort` directly: one sets a live mark
through the real setter, runs the hook, and asserts the mark is gone on disk
and the outcome is native (never `Outcome::Delegate`, which would mean the
clear path was never actually reached); the other makes the clear's own
`write_json_atomic` call fail (only `.bee/`'s own tmp-file creation is
blocked — `locks/`, `cache/`, and `logs/` stay writable) and asserts the hook
still resolves natively, the failure is crash-logged, and the reminder
pipeline's own inject-cache write still completes — proving a failing clear
never takes the turn down with it.

**Every surface a reader checks for run status also names what the run is
waiting on (awaiting-human D1/D5, ah-3, 2026-08-14).** Trigger: `bee status
--json`, `bee orient`, the text status renderer, or the session preamble.
What happens: `status_full/build.rs` emits the live `waiting_on` value
alongside `run_state`; `status_full/orient.rs` treats a live mark as a
**blocker** and also exposes it at `where.waiting_on`, so a routing reader
can tell the run is stopped on a person rather than running; the text
renderer (`status_full/render.rs`) prints it; the session preamble
(`hooks/session_preamble/render.rs`, `waiting_on_note`) names it at session
start using the same liveness check (`waiting_on_is_live`) every other
reader shares. Every pre-existing `bee status --json` key keeps its name,
type, and meaning — additive only, per the same published-contract
discipline R109's own `run_state` exposure already follows.

**A fifth surface, the post-compaction capsule, names the same live wait
right after its phase line (awaiting-human D1, ah-3 rework, 2026-08-14).** A
fresh repo-wide census of every `run_state`/`waiting_on`/phase-and-gate
reporting site (`docs/history/awaiting-human/reports/ah-3-rework.md`) found
`build_compact_capsule` (`hooks/compaction.rs`) already reading `waiting_on`
via `read_state_failopen` but not yet naming it to the reader. What
happens: the capsule appends `- Waiting on human — {kind}: {subject}`
immediately after its `- Phase: … | Mode: … | Feature: … | Lane: …` line,
reusing `session_preamble::waiting_on_note` — the SAME helper the
preamble's own Gates line calls — rather than re-deriving "live" a second
way; this is the text a session reads immediately after a context
compaction, precisely when it cannot see the rest of the transcript, so it
is the moment it most needs to know it is stopped on a person. The census's
sixth candidate, `bee status --brief` (`verbs/status_brief.rs`), is
deliberately left unwired: its `BriefState` struct reads only
`phase/feature/mode/gates/route`, a frozen 7-key minimal surface scoped as
a cheap worker-startup liveness ping, not a general routing surface
(status-diet D1/D2, `docs/history/status-diet/CONTEXT.md`) — the same
category of fact D1 already excludes (handoff resolution) covers a live
human wait too, and widening that frozen shape is a scoped decision for the
human, not a silent rework-pass addition. No seventh candidate exists; every
other `Phase:`/`Gates:`/`approved_gates` hit in the tree is internal
state-projection/merge machinery, never a reader-facing surface.

**`bee state waiting-on set`/`bee state waiting-on clear` are the
agent-facing verbs — the mark existed and could end three ways before this,
but nothing outside the crate could set or clear it (ah-4, 2026-08-14).**
Trigger: any `bee state waiting-on set|clear` call. What happens: `set`
takes `--kind`, `--subject`, and resolves the owning session through the
crate's usual session-resolution precedence; `clear` takes only the usual
routing flags and is a no-op when nothing is live. Both work with no active
feature per D3, writing the session-scoped mark on the default record in
that case. Both call ah-1's and ah-2's own store functions rather than a
second implementation. `PINNED_FLAG_COUNT` (`catalog.rs`) was bumped for the
new `--subject` flag, and `generated/registry_payload.json` was hand-edited
in the same cell — no generator for that file exists in this repo (its
generator script was retired with the Node runtime; see R108's own
already-flagged gap for `bee state gate`'s flags), so a future flag addition
anywhere in the crate still needs the same manual step or a restored
generator; `p-62f0566d` (backlog) already tracks closing that gap generally.

## Business Rules

- R106 — A gate entry's `approved` boolean equals `state == "approved"`
  after every one of the four entry-authoring paths
  (`default_gate_entry`/`merge_gates`, `gates_patch_from_record`,
  `legacy_gates_to_workflow_gates`); an entry in the pre-D3 shape (no
  `state` field) derives `state` from its `approved` boolean on read; an
  unknown `state` or `actor` value refuses with a typed error, writing
  nothing (traceable-runs D3, trun-1).
- R107 — The RECORD entry's `approved` boolean stays `true` after a
  `plan-rev bump` invalidates it; only the PROJECTED boolean
  (`workflowGatesToApprovedGates`, R66) reads `false` for a stale
  plan-rev. This is the intended, tested divergence between the record
  layer (traceable-runs D3, trun-1) and the projection layer (R66) — never
  "repaired" to make the record itself flip.
- R108 — `bee state gate --actor auto` refuses outright, zero mutation,
  unless both `--bypass-level` and `--reason` are also given; `gate_bypass`
  decides only whether the run halts at an approval, never whether the
  approval record exists (traceable-runs D2, trun-2).
- R109 — `run_state` is a closed vocabulary
  (`shaping`/`awaiting-approval`/`running`/`blocked`/`done`), derived fresh
  and written on every workflow-record create/update, never taken from a
  caller's patch; it reads `awaiting-approval` whenever a gate entry is
  `pending` and no later gate is approved, OR a live `waiting_on` mark
  exists (awaiting-human D1 widens this from the gate-only condition;
  R125) — one state, two sources, never a conflict when both hold at once;
  an unknown value refuses with a typed error; no new cell `status` value
  was introduced alongside it (traceable-runs D4, trun-7).
- R125 — `waiting_on` is `null` (no live wait) or an object with `kind` in
  the closed vocabulary `gate`/`question`/`turn-end`, a non-empty `subject`,
  a non-empty owning `session`, and an `asked_at` timestamp; an unknown kind
  or an empty subject/session refuses with a typed error and writes
  nothing; the mark lives beside `run_state` on both the workflow record
  and the default `.bee/state.json` record, so a wait recorded with no
  active feature is still readable (awaiting-human D1/D3, ah-1). `turn-end`
  is the Stop hook's every-turn mark (auto-wait-mark D1/D3, 2026-08-18):
  control is back with the human, but nothing is owed, so `bee orient`
  excludes it from `blockers` while every display surface keeps showing it
  (D4).
- R126 — A live `waiting_on` mark ends only three ways, never a fourth: the
  `UserPromptSubmit` hook's best-effort clear (never able to fail the hook
  or the turn), an explicit agent-issued clear (a no-op when nothing is
  live), or stale expiry — which reuses `cells/claims.rs`'s dual-condition
  rule verbatim (age past threshold AND the owning session's heartbeat also
  stale); age past threshold alone never clears a mark (awaiting-human
  D2/D4, ah-2).
- R127 — Every surface that reports `run_state` also reports a live
  `waiting_on`, additively: `bee status --json` (`build.rs`), `bee orient`
  (a live mark is a blocker, also at `where.waiting_on`), the text
  renderer, the session preamble, and the post-compaction capsule
  (`hooks/compaction.rs`, `build_compact_capsule`, reusing
  `session_preamble::waiting_on_note`); no pre-existing `bee status --json`
  key changes name, type, or meaning (awaiting-human D1/D5, ah-3; capsule
  added by ah-3 rework, commit `c5dc3d70`). `bee status --brief` is
  deliberately NOT one of these surfaces — its `BriefState` struct stays a
  frozen 7-key minimal shape (status-diet D1/D2), a recorded deviation, not
  a gap.
- R128 — `bee state waiting-on set`/`clear` are the only agent-facing doors
  onto the mark; both call ah-1's/ah-2's own store functions rather than a
  second implementation, and both work with no active feature, writing the
  session-scoped mark on the default record in that case (awaiting-human
  D1/D2/D3, ah-4).
- R129 — `bee state waiting-on clear` resolves a named feature to its newest
  workflow record regardless of that record's status, so a feature whose
  only record already reads `closed` still resolves; only a feature with no
  workflow record at all refuses. The clear then nulls the LANE
  PROJECTION's `waiting_on`/`run_state` pair, `run_state` only when it
  reads `awaiting-approval` — every other record-derived value is left
  alone. `bee state waiting-on set` keeps the narrower resolution and its
  refusal wording unchanged (merge-closes-the-lane D3, 500fa2f9, extends
  waiting-on-pair-clear f9fd9d46).
- R130 — R128's "only agent-facing doors" is not "only writers": the
  session-stop hook also sets a mark, on EVERY turn end, without inferring
  anything from what the agent said. A turn end is the definitional fact
  that control returned to the human, so no text heuristic, phrase table,
  or language detection decides it — the event alone does. That mark's
  `kind` is `turn-end`, and it is what separates "the run is idle, a person
  holds the next move" from "the run is blocked on a person": a reader
  wanting only real blocks filters `kind != turn-end`. The hook never
  overwrites a live mark, so an agent-declared `gate` or `question` always
  outranks it; it fills an empty slot or does nothing. Its `subject` is the
  turn's last non-empty assistant line, trimmed and truncated, with a fixed
  fallback when that line is blank, because R125 refuses an empty subject.
  The write is best-effort in the same shape as the `UserPromptSubmit`
  clear: a failure is logged and the hook still emits its normal output
  (auto-wait-mark D1/D2/D5/D6, awm-2).
- R131 — A surface that renders a live wait as a BLOCKER excludes a
  `turn-end` mark, because a turn end is by definition not a blocker; a
  surface that merely DISPLAYS the wait keeps showing all three kinds.
  `bee orient` is the one blocker-shaped surface and carries the exception;
  `bee status`, the text status renderer, the session preamble and the
  compact capsule all keep reporting every kind. This is why R127's "every
  surface that reports run_state also reports a live wait" still holds
  while orient's blockers array does not (auto-wait-mark D3/D4, awm-1).
- R132 — The hook reads the session transcript exactly once per turn end.
  One resolver and one read serve both the performance rollup and the
  turn-end subject: the rollup hands its already-parsed events onward, and
  every fallible bookkeeping step that follows is arranged so a late
  failure cannot discard them. Two successive attempts leaked a second read
  here — the first re-read outright, the second re-read only when the
  rollup failed after its own read, while carrying a comment asserting that
  path had no caller. Both were caught by review, not by the suite that was
  green at the time; the guard that holds now is a test counting reads, not
  a reading of the call graph (auto-wait-mark, awm-2 rework 2).
- R105 — A feature's workflow record is created through one idempotent seam
  (asked twice, it changes nothing) and retired through a verb taking exactly one
  of three modes — by feature, by id, or all-but-active; the active feature's own
  record is never closed by a sweep, and a sweep that cannot resolve an active
  feature refuses rather than closing everything (workflow-lifecycle, cells
  wl-1/wl-2, 2026-08-05).
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
  == null || approved_for_plan_rev === plan_rev)`, applied generically to
  every gate. Context and review are never stamped with a `plan_rev` by any
  approval path. Shape and execution are stamped by whichever path approved
  them: the standalone `--name` path stamps only the one gate it names; the
  merged `--merge` path stamps both together, so the two can never end up
  half-revoked after a later bump (multisession-native D7, advisor condition
  C2; scope widened from execution-only to both merge-approved fields by
  validation-diet D2/D15, cell vd-3). `bee state plan-rev bump` bumps and
  rebuilds exactly one named lane's workflow, touching no other workflow's
  `plan_rev`, gates, or claims.
- R67 — Every state-mutation write path — default and lane alike — locks and
  routes through its own feature's workflow record via `withMutationLock`,
  never a single lock scoped to the whole workflow branch regardless of which
  record it actually writes; the sole standing exceptions are a `--feature`
  swap on the default record (writes `state.json` directly, because identity
  is immutable on a workflow record) and a repo with zero workflow records
  (multisession-native D1, advisor condition C5, closed by
  `multisession-native-10`; per-record scoping finalized by
  state-phase-lock-race D13 — see R79).
- R79 — Every writer of `.bee/state.json` or of a lane projection acquires the
  lock scoped to the exact record it writes — `'state'` for `.bee/state.json`,
  `lane:<feature>` for `.bee/lanes/<feature>.json` — never a lock scoped to a
  code path that happens to write near several records; the single global
  acquisition order is `workflow:<id>` → `'state'` → `lane:<feature>`, most
  paths take only a prefix, and no path takes any two of these names in the
  reverse order. `rebuildStateProjection`/`rebuildLaneProjection` acquire
  nothing themselves and stay caller-serialized, unchanged from D3
  (state-phase-lock-race D1-D4/D13, GH #70).
- R80 — `handleStatePlanRevBump` resolves its target workflow before taking
  any lock, then acquires `withWorkflowLock(ctrlRoot, wf.id, () =>
  withStoreLock(root, 'state', body))` with `updateWorkflowAssumingLock`
  inside — the canonical `workflow:<id>` → `'state'` order — rather than
  holding `'state'` and calling the self-locking `updateWorkflow`, which was
  the repo's one live inverse-order edge (state-phase-lock-race D9,
  advisor-found).
- R74 — Every workflow-record and session/claim call site `state.mjs` and
  `bee` make resolves through `controlRootFor(root)`, never the writing
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
- A lock scoped to a code path rather than to the record it writes can be
  worse than no lock at all: `splr-1`'s blanket `'state'` wrap over the whole
  workflow branch left the lane record with no lock of its own, while
  forcing every lane mutation to wait out an unrelated default-record hold —
  turning a near-instant, correct invariant into a ~5 s timeout
  (state-phase-lock-race D12/D13; `test_cli_state.mjs`).
- A workflow record created (or last updated) before `run_state` shipped has
  no `run_state` key at all until its next create/update call — `bee status
  --json` reads it as `null` in the meantime rather than a computed
  best-guess; the field is written fresh on the next mutation, never
  backfilled by a read (traceable-runs D4, trun-7).
- A record created (or last updated) before `waiting_on` shipped has no
  `waiting_on` key at all until its next create/update call — it reads as
  `null` (not live) in the meantime, the same absent-is-safe behavior R109's
  own pre-`run_state` edge case already establishes (awaiting-human D1,
  ah-1).
- A pending gate and a live `waiting_on` mark at the same time still yield
  exactly one `awaiting-approval`, never a conflict and never a second
  value — the two sources are ORed, not raced (awaiting-human D1, ah-1).
- A lease past the stale-expiry age threshold whose owning session
  heartbeat is still fresh leaves the mark untouched — expiry never fires
  on age alone (awaiting-human D4, ah-2).

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
  `state-projection.mjs`; `handleStateGate`'s gate stamping (standalone
  `--name` and merged `--merge`, `findGateStamp` normalizing the stamp array)
  and `writeLaneRecordThroughProjection`'s optional `gateStamp` param, and the
  `state plan-rev bump` verb, in `the bee binary` +
  `lib/command-registry.mjs`. Proved red-first (a `plan_rev` bump flips only
  the targeted workflow's projected execution boolean; a sibling workflow is
  untouched). Evidence: trace `.bee/cells/multisession-native-9.json`, commit
  2dd834f. Widened so `--merge` stamps both `shape` and `execution` together
  (validation-diet D2/D15): `bee state gate --merge --approved true` flips
  both fields in one call, mutually exclusive with `--name`; a plain `--name`
  approval is byte-for-byte unchanged. Evidence: trace `.bee/cells/vd-3.json`;
  6 new tests in `packages/bee/tests/test_state_projection.mjs` proving one
  `--merge` call flips both gates, `--merge --name` refuses together, and
  `plan-rev bump` revokes both gates when approved via `--merge`.
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
  from every workflow/session/claim site this file and `bee` own; see
  `areas/worktree-parallelism/control-plane-topology.md` for the resolver,
  the declared plane split, and the msn-18 honest-block re-slice that swept
  it in. Evidence: traces `.bee/cells/multisession-native-{18a,18b,18c}.json`,
  commits 5d0ec3c, a1431448, d69d81e.
- Legacy handoff projection's audited writer set (msn-24): see
  `areas/workflow-state/handoff.md` (R77) for the full behavior; the
  grep-audit test itself lives in
  `packages/bee/tests/test_state.mjs`. Evidence: trace
  `.bee/cells/multisession-native-24.json`, commit cee2d5f.
- Per-record lock order (R79/R80, GH #70): `withMutationLock`'s `if (wf)`
  branch nests `withStoreLock(root, 'state', fn)` inside
  `withWorkflowLock(ctrlRoot, wf.id, …)` for default-record mutations, and a
  sibling `lane:<feature>`-scoped wrap for lane mutations, both in
  `the bee binary`; the two previously-unlocked writers
  (`handleStateStartFeature`'s post-rebuild at `bee:3284`,
  `handleStateRebuildProjections` at `bee:3304`) are each wrapped in
  `withStoreLock(root, 'state', …)`; `handleStatePlanRevBump`
  (`bee:2921`) restructured to resolve its workflow first, then
  `withWorkflowLock(…, () => withStoreLock(root, 'state', body))` with
  `updateWorkflowAssumingLock` inside, closing the one live inverse-order
  edge (D9). Fix vendored byte-identical into `.bee/bin/bee` (D10, `md5sum`
  parity with `the bee binary`). Proof: `scripts/tests/test_state_projection_race.mjs`
  — real OS child processes (never in-process async) asserting mutual
  exclusion between a `'state'`-locked writer and a `workflow:<id>`-locked
  writer, an exact-count lost-update assertion, and a negative control (the
  pre-fix workflow-lock-only arrangement) that must itself produce
  violations, so the suite cannot pass vacuously; red-first against
  unmodified `bee`. Evidence: cells `splr-1`/`splr-2`/`splr-3`, commits
  `e787819a`/`73eadc5f`/`ebc68f04`, 2026-07-27; full 109-suite verify green,
  verified independently by the orchestrator. Advisor consult:
  `docs/history/state-phase-lock-race/reports/advisor-consult.md`.
- Persisted gate record (R106/R107, D3, trun-1): `default_gate_entry`,
  `merge_gates`, `valid_run_state_value`-sibling gate-value checks, and
  `read_workflow_record`'s legacy-shape `state`-derivation-from-`approved`
  all in `packages/bee-rs/crates/bee/src/verbs/workflow_store/record.rs`;
  `gates_patch_from_record` (the real entry author behind `bee state gate`)
  in `workflow_store/handoff.rs`; `legacy_gates_to_workflow_gates` in
  `verbs/state_group/feature.rs`. Tests covering all four authoring paths,
  the legacy-shape read, invalid-value refusal, and the intended plan-rev
  divergence: `workflow_store/tests.rs`. Evidence: trace
  `.bee/cells/trun-1.json`.
- `bee state gate`'s new `--actor`/`--bypass-level`/`--reason` flags
  (R108, D2, trun-2): the accepted-flags allowlist and the auto-actor
  precondition in `run_gate_body`
  (`packages/bee-rs/crates/bee/src/verbs/state_group/set_gate.rs`), which
  issues its own still-lock-held `update_workflow_assuming_lock_with` patch
  after `write_through_projection` lands the legacy projection. Evidence:
  trace `.bee/cells/trun-2.json`. Known gap: the generated CLI registry
  payload (`packages/bee-rs/crates/bee/src/generated/registry_payload.json`,
  fed by `bee --help --json`) does not list these three flags for `state
  gate` — the flags are fully enforced by `run_gate_body`'s own
  `keys_known` allowlist regardless, so behavior is correct and tested; the
  gap is documentation-only, flagged by trun-2's own deviation note, not
  re-filed as a fresh backlog row here.
- `run_state` (R109, D4, trun-7): `RUN_STATE_VALUES`, `derive_run_state`,
  `check_patch_run_state`, `base_workflow_defaults`'s `run_state: null`
  default in `workflow_store/record.rs`; carried into `.bee/state.json` by
  `apply_workflow_d1_fields` in `workflow_store/projections.rs` — the seam
  the plan named as the one that makes the byte-identical-rebuild proof
  pass vacuously if skipped. Evidence: trace `.bee/cells/trun-7.json`.
- `bee status --json`'s `gate_records`/`run_state` exposure (D7 boundary,
  trun-3): `build_gate_records` in
  `packages/bee-rs/crates/bee/src/verbs/status_full/build.rs`, reading the
  live workflow record rather than the projection; rendered one line per
  non-approved gate in `status_full/render.rs`. Evidence: trace
  `.bee/cells/trun-3.json`.
- Creation seam and retirement verb (R105): `ensure_workflow_record_for_feature`
  in `packages/bee-rs/crates/bee/src/verbs/state_group/feature.rs:337-385`
  (idempotent by feature; refuses an empty slug), wired into the feature start
  at `state_group/policy.rs:684` — and since state-swap-hygiene cell ssh-1
  (2026-08-11) into the `state set --feature` SWAP path too (`run_set_body`,
  after the C1 direct write and outside the mutation locks): a swap closes
  every other live record and ensures the incoming feature's, so the
  deliberate C1 write carve-out no longer orphans the outgoing record
  (PBI p-71d014e7's residue; the start-feature half was wl-2); `close_workflows_for_feature`
  (`feature.rs:390-406`) closes each record under its own `workflow:<id>` lock,
  sequentially, never nested inside the `state` lock. The verb halves are
  `run_workflows_list` (`state_group/workflows.rs:38-71`) and
  `run_workflows_close` (`workflows.rs:106-259`), which requires exactly one of
  `--feature` / `--id` / `--all-but-active`. Evidence: cells `wl-1`/`wl-2`,
  `docs/history/workflow-lifecycle/promote-proposals.md` — the seam names in
  that proposal are the retired runtime's; these are the ported equivalents.
- The `waiting_on` mark (R125, D1/D3, ah-1): `WAITING_ON_KIND_VALUES`,
  `build_waiting_on`, `waiting_on_is_live`, `check_patch_waiting_on`,
  `base_workflow_defaults`'s `waiting_on: null` default, and the
  feature-scoped/default-record setters (`set_workflow_waiting_on`,
  `set_default_state_waiting_on`) all in
  `packages/bee-rs/crates/bee/src/verbs/workflow_store/record.rs`; the
  widened `derive_run_state` reads a live mark as a second source of
  `awaiting-approval` beside the pending-gate condition, in the same
  function. Carried into `.bee/state.json` by `apply_workflow_d1_fields`
  (`workflow_store/projections.rs`), extended the same way R109's own
  `run_state` carry-through was. Evidence: trace `.bee/cells/ah-1.json`,
  commit `b306201f`.
- The three clearing paths (R126, D2/D4, ah-2):
  `clear_and_reap_waiting_on_best_effort`,
  `clear_workflow_waiting_on`/`clear_default_state_waiting_on`, and
  `reap_stale_waiting_on` (the dual-condition reap, mirroring
  `cells/claims.rs`'s reclaim rule) in `hooks/prompt_context.rs` and
  `workflow_store/record.rs`. Evidence: trace `.bee/cells/ah-2.json`,
  commit `39facee4`. Rework (ah-2, commit `b7b422f9`): the human-message
  path proved through the real `prompt_context::run` hook entry point, not
  only the inner clear function —
  `run_clears_a_live_default_waiting_on_mark_through_the_real_hook_entry_point`
  and
  `run_survives_a_failing_clear_without_failing_the_hook_or_skipping_its_output`
  (the latter a real `.bee/`-directory permission-based write failure
  injection, `#[cfg(unix)]`) in `hooks/prompt_context.rs`'s own test module.
- The reporting surfaces (R127, D1/D5, ah-3): the `waiting_on` key in
  `status_full/build.rs`, the blocker + `where.waiting_on` in
  `status_full/orient.rs`, the text line in `status_full/render.rs`, and
  `waiting_on_note` in `hooks/session_preamble/render.rs`. Evidence: trace
  `.bee/cells/ah-3.json`, commit `fe0b8885`. Rework (ah-3, commit
  `c5dc3d70`): full census in
  `docs/history/awaiting-human/reports/ah-3-rework.md` found and wired a
  fifth surface, the post-compaction capsule's
  `build_compact_capsule` (`hooks/compaction.rs`), reusing
  `session_preamble::waiting_on_note`; tests
  `the_capsule_names_a_live_wait_right_after_the_phase_line` and
  `the_capsule_stays_silent_without_a_live_wait` in the same file. A sixth
  candidate, `bee status --brief` (`verbs/status_brief.rs`), was found and
  deliberately left unwired — recorded deviation, status-diet D1/D2.
- `bee state waiting-on set`/`clear` (R128, ah-4):
  `resolve_waiting_on_target`, `run_waiting_on_set`, `run_waiting_on_clear`
  in the new `packages/bee-rs/crates/bee/src/verbs/state_group/waiting_on.rs`,
  wired into `state_group/mod.rs`; `PINNED_FLAG_COUNT` bumped in
  `catalog.rs` and `generated/registry_payload.json` hand-edited in the same
  cell (no generator exists for that file in this repo). Proved end to end
  through the real CLI binary, not only at the unit level, in
  `packages/bee-rs/crates/bee/tests/workflow_verbs.rs`. Evidence: trace
  `.bee/cells/ah-4.json`, commit `00c67cb6`.
- Closed-record fallback + lane-pair clear (R129, merge-closes-the-lane D3):
  new `pub(crate)` helper `clear_lane_waiting_on_pair` in
  `packages/bee-rs/crates/bee/src/verbs/state_group/waiting_on.rs`. Evidence:
  commit `a4d5ca90`; full suite `cargo test --release --manifest-path
  packages/bee-rs/Cargo.toml` — 2033 passed, 0 failed. Live proof after
  installing the rebuilt binary: `bee state waiting-on clear --lane
  staging-lane` left that lane at `run_state: null` where the old binary
  left `awaiting-approval`.
