---
type: bee.area
title: "Workflow State — working sessions, self-derived identity, lanes, and the renewing heartbeat"
description: "Who the acting session is (resolved from its own environment, never handed down), how a feature gets its own pipeline lane that every reader resolves through, how a live session's heartbeat renews itself and carries its claims and holds forward with it, how lane binding now shares the same store lock as the heartbeat so the two writers of one session record can never lose each other's update, how active workers are always a computed join of live sessions and claims rather than a stored array, and which lane changes a recorded route will accept."
timestamp: 2026-08-06
bee:
  id: workflow-state-sessions-lanes-and-identity
  lifecycle: active
  areas: [workflow-state]
  required_context: [areas/workflow-state/overview.md, areas/worktree-parallelism/control-plane-topology.md]
  decisions: [multi-session-hardening D3/D5 with Δ1-Δ6 amendments (session self-derivation; throttled heartbeat and lease renewal), "fresh-session-handoff D2 (a lane never borrows the default pipeline's authority)", "hardening-1-7-10 (the durable single-fresh-session identity fallback, audited, at library and CLI levels)", i54-closeout D7, "multisession-native D10a (issue #56 3.8 — bindSessionLane/unbindSessionLane serialize under the same sessions store lock heartbeatSession already uses, closing the lost-update race between them)", "multisession-native D6 (active workers derived from live-heartbeat sessions + lane/workflow binding + claims, never the stored workers array; advisor condition C3 — startFeature excludes the calling session's own heartbeat)", "multisession-native D2/D3 (slice 4: session creation re-roots onto controlRoot; a session's own record carries workspace_id, auto-looked-up and stamped onto its claims too — docs/history/multisession-native/CONTEXT.md, decision e1ceca12)", "worker-conformance D11/D12 (R82's feature-boundary door arms on two markers — proof relocated and proof never recorded — with the freshness clock running over the union of both; cells wc-1/wc-2, docs/history/worker-conformance/CONTEXT.md, 2026-07-29)", "132362c7 (route-identity: a route carries the feature it was triaged for, a feature start drops the previous one's route, and the never-demote rule is scoped to one feature's own history — cell rti-1, 2026-08-06)", "hook-teeth D5/D7 (docs/history/hook-teeth/CONTEXT.md, 2026-08-04 — re-lane transitions are validated where the route is recorded, and each refusal names the rule it broke)", "counter-teeth D4 with refinement 64ad772d (docs/history/counter-teeth/CONTEXT.md, 2026-08-04 — the route-less claim warning escalates to a refusal from a session's second claim; the counter is scoped per feature and session, and the contention refusal outranks it)"]
  sources: ["fresh-session-handoff cells fsh-3/fsh-4 (lane store, resolvePipeline, lane-mode startFeature; validation-s2, 2026-07-13)", "multi-session-hardening cells msh-1..7 (traces in .bee/cells/, reports docs/history/multi-session-hardening/reports/, 2026-07-19)", hardening-1-7-10 cells 1710-1..1710-11 (2026-07-21), "docs/specs/workflow-state.md#B12", "docs/specs/workflow-state.md#B13", "docs/specs/workflow-state.md#B22", "docs/specs/workflow-state.md#B24", "docs/specs/workflow-state.md#R38", "docs/specs/workflow-state.md#R55", "docs/specs/workflow-state.md#E22", "docs/specs/workflow-state.md#P14", "i54-closeout cell i54-closeout-7 (resolveMutationTarget lane auto-resolve for state-write verbs; trace in .bee/cells/, 2026-07-24)", "multisession-native cell multisession-native-1 (trace .bee/cells/multisession-native-1.json, commit c794eda, 2026-07-24)", "multisession-native cell multisession-native-8 (activeWorkers derivation, trace .bee/cells/multisession-native-8.json, commit c435add, 2026-07-25)", "multisession-native cell multisession-native-10 (default-path mutation now also routes through its workflow record; trace .bee/cells/multisession-native-10.json, commit e7f365a, 2026-07-25)", "multisession-native cell multisession-native-19 (createSession/claimCellFile stamp workspace_id; bee-session-init.mjs re-roots onto resolveContext.controlRoot and lazily auto-registers the workspace; trace .bee/cells/multisession-native-19.json, commit 09e1ed0, 2026-07-25; see areas/worktree-parallelism/control-plane-topology.md)", "route-identity cell rti-1 (start_default drops the carried route, route --set stamps its feature, run_route gates the demote check by it; commit 68beab21, capped 2026-08-06)", "hook-teeth cell bh-5 (re-lane validation: downward once, highest-risk never, mandatory-ceremony flags block, promotion free; trace .bee/cells/bh-5.json, commit 95fe412d, 2026-08-04 — state_group 56 passed)", "counter-teeth cell ct-5 (per-(feature, session) no-route claim counter; warn once then refuse, contention refusal outranks it; trace .bee/cells/ct-5.json, commits 4a0d1b82 and 95ec0639, 2026-08-04 — cells 75 passed, concurrency 13 passed, full suite green)"]
  authoritative_for: "workflow-state: session identity, per-feature lanes, and heartbeat/lease renewal"
---

# Workflow State — working sessions, self-derived identity, lanes, and the renewing heartbeat

