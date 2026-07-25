// reservations.mjs — same-session file reservations for swarms.
// Store: .bee/reservations.json { reservations: [ {agent, cell, path, ttl_seconds, reserved_at, released_at|null} ] }

import path from 'node:path';
import { readJson, writeJsonAtomic } from './fsutil.mjs';
import { withStoreLock } from './lock.mjs';
import { resolveSessionId, isConcurrentMode } from './claims.mjs';

const DEFAULT_TTL_SECONDS = 3600;

// Intent/lease classification (multisession-native-13, CONTEXT.md D4 —
// advisor consult slice 3 condition D). A reservation row's `kind` decides
// whether the WRITE GUARD (guards.mjs checkWrite) treats an overlap against
// it as a hard deny or an advisory warning:
//   - 'lease' (the default — every row from before this cell, and every row
//     a worker's own write-time `reservations reserve` call makes unless it
//     opts in otherwise) is a HARD conflict, byte-unchanged from before this
//     cell.
//   - 'intent' is a planning-declared broad/glob scope: advisory only when
//     the overlap is broad (a directory-prefix or glob-suffix containment),
//     still hard when it collapses onto the exact same resource as the
//     write target. See guards.mjs's isAdvisoryIntentConflict for the exact
//     decision, and its module comment for why pathsOverlap itself stays
//     unchanged (schedule.mjs/state.mjs/cells.mjs still need broad overlap
//     for wave-planning purposes, unrelated to this hard/advisory split).
export const RESERVATION_KINDS = ['intent', 'lease'];
const DEFAULT_RESERVATION_KIND = 'lease';

function utcNow() {
  return new Date().toISOString();
}

export function reservationsPath(root) {
  return path.join(root, '.bee', 'reservations.json');
}

function readStore(root) {
  const store = readJson(reservationsPath(root), null);
  if (!store || typeof store !== 'object' || !Array.isArray(store.reservations)) {
    return { reservations: [] };
  }
  return store;
}

function writeStore(root, store) {
  writeJsonAtomic(reservationsPath(root), store);
}

function isExpired(reservation, nowMs) {
  const ttl = reservation.ttl_seconds;
  if (!Number.isFinite(ttl) || ttl <= 0) return false;
  const reservedMs = Date.parse(reservation.reserved_at);
  if (!Number.isFinite(reservedMs)) return false;
  return reservedMs + ttl * 1000 <= nowMs;
}

function isActive(reservation, nowMs = Date.now()) {
  return reservation.released_at == null && !isExpired(reservation, nowMs);
}

function normalizePath(value) {
  return String(value || '')
    .replace(/\\/g, '/')
    .replace(/^\.\/+/, '')
    .replace(/\/+$/, '');
}

/**
 * Two reservation paths overlap when: exact match, one is a directory prefix
 * of the other, or one is a trivial `*` glob suffix (e.g. `src/api/*`)
 * whose prefix contains/covers the other.
 */
export function pathsOverlap(a, b) {
  const left = normalizePath(a);
  const right = normalizePath(b);
  if (!left || !right) return false;
  if (left === right) return true;

  const leftGlob = left.endsWith('*');
  const rightGlob = right.endsWith('*');
  const leftBase = leftGlob ? left.replace(/\*+$/, '').replace(/\/+$/, '') : left;
  const rightBase = rightGlob ? right.replace(/\*+$/, '').replace(/\/+$/, '') : right;

  if (leftBase === rightBase) return true;
  if (leftBase === '' || rightBase === '') return true; // bare "*" covers everything
  return (
    leftBase.startsWith(`${rightBase}/`) || rightBase.startsWith(`${leftBase}/`)
  );
}

/**
 * isHardConflict(reservation, targetPath) — the single shared intent/lease
 * classification (multisession-native-13, D4 — advisor consult slice 3
 * condition D), used by BOTH reserve()'s own conflict pre-check below and
 * guards.mjs's write guard (checkWrite), so a declared 'intent' can never
 * hard-block anyone through EITHER chokepoint: not a fellow reserve() caller
 * staking out their own exact write path (which would otherwise silently
 * refuse before the write guard is ever consulted — the exact "hard deny
 * from an intent record" this cell's must_haves prohibit), and not an
 * eventual write into that path.
 *
 * true (hard) unless `reservation.kind === 'intent'` AND its stored path is
 * NOT the exact same resource as `targetPath` (a broad/glob-only overlap —
 * a directory prefix or glob-suffix containment, matched via pathsOverlap
 * but not string-identical). An 'intent' that collapses onto the exact
 * target is still a hard, same-resource collision regardless of its label.
 * `reservation.kind` absent/undefined (every pre-existing row, and every
 * row from a reserve() call that never passed `kind`) reads as the default
 * 'lease' — always hard, byte-unchanged from before this cell.
 */
