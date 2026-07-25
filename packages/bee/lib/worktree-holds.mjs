// worktree-holds.mjs — shared cross-worktree holds ledger (xwh-1, additive).
// WIRED since xwh-2 into bee.mjs's handleReservationsReserve/Release/Sweep/
// List (this stale header used to say "UNWIRED" — corrected in
// hardening-1-7-10 D3, see below). Mirrors a path-level hold from any
// checkout (the ordinary checkout, holder 'main', or a granted linked
// worktree, holder = its git-verified worktree id) into ONE shared ledger so
// a different checkout can discover a foreign hold on a path it is about to
// write to, before same-checkout reservation/state guards ever see it.
//
// hardening-1-7-10 (D3): the reserve seam used to be check-then-act — an
// UNLOCKED findForeignHolds read, then a local reserve, then a separately
// locked mirrorHold — three independent critical sections with real gaps
// between them, so two checkouts racing the SAME path could both pass the
// foreign-hold check before either had mirrored, and both land an active
// grant (test_worktree_holds_race.mjs's same-path scenario demonstrates the
// double-grant against that old shape). `withHoldsLock` is now exported so a
// caller (bee.mjs) can run the check + local reserve + mirror-insert as ONE
// atomic section under this module's own shared lock — `insertHold` is the
// unlocked core `mirrorHold` already wrapped in that lock, exposed
// separately so it can be called from INSIDE a section that already holds
// the lock (calling the locked `mirrorHold` there would self-deadlock: the
// lock file is per-process-exclusive, not reentrant). `renewHolds` is new
// too: it refreshes `mirrored_at` in place for a session's still-active
// holds (never a new row — mirrorHold/insertHold stay append-only), wired
// into the existing heartbeat/lease-renewal hook so a live worker never
// silently loses its cross-worktree hold at the 1h TTL.
//
// Store: <mainRoot>/.bee/runtime/cross-worktree-holds.json — ALWAYS the
// MAIN checkout's store, never a worktree's own `.bee/` (same asymmetry
// worktree-store.mjs's readGrants/writeGrant rely on for their own security
// property: a worktree cannot self-claim anything by writing to its own
// store, because nothing here ever reads a worktree's own `.bee/runtime/`).
// Atomic tmp+rename writes, same shape as worktree-store.mjs's
// writeGrantsFileAtomic. Every mutation runs inside
// withStoreLock(mainRoot, 'cross-worktree-holds') (lock.mjs) so two
// concurrent mirrorHold/releaseHolds/sweepExpiredHolds calls — from two
// separate OS processes, e.g. two different worktree checkouts — can never
// both read the same pre-mutation snapshot and have the later write silently
// drop the earlier one (same D2 lost-update fix reservations.mjs's
// reserve()/release()/sweepExpired() already apply to .bee/reservations.json).
//
// TTL-only expiry (reservations.mjs's sweepExpired precedent). Expired
// entries are pruned ON READ (never returned by findForeignHolds), and
// separately reconciled to disk (marked released) only by the explicit
// sweepExpiredHolds() call — the exact same two-tier shape (`isActive`
// read-time filter vs. `sweepExpired` disk write) reservations.mjs already
// uses. `renewHolds` (D3) is the renewal primitive that TTL-only expiry was
// missing: it pushes `mirrored_at` forward for a session's still-active
// holds so a live session's TTL clock keeps resetting instead of only ever
// counting down to sweepExpiredHolds pruning it.
//
// pathsOverlap is REUSED, not reimplemented: imported directly from
// reservations.mjs, which is the established sharing style already used by
// schedule.mjs, state.mjs, and cells.mjs (all import { pathsOverlap } from
// './reservations.mjs' rather than duplicating the predicate) — see each
// module's own header comment. reservations.mjs imports only fsutil.mjs +
// lock.mjs + claims.mjs, so importing it here creates no cycle.
//
// Production callers: bee.mjs's handleReservationsReserve/Release/Sweep/List
// (xwh-2) and the atomic reserve section added in hardening-1-7-10 D3;
// hooks/bee-state-sync.mjs's heartbeat renewal path (D3) calls renewHolds
// alongside its existing claims/reservations lease renewal.

