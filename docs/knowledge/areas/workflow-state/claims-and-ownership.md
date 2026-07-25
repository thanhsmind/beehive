---
type: bee.area
title: "Workflow State — atomic claims, typed refusals, and who may mutate a claimed unit"
description: "The single-winner claim primitive every claim path shares, the typed refusal a loser receives instead of a crash, the gate under which a live claim is adopted or reclaimed, the fencing token that refuses a stale mutation once a claim has been adopted, and the ownership check — with its audited rescue door — that guards every mutation of a claimed unit."
timestamp: 2026-07-25
bee:
  id: workflow-state-claims-and-ownership
  lifecycle: active
  areas: [workflow-state]
  required_context: [areas/workflow-state/overview.md]
  decisions: ["multi-session-hardening D1/D4 with Δ2/Δ5 amendments (docs/history/multi-session-hardening/CONTEXT.md; audit 12f54e88, locked 17a624dc)", fresh-session-handoff D1/D3 (atomic exclusive creation; gate-protected adoption and reclaim), "multisession-native D4/D9 invariant 10 (slice 3: claims stamp and bump a fence_epoch on adoption; renew/release may present it and refuse typed CLAIM_FENCE_STALE when stale — docs/history/multisession-native/CONTEXT.md)"]
  sources: ["multi-session-hardening cells msh-1..7 (traces in .bee/cells/, reports docs/history/multi-session-hardening/reports/, 2026-07-19)", "fresh-session-handoff S1 cells fsh-1/fsh-2 (race proofs on Linux/WSL2 and Windows, 2026-07-13)", "critical pattern 20260710 — never release another agent's holdings on a stall signal alone", "docs/specs/workflow-state.md#B11", "docs/specs/workflow-state.md#B23", "docs/specs/workflow-state.md#R17", "docs/specs/workflow-state.md#R18", "docs/specs/workflow-state.md#R36", "docs/specs/workflow-state.md#R39", "docs/specs/workflow-state.md#R40", "docs/specs/workflow-state.md#E19", "docs/specs/workflow-state.md#E20", "docs/specs/workflow-state.md#E21", "docs/specs/workflow-state.md#P13", "docs/specs/workflow-state.md#P18", "multisession-native cell multisession-native-12 (fence_epoch on claims; trace .bee/cells/multisession-native-12.json, commit 8c002a1, 2026-07-25; advisor digest docs/history/multisession-native/reports/advisor-digest-slice3.md condition F)"]
  authoritative_for: "workflow-state: claim exclusivity, typed contention refusals, claim fencing, and claimed-unit ownership"
---

# Workflow State — atomic claims, typed refusals, and who may mutate a claimed unit

Two sessions wanting the same unit at the same moment is not an error case, it
is the normal case — so ownership is decided by a storage-level operation that
cannot succeed twice, never by checking and then writing. Everything else here
follows from that one choice: the loser gets a typed "no" and stays healthy, a
live claim is only ever mutated under its own gate, and a rescue that overrides
ownership always leaves a permanent trace.

## Behaviors & Operations

**B11 — Concurrent sessions coordinate through atomic claims, on every claim
path, not only the cross-session picker.** Trigger: a working session wants
exclusive ownership of a unit of work while other sessions may want the same
unit at the same moment — whether it asks for a specific unit by identity or
pulls the next available one. What happens: the claim is created by exclusive
creation — a storage-level operation that cannot succeed twice — so exactly one
claimant wins; every other claimant receives the typed refusal `CLAIMED`,
naming the winner and its expiry, and remains free to pick other work. The
winning claim carries its owner, lifetime, and heartbeat; a claim with no
owning session (a single-user, ownerless claim) is a legal, supported shape.
The exclusive token behind a claim is released on EVERY transition that clears
it — completion, hand-back, block, drop, or reopen — not only the
cross-session-picker's own unwind, so a same-session round trip (claim, block,
reopen, claim again) never self-refuses against its own prior claim. Mutating a
live claim (adoption to a successor session; reclaim of an abandoned one)
happens only under that claim's own exclusive gate, with the claim record
continuously present throughout — an observer polling at any instant sees the
unit owned by exactly one session, never unowned mid-transfer. Reclaim
additionally re-verifies, while holding the gate, that the lifetime is expired
AND the heartbeat is stale. Single-winner behavior is proven by repeated
multi-process races on both supported platforms (Linux/WSL2 and Windows),
exercised through every claim path, not only the picker. What each actor
observes today: the full flow is wired — sessions and lane bindings are
commandable (B12), the readers consult them (B13), cross-session holds are
enforced at write time (B14), a finished task hands itself to a fresh session
(B15) which can then pull further approved work (B16), a shared coordination
store never silently drops a concurrent write (B21), a session's identity is
never handed down by another party (B22), mutating a claimed unit checks
ownership (B23), and a live session's heartbeat and leases renew themselves
(B24).

