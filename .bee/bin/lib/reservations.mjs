// reservations.mjs — same-session file reservations for swarms.
//
// multisession-native-16 (D4/D5, issue #56 3.3, advisor consult slice 3
// conditions B/C/E): the storage-touching verbs below (reserve/release/
// sweepExpired/renewHoldsBySession/listReservations/findConflicts/
// findSessionConflicts) are now a SHIM over lib/lease-store.mjs's sharded
// per-resource lease files (`.bee/runtime/leases/paths/<hash>.json`) — every
// reservation is a `type: 'path'` lease. `.bee/reservations.json` (this
// file's historic single-JSON store) is DEMOTED to a rebuildable PROJECTION
// for legacy readers only (rebuildReservationsProjection below, wired into
// state-projection.mjs's rebuildAllProjections) — it is never read or
// written by any function in this module anymore. The CLI surface (`bee
// reservations *` in bee.mjs) is untouched and stays byte-compatible: same
// flags, same output shapes, because every handler there calls straight
// through to the exported functions below, which keep their exact
// signatures and return shapes.
//
// Pure predicates (pathsOverlap, isHardConflict, RESERVATION_KINDS,
// reservationsPath) are UNCHANGED — they never touched storage and stay
// exactly as they were, since other modules (worktree-holds.mjs,
// schedule.mjs, state.mjs, guards.mjs) import them directly and must keep
// seeing byte-identical behavior. The pre-msn-16 private isExpired/isActive
// pair (operating on a reservation's own {reserved_at, ttl_seconds}) is
// GONE — expiry is now decided once, on the raw lease record's own
// `expires_at`, by isLeaseRecordExpired below, before any translation to the
// reservation shape happens (see that function's own comment for why the
// translated shape is unsafe to filter on directly).
//
// ─── the lease<->reservation field mapping ──────────────────────────────────
// A lease record's shape (lease-store.mjs) is {resource, mode, workflow_id,
// session_id, workspace_id, epoch, acquired_at, expires_at, kind} and
// requires workflow_id/session_id/workspace_id to be non-empty strings and
// epoch to be a finite number — none of which reservations have a native
// equivalent for except workflow_id (reservations always require a `cell`,
// which is exactly "the unit of work this hold belongs to" — the same role
// workflow_id plays for lease-store). The remaining three required-but-
// reservation-less fields are repurposed, documented here so the mapping is
// never rediscovered by reading call sites:
//   - workflow_id  <- cell (always present on a reservation; no synthesis
//                      needed)
//   - session_id   <- the resolved session id, or SESSIONLESS_SESSION_ID (a
//                      control-char-wrapped sentinel, same style as bee.mjs's
//                      LIST_ALL_HOLDS_SENTINEL) when the reservation is
//                      genuinely session-less — never a real session id,
//                      which always comes from claims.mjs's resolveSessionId
//                      (env var or a live session record's own id).
//   - workspace_id <- AGENT_WORKSPACE_PREFIX + agent. Reservations have no
//                      workspace-identity concept yet (advisor consult slice
//                      3 condition A: workspace/controlRoot differentiation
//                      is slice-4 work) and lease-store's path-type record
//                      shape has no dedicated `agent` slot, so this field
//                      carries the reservation's `agent` through the shim
//                      instead. NOT real workspace identity — do not read it
//                      as such anywhere else.
//   - epoch        <- RESERVATION_LEASE_EPOCH (0), always. Reservations carry
//                      no fencing concept (that is msn-12's claim-fencing
//                      epoch, a wholly separate mechanism on claims.mjs, not
//                      reused here) — this is lease-store's required field
//                      only, never compared or consumed by anything in this
//                      module.
//   - mode         <- RESERVATION_LEASE_MODE ('write'), always — reservations
//                      exist to guard write-heavy work (AGENTS.md critical
//                      rule 4), so this constant documents intent even though
//                      nothing currently branches on it.
// `ttl`/`kind` map straight through unchanged (lease-store's own TTL/kind
// vocabulary already matches reservations.mjs's, see below).
//
// ─── TTL semantics ───────────────────────────────────────────────────────
// lease-store's computeExpiresAt treats a non-positive ttl as "never
// expires" (expires_at: null) — DIFFERENT from this module's pre-msn-16
// reserve(), which silently coerced a non-positive ttl to DEFAULT_TTL_SECONDS
// (in practice unreachable through the CLI anyway: bee.mjs's --ttl flag
// parsing already refuses non-positive values before reserve() ever sees
// them). msn-16 adopts lease-store's semantics as the correct behavior going
// forward — "non-positive never expires" — matching this module's own
// isExpired() function, whose ttl<=0 branch already encoded that meaning but
// was previously unreachable via reserve(). A reconstructed reservation row's
// `ttl_seconds` uses the same 0-means-never-expires sentinel when a lease has
// no `expires_at` (leaseToReservation below).
//
// ─── overlap-conflict race window (accepted trade-off) ─────────────────────
// reservations' conflict model is OVERLAP-based (directory-prefix / trivial
// glob containment, pathsOverlap) across potentially DIFFERENT exact path
// strings — lease-store's O_EXCL create is EXACT-RESOURCE-KEY exclusion only.
// For the identical-path case these coincide exactly (isHardConflict already
// treats an identical path as always-hard regardless of kind, so
// acquireLeases' O_EXCL for that same key is race-free and matches the old
// D2 lost-update-free guarantee byte for byte — see reserve() below). For a
// genuinely overlapping-but-different path (e.g. "src/api" vs
// "src/api/router.ts"), the overlap pre-check (findConflicts) is an unlocked
// read, same lock-free posture as every other lease-store read — so two
// reserve() calls racing on DIFFERENT-but-overlapping paths in the same
// instant could theoretically both pass the pre-check and both land distinct
// lease files. This is the one place this migration narrows the prior
// whole-file-lock guarantee (which serialized every reserve() unconditionally
// via a single store-wide 'reservations' lock). Deliberately accepted per
// this cell's own hot-path requirement ("no global 'reservations' store lock
// ... on reserve/release/renew"): reservations coordinate cooperating
// swarm agents at human/agent working pace, not adversarial high-frequency
// locking, and the one case that matters most in practice — two agents
// grabbing the literal same file — stays fully race-free through
// acquireLeases' O_EXCL. Documented here rather than silently narrowed.