export function isHardConflict(reservation, targetPath) {
  return !(reservation.kind === 'intent' && normalizePath(reservation.path) !== normalizePath(targetPath));
}

export function listReservations(root, { activeOnly = false } = {}) {
  const store = readStore(root);
  const nowMs = Date.now();
  if (!activeOnly) return store.reservations;
  return store.reservations.filter((reservation) => isActive(reservation, nowMs));
}

/** Active reservations held by *other* agents covering any of the given paths. */
export function findConflicts(root, agent, paths) {
  const requested = (Array.isArray(paths) ? paths : [paths]).filter(Boolean);
  if (requested.length === 0) return [];
  return listReservations(root, { activeOnly: true }).filter(
    (reservation) =>
      reservation.agent !== agent &&
      requested.some((requestedPath) => pathsOverlap(reservation.path, requestedPath)),
  );
}

/**
 * Active reservations owned by a DIFFERENT session covering any of the given
 * paths (fresh-session-handoff D3 — cross-session hold conflict finder, the
 * session-keyed sibling of findConflicts' agent-keyed check). A reservation
 * with no `session` field is a legacy/intra-swarm-only row and never
 * conflicts here — only rows explicitly bound to a session can deny another
 * session's write; the acting session's own rows never conflict either.
 */
export function findSessionConflicts(root, sessionId, paths) {
  const requested = (Array.isArray(paths) ? paths : [paths]).filter(Boolean);
  if (requested.length === 0) return [];
  const acting = typeof sessionId === 'string' ? sessionId.trim() : '';
  return listReservations(root, { activeOnly: true }).filter(
    (reservation) =>
      typeof reservation.session === 'string' &&
      reservation.session.trim() &&
      reservation.session !== acting &&
      requested.some((requestedPath) => pathsOverlap(reservation.path, requestedPath)),
  );
}

/**
 * D2: the read-check-write body (conflict check + append + write) runs
 * inside withStoreLock('reservations') so two concurrent reserves can no
 * longer both pass the conflict check against the same snapshot and have
 * the later write silently drop the earlier hold — the conflict check reads
 * the store fresh under the lock, never a pre-lock snapshot.
 *
 * `kind` (multisession-native-13, D4): OPTIONAL, defaults to `'lease'`
 * (RESERVATION_KINDS above) — a worker's own write-time reservation stays a
 * hard conflict exactly as before this cell. Pass `kind: 'intent'` to
 * declare a broad/glob planning-time scope instead; guards.mjs's checkWrite
 * downgrades a conflict against an 'intent' row to an advisory warning
 * unless it collapses onto the exact write target (see guards.mjs
 * isAdvisoryIntentConflict). Invalid values throw synchronously, before the
 * store lock is ever taken.
 *
 * D3: when `session` is absent, it self-derives (explicit flag -> `resolve
 * SessionId`'s own BEE_SESSION_ID/CLAUDE_CODE_SESSION_ID env fallback ->
 * null), so a top-level-session reserve becomes cross-session-visible by
 * default; a genuinely absent id (no flag, no env) still writes a
 * session-less row, byte-identical to today's shape — UNLESS another
 * session is concurrently live (hardening-4a): see the SESSION_REQUIRED
 * refusal below, the reserve-side sibling of claims.mjs claimCellFile's own
 * sessionless-in-concurrent-mode refusal.
 */
