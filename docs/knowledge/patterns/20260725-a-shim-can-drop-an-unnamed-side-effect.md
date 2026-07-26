---
type: bee.pattern
title: A shim that preserves a CLI surface can still drop a side-effect that surface never named
description: "The cross-worktree mirror write lived beside the reservation store's write, not inside it — a neighbor, not a return value. Retiring the store's own implementation without deliberately carrying that neighbor along would have lost cross-worktree coordination silently, because every visible test of the shim's own contract (reserve/release/renew) would still pass."
tags: [shim, migration, side-effect, coordination, cross-worktree]
timestamp: 2026-07-25
bee:
  id: pattern-20260725-a-shim-can-drop-an-unnamed-side-effect
  lifecycle: active
  sources: [multisession-native-16 (reservations.mjs shim over lease-store.mjs; advisor consult slice 3 condition B named the atomic findForeignHolds+reserve()+insertHold cross-worktree mirror-write seam in bee.mjs as the biggest risk of the cell; the seam was deliberately left byte-for-byte untouched and a new CLI-level regression test was written specifically to prove the mirror write and the foreign-hold deny still fire through the shim), docs/history/multisession-native/reports/advisor-digest-slice3.md (condition B), .bee/cells/multisession-native-16.json]
  polarity: pitfall
  critical: true
---

# A shim that preserves a CLI surface can still drop a side-effect that surface never named

Retiring a store's own internal implementation and re-platforming its public
verbs (`reserve`/`release`/`renew`/`sweep`) onto a new backing structure looks
complete once every one of those verbs' own tests pass. It is not complete: a
neighbor module can be reaching into the OLD implementation's write path for
a side-effect the verb's own contract never promised — here, a cross-worktree
mirror insert that lived beside the reservation write inside one atomic
locked section, not as anything the `reserve()` return value ever surfaced.
Nothing about the new implementation's own test suite would ever exercise
that seam, because the seam was never the shimmed module's job to begin
with — it belonged to the caller composing them together.

The save here was structural, not lucky: the migration explicitly enumerated
every known caller of the retiring internals *before* writing the shim,
found the one seam whose behavior depended on the old write happening inside
a specific lock section, declared it untouchable in the plan, and wrote a
dedicated regression test proving the untouched seam still worked through
the new shim. Absent that inventory, the shim's own green suite would have
shipped a version where cross-worktree writes silently stopped
double-writing to the shared ledger — every reservation call still
"working," a foreign checkout's edit simply invisible from then on.

**The tell:** a module you are about to retire is not imported only by its
own tests — grep every OTHER file that imports it, and for each hit ask
whether it is calling the public verb for the verb's own contract, or
reaching past it into a specific locked section for a side-effect the
contract never named.