import fs from 'node:fs';
import path from 'node:path';
import { writeJsonAtomic } from './fsutil.mjs';
import { resolveSessionId, isConcurrentMode } from './claims.mjs';
import {
  acquireLeases,
  releaseLease,
  renewLease,
  listLeases,
  LeaseStoreError,
} from './lease-store.mjs';

// ─── msn-18b: control-plane root resolution (self-contained) ───────────────
// Leases are control-plane (PLANE RULE, docs/history/multisession-native
// CONTEXT.md D2/D3) — a lease taken from a linked worktree must land in
// MAIN's shared `.bee/runtime/leases/` store, not a worktree-local one, the
// same way msn-18a re-rooted claims/sessions/workflow reads onto
// state.mjs's `controlRootFor(root)`.
//
// This module CANNOT import controlRootFor (or resolveContext/resolveRoots)
// from state.mjs: state.mjs already imports THIS module directly
// (`import { pathsOverlap, listReservations } from './reservations.mjs'`,
// used by resolvePipeline/startFeature/etc.), so the reverse import would be
// a straight two-file cycle. Threading a `controlRoot` parameter from every
// caller was the other option (see this cell's own instructions), but
// `reservations reserve`/`release`/`list`/`sweep-expired` are called
// DIRECTLY from bee.mjs's CLI dispatcher with the bare, workspace-local
// `root` (bee.mjs is out of this cell's scope — msn-18c) — threading would
// leave every one of those call sites silently unfixed until 18c lands,
// which is exactly the "no coordination store left worktree-local in these
// modules" prohibition this cell must not violate.
//
// So this module resolves its OWN control root instead: `findMainRoot`
// below is a minimal, self-contained replica of state.mjs's
// resolveRootsCore/resolveContext linked-worktree walk-up — ONLY the
// mainRoot-finding portion (the grant-registry/workspaceId half of
// resolveContext is irrelevant here; a lease always targets the SHARED
// control store regardless of whether the worktree is grant-registered for
// its OWN local coordination store). This is a deliberate second
// implementation of "find the git-common mainRoot", accepted per
// advisor-digest-slice4 binding condition 6 ("resolveContext is THE single
// git-common-dir resolver ... see herding.mjs's resolveHerdingMainRoot for
// the one call site left independent, and why") — this is the SECOND such
// documented exception, for the same reason (a real import-cycle
// constraint, not convenience). Any future refactor that extracts this walk
// into its own zero-dependency leaf module (so state.mjs and this module
// share one implementation instead of two) should retire this copy.
//
// DELIBERATE divergence from state.mjs's resolveRootsCore: that function
// THROWS WorktreeLinkInvalidError for a malformed linked-worktree `.git`
// file — correct for a CLI dispatcher that can surface the error to a
// human. This module cannot make the same choice: guards.mjs's checkWrite
// (the write-guard hot path, called on every tool write) imports
// findConflicts/findSessionConflicts straight from here and has its OWN
// documented "unresolvable topology fails OPEN, never denies, never
// throws" contract (xwh-4) for exactly this malformed-link shape — proven
// red-first by test_guards.mjs's own "unresolvable topology fails OPEN"
// row, which this function's first draft (a throwing version, mirroring
// state.mjs 1:1) broke by turning that guard's fail-open path into an
// uncaught throw. So `findMainRoot` below FAILS OPEN on every malformed
// shape (returns `null`, exactly like "no git root reachable at all") —
// never throws — matching this module's own pre-existing "hook/guard-
// reachable, must never throw" posture (D5 "hooks never wait on the lock").
// A caller that specifically wants the CLI-facing throw-on-malformed
// behavior already has it via state.mjs's own resolveRoots/resolveContext.
function readGitdirFileForRoot(file, base) {
  try {
    let raw = fs.readFileSync(file, 'utf8').trim();
    if (!raw) return null;
    if (raw.startsWith('gitdir:')) raw = raw.slice('gitdir:'.length).trim();
    return path.resolve(base, raw.replace(/\\/g, path.sep));
  } catch {
    return null;
  }
}