**B23 — Mutating a claimed unit of work requires the caller to own it, with an
audited rescue door.** Trigger: any operation that would change a claimed
unit's state — recording verification, completing it, blocking it, releasing
it, or reopening it. What happens: the operation compares the caller's own
derived identity (B22) against the unit's live claim. A live claim owned by a
different session refuses — typed, naming the owner and when its claim
expires. An expired claim, an absent claim, an ownerless claim, or a claim the
caller itself owns all proceed exactly as before — a single working session
never encounters this refusal. An explicit forced override exists for genuine
rescue: it proceeds regardless of ownership and always appends a permanent,
append-only audit entry to the unit's own record — never silently — and that
entry survives the unit's own completion, kept apart from any other audit
trail the unit already carries so a later mutation can never overwrite it. A
forced release of a claim also clears or hands off the underlying claim
record, so the rescued unit is not left unclaimable by anyone. What each actor
observes: normal single-session work is unaffected; a session that tries to
mutate another live session's claimed work gets a clear refusal instead of
silently overwriting it; a deliberate rescue always leaves a trace (D4).

**A claim carries a fencing token that only widens on adoption, never
narrows (multisession-native D4/D9 invariant 10, slice 3).** Trigger: a claim
is created, adopted, renewed, or released. What happens: every claim stamps
`fence_epoch: 1` at creation (`claimCellFile`); adopting it to a successor
session (B15/B16's adoption, or a mailbox handoff's own adopt step) bumps
`fence_epoch` by exactly 1, in the SAME atomic write as the ownership
rewrite — never a separate step that could observe a half-adopted claim.
`renewClaimTTL` and `releaseClaim` accept an optional `presentedEpoch`; given
one behind the claim's current `fence_epoch`, both refuse typed
`CLAIM_FENCE_STALE`, naming the current epoch — a takeover already moved
ownership forward, and the stale caller must re-adopt before writing again.
Omitted (every production caller today), both verbs stay byte-unchanged from
before fencing existed — full mandatory presentation arrives only once
workspace identity lands in a later slice. The name is deliberately
`fence_epoch`, never bare "epoch" — `cells.mjs`'s own unrelated
budget-collapse sense of "epoch" already exists elsewhere in the record, and
the two must never be confused. `sweepExpiredClaims` and `clearClaim` are
untouched by fencing: a sweeper reclaiming an abandoned claim, or an
unconditional clear, is never a holder "presenting" a token to be checked
against. What each actor observes: a session that adopted a claim, then
tries to renew or release it using a stale epoch it captured before someone
else's takeover, gets a clear typed refusal instead of silently clobbering
the new owner's state; every caller that never presents an epoch sees no
behavior change at all.

## Business Rules

