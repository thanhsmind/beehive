---
type: bee.pattern
title: "A lost-update race between two writers of one record closes only when BOTH lock — locking one side moves the race, it does not close it"
description: A shared record with two read-modify-write code paths still races even after one path takes a lock; the race closes only once every writer of that record shares the same lock.
tags: [architecture, concurrency, locks, race-condition, sessions, advisor]
timestamp: 2026-07-24
bee:
  id: pattern-20260724-a-lost-update-race-closes-only-when-both-writers-lock
  lifecycle: active
  sources: ["multisession-native cell multisession-native-1 (trace .bee/cells/multisession-native-1.json, commit c794eda, 2026-07-24)", "advisor consult finding C1, docs/history/multisession-native/CONTEXT.md D10a"]
  polarity: pitfall
  critical: true
---

# A lost-update race between two writers of one record closes only when BOTH lock — locking one side moves the race, it does not close it

Heartbeat renewal already ran its session-record read-modify-write under the `sessions` store
lock. `bindSessionLane`/`unbindSessionLane` did not — they read, mutated, and wrote the same
record with no lock at all. Locking heartbeat's side alone protected nothing: a bind or unbind
landing in the still-unlocked window could be silently clobbered by heartbeat's later write of
its stale in-memory copy of the record, or an unbind could be resurrected the same way. The
advisor's finding: **a lost-update race between two writers of one record is only closed when
BOTH writers' read-modify-write run under the same lock.** Locking one side while the other
stays lock-free does not shrink the race, let alone close it — it only decides, silently, which
writer's write is the one that survives the interleaving.

**Rule.** When two code paths perform a read-modify-write on the same durable record, the record
— not either path in isolation — is the unit that needs protecting. Auditing "does this call
site take a lock?" one path at a time will pass every individual review while the pair keeps
racing; the question that actually catches it is "who else writes this same record, and do they
share my lock?" Fixed here by moving `bindSessionLane`/`unbindSessionLane`'s read inside the
same lock hold `heartbeatSession` already used — no new lock name, the same store lock, both
writers now serialize against each other; bounded-retry exhaustion on either path returns the
same typed `LOCK_BUSY` refusal. Proven with forced-interleaving regression tests (a seam hook,
not a sleep or a real thread) reconstructed red against the pre-fix build in both directions
(bind-vs-heartbeat, unbind-vs-heartbeat) — 10/10 rounds failing before the fix, green after.