export async function reserve(root, { agent, cell, path: reservedPath, ttl = DEFAULT_TTL_SECONDS, session = null, kind = DEFAULT_RESERVATION_KIND }) {
  if (typeof agent !== 'string' || !agent.trim()) {
    throw new Error('reserve: agent is required.');
  }
  if (typeof cell !== 'string' || !cell.trim()) {
    throw new Error('reserve: cell id is required.');
  }
  if (typeof reservedPath !== 'string' || !reservedPath.trim()) {
    throw new Error('reserve: path is required.');
  }
  if (!RESERVATION_KINDS.includes(kind)) {
    throw new Error(`reserve: kind must be one of ${RESERVATION_KINDS.join('/')} (got ${JSON.stringify(kind)}).`);
  }
  // hardening-1-7-10 D5/1710-10: `root` is passed through so
  // resolveSessionId's durable single-live-session fallback can adopt an
  // identity here too — a solo native Codex session has a real session
  // record but no env var identifying it, so without `root` this call
  // always fell through to the SESSION_REQUIRED check below and refused
  // (isConcurrentMode(root) sees that session's own live heartbeat and, with
  // no id to exclude it by, reads it as "another" session). Exactly one
  // fresh live session now resolves and adopts before isConcurrentMode is
  // ever consulted; two-or-more still leaves resolvedSession null and hits
  // the unchanged refusal below.
  const resolvedSession = resolveSessionId({ flag: session, root });
  // hardening-4a: mirrors claimCellFile's typed refusal — a solo caller
  // (nobody else live) keeps today's sessionless-reserve behavior
  // byte-unchanged; `conflicts: []` is included defensively alongside `code`
  // so any existing caller that only inspects `.conflicts` on !ok never
  // crashes on this new failure shape.
  if (resolvedSession == null && isConcurrentMode(root)) {
    return {
      ok: false,
      code: 'SESSION_REQUIRED',
      reason: `reserve: cannot reserve "${reservedPath.trim()}" without identifying the acting session while another session is active — pass --session-id or set BEE_SESSION_ID (CLAUDE_CODE_SESSION_ID is also honored).`,
      conflicts: [],
    };
  }
  return withStoreLock(root, 'reservations', () => {
    // multisession-native-13 (D4): ONLY a hard conflict refuses the reserve
    // — a pre-existing 'intent' row (declared broad/glob scope) never blocks
    // a fellow caller from staking out their own exact write-time lease
    // inside that scope (isHardConflict above). Both records end up active
    // simultaneously; the intent stays purely advisory/informational.
    const conflicts = findConflicts(root, agent.trim(), [reservedPath]).filter((c) => isHardConflict(c, reservedPath));
    if (conflicts.length > 0) {
      return { ok: false, conflicts };
    }
    const store = readStore(root);
    const reservation = {
      agent: agent.trim(),
      cell: cell.trim(),
      path: normalizePath(reservedPath),
      ttl_seconds: Number.isFinite(ttl) && ttl > 0 ? Math.floor(ttl) : DEFAULT_TTL_SECONDS,
      reserved_at: utcNow(),
      released_at: null,
      // session is OPTIONAL and OMITTED entirely when absent (mirrors claims.mjs's
      // lane-omission pattern): every pre-existing row and every call that never
      // resolves a session keeps today's exact shape, byte for byte.
      ...(resolvedSession ? { session: resolvedSession } : {}),
      // kind (multisession-native-13, D4): ALWAYS stamped explicitly, unlike
      // `session` above — every row is unambiguously a lease or an intent,
      // never merely "no field means legacy". Callers that never pass `kind`
      // get 'lease', which is exactly what every pre-existing row already
      // behaved as (hard-conflict-by-default) — semantically byte-unchanged.
      kind,
    };
    store.reservations.push(reservation);
    writeStore(root, store);
    return { ok: true, reservation };
  });
}

export async function release(root, { agent, cell = null }) {
  if (typeof agent !== 'string' || !agent.trim()) {
    throw new Error('release: agent is required.');
  }
  return withStoreLock(root, 'reservations', () => {
    const store = readStore(root);
    const releasedAt = utcNow();
    let released = 0;
    for (const reservation of store.reservations) {
      if (reservation.released_at != null) continue;
      if (reservation.agent !== agent.trim()) continue;
      if (cell && reservation.cell !== cell) continue;
      reservation.released_at = releasedAt;
      released += 1;
    }
    if (released > 0) writeStore(root, store);
    return { released };
  });
}

export async function sweepExpired(root) {
  return withStoreLock(root, 'reservations', () => {
    const store = readStore(root);
    const nowMs = Date.now();
    const releasedAt = utcNow();
    let released = 0;
    for (const reservation of store.reservations) {
      if (reservation.released_at != null) continue;
      if (!isExpired(reservation, nowMs)) continue;
      reservation.released_at = releasedAt;
      released += 1;
    }
    if (released > 0) writeStore(root, store);
    return released;
  });
}

/**
 * D5 — same-session-only TTL renewal for this session's active holds (the
 * reservations-side sibling of claims.mjs's renewClaimTTL): refreshes
 * `reserved_at` — the expiry clock — for every non-released row owned by
 * sessionId. Never touches another session's rows, never revives an
 * already-released one (released_at != null is a hard skip, matching
 * findConflicts/findSessionConflicts' own posture). Runs under the same
 * store lock as reserve/release/sweepExpired so a renewal can never race a
 * concurrent reservation mutation into a lost update; hook callers (D5 Δ3 —
 * "hooks never wait on the lock") pass `{ maxAttempts: 1 }` through
 * lockOptions unchanged, matching claims.mjs heartbeatTouch's own posture.
 */
export async function renewHoldsBySession(root, sessionId, { now = Date.now(), lockOptions } = {}) {
  const session = typeof sessionId === 'string' ? sessionId.trim() : '';
  if (!session) return { ok: true, renewed: 0 };
  return withStoreLock(
    root,
    'reservations',
    () => {
      const store = readStore(root);
      const nowIso = new Date(now).toISOString();
      let renewed = 0;
      for (const reservation of store.reservations) {
        if (reservation.released_at != null) continue;
        if (reservation.session !== session) continue;
        reservation.reserved_at = nowIso;
        renewed += 1;
      }
      if (renewed > 0) writeStore(root, store);
      return { ok: true, renewed };
    },
    lockOptions,
  );
}