- R17 — Concurrent ownership is decided by atomic exclusive creation, never by
  check-then-write; a live claim is mutated only under its own exclusive gate;
  reclaim requires expired lifetime AND stale heartbeat, re-verified under that
  gate (fresh-session-handoff D1/D3; critical pattern 20260710 — never release
  another agent's holdings on a stall signal alone).
- R18 — Contention is answered with a typed refusal carrying a code and reason,
  never an exception; a refused claimant is healthy and free to take other work
  (fresh-session-handoff S1, validation repair).
- R36 — Every claim path — a direct claim by identity as well as the
  cross-session picker — acquires the same exclusive token before any claim
  state changes, and that token is released on every transition that clears a
  claim (completion, hand-back, block, drop, reopen), so a same-session round
  trip never self-refuses against its own prior claim (multi-session-hardening
  D1, Δ2-amended).
- R39 — Mutating a claimed unit of work is refused when a live claim names a
  different session, naming the owner and expiry; an expired, absent,
  ownerless, or matching claim proceeds unchanged, and a forced override
  always appends a permanent audit entry that survives the unit's own
  completion (multi-session-hardening D4, Δ5-amended).
- R40 — A worker never establishes its own ownership of a unit of work; the
  dispatching orchestrator wins the claim before the worker starts, and the
  worker only validates the ownership it was handed (multi-session-hardening
  D1 worker-execution-contract amendment).
- R72 — A claim stamps `fence_epoch: 1` at creation and bumps it by exactly 1,
  atomically with the ownership rewrite, on every adoption; `renewClaimTTL`/
  `releaseClaim` may optionally present it and are refused typed
  `CLAIM_FENCE_STALE` when it is behind the claim's current `fence_epoch`,
  record untouched; omitted, both stay byte-unchanged from before fencing
  existed (multisession-native D4/D9 invariant 10).

## Edge Cases Settled

- Project directories on network file systems are declared unsupported for
  session coordination: exclusive creation is not reliable there. The
  supported topologies are a local Linux/WSL2 disk and a local Windows disk
  (both race-proven).
- A same-session round trip on one unit of work — claim, block, reopen, claim
  again — never self-refuses: the exclusive token is released on every
  claim-clearing transition, not only completion (multi-session-hardening
  D1, Δ2-amended).
- A forced ownership override always leaves a permanent audit trace naming the
  verb, who forced it, whose ownership was bypassed, and when — kept apart
  from any other audit trail on the unit so a later mutation can never
  overwrite it — and a forced release of the claim leaves the unit claimable
  again, never stuck self-refusing (multi-session-hardening D4, Δ5-amended).
- A caller presenting no `presentedEpoch` (every production caller today) is
  wholly unaffected by fencing — the `CLAIM_FENCE_STALE` check never runs
  for it (multisession-native D4/D9 invariant 10, msn-12).

## Pointers (implementation)

- Session coordination (B11/R17/R18): `skills/bee-hive/templates/lib/claims.mjs`
  (byte-mirrored to `.bee/bin/lib/`) — sessions under `.bee/sessions/`, claims
  under `.bee/claims/`, per-claim gate `<cell>.adopting`; race orchestrator
  `skills/bee-hive/templates/tests/race_claims_child.mjs` (3 scenarios using
  barrier-synchronized isolated Worker racers in `test_lib.mjs`). Evidence:
  traces `.bee/cells/fsh-{1,2}.json` (win32 +
  linux probe PASS lines), commits 0224f6c, edfac87; validation
  `docs/history/fresh-session-handoff/reports/validation-s1.md`.
- Multi-session hardening (B11/B21-B24, R36-R40): coordination lock primitive
  `withStoreLock` in `skills/bee-hive/templates/lib/lock.mjs` (byte-mirrored
  to `.bee/bin/lib/`), O_EXCL acquire with stale-holder takeover by atomic
  rename, forked-racer suite `scripts/tests/test_store_lock.mjs`; `cells claim --id`
  re-backed by the same claim-file gate `claim-next` uses
  (`claimCellCrossSession` in `lib/cells.mjs`), forked-racer suite
  `scripts/tests/test_claim_race.mjs`; session id self-derivation `resolveSessionId`
  in `lib/claims.mjs`; claim-clearing release on cap/unclaim/block/drop/reopen
  via `clearClaim` in `lib/claims.mjs`; reservation read-modify-write and
  session auto-derive under the lock in `lib/reservations.mjs`
  (`reserve`/`release`/`sweepExpired`), forked-racer suite
  `scripts/tests/test_reservation_race.mjs`; ownership guard on cell mutators
  (`checkClaimOwnership`/`guardClaimOwnership`, `--force-ownership`, the
  `trace.ownership_overrides` audit key kept apart from `trace.deviations`)
  in `lib/cells.mjs`; throttled heartbeat-and-lease renewal
  (`heartbeatTouch`, `renewClaimTTL` in `lib/claims.mjs`,
  `renewHoldsBySession` in `lib/reservations.mjs`) wired into
  `hooks/bee-prompt-context.mjs` and `hooks/bee-state-sync.mjs` in try-once
  mode, suite `scripts/tests/test_heartbeat_touch.mjs`; state logical
  read-modify-write verbs (`startFeature` in `lib/state.mjs`;
  `handleStateSet`/`handleStateGate`/`stateWorkerMutate`/
  `handleStateScribingRun` in `bee.mjs`) serialized under the same lock,
  waiting normally. Orchestrator-claims-before-spawn doctrine in
  `skills/bee-executing/SKILL.md` + `references/worker-details.md` and
  `skills/bee-swarming/SKILL.md` + `references/swarming-reference.md`; the
  four new suites added to `.bee/config.json` `commands.verify`. Evidence:
  `docs/history/multi-session-hardening/CONTEXT.md` (D1-D7, Δ1-Δ6); traces
  `.bee/cells/msh-{1..7}.json`; reports
  `docs/history/multi-session-hardening/reports/msh-{1..7}.md`.
- Claim fencing (multisession-native D4/D9 invariant 10): `fence_epoch`
  stamped in `claimCellFile` and bumped in `adoptClaim` (the SAME atomic
  write as the ownership rewrite); `CLAIM_FENCE_STALE` refusal in
  `renewClaimTTL`/`releaseClaim`, all in
  `skills/bee-hive/templates/lib/claims.mjs` (byte-mirrored to
  `.bee/bin/lib/`). Red-first: new refusal tests captured failing against
  pristine `claims.mjs`, then green after the implementation. Evidence:
  trace `.bee/cells/multisession-native-12.json`, commit 8c002a1; advisor
  digest `docs/history/multisession-native/reports/advisor-digest-slice3.md`
  condition F.
