// bee reservations — native port of the `reservations` verb group.
//
// Ported argv shapes (everything else returns None BEFORE any output and the
// whole command re-runs under Node):
//   reservations list  [--active-only] [--json]
//   reservations reserve --agent A --cell C --path P [--ttl N] [--session S]
//                        [--kind intent|lease] [--json]
//   reservations release --agent A [--cell C] [--json]
//   reservations sweep [--json]
//
// Provenance: bee.mjs handleReservationsReserve/Release/List/Sweep +
// reservePathAtomic/releaseReservationsForAgent/resolveMainRoot/
// resolveHoldTopology/holdForeignExpiry, lib/reservations.mjs (the msn-16
// lease-store shim: listReservations/findConflicts/reserve/release/
// sweepExpired/pathsOverlap/isHardConflict/leaseToReservation/findMainRoot/
// controlRootFor), lib/lease-store.mjs (acquireLeases' O_EXCL create /
// releaseLease's plain rm / resolveResourceFile), lib/worktree-holds.mjs
// (readStore/insertHold/findForeignHolds/releaseHolds/sweepExpiredHolds/
// withHoldsLock) and lib/claims.mjs (resolveSessionId/listSessionRecords/
// heartbeatStale/isConcurrentMode).
//
// Locking: the mutating verbs contend on the SAME cross-process lock files
// Node uses — crate::lock::acquire_store_lock with the exact Node lock name
// "cross-worktree-holds" (worktree-holds.mjs CROSS_WORKTREE_HOLDS_LOCK), so
// the two runtimes serialize against each other mid-campaign. The Node
// verbs are async solely because withStoreLock is async — the sync Rust
// lock covers the same semantics (contract C1).
//
// Root topology: WORKTREE-NATIVE (see roots.rs's header for the per-verb
// flip list). The four verbs here resolve through
// crate::roots::resolve_store_root_worktree and carry both of bee.mjs's
// topology helpers for real:
//
//   * resolveMainRoot(root) — where the shared cross-worktree holds LEDGER
//     lives. `<mainRoot>/.bee/runtime/cross-worktree-holds.json`, always the
//     MAIN checkout's store, never a worktree's own (the same asymmetry the
//     grant registry relies on). `list` and `sweep` use it unconditionally.
//   * resolveHoldTopology(root) — `{mainRoot, holder}` for the two
//     hold-worthy topologies and `null` for everything else:
//       ordinary checkout        -> {workRoot, "main"}
//       GRANTED linked worktree  -> {mainRoot, its git-verified id}
//       UNGRANTED linked worktree-> null: `root` already IS the shared main
//                                  store, so mirroring under a synthetic
//                                  identity would just duplicate a
//                                  reservation the store already carries.
//                                  reserve/release skip the ENTIRE
//                                  cross-worktree section — no lock taken,
//                                  no foreign-hold check, no mirror row.
//
// A BROKEN link still delegates (resolveRoots throws in Node before dispatch).
// controlRootFor (reservations.mjs's own cycle-free findMainRoot walk, which
// cannot import state.mjs) is a THIRD, separate resolver, ported in full
// including its linked branch: it is where the LEASE files live, and it
// answers mainRoot for a granted worktree from the git link alone.
//
// CUTOVER (2026-08-01): corrupt JSON no longer delegates. The cross-worktree
// holds ledger and the session records both FAIL OPEN exactly as
// `readJson(file, fallback)` did — one `bee: could not parse JSON at …` line
// (crate::fsutil::warn_corrupt_json) and then the same fallback: an empty
// `{ holds: [] }` ledger, or that one session record skipped. The
// `|n| >= 1e21` class is retired too: jsjson::js_f64_to_string implements the
// full ECMA Number::toString, so js_numberify accepts every finite number.
//
// Delegation rules beyond the argv shape (all pre-checked BEFORE any store
// write): `null` ledger entries (a JS
// property access crash), date strings outside the ISO subset this port
// models (Date.parse's V8 fallback grammar), and unmodelable Windows path
// spellings. One accepted
// residual: a prechecked file turning corrupt DURING the run (external,
// non-locking writer) can still surface as a late None — the Node re-run
// then operates on the partially-mutated store, the same outcome as two
// sequential CLI invocations. A second documented deviation: Node's
// insertHold re-reads the holds ledger inside the atomic reserve section;
// every production ledger writer serializes on this same lock, so the store
// value read at the section's own findForeignHolds step is reused instead.














// ─── tests ─────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests;

mod jsval;
mod flags;
mod leases;
mod emit;
mod reserve;
mod release;
pub(crate) use self::jsval::*;
pub(crate) use self::flags::*;
pub(crate) use self::leases::*;
pub(crate) use self::emit::*;
pub(crate) use self::reserve::*;
pub(crate) use self::release::*;
