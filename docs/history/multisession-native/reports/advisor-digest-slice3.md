# Advisor consult — slice 3 execution (sharded leases + handoff mailboxes)

Advisor: claude/fable (ceiling), read-only, 2026-07-25. VERDICT: proceed-with-conditions.

Binding conditions:
- A (msn-11): .bee/runtime/leases placement is a NOTE only — no cross-workspace lease read against worktree-local stores before the slice-4 controlRoot re-root (cross-worktree visibility rides worktree-holds' mainRoot ledger, worktree-holds.mjs:28-32).
- B (msn-16, biggest risk): preserve the atomic `findForeignHolds + <lease write> + insertHold` reserve seam (bee.mjs:1525-1547, withHoldsLock) byte-for-byte in behavior; worktree-holds.mjs in read_first; cross-worktree double-write regression test required.
- C (msn-16): reservations.json projection rebuilt SYNCHRONOUSLY on every reserve/release, or migrate the direct reader state.mjs:1778-1791 (listActiveReservationsForStart → startFeature precondition d) onto the shim — lazily-rebuilt projection lets startFeature green-light a colliding start.
- D (msn-13): intra-swarm agent-keyed reservations (guards.mjs:733-748) and cross-session holds (guards.mjs:645-656) STAY hard leases; only planning-declared broad/glob paths become advisory intents. Do not conflate intent/lease with same/cross-workspace (that is msn-14). Swarm-reservation-stays-hard test required.
- E (msn-11/16): lease renewal wired into bee-state-sync.mjs:34-70 heartbeat with the same {maxAttempts:1} try-once posture — reservations.renewHoldsBySession must keep working over the shim or be replaced; otherwise live leases lapse at TTL.
- F (msn-12): claims carry NO fencing epoch today (adoptClaim claims.mjs:686-711 bumps adopted_from/at only; "epoch" at cells.mjs:2408 is a different, budget-collapse sense — do not reuse the word loosely). msn-12 stamps+bumps claim fencing epoch in adoptClaim and enforces on renew (renewClaimTTL:725, renewHoldsBySession).
- G (msn-15): state-projection.mjs has NO handoff rebuild — msn-15 ADDS rebuildHandoffProjection, registers in rebuildAllProjections, preserves kind normalization (state.mjs:974-976); byte-identical readers: hooks/bee-session-close.mjs:520, recovery.mjs, state handoff show/write/adopt, AGENTS.md startup step 5. Document (not leave implicit) that the single legacy HANDOFF.json projects only ONE workflow's newest open handoff during compat.

Order 11→12→13→14, 15 after 12, 16 after 13+15 — correct (13←12 and 16←15 are edit-surface serialization, fine).