import fs from 'node:fs';
import path from 'node:path';
import { readJson, writeJsonAtomic } from './fsutil.mjs';
import { withStoreLock } from './lock.mjs';
import { pathsOverlap } from './reservations.mjs';

const DEFAULT_TTL_SECONDS = 3600;

function utcNow() {
  return new Date().toISOString();
}

function holdsLedgerPath(mainRoot) {
  return path.join(mainRoot, '.bee', 'runtime', 'cross-worktree-holds.json');
}

/** Missing file reads as an empty ledger (fail-open read, same posture as
 * worktree-store.mjs's readGrants and reservations.mjs's readStore) — never
 * throws for an absent/malformed store; readJson already warns to stderr and
 * falls back on malformed JSON. */
function readStore(mainRoot) {
  const store = readJson(holdsLedgerPath(mainRoot), null);
  if (!store || typeof store !== 'object' || !Array.isArray(store.holds)) {
    return { holds: [] };
  }
  return store;
}

/** Atomic tmp+rename write, imitating worktree-store.mjs's
 * writeGrantsFileAtomic (the grants-registry writer this module's ledger is
 * modeled on) rather than re-deriving a third atomic-write shape. */
function writeStore(mainRoot, store) {
  writeJsonAtomic(holdsLedgerPath(mainRoot), store);
}

function normalizePath(value) {
  return String(value || '')
    .replace(/\\/g, '/')
    .replace(/^\.\/+/, '')
    .replace(/\/+$/, '');
}

function isExpired(entry, nowMs) {
  const ttl = entry.ttl_seconds;
  if (!Number.isFinite(ttl) || ttl <= 0) return false;
  const mirroredMs = Date.parse(entry.mirrored_at);
  if (!Number.isFinite(mirroredMs)) return false;
  return mirroredMs + ttl * 1000 <= nowMs;
}

function isActive(entry, nowMs = Date.now()) {
  return entry.released_at == null && !isExpired(entry, nowMs);
}

// The single named lock every mutation in this module runs under. Exported
// (D3) so a caller that needs to run MORE than one of this module's
// operations as one atomic section (bee.mjs's reserve seam: re-check foreign
// holds, do the local reserve, insert the mirror row, all without releasing
// the shared lock in between) can acquire it directly instead of composing
// several independently-locked calls with gaps between them.
export const CROSS_WORKTREE_HOLDS_LOCK = 'cross-worktree-holds';

/**
 * Runs `fn` with this module's shared cross-worktree-holds lock held
 * (lock.mjs's withStoreLock under the hood). `options` passes straight
 * through to withStoreLock (e.g. `{ maxAttempts: 1 }` for a hook's
 * try-once/never-block posture). Lock order for any caller composing this
 * with a second, DIFFERENT lock (e.g. reservations.mjs's own local
 * 'reservations' lock): this shared lock outermost, the local store's own
 * lock inside — and never hold this lock across a child-process spawn.
 */
export function withHoldsLock(mainRoot, fn, options) {
  return withStoreLock(mainRoot, CROSS_WORKTREE_HOLDS_LOCK, fn, options);
}

/**
 * The unlocked CORE of mirrorHold — reads the store, appends one hold row,
 * writes it back. Never acquires a lock itself: callers already inside a
 * withHoldsLock(mainRoot, ...) section (bee.mjs's atomic reserve seam, D3)
 * call this directly, because calling the LOCKED `mirrorHold` from inside a
 * section that already holds the same lock would self-deadlock (the lock
 * file is per-process-exclusive via O_EXCL, never reentrant). Always
 * appends a new entry (never upserts) — the same posture reservations.mjs's
 * reserve() takes: repeated calls from the same holder on the same path
 * simply accumulate rows, and isActive()/findForeignHolds() read-time-filter
 * which of them still matter. No conflict check runs here; this module only
 * records visibility, it does not itself decide allow/deny.
 */
