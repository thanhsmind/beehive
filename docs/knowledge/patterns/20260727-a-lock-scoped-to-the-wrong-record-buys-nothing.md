---
type: bee.pattern
title: A lock that guards the wrong record buys nothing and costs an invariant
description: "Serializing a lane mutation on the shared 'state' lock closed nothing for lanes — the write it guards (lanePath) is never touched by that lock's other holders — and it turned a correct, live invariant red by forcing an unrelated writer to wait out the lock's own timeout."
tags: [architecture, concurrency, locks, race-condition, projections, state-phase-lock-race]
timestamp: 2026-07-27
bee:
  id: pattern-20260727-a-lock-scoped-to-the-wrong-record-buys-nothing
  lifecycle: active
  sources: ["state-phase-lock-race cell splr-1 (blanket 'state' wrap over the whole workflow branch, trace .bee/cells/splr-1.json, commit e787819a, 2026-07-27)", "state-phase-lock-race cell splr-3 (per-record fix, decision D13, trace .bee/cells/splr-3.json, commit ebc68f04, 2026-07-27)", docs/history/state-phase-lock-race/CONTEXT.md D1-D4, decision 61e21a42-39b2-4f8a-bcb8-2a4d99f00154 (D13)]
  polarity: pitfall
  critical: true
---

# A lock that guards the wrong record buys nothing and costs an invariant

`splr-1`'s first fix for the GH #70 lost-update race wrapped the entire
`if (wf)` branch of `withMutationLock` — default-record mutation **and** lane
mutation alike — in the shared `'state'` lock, reasoning that "every writer of
the shared projection" meant every writer reachable from that branch. It did
not. `state.mjs:1704`'s lane write (`writeLaneRecordThroughProjection` →
`rebuildLaneProjection` → `writeLane`) touches only
`.bee/lanes/<feature>.json` — a file the `'state'` lock's other holders (the
`bee-state-sync` hook, the default-record writers) never write and never
read. Wrapping it in `'state'` closed nothing: the actual lane record still
had no lock of its own protecting it from another lane writer, while the
wrap's only real effect was to force every lane mutation to queue behind
whatever unrelated default-record work happened to be holding `'state'` at
that moment.

The cost was not neutral. `test_cli_state.mjs` encoded a genuine, previously
correct invariant — *"a lane mutation with a live workflow record never needs
the shared `'state'` lock"* — and under the blanket wrap that assertion went
from near-instant to 4995 ms: the full lock-acquire timeout, because the test
now waited on a lock its own operation had no reason to want. A fix that
turns a live green invariant red is not a stricter fix; it is evidence the
lock was scoped to the wrong record.

**Rule.** Before wrapping a write in a shared lock, name the exact file (or
record) that write touches, then check whether that lock's *other* holders
touch the *same* file. A lock scoped to "the code path near this write"
rather than "the record this write mutates" either protects nothing (if the
path's actual write target has no other writers sharing the lock) or
over-serializes unrelated work (if it does) — and a project with more than
one independently-written shared record needs one lock **per record**, not
one lock per code path that happens to pass near several records.
`state-phase-lock-race` D13 landed the corrected shape: a single acyclic
global order `workflow:<id>` → `'state'` → `lane:<feature>`, where `'state'`
guards only `.bee/state.json` and `lane:<feature>` guards only
`.bee/lanes/<feature>.json` — each lock scoped to exactly the record its own
name says it guards, no more and no less.

See also [[pattern-20260724-a-lost-update-race-closes-only-when-both-writers-lock]]:
that pattern establishes that a lost update closes only when every writer of
a record shares its lock; this one is the mirror-image failure mode —
applying the right *kind* of fix (shared locking) to the wrong *scope*
(a record that lock does not actually own) produces the same open race for
the record that needed it, while silently taxing a record that never did.