function locateGitRootForRoot(start) {
  let dir = path.resolve(start || process.cwd());
  while (true) {
    const marker = path.join(dir, '.git');
    if (fs.existsSync(marker)) return { workRoot: dir, marker };
    const parent = path.dirname(dir);
    if (parent === dir) return null;
    dir = parent;
  }
}

/**
 * findMainRoot(root) — the git-common mainRoot for `root` (byte-for-byte the
 * same value state.mjs's resolveContext(root).controlRoot resolves for
 * every WELL-FORMED topology, minus the grant-registry workspaceId
 * computation this module never needs). An ORDINARY checkout (no `.git`
 * file, i.e. not a linked worktree) returns `root`'s own git root —
 * byte-identical to `root` for every solo/main repo, so main-checkout
 * callers see zero behavior change. A LINKED worktree returns the MAIN
 * checkout's root. Never throws (see the block comment above for why this
 * diverges from state.mjs's own WorktreeLinkInvalidError-throwing
 * resolveRoots): a malformed linked-worktree `.git` file, OR no git root
 * reachable at all, both resolve to `null` — the caller (controlRootFor)
 * falls back to `root` itself, the exact pre-msn-18b behavior for that
 * checkout.
 */
export function findMainRoot(root) {
  const located = locateGitRootForRoot(root);
  if (!located) return null;
  const { workRoot, marker } = located;
  let isFile = false;
  try {
    isFile = fs.statSync(marker).isFile();
  } catch {
    return null;
  }
  if (!isFile) return workRoot; // ordinary checkout: mainRoot === workRoot

  const gitdir = readGitdirFileForRoot(marker, workRoot);
  if (!gitdir) return null; // malformed — fail open, see block comment above
  const worktreesRoot = path.resolve(gitdir, '..');
  const commonGitDir = path.resolve(worktreesRoot, '..');
  if (path.basename(commonGitDir) !== '.git' || path.basename(worktreesRoot) !== 'worktrees') {
    return null; // outside the expected .git/worktrees namespace — fail open
  }
  const id = path.basename(gitdir);
  if (!id || id === '.' || id === '..') return null; // empty id — fail open
  const reverse = readGitdirFileForRoot(path.join(gitdir, 'gitdir'), gitdir);
  if (!reverse || path.resolve(reverse) !== path.resolve(marker)) {
    return null; // reverse pointer missing/mismatched — fail open
  }
  return path.dirname(commonGitDir);
}

