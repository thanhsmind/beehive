---
type: bee.area
title: Workflow State — cross-session file holds and the coordination lock behind every shared write
description: "The write-time refusal that names the live session holding a path and when its hold expires, the bounded-wait lock every shared coordination store's read-modify-write body runs inside — including exactly when a stale holder may be taken over and when it may not — the fail-open contention telemetry every lock acquire now records, surfaced as a bounded summary in bee status, the per-workflow lock order that keeps two features' state mutations from ever contending on one lock, and the sharded per-resource lease store that now backs every reservation instead of one shared file."
timestamp: 2026-07-25
bee:
  id: workflow-state-holds-and-the-coordination-lock
  lifecycle: active
  areas: [workflow-state]
  required_context: [areas/workflow-state/overview.md, areas/worktree-parallelism/control-plane-topology.md]
  decisions: ["multi-session-hardening D2 with Δ1/Δ3 amendments (the coordination lock: verbs wait bounded, checkpoints try once)", "fresh-session-handoff D3 (a write into another live session's held path is refused at write time)", hardening-1-7-10 (liveness-probed stale takeover with a one-hour pid-reuse ceiling; no timer heartbeat by design), "multisession-native (stage 0-1: lock-acquire outcomes recorded as fail-open contention telemetry; bee status surfaces a bounded contention summary from that telemetry)", "multisession-native D1 (state mutation locks its own feature's workflow:<id>, never a blanket state lock; advisor condition C4 — the sessions lock and a workflow lock are never held together)", "multisession-native D4 (slice 3, issue #56: reservations become sharded per-resource lease records; a declared intent scope is advisory, never a hard block; fencing on leases; .bee/reservations.json retired to a rebuildable projection — docs/history/multisession-native/CONTEXT.md)", "multisession-native advisor-digest-slice3 conditions A/B/C/D/E (docs/history/multisession-native/reports/advisor-digest-slice3.md — leases root stays worktree-local until slice 4's controlRoot re-root; the cross-worktree mirror-write reserve seam preserved byte-for-byte; the reservations projection must never let a lazy rebuild green-light a colliding feature start; intra-swarm/cross-session conflicts stay hard leases, only planning-declared broad/glob paths become advisory intents; lease renewal rides the existing heartbeat try-once posture)", "multisession-native D2 (slice 4: leases are control-plane — every lease-store call site resolves through controlRootFor(root), closing slice 3's deferred condition A — docs/history/multisession-native/CONTEXT.md, decision e1ceca12)", "multisession-native D3/msn-21 (a lease's stamped workspace_id, dormant since msn-11, now decides a same-path conflict: same-workspace stays a hard deny, a different workspace downgrades to advisory — docs/history/multisession-native/CONTEXT.md)"]
  sources: ["fresh-session-handoff cells fsh-7/fsh-8 (phase-independent deny + fail-closed corrupt-store branch; validation-s3, 2026-07-13)", "multi-session-hardening cells msh-1..7 (coordination lock primitive and forked-racer suites, 2026-07-19)", hardening-1-7-10 cells 1710-1..1710-11 (2026-07-21), "docs/specs/workflow-state.md#B14", "docs/specs/workflow-state.md#B21", "docs/specs/workflow-state.md#R37", "docs/specs/workflow-state.md#R52", "docs/specs/workflow-state.md#P15", "multisession-native cell multisession-native-3 (contention telemetry in lock.mjs; trace .bee/cells/multisession-native-3.json, commit 2d66ccc, 2026-07-24)", "multisession-native cell multisession-native-4 (bee status contention summary; trace .bee/cells/multisession-native-4.json, commit 1865cae, 2026-07-24)", "multisession-native cell multisession-native-10 (per-workflow withMutationLock replacing the blanket state lock; trace .bee/cells/multisession-native-10.json, commit e7f365a, 2026-07-25; advisor digest docs/history/multisession-native/reports/advisor-digest-slice2.md condition C4)", "multisession-native cell multisession-native-11 (sharded lease-store.mjs; trace .bee/cells/multisession-native-11.json, commit ad19826, 2026-07-25)", "multisession-native cell multisession-native-13 (intent vs write lease split; trace .bee/cells/multisession-native-13.json, commit debe0d9, 2026-07-25)", "multisession-native cell multisession-native-16 (reservations.mjs shim over lease-store; .bee/reservations.json retired to a projection; trace .bee/cells/multisession-native-16.json, commit 81a936c, 2026-07-25)", "multisession-native cell multisession-native-18b (reservations.mjs's lease-store gateway re-rooted onto controlRootFor via its own fail-open findMainRoot replica; trace .bee/cells/multisession-native-18b.json, commit a1431448, 2026-07-25)", "multisession-native cell multisession-native-21 (guards.mjs checkWrite reads a lease's stamped workspace_id to scope the same-path conflict; trace .bee/cells/multisession-native-21.json, commit 3f56916, 2026-07-25; see areas/worktree-parallelism/control-plane-topology.md)"]
  authoritative_for: "workflow-state: cross-session file holds and the shared-store coordination lock"