Several features can be in flight at once, and several terminals can be working
the same checkout at once — so "where does the workflow stand" is always asked
by *somebody*. This concept owns that somebody: the session, its
self-resolved identity, the lane it is bound to, and the heartbeat that keeps its
claims and holds alive while it is genuinely working.

Note on the source's block boundaries: `B24`'s block in the pinned source runs
to the end of the `### Closing a feature` subsection, because a `###` heading
does not close an anchor's block. That subsection's prose therefore travels here
with `B24` verbatim, and also appears — as prose, not as a claim — in
`gates.md`, which is where the closing tail belongs topically. Only the anchor
CLAIM is unique; the prose is deliberately in both.

## Behaviors & Operations

**B12 — A feature can start as its own lane, and every lane mutation is
commandable.** Trigger: new work begins while other features are mid-flight.
What happens: starting a feature *as a lane* creates that feature's own
pipeline record and resets exactly its four gate fields in one atomic write, leaving
the default record and every other lane byte-identical. Its preconditions are
lane-scoped, with attribution **derived from existing records, never new
fields**: an unfinished unit blocks only if it belongs to this feature; a
pause snapshot blocks only if it names this feature; a registered worker
blocks only if its unit belongs to this feature; and — globally — declared
intended paths refuse when they overlap another session's live holds. The
default (non-lane) feature start keeps its original whole-repo semantics
unchanged. Every lane mutation has a command verb: the state mutation verbs
accept a lane selector routing the write to that lane's record (with a safety
refusal when a mutation would silently rename a lane's identity), lanes are
listable with their phases/gates/bindings, and sessions are listable and
bindable/unbindable to a lane. Every published command example is executed by
the suite against the real operation. What each actor observes: an agent in a
zero-lane repo sees exactly the pre-lane behavior of every verb; an agent
using lanes sees per-feature pipelines whose gates never bleed into each
other.

**Lane-scoped writes auto-resolve the same way lane reads already do
(i54-closeout D7).** `resolveMutationTarget` — the shared resolution behind
`state set`, `state gate`, `state scribing-run`, and `state advisor-ref record`
— picks its target in one fixed precedence: an explicit `--lane` always wins;
absent that, the calling session's own bound lane (identity self-resolved at
the moment of the operation, per B22) is used; absent both, the default record
is used, exactly as before lanes existed. `--no-lane` forces the default record
even from a bound session; passing it together with an explicit `--lane` is
refused. A missing or corrupt bound lane refuses the write loudly, with zero
writes performed — it never silently falls back to the default record (the
same never-borrow-the-default's-authority discipline as B13's read path,
fresh-session-handoff D2). A background writer that has already decided which
record owns its fact names that record directly, and forces the default record
when that is the answer; it never lets this shared resolver choose, because the
resolver prefers the calling session's bound lane and would land the write on
another feature's record (merge-ready-fact cell mrf-1). An unbound session sees no behavior change at all:
every one of the four mutation verbs resolves to the default record exactly as
it always did. `--owner`, where a verb accepts it, is still checked against
the *selected* record's own pre-mutation phase, never the default's. **Since
multisession-native slice 2, the selected record's mutation itself is no
longer a direct file write:** whichever record `resolveMutationTarget` picks
— lane or default — its write now routes through that feature's own workflow
record and its projection, under that workflow's own lock, exactly as
described in `workflow-records-and-projections.md` (required for both paths as
of `multisession-native-10`, closing the interim gap where only the lane path
routed through a workflow record). The precedence, the refusal shapes, and
everything an unbound or zero-workflow-record session observes are unchanged
by this — only what backs the write moved.