/** controlRootFor(root) — findMainRoot(root), falling back to `root` itself
 * when nothing is resolvable (no git root, no onboarding marker reachable,
 * OR a malformed linked-worktree link — findMainRoot fails open on all
 * three, never throws), so every exported store-touching function below can
 * call this unconditionally with no try/catch of its own. Every
 * lease-store.mjs call in this module goes through this — see each
 * function's own use below. This is what keeps reserve/release/
 * sweepExpired/renewHoldsBySession/listReservations/findConflicts/
 * findSessionConflicts safe to call from guards.mjs's write-guard hot path
 * (swarming-reservation check), which has its OWN documented "unresolvable
 * topology fails OPEN, never throws" contract (xwh-4) — see findMainRoot's
 * own doc comment for the regression this fail-open design fixes
 * (test_guards.mjs's "unresolvable topology fails OPEN" case). */
function controlRootFor(root) {
  return findMainRoot(root) ?? root;
}

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

// ─── lease-store mapping constants (msn-16, see module header) ─────────────
const PATH_RESOURCE_PREFIX = 'path:';
const AGENT_WORKSPACE_PREFIX = 'agent:';
// Control-char-wrapped so it can never collide with a real session id (every
// real one comes from claims.mjs's resolveSessionId — an env var value or a
// live session record's own id, never containing NUL bytes) — same
// unforgeable-sentinel style as bee.mjs's LIST_ALL_HOLDS_SENTINEL.
const SESSIONLESS_SESSION_ID = '\u0000bee-reservation-sessionless\u0000';
const RESERVATION_LEASE_MODE = 'write';
const RESERVATION_LEASE_EPOCH = 0;

function utcNow() {
  return new Date().toISOString();
}

export function reservationsPath(root) {
  return path.join(root, '.bee', 'reservations.json');
}