export function insertHold(mainRoot, { path: holdPath, holder, feature = null, session = null, cell = null, ttl = DEFAULT_TTL_SECONDS } = {}) {
  if (typeof holdPath !== 'string' || !holdPath.trim()) {
    throw new Error('insertHold: path is required.');
  }
  if (typeof holder !== 'string' || !holder.trim()) {
    throw new Error('insertHold: holder is required.');
  }
  const store = readStore(mainRoot);
  const hold = {
    path: normalizePath(holdPath),
    holder: holder.trim(),
    feature: typeof feature === 'string' && feature.trim() ? feature.trim() : null,
    session: typeof session === 'string' && session.trim() ? session.trim() : null,
    cell: typeof cell === 'string' && cell.trim() ? cell.trim() : null,
    ttl_seconds: Number.isFinite(ttl) && ttl > 0 ? Math.floor(ttl) : DEFAULT_TTL_SECONDS,
    mirrored_at: utcNow(),
    released_at: null,
  };
  store.holds.push(hold);
  writeStore(mainRoot, store);
  return { ok: true, hold };
}

/**
 * Mirrors ONE path-level hold into the shared ledger under
 * withHoldsLock(mainRoot, ...) — the standalone, self-locking public API
 * (unchanged call shape from before D3). Callers that need this insert as
 * part of a LARGER atomic section (bee.mjs's reserve seam) should call
 * `insertHold` directly from inside their own `withHoldsLock` block instead,
 * to avoid nesting two acquisitions of the same lock.
 */
export async function mirrorHold(mainRoot, opts) {
  return withHoldsLock(mainRoot, () => insertHold(mainRoot, opts));
}

/**
 * Active (unreleased, unexpired) holds owned by a DIFFERENT holder that
 * overlap any of `paths` (pathsOverlap semantics reused from
 * reservations.mjs — exact match, directory prefix, or trivial `*` glob).
 * Pure read: no lock, mirrors reservations.mjs's findConflicts/
 * findSessionConflicts (also unlocked reads run outside withStoreLock).
 * `paths` accepts a single path string or an array.
 */
export function findForeignHolds(mainRoot, holder, paths) {
  const requested = (Array.isArray(paths) ? paths : [paths]).filter(Boolean);
  if (requested.length === 0) return [];
  const acting = typeof holder === 'string' ? holder.trim() : '';
  const nowMs = Date.now();
  const store = readStore(mainRoot);
  return store.holds.filter(
    (hold) =>
      isActive(hold, nowMs) &&
      hold.holder !== acting &&
      requested.some((requestedPath) => pathsOverlap(hold.path, requestedPath)),
  );
}

/**
 * Releases every unreleased hold owned by `holder`, optionally narrowed by
 * `session` and/or `cell` (both `null` by default — a null filter matches
 * any value, same "absent filter = no filter" posture as
 * reservations.mjs's release({agent, cell=null})). Runs under
 * withStoreLock(mainRoot, 'cross-worktree-holds'). Returns
 * `{ released: <count> }`; the store is only rewritten when at least one row
 * changed.
 */
export async function releaseHolds(mainRoot, { holder, session = null, cell = null } = {}) {
  if (typeof holder !== 'string' || !holder.trim()) {
    throw new Error('releaseHolds: holder is required.');
  }
  const actingHolder = holder.trim();
  return withHoldsLock(mainRoot, () => {
    const store = readStore(mainRoot);
    const releasedAt = utcNow();
    let released = 0;
    for (const hold of store.holds) {
      if (hold.released_at != null) continue;
      if (hold.holder !== actingHolder) continue;
      if (session != null && hold.session !== session) continue;
      if (cell != null && hold.cell !== cell) continue;
      hold.released_at = releasedAt;
      released += 1;
    }
    if (released > 0) writeStore(mainRoot, store);
    return { released };
  });
}