---

# Workflow State — cross-session file holds and the coordination lock behind every shared write

Two guards keep concurrent work from quietly destroying each other. One is
visible to the person writing: a path another live session holds is refused at
write time, naming the holder and when the hold lapses. The other is invisible
until it is missing: every shared store's read-modify-write body runs inside a
lock, and a holder that is merely old is a takeover *candidate*, never an
automatic steal.

## Behaviors & Operations

**B14 — A write into another live session's held path is refused at write
time.** Trigger: any write attempt while the acting session's identity is
known. What happens: when the path overlaps a hold owned by a *different*
session that is still live (within its lifetime), the write is refused with a
typed message naming the holder — its session and its worker name — and when
the hold expires. What never blocks: the acting session's own holds, expired
holds, and legacy holds that predate session ownership (they carry no owning
session and keep their original worker-level meaning). The refusal is
unconditional on workflow phase — it fires in every phase, including mid-
execution. When the hold ledger exists but cannot be read, a session-aware
write is refused (fail-closed, as a returned refusal that survives the
guard's fail-open crash handling — never a thrown error, which the guard
would swallow into an allow); a ledger that simply does not exist blocks
nothing. What each actor observes: the blocked session gets the who-and-until
message and stays healthy (free to pick other work); the holding session is
undisturbed; a repo with no session-owned holds behaves exactly as before
(fresh-session-handoff D3).

**B21 — A shared coordination store serializes its own concurrent writes.**
Trigger: two or more sessions attempt a read-modify-write against the same
shared coordination store (a hold ledger, the durable workflow record) at the
same moment. What happens: each read-modify-write body runs inside that
store's coordination lock (Data Dictionary) — acquired by exclusive creation
with bounded retry, and a stale holder is taken over by an atomic handoff
rather than an unconditional removal, so a waiter can never delete a fresh
holder's lock out from under it; staleness is re-verified at every retry,
never cached. Crossing the ordinary staleness age only makes a holder a
takeover candidate: the actual steal requires either a liveness probe to find
the recorded owner process provably dead (a probe that comes back
permission-denied still counts as alive, never as dead — an inconclusive
answer is never license to steal) or an absolute one-hour ceiling to have
passed regardless of what the probe reports, guarding against a reused pid
being mistaken for the original owner. A holder that is genuinely still alive
therefore keeps the lock legitimately across a long synchronous child spawn —
a worktree merge running its verify command is the motivating case — for as
long as the ceiling allows; no timer heartbeat renews the hold from inside the
holder by design, since the holder's own synchronous child spawns already
block the event loop a timer would need (hardening-1-7-10). A command-line
verb waits for the lock, bounded; a lifecycle checkpoint never waits — it
tries once and, if busy, skips its own update silently, preserving the
checkpoint's existing fail-open discipline. On a genuine timeout the caller
receives the typed `LOCK_BUSY` refusal naming the current holder — never a
silent fall-through to an unlocked write, because mutual exclusion is the
entire point. What each actor observes: no hold and no
coordination-record write is ever silently dropped by a second concurrent
writer; a checkpoint that loses the race simply skips one opportunistic
refresh, with the next one along shortly (D2).

