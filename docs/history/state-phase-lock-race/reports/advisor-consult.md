# Advisor consult — state-phase-lock-race (GH #70)

Date 2026-07-27 · Advisor `fable` (`models.claude.advisor`) · read-only.

**Verdict: PROCEED WITH CHANGES.**

## Refuted: the plan's claim that no `'state'` → `workflow:<id>` edge exists

A live inverse edge exists. `handleStatePlanRevBump` holds
`withStoreLock(root, 'state', …)` at `packages/bee/bee.mjs:2921`, and inside that
closure calls `updateWorkflow` at `bee.mjs:2944`, which self-locks
`withWorkflowLock` → `workflow:<id>` (`workflow-store.mjs:389`).

Adding this feature's `workflow:<id>` → `'state'` edge without repairing that one
produces a deterministic lock-order inversion, in exactly the concurrent-session
scenario the fix targets:

- Session A (`state gate`/`set` on lane F) holds `workflow:W`, waits on `'state'`.
- Session B (`state plan-rev bump` on lane F) holds `'state'`, waits on `workflow:W`.

Not a permanent hang — `lock.mjs` is timeout-bounded and neither holder is
stale-eligible — but ~5 s of dual `LockBusyError`. **Verified independently by the
orchestrator** against `bee.mjs:2915-2950` and `workflow-store.mjs:385-390`.

## Required change 1 — repair the inversion in the same cell

Restructure `handleStatePlanRevBump` to the canonical order: resolve the
lane/workflow first, then
`withWorkflowLock(ctrlRoot, wf.id, () => withStoreLock(root, 'state', body))`
with `updateWorkflowAssumingLock` inside, so `workflow:<id>` → `'state'` is the
single global order.

## Required change 2 — the writer inventory is incomplete

Add to it: `state.mjs:2592` (`startFeature` legacy write, already `'state'`),
`bee.mjs:2971` (`stateWorkerMutate`, already `'state'`), and the narrow
`bee.mjs:2465` workflow-branch fallback (holds `workflow:<id>` only when the
workflow closes between lock-branch selection at `:2551` and the re-list at
`:2463` — silently fixed by the wrap). The test must assert the real invariant:
**every production `writeState` runs under `'state'`**.

## Recommended 3

- Record the msn-10/D9 concurrency walk-back explicitly.
- Note the hook's increased `maxAttempts: 1` skip rate — a skipped tick delays
  only `cells`/`last_activity` to the next tick.
- Deploy the fix into `.bee/bin/`. **Verified:** `md5sum` of `.bee/bin/bee.mjs`
  equals `packages/bee/bee.mjs` today, so this repo executes the vendored copy
  and an un-vendored fix ships nothing.

## Confirmed clean

- No self-deadlock. `withMutationLock`'s four callers are top-level dispatch
  handlers. `handleStateStartFeature`'s rebuild (`bee.mjs:3284`) is genuinely
  outside `startFeature`'s `'state'` hold, which releases in `lock.mjs`'s
  `finally` before `createWorkflow` at `state.mjs:2607`.
- Claim (b) holds: `rebuildReservationsProjection`
  (`reservations.mjs:701-709`) is `listReservations` + `writeJsonAtomic`, zero
  locks.
- `startFeature`/`startLane`'s `'state'` closure acquires nothing —
  `seedLegacyWorkflows` runs *before* the lock (`state.mjs:2454`).
- Handoff locks never nest with `'state'`.

## Does the fix close the symptom

Yes, for the lost-update class. Post-fix every production state.json
read-derive-write pair holds `'state'` for its whole span, and workflow-record
writes are atomic, so every write derives from a consistent snapshot. Lockless
readers (write-guard, status) can still catch `state.json` between a record write
and its follow-up rebuild, but they read a consistent recent snapshot — never a
lost-update artifact.

## Alternatives rejected

- **CAS / version field** — does not serialize the read-to-write span, and adds
  schema churn to a file slated for retirement.
- **Sole-writer projection refactor** — the fix already achieves this in effect.
- **Guard reads workflow records instead of the projection** — genuinely
  attractive as a *complement*, since it removes the intake guard's dependency on
  projection freshness entirely, but it does not replace the lock fix (status and
  every other projection reader would still see lost updates). Filed as a
  follow-up hardening PBI, not a change to this plan.
