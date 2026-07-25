---
type: bee.area
title: "Workflow State — working sessions, self-derived identity, lanes, and the renewing heartbeat"
description: "Who the acting session is (resolved from its own environment, never handed down), how a feature gets its own pipeline lane that every reader resolves through, how a live session's heartbeat renews itself and carries its claims and holds forward with it, how lane binding now shares the same store lock as the heartbeat so the two writers of one session record can never lose each other's update, and how active workers are always a computed join of live sessions and claims rather than a stored array."
timestamp: 2026-07-25
bee:
  id: workflow-state-sessions-lanes-and-identity
  lifecycle: active
  areas: [workflow-state]
  required_context: [areas/workflow-state/overview.md]
  decisions: [multi-session-hardening D3/D5 with Δ1-Δ6 amendments (session self-derivation; throttled heartbeat and lease renewal), "fresh-session-handoff D2 (a lane never borrows the default pipeline's authority)", "hardening-1-7-10 (the durable single-fresh-session identity fallback, audited, at library and CLI levels)", i54-closeout D7, "multisession-native D10a (issue #56 3.8 — bindSessionLane/unbindSessionLane serialize under the same sessions store lock heartbeatSession already uses, closing the lost-update race between them)", "multisession-native D6 (active workers derived from live-heartbeat sessions + lane/workflow binding + claims, never the stored workers array; advisor condition C3 — startFeature excludes the calling session's own heartbeat)"]
  sources: ["fresh-session-handoff cells fsh-3/fsh-4 (lane store, resolvePipeline, lane-mode startFeature; validation-s2, 2026-07-13)", "multi-session-hardening cells msh-1..7 (traces in .bee/cells/, reports docs/history/multi-session-hardening/reports/, 2026-07-19)", hardening-1-7-10 cells 1710-1..1710-11 (2026-07-21), "docs/specs/workflow-state.md#B12", "docs/specs/workflow-state.md#B13", "docs/specs/workflow-state.md#B22", "docs/specs/workflow-state.md#B24", "docs/specs/workflow-state.md#R38", "docs/specs/workflow-state.md#R55", "docs/specs/workflow-state.md#E22", "docs/specs/workflow-state.md#P14", "i54-closeout cell i54-closeout-7 (resolveMutationTarget lane auto-resolve for state-write verbs; trace in .bee/cells/, 2026-07-24)", "multisession-native cell multisession-native-1 (trace .bee/cells/multisession-native-1.json, commit c794eda, 2026-07-24)", "multisession-native cell multisession-native-8 (activeWorkers derivation, trace .bee/cells/multisession-native-8.json, commit c435add, 2026-07-25)", "multisession-native cell multisession-native-10 (default-path mutation now also routes through its workflow record; trace .bee/cells/multisession-native-10.json, commit e7f365a, 2026-07-25)"]
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
pipeline record and resets exactly its four gates in one atomic write, leaving
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
fresh-session-handoff D2). An unbound session sees no behavior change at all:
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
judge the acting session's own lane. What each actor observes in a zero-lane
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
- R60 — Active workers are always a computed join of live-heartbeat sessions
  with lane/workflow binding and cell claims, never the stored `workers`
  array; `startFeature`'s worker precondition (default and `--as-lane` paths
  alike) excludes the calling session's own heartbeat from that computed view
  (multisession-native D6, advisor condition C3).

## Edge Cases Settled

- A single-user workspace with no session identity anywhere in the
  environment behaves exactly as before: claims and holds are recorded
  ownerless, and the new ownership check never fires against an ownerless
  claim (multi-session-hardening D3/D4).

## Pointers (implementation)

- Lanes (B12): lane store + `resolvePipeline` + lane-mode `startFeature` in
  `skills/bee-hive/templates/lib/state.mjs`; `bindSessionLane`/`unbindSessionLane`
  in `lib/claims.mjs`; CLI: `--lane` on `state.set/gate/scribing-run`,
  `--as-lane/--session-id/--paths` on `state.start-feature`, `state.lanes`,
  `state.session.list/bind/unbind` (`lib/command-registry.mjs` + `bee.mjs`,
  runExample rows in `test_bee_cli.mjs`). Evidence: traces
  `.bee/cells/fsh-{3,4}.json`, commits 257d6b5, 6fa4f89;
  `docs/history/fresh-session-handoff/reports/validation-s2.md`.
- Lock-serialized bind/unbind (D10a): `bindSessionLane`/`unbindSessionLane`
  read-modify-write moved inside `acquireSessionsLock` in
  `skills/bee-hive/templates/lib/claims.mjs`, same bounded-retry/typed
  `LOCK_BUSY` shape as `heartbeatSession`'s own lock hold. Two forced-
  interleaving regression tests (`_raceSeam` hook, same style as
  `lock.mjs`'s `_takeoverSeam`/`_postRenameSeam`) in
  `skills/bee-hive/templates/tests/test_claims.mjs`, proven red-first against
  a reconstructed pre-fix build (10/10 rounds failing both directions).
  Evidence: trace `.bee/cells/multisession-native-1.json`, commit c794eda.
- Active workers (D6): `activeWorkers(root, {excludeSessionId})` in
  `skills/bee-hive/templates/lib/claims.mjs`; `startFeature`'s worker
  precondition (default and `startLane`) reads it instead of
  `state.workers`; `buildStatus`/`renderStatusText` in `bee.mjs` gain a
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