**Every store-lock acquire outcome is recorded as fail-open contention
telemetry (multisession-native, stage 0-1).** Trigger: any attempt to acquire
a shared coordination store's lock — both the retrying path a command-line
verb waits on and the try-once path a lifecycle checkpoint uses. What
happens: the outcome — acquired immediately, acquired after retry, or
`LOCK_BUSY` exhaustion — appends one line to a dedicated contention log,
carrying when it happened, which lock, how long the caller waited, who
currently holds the lock, who is asking, and (reserved for a later cell to
populate) which workflow, workspace, and resource the wait was over. The
append is itself lock-free — a plain best-effort file write, not gated by the
lock it is reporting on — and every failure writing it is swallowed, up to
and including the log file being unwritable: telemetry can never turn a
successful acquire into a reported failure, and a broken log can never be the
reason an acquire or a hook's heartbeat trips. What each actor observes: a
session that later asks "why am I waiting" has a record to answer from; a
session that never contends a lock produces no new telemetry, no behavior
change, and no measurable extra latency.

**`bee status` surfaces the same contention log as a bounded summary
(multisession-native, stage 0-1).** Trigger: `bee status` or
`bee status --json` runs while the contention log holds at least one busy
(`LOCK_BUSY`) event inside its recent window. What happens: status reads a
fixed, bounded 64KB tail of the log — never a full-file scan — through the
same windowed, malformed-line-skipping, never-throws reader already used to
recover a crashed session's own transcript tail, itself sometimes multiple
megabytes; a line that fails to parse is skipped rather than aborting the
read. From that window it reports how many busy events occurred, the busiest
locks (at most 5, worst wait first), the single worst wait and which lock it
fell on (measured across every event in the window, not only the busy ones,
so a caller that eventually acquired after a long retry still shows up), and
the most recent busy events (at most 5, newest first). What each actor
observes: a session waiting on a contended lock gets a concrete answer
instead of silence; a repo with no contention in the window — including one
with no contention log at all — sees no contention information in status,
the same additive silence status already uses elsewhere for a signal with
nothing to report.