/**
 * Releases EVERY unreleased hold for `id`, ignoring session/cell entirely —
 * the unconditional sibling of releaseHolds, for a holder going away
 * outright (e.g. a granted worktree being unregistered/removed) rather than
 * one narrowed release. Implemented as a thin delegate to releaseHolds with
 * no session/cell filter, not a second store-mutation body.
 */
export async function releaseAllForHolder(mainRoot, id) {
  const { released } = await releaseHolds(mainRoot, { holder: id });
  return { released };
}

/**
 * Marks every TTL-expired, still-unreleased hold as released (sets
 * `released_at`) — the disk-persisting sibling of the read-time `isActive`
 * filter every query function above already applies. Mirrors
 * reservations.mjs's sweepExpired exactly: same lock, same "only write when
 * something actually changed" guard. Returns the count of holds released.
 */
export async function sweepExpiredHolds(mainRoot) {
  return withHoldsLock(mainRoot, () => {
    const store = readStore(mainRoot);
    const nowMs = Date.now();
    const releasedAt = utcNow();
    let released = 0;
    for (const hold of store.holds) {
      if (hold.released_at != null) continue;
      if (!isExpired(hold, nowMs)) continue;
      hold.released_at = releasedAt;
      released += 1;
    }
    if (released > 0) writeStore(mainRoot, store);
    return released;
  });
}

/**
 * D3 — renews this session's still-active holds by pushing `mirrored_at`
 * forward to now, WITHOUT touching `released_at` or appending any row (the
 * renewal exception to this module's otherwise append-only mutation shape:
 * mirrorHold/insertHold always append, releaseHolds/sweepExpiredHolds only
 * ever set `released_at`; renewHolds is the one path that rewrites an
 * existing row's `mirrored_at` in place). Only rows that are still
 * `isActive` (unreleased AND not yet expired) at call time are renewed — a
 * hold that already lapsed past its TTL is left alone rather than resurrected,
 * matching findForeignHolds' own "no longer visible" verdict for it. Runs
 * under withHoldsLock so a renewal can never race a concurrent
 * insert/release/sweep into a lost update. `options` passes straight through
 * to withHoldsLock (hook callers pass `{ maxAttempts: 1 }`, the same
 * try-once/never-block posture claims.mjs's heartbeatTouch and
 * reservations.mjs's renewHoldsBySession already use — a missed renewal here
 * is skipped, never waited on). Returns `{ renewed: <count> }`.
 */
export async function renewHolds(mainRoot, sessionId, options) {
  const session = typeof sessionId === 'string' ? sessionId.trim() : '';
  if (!session) return { renewed: 0 };
  return withHoldsLock(
    mainRoot,
    () => {
      const store = readStore(mainRoot);
      const nowMs = Date.now();
      const nowIso = utcNow();
      let renewed = 0;
      for (const hold of store.holds) {
        if (!isActive(hold, nowMs)) continue;
        if (hold.session !== session) continue;
        hold.mirrored_at = nowIso;
        renewed += 1;
      }
      if (renewed > 0) writeStore(mainRoot, store);
      return { renewed };
    },
    options,
  );
}

/**
 * True when the ledger file exists but is unreadable/malformed JSON — false
 * for a missing file (today's open/empty-ledger behavior, never "corrupt")
 * and false once it parses cleanly. Mirrors guards.mjs's private
 * reservationStoreCorrupt (guards.mjs:108-117) byte-for-byte in shape, as a
 * PUBLIC export here for later callers (a hold-aware write guard) that need
 * to fail closed on a torn/corrupt cross-worktree ledger rather than
 * silently treating it as empty. Never locked (mirrors the same read-only,
 * unlocked posture as reservationStoreCorrupt and findForeignHolds above).
 */
export function holdsStoreCorrupt(mainRoot) {
  const file = holdsLedgerPath(mainRoot);
  if (!fs.existsSync(file)) return false; // missing store = today's open behavior
  try {
    JSON.parse(fs.readFileSync(file, 'utf8'));
    return false;
  } catch {
    return true;
  }
}