// isLeaseRecordExpired — the ONE place an activeOnly/sweep decision is made,
// operating on the RAW lease record's own `expires_at` (an absolute
// timestamp lease-store already computed), never on a translated
// reservation's derived {reserved_at, ttl_seconds} pair. This matters
// because that derived pair is LOSSY for expiry purposes once a renewal
// moves `expires_at` to before the lease's original `acquired_at` (e.g. a
// test backdating a lease into the past, or in principle any renewal whose
// `now` predates the original acquire) — reconstructing `ttl_seconds` as
// `round((expires_at - acquired_at) / 1000)` would go negative, and this
// module's own isHardConflict-adjacent "non-positive ttl means never
// expires" convention would then misread an ALREADY-EXPIRED lease as
// never-expiring. Mirrors lease-store.mjs's own private isLeaseExpired
// (deliberately not imported — same small-duplicate-over-new-export
// precedent as this module's other borrowed conventions).
function isLeaseRecordExpired(record, nowMs) {
  if (record.expires_at == null) return false; // no record, or "never expires"
  const expiresMs = Date.parse(record.expires_at);
  if (!Number.isFinite(expiresMs)) return false;
  return expiresMs <= nowMs;
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

// ─── lease <-> reservation translation (msn-16) ─────────────────────────────

function isPathLease(record) {
  return Boolean(record) && typeof record.resource === 'string' && record.resource.startsWith(PATH_RESOURCE_PREFIX);
}

function leaseAgent(record) {
  const workspaceId = record.workspace_id;
  return typeof workspaceId === 'string' && workspaceId.startsWith(AGENT_WORKSPACE_PREFIX)
    ? workspaceId.slice(AGENT_WORKSPACE_PREFIX.length)
    : workspaceId;
}

// Shared by leaseToReservation (display) and renewHoldsBySession (renewal —
// a renewal must extend by the lease's OWN original window, not some
// unrelated default, to match the pre-msn-16 renewHoldsBySession contract of
// "reset the anchor time, never touch ttl_seconds"). 0 sentinel: never
// expires (matches isExpired's own ttl<=0 convention).
function leaseTtlSeconds(record) {
  return record.expires_at != null
    ? Math.max(0, Math.round((Date.parse(record.expires_at) - Date.parse(record.acquired_at)) / 1000))
    : 0;
}

function leaseToReservation(record) {
  const ttlSeconds = leaseTtlSeconds(record);
  return {
    agent: leaseAgent(record),
    cell: record.workflow_id,
    path: record.resource.slice(PATH_RESOURCE_PREFIX.length),
    ttl_seconds: ttlSeconds,
    reserved_at: record.acquired_at,
    // A released lease file is DELETED, never soft-deleted — so every row
    // this module can still see is, by construction, not-yet-released. This
    // field is kept in the shape (rather than dropped) purely for CLI/shape
    // compatibility; see module header for the observable consequence
    // (`bee reservations list` without --active-only can no longer show
    // release HISTORY, only currently-live-or-not-yet-swept rows).
    released_at: null,
    ...(record.session_id && record.session_id !== SESSIONLESS_SESSION_ID ? { session: record.session_id } : {}),
    kind: record.kind || DEFAULT_RESERVATION_KIND,
  };
}

// msn-18b: the single chokepoint every reader below (listReservations,
// release, sweepExpired, renewHoldsBySession) goes through — re-rooted once
// here via controlRootFor so a linked worktree's read lands in main's shared
// leases store, never a worktree-local one.
function listPathLeaseRecords(root) {
  return listLeases(controlRootFor(root)).leases.filter(isPathLease);
}

export function listReservations(root, { activeOnly = false, now = Date.now() } = {}) {
  const records = listPathLeaseRecords(root);
  // Filter on the RAW records (isLeaseRecordExpired, using expires_at
  // directly) BEFORE translating to the reservation shape — see that
  // function's own comment for why filtering the translated shape instead
  // would be lossy.
  const active = activeOnly ? records.filter((record) => !isLeaseRecordExpired(record, now)) : records;
  return active.map(leaseToReservation);
}

/** Active reservations held by *other* agents covering any of the given paths. */
export function findConflicts(root, agent, paths, { now = Date.now() } = {}) {
  const requested = (Array.isArray(paths) ? paths : [paths]).filter(Boolean);
  if (requested.length === 0) return [];
  return listReservations(root, { activeOnly: true, now }).filter(
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
export function findSessionConflicts(root, sessionId, paths, { now = Date.now() } = {}) {
  const requested = (Array.isArray(paths) ? paths : [paths]).filter(Boolean);
  if (requested.length === 0) return [];
  const acting = typeof sessionId === 'string' ? sessionId.trim() : '';
  return listReservations(root, { activeOnly: true, now }).filter(
    (reservation) =>
      typeof reservation.session === 'string' &&
      reservation.session.trim() &&
      reservation.session !== acting &&
      requested.some((requestedPath) => pathsOverlap(reservation.path, requestedPath)),
  );
}

/**
 * reserve() — msn-16: acquires a per-resource lease-store lease keyed by the
 * exact path, after an overlap-based conflict pre-check (see module header,
 * "overlap-conflict race window"). No global 'reservations' store lock is
 * ever taken here — the exact-path case is race-free via acquireLeases'
 * O_EXCL create (LEASE_HELD is translated back into the same `{ok:false,
 * conflicts:[...]}` shape reserve() has always returned on conflict), which
 * is exactly the collision case isHardConflict always treats as hard
 * regardless of kind (see its own doc comment) — so this loses none of the
 * old D2 lost-update guarantee for the case that matters most (two callers
 * racing the literal same path).
 *
 * `kind` (multisession-native-13, D4): OPTIONAL, defaults to `'lease'`
 * (RESERVATION_KINDS above) — a worker's own write-time reservation stays a
 * hard conflict exactly as before this cell. Pass `kind: 'intent'` to
 * declare a broad/glob planning-time scope instead; guards.mjs's checkWrite
 * downgrades a conflict against an 'intent' row to an advisory warning
 * unless it collapses onto the exact write target (see guards.mjs
 * isAdvisoryIntentConflict). Invalid values throw synchronously, before any
 * lease file is ever touched.
 *
 * D3: when `session` is absent, it self-derives (explicit flag -> `resolve
 * SessionId`'s own BEE_SESSION_ID/CLAUDE_CODE_SESSION_ID env fallback ->
 * null), so a top-level-session reserve becomes cross-session-visible by
 * default; a genuinely absent id (no flag, no env) still omits the `session`
 * key from the returned reservation shape entirely, byte-identical to
 * today's shape — UNLESS another session is concurrently live
 * (hardening-4a): see the SESSION_REQUIRED refusal below.
 */
export async function reserve(
  root,
  { agent, cell, path: reservedPath, ttl = DEFAULT_TTL_SECONDS, session = null, kind = DEFAULT_RESERVATION_KIND, now = Date.now() },
) {
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
  // msn-18b (PLANE RULE): sessions are control-plane — resolved once here
  // via controlRootFor and reused for every claims.mjs/lease-store.mjs touch
  // below, so a reserve taken from a linked worktree sees the SAME live
  // sessions and lands its lease in the SAME shared store main uses.
  const controlRoot = controlRootFor(root);
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
  const resolvedSession = resolveSessionId({ flag: session, root: controlRoot });
  // hardening-4a: mirrors claimCellFile's typed refusal — a solo caller
  // (nobody else live) keeps today's sessionless-reserve behavior
  // byte-unchanged; `conflicts: []` is included defensively alongside `code`
  // so any existing caller that only inspects `.conflicts` on !ok never
  // crashes on this new failure shape.
  if (resolvedSession == null && isConcurrentMode(controlRoot)) {
    return {
      ok: false,
      code: 'SESSION_REQUIRED',
      reason: `reserve: cannot reserve "${reservedPath.trim()}" without identifying the acting session while another session is active — pass --session-id or set BEE_SESSION_ID (CLAUDE_CODE_SESSION_ID is also honored).`,
      conflicts: [],
    };
  }

  const trimmedAgent = agent.trim();
  const trimmedCell = cell.trim();
  const normalizedTargetPath = normalizePath(reservedPath);

  // Overlap pre-check — see module header for the accepted race window on a
  // non-identical overlapping path. multisession-native-13 (D4): ONLY a hard
  // conflict refuses the reserve — a pre-existing 'intent' row (declared
  // broad/glob scope) never blocks a fellow caller from staking out their
  // own exact write-time lease inside that scope (isHardConflict above).
  const overlapConflicts = findConflicts(controlRoot, trimmedAgent, [reservedPath], { now }).filter((c) =>
    isHardConflict(c, reservedPath),
  );
  if (overlapConflicts.length > 0) {
    return { ok: false, conflicts: overlapConflicts };
  }

  const sessionIdForLease = resolvedSession || SESSIONLESS_SESSION_ID;
  let leaseRecord;
  try {
    [leaseRecord] = acquireLeases(
      controlRoot,
      [
        {
          type: 'path',
          id: reservedPath,
          mode: RESERVATION_LEASE_MODE,
          workflow_id: trimmedCell,
          session_id: sessionIdForLease,
          workspace_id: `${AGENT_WORKSPACE_PREFIX}${trimmedAgent}`,
          epoch: RESERVATION_LEASE_EPOCH,
          ttl,
          kind,
        },
      ],
      { now },
    );
  } catch (error) {
    if (error instanceof LeaseStoreError && error.code === 'LEASE_HELD') {
      // Lost an exact-path race the overlap pre-check (unlocked read) missed
      // — race-free thanks to acquireLeases' O_EXCL. Report it exactly like
      // a conflict caught by the pre-check.
      const holderReservation = error.holder && isPathLease(error.holder) ? leaseToReservation(error.holder) : null;
      return { ok: false, conflicts: holderReservation ? [holderReservation] : [] };
    }
    throw error;
  }

  const reservation = leaseToReservation(leaseRecord);
  // normalizedTargetPath is already reflected in reservation.path via the
  // lease's own canonicalized resource key (lease-store's canonicalizePath
  // mirrors this module's normalizePath byte for byte — see lease-store.mjs
  // module header) — asserted implicitly by every existing overlap test
  // rather than re-derived here.
  void normalizedTargetPath;
  return { ok: true, reservation };
}

export async function release(root, { agent, cell = null }) {
  if (typeof agent !== 'string' || !agent.trim()) {
    throw new Error('release: agent is required.');
  }
  // msn-18b (PLANE RULE): see reserve()'s own comment.
  const controlRoot = controlRootFor(root);
  const trimmedAgent = agent.trim();
  const matches = listPathLeaseRecords(controlRoot).filter((record) => {
    if (leaseAgent(record) !== trimmedAgent) return false;
    if (cell && record.workflow_id !== cell) return false;
    return true;
  });
  let released = 0;
  for (const record of matches) {
    const result = await releaseLease(controlRoot, { type: 'path', id: record.resource.slice(PATH_RESOURCE_PREFIX.length) });
    if (result.released) released += 1;
  }
  return { released };
}

/**
 * sweepExpired — per-record (each expired lease is its own releaseLease
 * call), scoped to PATH-type leases only: this module never sweeps a
 * 'cell'-type lease, even though lease-store's own sweepExpiredLeases would
 * (that resource type belongs to a different, not-yet-production consumer of
 * lease-store — see lease-store.mjs module header). No global 'reservations'
 * lock; each release only ever contends with a concurrent mutation of that
 * SAME resource file (lease-store's own per-file lock/O_EXCL discipline).
 */
export async function sweepExpired(root, { now = Date.now() } = {}) {
  // msn-18b (PLANE RULE): see reserve()'s own comment.
  const controlRoot = controlRootFor(root);
  let released = 0;
  for (const record of listPathLeaseRecords(controlRoot)) {
    // isLeaseRecordExpired on the RAW record — see its own comment for why
    // this must never go through the translated reservation shape.
    if (!isLeaseRecordExpired(record, now)) continue;
    const result = await releaseLease(controlRoot, { type: 'path', id: record.resource.slice(PATH_RESOURCE_PREFIX.length) });
    if (result.released) released += 1;
  }
  return released;
}

/**
 * D5 — same-session-only TTL renewal for this session's active holds (the
 * reservations-side sibling of claims.mjs's renewClaimTTL / lease-store's own
 * renewLeasesBySession, which this delegates to per-record but SCOPED to
 * path-type leases only, same reasoning as sweepExpired above). Never
 * touches another session's rows. Runs under lease-store's own per-file lock
 * (never a store-wide lock) so a renewal can never race a concurrent
 * mutation of that SAME resource into a lost update; hook callers (D5 Δ3 —
 * "hooks never wait on the lock", advisor consult slice 3 condition E) pass
 * `{ maxAttempts: 1 }` through lockOptions unchanged, matching
 * claims.mjs heartbeatTouch's own posture.
 */
export async function renewHoldsBySession(root, sessionId, { now = Date.now(), lockOptions } = {}) {
  const session = typeof sessionId === 'string' ? sessionId.trim() : '';
  if (!session) return { ok: true, renewed: 0 };
  // msn-18b (PLANE RULE): see reserve()'s own comment.
  const controlRoot = controlRootFor(root);
  let renewed = 0;
  for (const record of listPathLeaseRecords(controlRoot)) {
    if (record.session_id !== session) continue;
    // Renew by the lease's OWN original ttl window, not lease-store's
    // renewLease default — the pre-msn-16 renewHoldsBySession contract was
    // "reset the anchor time (reserved_at), never touch ttl_seconds", so a
    // 60s reservation stays a 60s reservation across renewals, not silently
    // widened to lease-store's own DEFAULT_TTL_SECONDS (3600).
    const ttl = leaseTtlSeconds(record);
    try {
      await renewLease(controlRoot, { type: 'path', id: record.resource.slice(PATH_RESOURCE_PREFIX.length) }, { ttl, now, lockOptions });
      renewed += 1;
    } catch (error) {
      if (error instanceof LeaseStoreError && error.code === 'LEASE_MISSING') continue; // swept concurrently — fine, skip
      throw error;
    }
  }
  return { ok: true, renewed };
}

/**
 * rebuildReservationsProjection(root) — msn-16, must_have "projection
 * rebuildable: delete reservations.json, rebuild, legacy readers
 * unaffected": regenerates `.bee/reservations.json` from the CURRENT set of
 * active path leases (bounded by however many are actually live — never the
 * unbounded, ever-growing full history the pre-msn-16 file kept). Registered
 * into state-projection.mjs's rebuildAllProjections (mirrors msn-15's
 * rebuildHandoffProjection registration) so `bee state rebuild-projections`
 * (and the recovery path) regenerates it on demand. Deliberately NOT called
 * from reserve/release/renew/sweep themselves — see this module's header
 * ("overlap-conflict race window" section) and this cell's own hot-path
 * requirement: no reservation mutation ever pays for a synchronous
 * whole-projection rewrite. Legacy direct readers of the raw file (anything
 * that still parses `.bee/reservations.json` itself rather than importing
 * this module's functions) therefore see a snapshot as of the last rebuild,
 * not a live view — the one production reader that needed a LIVE view
 * (state.mjs's startFeature precondition, advisor consult slice 3 condition
 * C) was migrated onto listReservations() directly instead of reading this
 * projection, so no correctness-critical caller depends on this being fresh.
 *
 * msn-18b: `root` here is deliberately NOT re-rooted for the WRITE
 * (`reservationsPath(root)`) — `.bee/reservations.json` is a legacy,
 * single-checkout DISPLAY projection, same class as `.bee/state.json`/
 * `.bee/HANDOFF.json` (state-projection.mjs leaves those on the caller's own
 * workspace root too, see that module's header for the full reasoning). The
 * READ (`listReservations` below) IS control-rooted regardless, via
 * listPathLeaseRecords's own controlRootFor — so every checkout's local
 * projection file becomes an accurate full mirror of the SHARED lease data,
 * which is exactly what a display-only projection should do.
 */
export function rebuildReservationsProjection(root) {
  const rows = listReservations(root, { activeOnly: true });
  const sorted = [...rows].sort((a, b) => {
    if (a.reserved_at !== b.reserved_at) return a.reserved_at < b.reserved_at ? -1 : 1;
    if (a.path !== b.path) return a.path < b.path ? -1 : 1;
    return 0;
  });
  writeJsonAtomic(reservationsPath(root), { reservations: sorted });
  return { authoritative: true, count: sorted.length };
}