**"Active workers" is a computed view, never the hand-mutated array
(multisession-native D6).** Trigger: any read of who is currently working —
status, the session preamble, a start-feature precondition. What happens: the
answer is derived, not stored — live-heartbeat sessions (B24) joined with
their lane/workflow binding and their current cell claim, freshly computed on
every read. The record's own `workers` array (written and read by `state
worker add/update/remove/clear/prune`) stays fully commandable for display and
operator tooling, but it is documented as display-only: no gate or
precondition anywhere reads it as truth any more. `startFeature`'s worker
precondition — on both the default path and the lane path (`--as-lane`) —
checks this derived view and excludes the calling session's own heartbeat from
it, so a session starting a feature alone is never blocked by the fact that it
is itself alive (the same `excludeSessionId` pattern `isConcurrentMode`
already used). A worker with a stale heartbeat simply stops appearing in the
computed view on the next read — there is no separate "remove" call needed to
drop it. What each actor observes: `bee status` gains a `workers` line sourced
from the derived view (previously status reported no worker information at
all); a hand-written `workers` entry with no live session behind it no longer
blocks a new feature from starting; a live *other* session holding a claim
still blocks exactly as before; a solo starter's own heartbeat never
self-blocks.

**A quiet heartbeat is not a dead session, and a reader must not treat it as
one.** The renewal fires while a session works, so a session inside one long
operation emits nothing for as long as that operation runs — the same fact R97
states for the sweep path, and it binds every reader, not only a sweep. Age
therefore answers exactly one question: whether the record still falls inside
the liveness window. Inside it, the owner may be mid-thought or may have died
moments ago, and the record cannot tell the two apart; outside it, the owner is
treated as gone. Reading a flat heartbeat as "not listening" is a mistake two
separate sessions made independently within one hour on 2026-08-19, one of them
concluding an owner was ignoring messages it was in fact answering. When the
question is who to contact rather than what to reclaim, the honest report is
either the bound owner or "unowned" — never a confident aliveness verdict the
heartbeat was never able to give.

**B13 — Readers resolve through the acting session's lane.** Trigger: any
read of "where does the workflow stand" while lanes exist. What happens, per
reader: **claim authorization** — a unit of work is claimable only under its
own feature-lane's execution approval when such a lane exists; an unapproved
lane refuses even though the default gate is granted, an approved lane
authorizes even though it is not, and a corrupt lane record refuses loudly
rather than falling back (the lane never borrows the default pipeline's
authority — fresh-session-handoff D2). **Write gating** — the production write
guard passes the acting session's identity (carried on every guard event) into
the check, so a bound session is judged by its own lane's phase and gates; an
event without the identity is judged exactly as before. **Presentation** — the status surface lists every
lane with its phase, gates, and bound sessions; the session preamble, given a
bound session, shows that lane's view plus a one-line count of other active
lanes; the two lifecycle guardrails (mid-work warning, session-close warning)
judge the acting session's own lane. The session-close mid-phase warning
names all THREE sanctioned exits since revision-deadlock-visibility cell
rdv-2 (2026-08-11, PBI p-808487c4): finish and cap the work; write the
handoff record and release reservations; or record a capture stub for what
settled (decision 0017's road) and close cleanly — the third had been
defined by doctrine but omitted from the warning's text. What each actor observes in a zero-lane
repo: byte-identical output everywhere — the entire migration is invisible
until a lane exists.

**B22 — A session's identity is derived automatically, never handed to it by
another party.** Trigger: any operation that records or checks a session's
ownership — claiming, holding, or renewing. What happens: the acting session's
identity is resolved from its own runtime environment at the moment of the
operation, in a fixed order of preference, with an explicit override reserved
for tests; nothing else ever substitutes a different party's stated identity
for the caller's own. Below that environment lookup and above the ownerless
floor sits one durable fallback: when the session store shows exactly one
fresh, live-heartbeat session and the environment gave no answer, that lone
session is adopted as the caller's identity, audited with an `adopted` marker
rather than adopted silently; two or more fresh live sessions still refuse
(`SESSION_REQUIRED`) rather than guess between them. What each actor observes:
an operation with no
resolvable identity still records an ownerless (sessionless) entry exactly as
it always could; every other operation is now attributed to the real acting
session by default rather than only when a caller opted in, so cross-session
holds and claims become visible without any special handling (D3;
durable-fallback tier: hardening-1-7-10).

**A session's own record now carries its workspace, stamped once and reused
by its claims (multisession-native D2/D3, msn-19).** Trigger: session
creation, or claiming a cell file. What happens: `packages/bee/hooks/bee-session-init.mjs`
creates the session record at `resolveContext.controlRoot` rather than the
writing checkout's own root — closing the gap the `18c` adapter comment had
flagged as deferred to a later cell — and lazily auto-registers that
checkout's workspace (`workspace-store.mjs`) the first time a session touches
it. `claims.mjs`'s `createSession` and `claimCellFile` both stamp
`workspace_id` onto the record they write, auto-looked-up from the acting
session's own already-resolved workspace rather than accepted as a caller-
supplied value. What each actor observes: a session created inside a linked
worktree is visible at the same control-plane path any other checkout's
session list already reads; its claims now carry enough identity for the
write guard's same-workspace-vs-different-workspace lease check
(`holds-and-the-coordination-lock.md`) to answer correctly. See
`areas/worktree-parallelism/control-plane-topology.md` for the workspace
registry this stamping feeds and the write-policy decision it enables.

**B24 — A live session's heartbeat renews itself, throttled, and carries its
claims and holds forward with it.** Trigger: a working session performs any
tracked activity while it is already known to the coordination store. What
happens: the session's heartbeat record refreshes automatically, at most once
per a short throttle window (well under the staleness threshold that governs
reclaim), so routine work does not spam the shared store with writes. In the
same moment, every claim and hold the session owns has its lease renewed —
except a claim currently gated by an in-flight ownership transfer (B11
adoption), which is skipped rather than rewritten, so an automatic renewal can
never revert a transfer that is mid-flight. This automatic renewal is
opportunistic, not authoritative: it runs through the same never-wait,
try-once discipline as any other lifecycle-triggered write (B21), and a
failure to renew never blocks or delays the session's own primary work. What
each actor observes: a session that stays genuinely active never goes stale,
so its claims and holds are not mistakenly reclaimed out from under it; the
accepted residual is that a session idling in unrelated activity still
renews — the audited forced door (B23) and release on any claim-clearing
transition (B11) remain the rescue; the staleness threshold itself is
unchanged, so real silence that long still genuinely means the session is
gone (D5).

**Lane binding now serializes under the same lock heartbeat renewal already uses
(multisession-native D10a).** Trigger: a session's lane binding changes (bind
or unbind) at or near the same moment its own or another session's heartbeat
renewal (B24) is in flight. What happens: binding and unbinding a lane now
read and write the session record inside the identical store lock that
heartbeat renewal already acquires — a read-modify-write on a session record
is never performed lock-free, on any of the three paths that touch it.
Exhausting the lock's bounded-retry budget returns the same typed `LOCK_BUSY`
refusal (Data Dictionary) every other coordination-store contention already
answers with, naming the current holder, rather than silently proceeding
unlocked. What each actor observes: a bind or unbind landing in the same
instant as a heartbeat can no longer be silently clobbered (a fresh bind
overwritten by heartbeat's stale in-memory copy of the record) or resurrected
(an unbind reverted the same way) — the two writers of one session record are
now mutually exclusive, closing the session store's last lock-free
read-modify-write (issue #56 3.8).

**A lane binding is refused unless the lane already exists, and the guard that
enforces bindings never blocks the escape from a bad one (cells lgd-1/lgd-2,
2026-08-12).** Trigger: binding a session to a lane feature that has no lane
record. What happens: the bind is refused at the door, before the sessions lock
is taken, carrying the same lane-missing wording every other lane-resolving path
already uses — the one that names starting the lane as the fix. Nothing is
written. Before this, any well-formed lane id was accepted, which produced a
session record every other seam was then obliged to reject.

The second half of the rule is what made the first half urgent. The write
guard's command check used to resolve the acting lane record *before* it looked
at the command at all, so a binding that resolved to nothing refused **every**
shell command the session ran — including the unbind that the refusal itself
names as the remedy. A session could enter that state and not leave it; only a
human running the command outside the session could break the deadlock. The
check now reads the command first: a command that carries none of the version-control
verbs this guard judges is not its business and resolves nothing. What each
actor observes: a bad binding can no longer be created, and a session that
somehow holds one can still run its own way out. What is unchanged: the
version-control verbs the guard judges, and every file write, still refuse under
an unresolvable binding.

The durable rule behind both halves: **a refusal whose remedy names a command
must leave that command runnable**, and a guard resolves shared state only for
the inputs it actually judges.

### Closing a feature — the tail of the chain

Closing is the one stretch of the pipeline where each step must *prove* the step
before it happened. The phase vocabulary alone never granted that proof: the
names asserted history ("both the knowledge sync and the learning capture have
run"), while nothing checked whether either had. A feature could therefore be
marked closed straight from execution, and this is exactly what happened
repeatedly — the settled behavior of six completed units never reached the
specs, and the only trace was a knowledge-sync record that stayed empty.

Three rules now hold the tail together. Together they make "declare it closed"
impossible; the only way to close is to actually close.

**Entering learning capture is never an assertion.** The learning-capture phase
cannot be set directly, from any phase. It is *produced* — and only produced —
by recording a knowledge sync. Attempting to set it names the recording step as
the way. This means the phase is reachable if and only if a real sync was
stamped, because stamping it is the sole door.

**Recording a knowledge sync demands that work was executed.** The recording
step is refused unless the feature currently stands in a phase where execution
has actually happened (execution, independent review, or the sync itself). It is
not possible to sync the knowledge of work that was never done.

**Reaching the terminal state demands the phase before it AND zero spec debt.**
The terminal state may be entered only from learning capture, and only while no
completed behavior-changing unit is still missing from the specs. The refusal
names *every* such unit by identity — not a count — and discloses the waiver.
A refused close is side-effect-free: the phase is left exactly as it was.

**The waiver is a door, not a hole.** A feature whose settled behavior genuinely
belongs in no spec may still be closed, by waiving the debt explicitly. The
waiver permits the close and simultaneously records a durable decision naming
every unit whose behavior was left out. Nothing about it is silent, and nothing
about it is the default. It exists because a guard with no door gets a hole
punched in it — a fail-close with no sanctioned exit teaches its user to work
around the guard instead of through it.

Everything outside the tail stays permissive: moving backward to an earlier
phase is always legal (a failed feasibility check or a negative proof must be
able to return to planning), and returning to idle — the way an abandoned
exploration is dropped — is unaffected.

**What each actor observes.** The agent attempting a dishonest close gets a
refusal that says which step was skipped and how to perform it, and the record is
untouched. The human sees a feature that cannot be reported as finished until
its knowledge actually landed — the state and the specs can no longer disagree.

**B48 — Re-laning a feature is validated where the route is recorded, and each
refusal names the rule it broke (hook-teeth D5, 2026-08-04).** Trigger:
recording a route for a feature that already carries one, under a different
lane. What happens: four rules are checked in turn, and any violation refuses
with that rule stated in its own words. Work classified highest-risk never
leaves that lane, whatever the new lane would be. Demotion down the ceremony
ladder — the ordered run from the smallest lane, through the middle one, to the
standard one — is allowed at most once in a feature's whole life, and the first
demotion's moment is stamped so a second is recognisable. A demotion is refused
outright when the new classification carries any flag that makes ceremony
mandatory. Re-recording the same lane, promoting upward, and any move touching a
lane that sits off that ladder are always allowed, and an allowed same-lane
re-record carries the existing demotion history forward untouched. What each
actor observes: ceremony a feature earned cannot be shed quietly one step at a
time, while raising ceremony on discovered risk stays free at every moment. The
history this rule reads is one feature's own (decision 132362c7).

**B52 — An untriaged feature costs each session exactly one warning, then
refuses (counter-teeth D4, 2026-08-04).** Trigger: claiming a unit of a feature
that carries no route record. What happens: the first such claim by a given
session still succeeds and still warns, naming the recording verb as the remedy
and stating plainly that this session's next route-less claim will be refused.
From that session's second claim onward the claim is refused, with the same
remedy named. The count is kept per session *and* per feature, and it advances
only when a claim actually succeeds — a refused claim never spends the warning.
Why the scope is not per feature alone: a swarm fans several sessions out across
one feature, and a bare per-feature count would refuse every worker but the
first for a fault none of them can fix from where they stand. What each actor
observes: a session that forgot to triage learns once, cheaply, and is stopped
the second time; a race for the same unit still reports the contention first —
the already-claimed refusal outranks this one, so the loser of a real race is
told it lost, never that the feature is untriaged (D4 refinement, decision
64ad772d).

## Business Rules

- R38 — A session's identity is always self-resolved at the moment of the
  operation from its own runtime environment, never accepted as handed down by
  another party except an explicit test override; an operation with no
  resolvable identity still proceeds, recorded as ownerless
  (multi-session-hardening D3). Below the environment lookup, exactly one
  durable fallback applies before falling through to ownerless: when precisely
  one fresh live session exists in the store, it is adopted and the adoption
  is audited on both the result and the resulting claim; two or more fresh
  live sessions refuse rather than guess. The full chain, fallback included,
  applies at the library level and every CLI surface alike (claim, claim-next,
  reservations) (hardening-1-7-10).
- R55 — Session identity resolution carries one durable fallback below the
  environment lookup and above the ownerless floor: exactly one fresh live
  session in the store is adopted (audited, never silent); two or more refuse
  rather than guess. The chain applies identically at the library level and
  every CLI surface that resolves identity (hardening-1-7-10).
- R56 — A lane-scoped state mutation (`state set`, `state gate`, `state
  scribing-run`, `state advisor-ref record`) resolves its target in the order
  explicit `--lane` > the calling session's own bound lane > the default
  record, symmetric with the read-path resolution in B13; `--no-lane` forces
  the default from a bound session, and a missing or corrupt bound lane
  refuses the write loudly rather than falling back (i54-closeout D7).
- R57 — Binding and unbinding a session's lane acquire the same `sessions`
  store lock as heartbeat renewal around their own read-modify-write, with the
  same bounded-retry / typed `LOCK_BUSY` discipline; no path writes a session
  record without holding that lock (multisession-native D10a, issue #56 3.8).
- R58 — A lane binding is written only for a lane that already exists: binding
  to a feature with no lane record is refused before the store lock is taken,
  with the shared lane-missing wording, and writes nothing (cell lgd-1,
  2026-08-12).
- R59 — A guard resolves shared state only for the inputs it judges, and a
  refusal whose stated remedy names a command must leave that command runnable.
  The write guard's command check therefore reads the command before resolving
  the acting record: a command carrying none of the version-control verbs it
  judges resolves nothing and is allowed, so an unresolvable lane binding can no
  longer refuse the unbind that escapes it (cell lgd-2, 2026-08-12).
- R60 — Active workers are always a computed join of live-heartbeat sessions
  with lane/workflow binding and cell claims, never the stored `workers`
  array; `startFeature`'s worker precondition (default and `--as-lane` paths
  alike) excludes the calling session's own heartbeat from that computed view
  (multisession-native D6, advisor condition C3).
- R76 — A session record is created at `resolveContext.controlRoot`, never
  the writing checkout's own root, and lazily auto-registers that checkout's
  workspace on first touch; `createSession`/`claimCellFile` stamp
  `workspace_id` on the session and every claim it makes, auto-looked-up from
  the acting session's own resolved workspace, never accepted as a caller-
  supplied value (multisession-native D2/D3, msn-19).

- R88 — **A worker in a shared checkout touches only its own paths, never the
  whole tree.** Concurrency makes the repository a shared resource, so the
  worker's version-control surface narrows to four moves: inspect state, read
  a diff, read the log, and record its own work in one path-scoped commit made
  through its own private index — never the repository's. Every whole-tree
  operation (staging, stashing, checking out, resetting) is forbidden, because
  each one silently takes or discards what every sibling has not yet recorded;
  a worker that produced a wrong commit reports it and lets the delegator
  repair history. Three separate incidents in one wave proved each half: two
  index sweeps that lost commit attribution, and one whole-tree revert that
  destroyed a live worker's in-progress edit while its file reservation was
  held — reservations govern files, not the tree (skill-diet-wave2,
  2026-07-28).
- R86 — **One concurrency law, three tiers.** Work that can run at the same
  time runs at the same time: gathering fans out to read-only workers, the
  cells of a slice fan out to a wave, and independent features fan out to
  lanes (or their own checkouts). Serial is legal for exactly four reasons —
  declared file sets overlap (shared generated artifacts included, unless a
  wave barrier defers them), a true dependency, a single scarce external
  resource, or the human said so — and nothing else counts. Before dispatching
  anything, the orchestrator states the concurrency plan in one line: what runs
  together, what is forced serial, and which of the four reasons forces it —
  computed from declared paths and dependencies, never guessed. A lane refusal
  that names the holding claim is that computation's proof
  (concurrency-first cf-1, user directive 2026-07-28).
- R87 — **An artifact that survives a law change must be re-labelled, or it
  keeps teaching the old law.** When ownership of a step moves, every field,
  prompt, and rendered view that names that step states the new owner at the
  point of reading. Proven the hard way: after verification moved to the
  delegator, workers kept running suites — not from disobedience, but because
  the work record still handed them a field named "verify" holding a runnable
  command with no owner on it, and an artifact outranks a dispatch instruction.
  The field now renders its owner beside it (verify-owner-signal vo-1).
- R88 — **A restatement can be born stricter than the law it restates, and the
  reader obeys the restatement.** R87 covers a copy that survives a law change;
  this is its mirror, and no law has to change for it to happen. R86 has always
  required the concurrency plan *before dispatching anything*, but the two
  places that operationalized it — the router's decision list and the
  lanes-first-class paragraph — each restated it with a precondition the law
  never carried: act only when another feature is already live. The case R86
  exists for, two independent ready features and nothing busy, therefore
  triggered nothing. The evidence is the usage gap: seventeen lane records on
  disk against two occasions of genuinely concurrent features. A rule is only
  as strong as the narrowest restatement an agent reads first, so a law
  expressed in more than one place is audited against its own statement, not
  against its neighbours (lane-plan-unconditional lpu-1). Where the tier can be
  checked, check it: cell-level concurrency is not prose but the output of a
  required command, which an agent must argue against to go serial.
- R85 — **Per-turn rules live in the always-loaded layer, never behind an
  on-demand file.** The communication contract governs every single turn, so
  its operative form sits in the instruction surface that is present in every
  session by construction; the long-form contract may stay in a reference, but
  only as the expansion of a rule already loaded. A rule reachable only by a
  file nothing forces open is a rule nothing follows — proven the hard way: a
  full communication contract sat authored-and-unread with no body pointing at
  it, and the user experienced bee as having no communication style at all. A
  guard now pins the turn shape and the pre-send check to the always-loaded
  layer, so no future text migration can exile them again. This is the one
  standing exception to the thin-body doctrine (comms-always-loaded ca-1,
  user report 2026-07-28).
- R84 — **Every perceivable step is visible as it happens.** The pipeline no
  longer works invisibly and reports at the end: each perceivable step emits
  exactly one short line, on by default, in the user's own work language —
  route recorded, gate passed or auto-approved, work created, workers
  dispatched, results received, work capped, fix opened, verification started
  and its outcome, evidence recorded, sync paid, knowledge synced, learnings
  compounded, feature closed. One fixed shape: a state glyph (started, done,
  red, auto-approved), the event, and its key fact. **Bypass silences
  questions, never ticks**; an explicit quiet setting silences the stream but
  never a red or a refusal; the ship-visibility switch silences only its own
  PR-related lines. Ticks are what the agent writes as it goes, never a
  subsystem to build (step-ticks vt-1, user directive 2026-07-28).
- R83 — **Scribing and compounding are feature-close events.** A feature spans
  many slices and cells; one executing pass is a small part. Mid-feature,
  settlements are captured as same-turn one-line stubs (unchanged duty); the
  spec merge, knowledge sync, and learnings run exactly once, when the
  feature's work fully completes — after the close-time proof check (since
  2026-08-18 / 1f534837 the close checks each cap's recorded proof line
  and runs no suite itself). One boundary,
  three events: tests, sync, compound (feature-close-events fc-1, user
  philosophy decision 2026-07-28).
- R82 — *(Superseded 2026-07-31 — decision 412e9b3a,
  docs/specs/test-simple.md: the pending-cap path, the feature-verify
  record, and both debt markers are deleted. Live rule since 2026-08-18
  (decisions 58ec9664/1f534837, refining 13ce1858): the agent owns test
  scope — every cap records a proof line
  `<command> — <result> — <scope reason>`, `bee close` and `bee worktree
  merge` check that record and run nothing themselves, and CI runs the
  full declared command on every push. Kept below as the historical
  record.)*
  **The delegator verifies, at the shippable unit.** Workers implement,
  commit, and report — they run no suites; a cell caps through the sanctioned
  pending path with no per-cell proof. MAIN produces all evidence: red proof
  (bugfix repro) before dispatch at authoring, and ONE feature-level verify
  (impacted over the feature's diff, cache-assisted) when the full picture
  exists, recorded as a machine-readable feature-verify record (command,
  output hash, result). Leaving the execution phase is refused — typed, and
  no bypass level lifts it — while any cell still OWING proof lacks a green
  record newer than the newest owed cap. **Two markers, one debt** (amended by
  worker-conformance D11/D12): proof deliberately RELOCATED to the feature
  boundary, and proof never recorded AT ALL — a completion that asserted a pass
  with neither real output nor supplied evidence. The door cannot tell them
  apart and must not try, because in both cases the one thing that can still
  prove the cell is the feature-level green run. **The freshness clock runs
  over the UNION of both kinds**, which is the load-bearing half: reading it
  over the relocated caps alone would let a green record newer than the newest
  relocated cap but older than a newer unrecorded cap look fresh, and open the
  door on a cell that run never covered. The second marker is what keeps the
  evidence diet survivable — once the per-unit evidence doors stop refusing, an
  unproven completion carries no relocation marker, and a door armed on
  relocation alone would let a feature close with zero tests executed anywhere.
  A red feature verify opens fix cells in
  the same feature (never un-caps); per-cell commits and bisect localize.
  The classic per-cell evidence path survives for spot use and transition
  (main-verifies D1–D5, two user philosophy decisions 2026-07-28; the two-marker
  debt and the union freshness clock added by worker-conformance D11/D12,
  2026-07-29 — the per-unit teeth themselves live in
  areas/workflow-state/cells-completion-judge-and-archive.md R89–R93).
- R81 — **Worker orientation is brief, not the full status.** The status
  report has a brief form — phase, feature, mode, gates, bypass level, ship
  visibility, route only — that reads nothing but the state layer (no cell
  scans, no review or handoff resolution, no model tables). A dispatched
  worker receives the state line embedded in its dispatch and re-validates
  with the brief form; the claim record stays the sole claim authority. The
  full report remains the orchestrator's routing surface. Measured driver:
  the full report cost 372ms and 15KB per worker startup; brief costs ~70ms
  and ~0.5KB (status-diet D1/D2).
- R80 — **Triage leaves a machine-readable trace.** Every feature carries a
  route record — work class, lane, the counted risk flags, and the product-file
  count — written the same turn the mode gate counts them, through a validated
  verb that refuses free prose and unknown enum values with nothing written.
  The record lives on the feature's workflow record, appears in the runtime
  status report and as one session-preamble line, and a re-lane demotion
  rewrites the same record in place (one route per feature, never a second).
  Claiming a cell of a route-less feature warns once per session and refuses
  from that session's next claim onward — the safety net grew teeth (B52/R103).
  Counting without recording is the guess this rule kills (explicit-triage
  D1–D4; the warn-then-refuse escalation is counter-teeth D4, 2026-08-04).
- R80a — **A route belongs to the feature it was triaged for, and a feature
  starts with none.** The record now carries the feature it was recorded for,
  and starting a feature drops whatever route the previous one left behind —
  a freshly started feature is untriaged, which is exactly the state the
  route-less claim warning above exists to catch. The never-demote rule is
  therefore scoped to one feature's own history: a recorded route naming a
  different feature — or one written before this rule, carrying no feature at
  all — counts as no route, so the next `--set` is a first-time record rather
  than a refused demotion. Without this, the highest lane any finished feature
  ever reached became a permanent floor for every feature after it: measured
  live, two one-file bugfixes inherited a closed feature's high-risk label and
  could not be re-triaged, because the rule is right inside one feature and
  meaningless across a feature boundary (route-identity, cell rti-1,
  2026-08-06).
- R79 — **A feature's workflow record is closed, not abandoned.** Starting a
  new feature closes the outgoing feature's live workflow record(s) (terminal
  status) inside the same guarded mutation that creates the new one, and the
  projection's idle-bootstrap picker never selects a record whose phase is the
  terminal close phase — so a rebuild fired by a stopping subagent can never
  resurrect a finished feature into the live state. Settled after three
  same-day incidents where zombie-active records of closed features were
  picked on SubagentStop rebuilds (foundation-fixes D1/D2).
- R77 — Ship visibility is an opt-in workspace setting with exactly two
  values: off (the default when the key is absent) and draft-pr. The runtime
  status report always carries the resolved value; the session preamble adds
  one line only when the value is draft-pr (first cap opens a draft PR, every
  cap pushes — the push/PR act itself is orchestrator behavior under the
  routing contract, never a runtime side effect). An unrecognized configured
  value resolves to off with a one-line warning, never a failure
  (ship-visibility-config sv-1, spec #81 P1).

- R103 — A route-less feature costs each session one warning per feature and
  refuses every claim after it; the counter advances only on a successful claim,
  and the already-claimed contention refusal always outranks it (counter-teeth
  D4 with refinement 64ad772d, cell ct-5, 2026-08-04).
- R99 — Lane demotion is validated at record time: highest-risk work never
  demotes, a mandatory-ceremony flag on the new classification blocks demotion,
  a feature demotes at most once ever (stamped at the first), and same-lane
  re-records, promotions, and off-ladder moves are always allowed (hook-teeth
  D5, cell bh-5, 2026-08-04).
- R134 — **An unbound session may not blind-write the shared default record
  while lanes are live.** Recording a route from a session with no lane binding
  resolves to the default record — which in an active repo is another feature's
  real triage — so with one or more live lane records the write is refused
  before any field is set, naming the live lanes and what the default record
  currently holds. Both exits ride the refusal: bind the session to its lane, or
  force the default record on purpose. A featureless default record takes its
  own branch and says so, rather than naming a feature "none" whose triage
  would be lost. Route targeting is asymmetric by decision: the record selector
  is a refusal plus one forcing flag, never a second lane-naming flag, because
  the verb already spends that flag name on the triage classification
  (store-reach-gaps D1, 2026-08-21).

## Edge Cases Settled

- A single-user workspace with no session identity anywhere in the
  environment behaves exactly as before: claims and holds are recorded
  ownerless, and the new ownership check never fires against an ownerless
  claim (multi-session-hardening D3/D4).

## Pointers (implementation)

- Ship visibility (R77): `shipVisibility(root)` + `SHIP_VISIBILITY_VALUES` in
  `packages/bee/lib/state.mjs`; status field in `the bee binary`;
  preamble line in `packages/bee/lib/inject.mjs`; suite
  `scripts/tests/test_ship_visibility.mjs`; commits 5d46f40f, f0460a67.
- Lanes (B12): lane store + `resolvePipeline` + lane-mode `startFeature` in
  `packages/bee/lib/state.mjs`; `bindSessionLane`/`unbindSessionLane`
  in `lib/claims.mjs`; CLI: `--lane` on `state.set/gate/scribing-run`,
  `--as-lane/--session-id/--paths` on `state.start-feature`, `state.lanes`,
  `state.session.list/bind/unbind` (`lib/command-registry.mjs` + `bee`,
  runExample rows in `test_bee_cli.mjs`). Evidence: traces
  `.bee/cells/fsh-{3,4}.json`, commits 257d6b5, 6fa4f89;
  `docs/history/fresh-session-handoff/reports/validation-s2.md`.
- Bind existence check (R58) and the guard's git-first scan (R59):
  `bind_lane_missing` + the `lane_missing_refusal` reuse in
  `packages/bee-rs/crates/bee/src/verbs/state_group/sessions.rs`;
  `check_git_bash_command`'s `tokenize_deep` / `find_git_invocations` now ahead
  of `resolve_write_record` in
  `packages/bee-rs/crates/bee/src/hooks/write_guard/checks.rs`. Evidence: cells
  lgd-1/lgd-2, commits 896959ea, 0a5ec197, released in v2.4.8.
- Lock-serialized bind/unbind (D10a): `bindSessionLane`/`unbindSessionLane`
  read-modify-write moved inside `acquireSessionsLock` in
  `packages/bee/lib/claims.mjs`, same bounded-retry/typed
  `LOCK_BUSY` shape as `heartbeatSession`'s own lock hold. Two forced-
  interleaving regression tests (`_raceSeam` hook, same style as
  `lock.mjs`'s `_takeoverSeam`/`_postRenameSeam`) in
  `packages/bee/tests/test_claims.mjs`, proven red-first against
  a reconstructed pre-fix build (10/10 rounds failing both directions).
  Evidence: trace `.bee/cells/multisession-native-1.json`, commit c794eda.
- Active workers (D6): `activeWorkers(root, {excludeSessionId})` in
  `packages/bee/lib/claims.mjs`; `startFeature`'s worker
  precondition (default and `startLane`) reads it instead of
  `state.workers`; `buildStatus`/`renderStatusText` in `bee` gain a
  `workers` field/line sourced from it. `stateWorkerMutate` and
  `handleStateWorkerPrune` keep writing/reading the legacy `workers` array,
  now display-only. Tests: `test_claims.mjs` (unit coverage),
  `test_cli_state.mjs`/`test_state.mjs` (precondition coverage: hand-written
  entry no longer blocks, a live other session's claim still does, C3
  self-exclusion holds). Evidence: trace
  `.bee/cells/multisession-native-8.json`, commit c435add.
- Default/lane writes routed through their workflow record: see
  `workflow-records-and-projections.md` Pointers for `resolveMutationTarget`,
  `writeLaneRecordThroughProjection`/`writeStateRecordThroughProjection`, and
  the `workflow:<id>` lock they acquire.
- Session workspace stamping (R76): `packages/bee/hooks/bee-session-init.mjs`
  (control-plane session creation + lazy workspace auto-register);
  `createSession`/`claimCellFile` in `lib/claims.mjs`. Evidence: trace
  `.bee/cells/multisession-native-19.json`, commit 09e1ed0. Full workspace
  registry and write-policy mechanics:
  `areas/worktree-parallelism/control-plane-topology.md`.
- Route-less claim escalation (B52/R103): the per-(feature, session) counter at
  `.bee/no_route_claims/<feature>__<session-fingerprint>.json` under the control
  root — `no_route_claim_key`, `no_route_claim_count_path` and
  `bump_no_route_claim_count` in
  `packages/bee-rs/crates/bee/src/verbs/cells/handlers_write.rs:723-815`,
  bumped only after a successful claim (`handlers_write.rs:990`); the refusal is
  code `NO_ROUTE_RECORD` at `handlers_write.rs:1107-1113`, ordered after the
  already-claimed refusal (`handlers_write.rs:1059-1064`) so a race loser sees
  the contention refusal instead. Evidence: trace `.bee/cells/ct-5.json`,
  commits 4a0d1b82 and 95ec0639 (cells 75 passed, concurrency 13 passed, full
  suite green, 2026-08-04).
- Re-lane transition validation (B48/R99): `validate_route_lane_transition`
  with `triage_ladder_rank` (`tiny` 0 < `small` 1 < `standard` 2; `docs`,
  `spike` and `high-risk` are off-ladder) and `HARD_GATE_ROUTE_FLAGS` (`auth`,
  `authorization`, `data-model`, `audit-security`, `external-systems`,
  `proof-weakening`) in
  `packages/bee-rs/crates/bee/src/verbs/state_group/workflows.rs:311-413`; the
  once-ever bound is carried by the route record's `demoted_at` stamp, minted
  on an allowed demotion and preserved otherwise (`workflows.rs:405-412`).
  Red-first per hook-teeth D7: the classification tests landed against a stub
  before the refusals wired in. Evidence: trace `.bee/cells/bh-5.json`, commit
  95fe412d (state_group 56 passed, 2026-08-04).