**A workflow's own lock, not one shared store lock, now serializes state
mutation (multisession-native D1/C4).** Trigger: any state-mutation write
(`state set`, `state gate`, `state scribing-run`, `state advisor-ref record`,
`start-feature`) against a feature that has a live workflow record. What
happens: the write acquires that workflow's own `workflow:<id>` lock — never
the single blanket `state` lock every one of those verbs held through
`multisession-native-9` — so two different features' state mutations no
longer contend on one lock at all (closing issue #56 3.1/3.2). This sits
beside, not instead of, the `sessions` store lock (B21/R37-style: heartbeat
renewal, lane bind/unbind): the two are never held together in the same
operation — a fixed lock-order rule, not a convention to remember. A workflow
lock is acquired, its body runs, and it is released before any `sessions`
lock the same logical operation might need is taken (or the reverse never
happens at all on any of these paths) — an operation never asks for both at
once. Crossing a workflow's own lock with a *different* feature's workflow
lock is likewise impossible: each write names its own `id`, so the only lock
two writers could ever contend on is one they both genuinely target the same
workflow. See `workflow-records-and-projections.md` for the workflow record
itself and exactly which write paths route through this lock. What each actor
observes: a session mutating feature A's state is never blocked by another
session mutating feature B's state, even while both are mid-write at the
same instant; a missing or corrupt workflow record still refuses loudly
(typed `WORKFLOW_MISSING`/`WORKFLOW_CORRUPT`) rather than silently falling
back to an unlocked write.

**Reservations live as sharded per-resource lease records, not one shared
file (multisession-native D4, slice 3).** Trigger: any reservation acquire,
release, renew, or sweep — a swarming worker's path reservation, a
cross-session file hold. What happens: each lease lives at its own path —
`.bee/runtime/leases/cells/<cell-id>.json` or
`.bee/runtime/leases/paths/<path-hash>.json` — carrying `{resource, mode,
kind, workflow_id, session_id, workspace_id, epoch, acquired_at,
expires_at}`, so two unrelated resources never contend on a shared file at
all (the same "sharded beats whole-file" shift `workflow-records-and-
projections.md`'s own per-workflow lock already made for state mutation).
Acquiring one or more leases in a single call canonicalizes every request
first, then sorts them by the sha256 hash of their resource key *before* any
file is created — so two callers requesting the same resources in opposite
order still attempt creation in the identical global order, the standard
fixed-order deadlock-avoidance discipline applied to O_EXCL creates instead
of mutex acquisition. Each lease is O_EXCL-created; on the first collision,
every lease already created by that same call is rolled back (deleted)
before the typed `LEASE_HELD` refusal is thrown, naming the conflicting
holder — a partial acquire is never left standing. Release/renew/sweep are
per-record: no store-wide lock exists for this store at all; a renew takes
only a per-resource `lease:<hash>` lock, so renewing one lease never
contends with acquiring, renewing, or releasing a different one. TTL
semantics match the pre-existing store exactly: a non-positive TTL stores
`expires_at: null` and is never swept; every expiry decision reads the raw
`expires_at`, never a cached staleness verdict.

**A lease's stored epoch can fence a stale renew or release (multisession-
native D4/D9 invariant 10).** Trigger: `renewLease`/`releaseLease` called
with an optional `presentedEpoch`. What happens: `epoch` is stored verbatim
on every acquire — this store never compares it against a prior value at
acquire time, and a genuine takeover is release-then-acquire with the new
caller deciding the bumped value, never a forced-takeover primitive here.
Given a `presentedEpoch` behind the resource's currently stored `epoch`, a
renew or release refuses typed `LEASE_FENCE_STALE` and leaves the record on
disk untouched — a takeover already moved ownership forward. Omitted (every
production caller today — this store has no production wiring yet), both
verbs stay byte-unchanged from before fencing existed; full mandatory
presentation arrives with workspace identity in a later slice. See
`claims-and-ownership.md` for the equivalent `fence_epoch` token on claims
themselves (`CLAIM_FENCE_STALE`) — a deliberately distinct name and a
deliberately separate mechanism, never shared code, per each store's own
structural-isolation rule.

**Leases moved onto the control plane, and their long-dormant `workspace_id`
field now decides a same-path conflict (multisession-native D2/D3, slice 4,
msn-18b/msn-21).** Trigger: any lease acquire, renew, release, or a write
whose path collides with another session's already-held exact-path lease.
What happens: `reservations.mjs` — the sole lease-store gateway — now resolves
its store root through its own fail-open `findMainRoot`/`controlRootFor`
replica rather than the writing checkout's own root, closing the gap slice
3's advisor condition A had deliberately deferred ("leases root stays
worktree-local until slice 4's controlRoot re-root"); every checkout now
reads and writes the identical lease files. Separately, the `workspace_id`
every lease has carried since `multisession-native-11` (forward groundwork,
unconsumed until now) is read by the write guard's conflict check: a
collision with a lease stamped with the **same** `workspace_id` as the acting
session stays a hard deny, exactly as before; a collision with a lease from a
**different** workspace downgrades to an allow (the write guard's own deny
class (b) — see `areas/worktree-parallelism/control-plane-topology.md`).
Legacy and solo-repo leases (no `workspace_id` recorded, defaulting to
`'main'` on both sides) are byte-identical under this check — the scoping
only changes behavior once two genuinely different workspaces are in play.
This is layered on top of, not instead of, the `'intent'`/`'lease'` kind
split described next: a same-workspace `'lease'`-kind conflict is still hard,
a same-workspace `'intent'`-kind conflict is still advisory, and now a
different-workspace conflict of either kind is advisory regardless of kind.
What each actor observes: two sessions holding the identical exact-path
lease inside the SAME workspace still collide exactly as they always did; the
same collision across two different granted worktrees now reads as a warning
naming the other workspace, not a refusal.

**A reservation's declared kind decides whether a same-workspace conflict is
a hard block or an advisory (multisession-native D4, slice 3, advisor
condition D).** Trigger: a swarming-phase write, or `reservations reserve`
itself, whose path overlaps another agent's already-declared reservation in
the SAME workspace. What happens: every reservation record now carries a
`kind` of `'intent'` or `'lease'`, defaulting to `'lease'` when the caller
omits it — every reservation before this cell stays semantically a lease,
byte-unchanged in behavior. `'lease'` marks an exact path a writer is
actually about to touch (hard, exactly as before this cell); `'intent'`
marks a planning-declared broad or glob scope (advisory: a warning plus a
scheduling input, never a hard block through either chokepoint it can reach —
the write guard or `reserve()`'s own pre-check). Both chokepoints classify a
conflict through the SAME shared predicate, `isHardConflict(reservation,
targetPath)`: true (hard) unless the conflicting reservation's kind is
`'intent'` AND its declared path only broadly covers the target rather than
matching it exactly. `reserve()`'s own conflict pre-check was fixed onto this
same predicate in the same cell — a pre-existing intent record would
otherwise have silently hard-refused a fellow worker's later exact `reserve()`
call through a different chokepoint than the write guard, caught by a
red-first regression test. Cross-session file holds (B14) and wave-planning's
`pathsOverlap` containment semantics (`schedule.mjs`/`state.mjs`/`cells.mjs`)
are completely untouched by this split — it applies only to the intra-swarm
agent-reservation conflict, never to a cross-session hold or to what counts
as "overlap" for dispatch scheduling. What each actor observes: an agent
whose declared exact-path lease collides with another agent's still gets the
same hard `reservation conflict` refusal as before; an agent whose write only
falls inside another agent's broadly-declared intent gets an allow plus a
non-blocking warning naming the declaring agent, its cell, and the declared
scope.

**`.bee/reservations.json` is now a rebuildable projection of the lease
store, never a second source of truth (multisession-native D4, slice 3,
msn-16, advisor condition C).** Trigger: `reserve`/`release`/`sweepExpired`/
`renewHoldsBySession`/`listReservations`/`findConflicts`/
`findSessionConflicts`, or any projection rebuild. What happens: every one of
those verbs now runs entirely on top of the sharded lease-store files instead
of one whole-file JSON store guarded by a single store-wide `'reservations'`
lock — two callers reserving different resources no longer contend on one
lock at all. `.bee/reservations.json` itself becomes a rebuildable
projection (`rebuildReservationsProjection`, registered in
`state-projection.mjs`'s `rebuildAllProjections` alongside the state/lane/
handoff projections) — never written synchronously on the reserve/release/
renew/sweep hot path. `state.mjs`'s own `listActiveReservationsForStart`
(the `start-feature` precondition that must never green-light a colliding
start) now reads through the live `listReservations()` shim instead of the
legacy file directly, so it always sees `reserve()`'s true current state
rather than a possibly-stale projection (closing the advisor's condition C
concern about a lazily-rebuilt projection). One seam is deliberately left
byte-for-byte untouched (advisor condition B, the biggest risk in this cell):
`bee.mjs`'s atomic cross-worktree seam — `findForeignHolds` + the reservation
write + the mirrored `insertHold` into the cross-worktree ledger — still runs
as ONE section under `withHoldsLock`, exactly as before the shim; a new
CLI-level regression test proves both the mirror write and the foreign-hold
deny still fire correctly through the shimmed store. See
`worktree-parallelism/cross-worktree-holds.md` for that ledger. What each
actor observes: reservation behavior is unchanged at every call site; the
legacy file, when read, always reflects the lease store's true current state
after the next projection rebuild, never a hand-maintained duplicate that can
drift from it.

## Business Rules

- R37 — A shared coordination store's read-modify-write body always serializes
  through its coordination lock: a command-line verb waits (bounded), a
  lifecycle checkpoint tries once and skips silently on contention — never a
  fall-through to an unlocked write (multi-session-hardening D2, Δ1/Δ3-amended).
- R52 — A stale coordination-lock holder is a takeover candidate only; the
  steal itself requires a liveness probe to find the recorded owner provably
  dead (a permission-denied answer counts as alive) or an absolute one-hour
  ceiling to have passed regardless of the probe, so a genuinely live holder
  keeps the lock across a long synchronous child spawn and no timer heartbeat
  is needed or attempted (hardening-1-7-10).
- R58 — Every store-lock acquire outcome — success with zero retries, a
  retried success, or `LOCK_BUSY` exhaustion — on both the async retrying
  path and the hook try-once path, appends one fail-open contention-telemetry
  line; the append never itself takes a lock, and a telemetry failure never
  turns a real acquire result into something else (multisession-native).
- R59 — `bee status`/`bee status --json` reads at most a bounded 64KB tail of
  the contention log and reports `busy_count`, `top_locks` (≤5),
  `worst_wait_ms`/`worst_wait_lock`, and `recent_busy` (≤5); the whole key is
  omitted when the log is absent or the window holds no busy event, and a
  malformed line is skipped rather than failing the read (multisession-native).
- R61 — State-mutation writes (default and lane alike) lock their own
  feature's `workflow:<id>`, never a blanket `state` lock; a `sessions` store
  lock and a `workflow:<id>` lock are never held together in the same
  operation (fixed lock order, no AB-BA), so two different features' state
  mutations never contend on a shared lock (multisession-native D1, advisor
  condition C4).
- R68 — Every reservation lives at its own per-resource file under
  `.bee/runtime/leases/{cells,paths}/`; acquiring a batch canonicalizes and
  hash-sorts every request before any file is created (deadlock-free
  regardless of caller order), and the first collision rolls back every
  lease already created by that same call before refusing typed
  `LEASE_HELD` — a partial acquire is never left standing (multisession-native
  D4, msn-11).
- R69 — `renewLease`/`releaseLease` accept an optional `presentedEpoch` and
  refuse typed `LEASE_FENCE_STALE`, record untouched, when it is behind the
  resource's currently stored `epoch`; omitted, both stay byte-unchanged from
  before fencing existed (multisession-native D4/D9 invariant 10, msn-12).
- R70 — A reservation's `kind` (`'intent'`|`'lease'`, default `'lease'`)
  decides whether a same-workspace conflict hard-blocks: an exact-path or
  default-kind conflict still denies exactly as before; a broadly-scoped
  `'intent'` downgrades to an allow plus a non-blocking warning, at both the
  write-guard chokepoint and `reserve()`'s own pre-check, via the same shared
  `isHardConflict` predicate. Cross-session holds and wave-scheduling's
  `pathsOverlap` containment are untouched (multisession-native D4, msn-13,
  advisor condition D).
- R71 — `.bee/reservations.json` is a rebuildable projection of the sharded
  lease store, never written synchronously on the reserve/release/renew/sweep
  hot path; `start-feature`'s own reservation precondition reads the live
  shim, never the possibly-stale file, so it can never green-light a
  colliding start. The cross-worktree mirror-write reserve seam
  (`findForeignHolds` + reservation write + `insertHold`, one atomic section
  under `withHoldsLock`) stays byte-for-byte unchanged through the shim
  (multisession-native D4, msn-16, advisor condition B/C).
- R75 — The lease store resolves through `controlRootFor(root)`, never the
  writing checkout's own root, closing slice 3's deferred condition A; a
  same-path lease conflict is a hard deny only when the colliding lease's
  stamped `workspace_id` matches the acting session's own — a different
  workspace's lease downgrades to advisory regardless of `'intent'`/`'lease'`
  kind; legacy/solo leases with no `workspace_id` stay byte-identical
  (multisession-native D2/D3, slice 4, msn-18b/msn-21).

## Edge Cases Settled

- A contention log that does not exist yet, or whose recent window holds zero
  `LOCK_BUSY` events, produces no `contention` key in status at all — silent,
  not an empty object and not an error (multisession-native).
- A malformed or partially written line inside the tail window is skipped,
  never treated as a parse failure that aborts the read (multisession-native).
- A lease request presenting no `presentedEpoch` (every production caller
  today) behaves byte-identically to before fencing existed on this store —
  the fencing check never runs (multisession-native D4/D9, msn-12).
- Releasing an absent lease is `{ ok: true, released: false }`, never an
  error, whether or not a `presentedEpoch` was given — there is nothing on
  disk left to fence against (multisession-native, msn-12).

## Pointers (implementation)

- Hold enforcement (B14): `findSessionConflicts` + optional `session` field in
  `skills/bee-hive/templates/lib/reservations.mjs`; phase-independent deny +
  fail-closed corrupt-store branch in `lib/guards.mjs` `checkWrite`;
  `payload.session_id` threaded at `hooks/bee-write-guard.mjs`; `--session` on
  the reservations verb. Evidence: traces `.bee/cells/fsh-{7,8}.json`, commits
  255757d, 4969e8c; `docs/history/fresh-session-handoff/reports/validation-s3.md`.
- Contention telemetry: `appendContentionTelemetry` in `lock.mjs`, called from
  both `withStoreLock` (retrying async path) and
  `acquireStoreLockOnceSync` (hook try-once path); schema `{ts, lock_name,
  lock_wait_ms, holder_session, caller_session, workflow_id, workspace_id,
  resource, result}` written to `.bee/logs/contention.jsonl` via a plain
  `fs.appendFileSync`, mirroring `bee.mjs`'s own `timings.jsonl`
  `recordTiming`. Evidence: `scripts/test_store_lock.mjs` scenario (i);
  trace `.bee/cells/multisession-native-3.json`, commit 2d66ccc.
- Status contention summary: `buildContentionSummary` in `bee.mjs`, reading
  through `readTranscriptTail` (`lib/recovery.mjs`) for the bounded,
  windowed, never-throws read. Evidence:
  `skills/bee-hive/templates/tests/test_contention_status.mjs` (seeded
  fixture aggregation, text-render mention, absent-log silence, malformed-
  line skip, 8MB garbage-head tail-window proof); trace
  `.bee/cells/multisession-native-4.json`, commit 1865cae.
- Per-workflow lock order (D1/C4): `withMutationLock` in
  `skills/bee-hive/templates/lib/state.mjs` (locks per-workflow instead of
  the blanket `state` lock every state-mutation verb held through
  `multisession-native-9`); two deterministic seam tests proving zero
  cross-workflow `LOCK_BUSY` and decoupling from the `sessions` lock. Trace
  `.bee/cells/multisession-native-10.json`, commit e7f365a. See
  `workflow-records-and-projections.md` Pointers for the full write path.
- Sharded lease store: `acquireLeases`/`releaseLease`/`renewLease`/
  `renewLeasesBySession`/`sweepExpiredLeases`/`listLeases` in
  `skills/bee-hive/templates/lib/lease-store.mjs` (imports only node
  builtins, `fsutil.mjs`, and `lock.mjs` — proven never to import
  `claims.mjs`/`state.mjs`/`reservations.mjs` by a static source-scan test,
  mirroring `workflow-store.mjs`'s own C4 proof). 12 tests in
  `test_lease_store.mjs` cover the round-trip, partial-acquire rollback (zero
  residue), disjoint-resource non-contention, hash-sort determinism under
  reversed request order, and TTL never-expires semantics. Evidence: trace
  `.bee/cells/multisession-native-11.json`, commit ad19826.
- Lease/claim fencing (msn-12): `LEASE_FENCE_STALE` in `lease-store.mjs`'s
  `renewLease`/`releaseLease`. Red-first: new refusal tests captured failing
  against pristine `lease-store.mjs`, then green after the implementation.
  Evidence: trace `.bee/cells/multisession-native-12.json`, commit 8c002a1.
- Intent vs write lease (msn-13): `RESERVATION_KINDS`, `isHardConflict` in
  `skills/bee-hive/templates/lib/reservations.mjs`; classification wired into
  `guards.mjs`'s `checkWrite` (swarming-phase branch) and `reserve()`'s own
  conflict pre-check; `--kind` flag on `bee reservations reserve`; equivalent
  `kind` field added to `lease-store.mjs` as forward groundwork. Red-first
  regression test proved a pre-existing intent record could otherwise refuse
  a fellow worker's exact `reserve()` call through the pre-check chokepoint.
  Evidence: trace `.bee/cells/multisession-native-13.json`, commit debe0d9.
- Reservations shim over the lease store (msn-16): `reserve`/`release`/
  `sweepExpired`/`renewHoldsBySession`/`listReservations`/`findConflicts`/
  `findSessionConflicts`/`leaseToReservation` in `reservations.mjs`;
  `rebuildReservationsProjection` (wired into `state-projection.mjs`'s
  `rebuildAllProjections`); `listActiveReservationsForStart` in `state.mjs`
  reading through the live shim. A CLI-level regression test proves the
  cross-worktree mirror write and the foreign-hold deny both still hold
  through the shim (`bee.mjs`'s `withHoldsLock` seam at the
  `findForeignHolds`/reserve/`insertHold` call site, untouched). Evidence:
  trace `.bee/cells/multisession-native-16.json`, commit 81a936c; advisor
  consult for this whole slice:
  `docs/history/multisession-native/reports/advisor-digest-slice3.md`
  (conditions A-G, verdict proceed-with-conditions).
- Leases onto the control plane + workspace-scoped conflict (R75):
  `reservations.mjs`'s own `findMainRoot`/`controlRootFor` replica (trace
  `.bee/cells/multisession-native-18b.json`, commit a1431448); the write
  guard's `workspace_id` read in `guards.mjs` `checkWrite` (trace
  `.bee/cells/multisession-native-21.json`, commit 3f56916). Full topology
  resolver and the guard's three deny classes:
  `areas/worktree-parallelism/control-plane-topology.md`.
