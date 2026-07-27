// claims.mjs — cross-session session identity + atomic per-cell claims.
// Stores (all repo-relative under .bee/ — never system temp, pattern 20260708):
//   .bee/sessions/<session-id>.json  { id, started_at, last_heartbeat, lane?,
//                                       workspace_id? }
//                                    (lane is OPTIONAL and OMITTED while
//                                     unbound — see bindSessionLane below;
//                                     workspace_id is likewise OPTIONAL and
//                                     OMITTED — msn-19, CONTEXT.md D2/D3:
//                                     stamped from the caller's resolved
//                                     resolveContext(cwd).workspaceId at
//                                     session-create time, same "omit rather
//                                     than write a placeholder" convention as
//                                     transcript_path below)
//   .bee/claims/<cell-id>.json       { cell, session?, ttl_seconds, claimed_at,
//                                      adopted_from?, adopted_at?, workspace_id? }
//                                    (session is OMITTED, never null, for a
//                                     sessionless single-user claim — D1 Δ2;
//                                     workspace_id — msn-19 — is OMITTED
//                                     unless the acting session's OWN record
//                                     carries one, auto-looked-up at claim
//                                     time — see claimCellFile below, never a
//                                     caller-supplied argument)
//   .bee/claims/<cell-id>.adopting   exclusive per-claim gate (adopt/sweep/release)
//
// Claim creation is exclusive-create ('wx' → O_EXCL): the probe-proven
// one-winner primitive (.bee/spikes/fresh-session-handoff/probe_atomic_claim.mjs,
// PASS 20x8 on linux AND win32). O_EXCL is not reliable on network filesystems,
// so project directories on network mounts (NFS and similar) are unsupported.
//
// Typed-failure contract (pinned, fresh-session-handoff validating repair):
// every mutating API returns { ok: true, ... } | { ok: false, code, reason }
// and never throws for contention. Codes: SESSION_EXISTS, SESSION_MISSING,
// CLAIMED, GATE_HELD, NOT_OWNER, NOT_FOUND, LOCK_BUSY (heartbeatSession,
// bindSessionLane, unbindSessionLane — rel180-2 / D10a, all three hold the
// same 'sessions' store-lock below). Bad arguments (empty/path-shaped ids)
// still throw, matching reservations.mjs.
//
// Ownership changes never delete-then-recreate the claim file: adoption and
// release run under the exclusive gate, and adoption rewrites the owner IN
// PLACE via atomic rename, so concurrent 'wx' claimers keep getting EEXIST
// throughout. Reclaim (sweepExpiredClaims) requires TTL expired AND heartbeat
// stale, re-verified under the gate — an expired TTL with a fresh heartbeat is
// never stolen (pattern 20260710).
//
// rel180-2 (fix-first, pre-existing race): the gate above only ever
// serialized adopt/sweep/release against EACH OTHER — heartbeatSession's
// write was never part of it, so a renewal landing strictly between the
// gate-protected heartbeat-staleness READ and the reclaim's rmSync was
// invisible to that decision (sweepExpiredClaims could reclaim a claim whose
// owner was, by the time of the write, alive and heartbeating). Closed by a
// second, SESSION-scoped store-lock ('sessions' — the same name
// heartbeatTouch already used) that both heartbeatSession's write and
// sweepExpiredClaims's heartbeat re-check + reclaim now hold as one unit
// (acquireSessionsLock below) — a renewal either completes fully before that
// unit starts (and is seen) or is forced to wait until it finishes (and only
// then lands, moot on an already-reclaimed claim). Bounded, never unbounded,
// matching the per-claim gate's own posture.
//
// D10a (multisession-native, issue #56 3.8): the SAME lost-update shape was
// still open on the lane-binding side — bindSessionLane/unbindSessionLane
// did their read-modify-write with NO lock at all, and heartbeatSession did
// its OWN read (of the whole session record, lane field included) OUTSIDE
// the 'sessions' lock above, so a bind/unbind landing strictly between that
// unlocked read and heartbeatSession's later locked write was invisible to
// the stale copy heartbeatSession then wrote back — silently erasing (bind)
// or resurrecting (unbind) the lane. Closed the same way: heartbeatSession's
// read moved INSIDE its existing lock section (now genuinely one atomic
// read-modify-write unit), and bindSessionLane/unbindSessionLane each now
// acquire the SAME 'sessions' lock (acquireSessionsLock) around their own
// read-modify-write, never a new lock name. All three call sites are
// call-site leaves — none nests inside another's lock acquisition (verified:
// the only bind/unbind callers are bee.mjs's state.session.bind/unbind CLI
// handlers, dispatched flat with no lock held) — so no self-deadlock risk
// against acquireSessionsLock's non-reentrant, bounded-retry primitive.

import fs from 'node:fs';
import path from 'node:path';
import { randomUUID } from 'node:crypto';
import { readJson, writeJsonAtomic, ensureDir } from './fsutil.mjs';
import { withStoreLock, acquireStoreLockOnceSync } from './lock.mjs';
// hardening-4b (sweep-reset): logs one audit line per cell reset below.
// decisions.mjs imports only fsutil.mjs/node builtins, so importing it here
// creates no cycle.
import { logDecision } from './decisions.mjs';
// Deliberately NOT importing reservations.mjs here (state.mjs:7-9 pins
// claims.mjs and reservations.mjs as cycle-free leaf modules — reservations.mjs
// already imports resolveSessionId from THIS file). heartbeatTouch below
// therefore renews only the session heartbeat + this session's claim files;
// its hold-renewal companion (renewHoldsBySession, reservations.mjs) is
// called by the HOOK callers alongside it (hooks/bee-prompt-context.mjs,
// hooks/bee-state-sync.mjs), composed at the call site the same way state.mjs
// composes both leaf modules without either importing the other.

export const DEFAULT_CLAIM_TTL_SECONDS = 3600;
export const DEFAULT_HEARTBEAT_STALE_SECONDS = 900;
// D5 — the hook-driven touch throttle: heartbeatTouch no-ops unless the
// stored heartbeat is older than this, so a burst of hook events (many
// prompts/tool calls in one turn) costs at most one refresh per window.
// Deliberately far below DEFAULT_HEARTBEAT_STALE_SECONDS (900) — the whole
// point is that stale-detection now has a real, frequently-running refresh
// behind it; the 900s threshold itself is unchanged (prohibition, msh-5).
export const HEARTBEAT_TOUCH_THROTTLE_SECONDS = 60;

// A missing file is the ordinary "record absent" case; any other errno
// (EACCES, EIO, EMFILE) is undetectable state a strict-mode caller must see
// as a thrown error, not a silent "absent" (review finding F1 sweep).
function rethrowUnlessMissing(err) {
  if (err && err.code === 'ENOENT') return;
  throw err;
}

function utcNow(nowMs) {
  return new Date(nowMs).toISOString();
}

/**
 * D3 (hardening-4a Δ-amended) — session id is resolved at mutation time,
 * never handed down: explicit flag (highest, for tests/CLI callers) ->
 * BEE_SESSION_ID env (runtime-neutral — set by any harness, not just
 * Claude Code) -> CLAUDE_CODE_SESSION_ID env (legacy, kept for back-compat)
 * -> hardening-1-7-10 D5's durable fallback (below) -> absent (null). A
 * blank/whitespace-only flag or env value is treated as absent, same as
 * omitting it. Callers that require a session (claim-next) still enforce
 * non-null themselves; this helper only resolves, it never refuses. lock.mjs's
 * withStoreLock duplicates the flag/env portion of this chain inline
 * (deliberately, to stay import-light, and never sees `root` — the durable
 * fallback is claims.mjs-only) — keep the two in sync by hand.
 *
 * D5 (Codex session bridge) — durable fallback: a native Codex session never
 * gets CLAUDE_CODE_SESSION_ID/BEE_SESSION_ID set the way Claude Code does, so
 * every mutation used to resolve null even though the session-init hook had
 * already registered a real, live session record for it — the "solo-session
 * SESSION_REQUIRED" bug. When `root` is supplied and flag/env both resolve to
 * nothing, list every LIVE (non-stale, same freshness predicate as
 * isConcurrentMode/heartbeatStale) session record: exactly ONE -> adopt its
 * id (the caller has no identity of its own, but there is exactly one live
 * worker in this checkout, so it is safe to infer that is who's asking); zero
 * or two-or-more is left null — a genuinely solo caller with nothing to adopt
 * stays null (existing sessionless-claim behavior), and real multi-session
 * ambiguity is never guessed at. Callers that pass no `root` (the large
 * majority — CLI flag/env resolution) see byte-identical behavior; passing an
 * `audit` object (mutated with `adopted: true` only on a successful adopt)
 * lets a caller like claimCellFile below tell an inferred identity apart from
 * an explicitly supplied one, without changing this function's plain
 * string|null return type for everyone else.
 */
export function resolveSessionId({ flag, root, audit } = {}) {
  if (typeof flag === 'string' && flag.trim()) return flag.trim();
  const beeEnv = process.env.BEE_SESSION_ID;
  if (typeof beeEnv === 'string' && beeEnv.trim()) return beeEnv.trim();
  const env = process.env.CLAUDE_CODE_SESSION_ID;
  if (typeof env === 'string' && env.trim()) return env.trim();
  if (root) {
    const fresh = listSessionRecords(root).filter((session) => !heartbeatStale(session));
    if (fresh.length === 1) {
      if (audit && typeof audit === 'object') audit.adopted = true;
      return fresh[0].id;
    }
  }
  return null;
}

function requireId(value, label) {
  if (typeof value !== 'string' || !value.trim()) {
    throw new Error(`${label} is required.`);
  }
  const id = value.trim();
  if (/[\\/]/.test(id) || id.includes('..')) {
    throw new Error(`${label} must be a plain id (no path separators).`);
  }
  return id;
}

function fail(code, reason, extra = {}) {
  return { ok: false, code, reason, ...extra };
}

export function sessionsDir(root) {
  return path.join(root, '.bee', 'sessions');
}

export function claimsDir(root) {
  return path.join(root, '.bee', 'claims');
}

export function sessionPath(root, sessionId) {
  return path.join(sessionsDir(root), `${requireId(sessionId, 'session id')}.json`);
}

export function claimPath(root, cellId) {
  return path.join(claimsDir(root), `${requireId(cellId, 'cell id')}.json`);
}

/** Exclusive per-claim gate file: <cellId>.adopting (adopt/sweep/release). */
export function claimGatePath(root, cellId) {
  return path.join(claimsDir(root), `${requireId(cellId, 'cell id')}.adopting`);
}

// ─── sessions ────────────────────────────────────────────────────────────────

// D5 (Codex session bridge) — `transcript_path` is OPTIONAL and OMITTED when
// absent/blank (same "never write a placeholder" convention as `lane` above):
// the session-init hook passes the real hook payload's transcript_path
// through here so recovery.mjs can resolve a session's transcript from the
// stored path directly instead of guessing via Claude's encoded-layout math
// (recovery.mjs prefers this when present; layout math stays the fallback).
//
// msn-19 (CONTEXT.md D2/D3): `workspace_id` is likewise OPTIONAL and OMITTED
// when absent/blank — the caller (hooks/bee-session-init.mjs) passes
// `state.resolveContext(cwd).workspaceId` through here so a session created
// from inside a registered worktree carries that worktree's workspace
// identity for the rest of its life; a caller with no workspace concept
// (most tests, and any pre-msn-19 caller) sees byte-identical session
// records, same as omitting transcript_path always has.
export function createSession(root, { id = randomUUID(), now = Date.now(), transcript_path, workspace_id } = {}) {
  const sessionId = requireId(id, 'session id');
  ensureDir(sessionsDir(root));
  const session = {
    id: sessionId,
    started_at: utcNow(now),
    last_heartbeat: utcNow(now),
    ...(typeof transcript_path === 'string' && transcript_path.trim() ? { transcript_path: transcript_path.trim() } : {}),
    ...(typeof workspace_id === 'string' && workspace_id.trim() ? { workspace_id: workspace_id.trim() } : {}),
  };
  try {
    fs.writeFileSync(sessionPath(root, sessionId), `${JSON.stringify(session, null, 2)}\n`, {
      encoding: 'utf8',
      flag: 'wx',
    });
  } catch (error) {
    if (error && error.code === 'EEXIST') {
      return fail('SESSION_EXISTS', `session "${sessionId}" already exists.`);
    }
    throw error;
  }
  return { ok: true, session };
}

/**
 * `strict` (review finding F1 sweep, worktree-concurrency-guard-controlroot-
 * port delta re-review): opt-in, default false — every existing caller keeps
 * `readJson`'s ordinary fail-open read (a missing/corrupt file reads as no
 * session). When true, bypasses `readJson` for the file read itself so a
 * hard error (EACCES/EIO/EMFILE) or a corrupt/unparseable record propagates
 * instead of silently reading as "session absent" — closing the same
 * fail-open shape F1 fixed at the directory level, now at the per-record
 * level `listSessionRecords(root, {strict:true})` reads through this.
 */
export function readSession(root, sessionId, { strict = false } = {}) {
  let file;
  try {
    file = sessionPath(root, sessionId); // fail-open: a malformed id reads as "no session"
  } catch {
    return null;
  }
  let session;
  if (strict) {
    let text;
    try {
      text = fs.readFileSync(file, 'utf8');
    } catch (err) {
      rethrowUnlessMissing(err);
      return null;
    }
    session = JSON.parse(text.charCodeAt(0) === 0xfeff ? text.slice(1) : text);
  } else {
    session = readJson(file, null);
  }
  if (!session || typeof session !== 'object' || session.id !== String(sessionId).trim()) return null;
  return session;
}

/**
 * List every readable session record under .bee/sessions/ (GH#20: the reader
 * cells.mjs's claim-next fallback pool needs to detect a lane's live owner).
 * Fail-open by default, matching readSession/heartbeatStale's posture: a
 * missing .bee/sessions/ directory is zero records, never an error, and an
 * unreadable/corrupt entry is silently skipped rather than surfaced — a
 * broken session record counts as absent, not as a reason to stop.
 *
 * `strict` (review finding F1, worktree-concurrency-guard-controlroot-port):
 * opt-in, default false so every existing caller (claim-next's fallback
 * pool, reservations.mjs, isConcurrentMode's other callers) keeps today's
 * fail-open behavior byte-identical. When true, a transient/hard error
 * reading the sessions directory (EACCES, EIO, EMFILE — anything but a
 * legitimately absent directory) propagates instead of silently reading as
 * "zero sessions": a caller that opts into strict is one for whom "could not
 * tell if anyone else is live" must never look identical to "nobody else is
 * live".
 */
export function listSessionRecords(root, { strict = false } = {}) {
  let entries;
  try {
    entries = fs.readdirSync(sessionsDir(root));
  } catch (err) {
    if (strict) rethrowUnlessMissing(err);
    return [];
  }
  const sessions = [];
  for (const entry of entries) {
    if (!entry.endsWith('.json')) continue;
    const record = readSession(root, entry.slice(0, -'.json'.length), { strict });
    if (record) sessions.push(record);
  }
  return sessions;
}

/**
 * rel180-2: the write is gated by the SAME 'sessions' store-lock
 * sweepExpiredClaims's heartbeat-staleness decision + reclaim now also holds
 * (acquireSessionsLock below) — closes the read-then-act race where a
 * renewal landing between the sweeper's staleness read and its reclaim was
 * invisible to that decision. Bounded, never an unbounded wait (matches the
 * per-claim gate's posture): a typed LOCK_BUSY failure on exhaustion rather
 * than a silent no-op — silently reporting ok:true without actually writing
 * would let a caller believe the heartbeat renewed when the on-disk record
 * never changed, which is exactly the inconsistency this fix exists to rule
 * out. `lockAttempts` lets a hook-driven caller (heartbeatTouch, Δ3: "hooks
 * never WAIT on a store lock") opt into a single non-retrying attempt; every
 * other caller gets the same bounded-retry budget as acquireGateWithRetry.
 */
export function heartbeatSession(root, sessionId, { now = Date.now(), lockAttempts = SESSIONS_LOCK_RETRY_ATTEMPTS, _raceSeam } = {}) {
  const lock = acquireSessionsLock(root, lockAttempts);
  if (!lock.acquired) {
    return fail(
      'LOCK_BUSY',
      `session "${sessionId}" heartbeat could not acquire the sessions lock after ${lockAttempts} bounded ${lockAttempts === 1 ? 'attempt' : 'attempts'} — never waited unboundedly.`,
    );
  }
  try {
    // D10a: the read moved INSIDE the lock (was: readSession before
    // acquireSessionsLock, above the fold) — a bind/unbind landing between
    // an unlocked read and this write is exactly the lost-update shape D10a
    // closes; see the rel180-2/D10a comment block above this function.
    const session = readSession(root, sessionId);
    if (!session) {
      return fail('SESSION_MISSING', `session "${sessionId}" has no record to heartbeat.`);
    }
    // _raceSeam: test-only synchronous hook, fired here (post-read,
    // pre-write, still holding the lock) so a regression test can prove the
    // window is closed without sleeps or real threads — same seam style as
    // lock.mjs's _takeoverSeam/_postRenameSeam. Never set by any production
    // caller; undefined is a complete no-op.
    if (typeof _raceSeam === 'function') _raceSeam();
    session.last_heartbeat = utcNow(now);
    withTransientFsRetry(() => writeJsonAtomic(sessionPath(root, sessionId), session));
    return { ok: true, session };
  } finally {
    lock.release();
  }
}

/** A session is stale when its heartbeat is older than staleSeconds — or unreadable/missing. */
export function heartbeatStale(session, nowMs = Date.now(), staleSeconds = DEFAULT_HEARTBEAT_STALE_SECONDS) {
  if (!session || typeof session !== 'object') return true;
  const beatMs = Date.parse(session.last_heartbeat);
  if (!Number.isFinite(beatMs)) return true;
  return beatMs + staleSeconds * 1000 <= nowMs;
}

/**
 * hardening-4a — "concurrent mode" is true when at least one OTHER session
 * record exists with a LIVE (non-stale) heartbeat, reusing the exact msh-5
 * staleness window heartbeatStale already applies everywhere else
 * (DEFAULT_HEARTBEAT_STALE_SECONDS, overridable only for tests). Backs the
 * sessionless-claim/reserve refusal below: a genuinely solo session (nobody
 * else's heartbeat is live) keeps today's sessionless behavior byte-
 * unchanged; only once a second live session shows up does an unidentified
 * caller get asked to say who it is. `excludeSessionId` lets a caller that
 * DOES hold a real session id exclude its own record — its own liveness is
 * never "another" session; a caller with no session id at all simply asks
 * "is anyone else out there right now."
 */
export function isConcurrentMode(root, { excludeSessionId = null, now = Date.now(), staleSeconds = DEFAULT_HEARTBEAT_STALE_SECONDS, strict = false } = {}) {
  const exclude = typeof excludeSessionId === 'string' ? excludeSessionId.trim() : '';
  return listSessionRecords(root, { strict }).some(
    (session) => session.id !== exclude && !heartbeatStale(session, now, staleSeconds),
  );
}

/**
 * activeWorkers(root, {excludeSessionId}) — D6 derived view (CONTEXT.md:
 * "Workers derived, never stored"; multisession-native-8). "Active workers"
 * are live-heartbeat sessions (same heartbeatStale staleness window every
 * other liveness check here uses) joined with (a) their lane binding —
 * session.lane, the ONLY session->feature binding that exists today; the
 * workflow_id binding CONTEXT.md D6 also names is msn-10's job, not yet on
 * any session record — and (b) their current cell claim, read the same way
 * state.mjs's listClaimHoldsForStart enumerates .bee/claims/*.json, but
 * keyed by session rather than by cell.
 *
 * `excludeSessionId` reuses isConcurrentMode's exact pattern (hardening-4a):
 * a caller holding a real session id excludes its OWN record from the view
 * — its own liveness is never "another" active worker. This is what makes
 * C3 possible: startFeature's precondition passes its own sessionId here so
 * a solo starter is never blocked by its own heartbeat.
 *
 * Returns one row per live, non-excluded session — the exact replacement
 * shape for the old hand-mutated state.workers array, but sourced from
 * ground truth instead of whatever a worker verb last wrote:
 *   { session_id, lane, cell, last_heartbeat }
 * `lane` is null while unbound; `cell` is null when the session holds no
 * currently-active claim (a live session between claims still counts as an
 * active worker, same as the old array could carry a worker with no cell in
 * flight). Never throws — a missing .bee/sessions or .bee/claims directory
 * reads as zero rows, same fail-open posture as listSessionRecords.
 */
export function activeWorkers(root, { excludeSessionId = null, now = Date.now(), staleSeconds = DEFAULT_HEARTBEAT_STALE_SECONDS } = {}) {
  const exclude = typeof excludeSessionId === 'string' ? excludeSessionId.trim() : '';
  const liveSessions = listSessionRecords(root).filter(
    (session) => session.id !== exclude && !heartbeatStale(session, now, staleSeconds),
  );
  if (liveSessions.length === 0) return [];

  let entries;
  try {
    entries = fs.readdirSync(claimsDir(root));
  } catch {
    entries = [];
  }
  const claimCellBySession = new Map();
  for (const entry of entries) {
    if (!entry.endsWith('.json')) continue;
    let claim;
    try {
      claim = readClaim(root, entry.slice(0, -'.json'.length));
    } catch {
      continue; // a filename that is not a plain cell id is no claim of ours
    }
    if (!claim || !claim.session || !isClaimActive(claim, now)) continue;
    // One row per worker (same shape as the old array's one-cell-per-nickname
    // convention): first active claim seen for a session wins.
    if (!claimCellBySession.has(claim.session)) claimCellBySession.set(claim.session, claim.cell);
  }

  return liveSessions.map((session) => ({
    session_id: session.id,
    lane: typeof session.lane === 'string' && session.lane ? session.lane : null,
    cell: claimCellBySession.get(session.id) ?? null,
    last_heartbeat: session.last_heartbeat,
  }));
}

// ─── session→lane binding (fresh-session-handoff fsh-3) ─────────────────────
// The lane field is OPTIONAL and OMITTED while unbound: createSession never
// writes it, so pre-existing session-record consumers see exactly the shape
// they always did. Binding does NOT verify the lane record exists — claims.mjs
// stays lane-agnostic (the same decoupling as sessionId-as-parameter), and
// state.mjs resolvePipeline owns the typed LANE_MISSING/LANE_CORRUPT refusal.

// D10a: bad-argument validation (requireId) still runs BEFORE the lock is
// touched at all — a malformed id throws exactly as before, with no lock
// acquisition attempted for garbage input (unchanged from pre-D10a).
export function bindSessionLane(root, sessionId, feature, { lockAttempts = SESSIONS_LOCK_RETRY_ATTEMPTS } = {}) {
  const session = requireId(sessionId, 'session id');
  const lane = requireId(feature, 'lane feature');
  const lock = acquireSessionsLock(root, lockAttempts);
  if (!lock.acquired) {
    return fail(
      'LOCK_BUSY',
      `session "${session}" bind to lane "${lane}" could not acquire the sessions lock after ${lockAttempts} bounded ${lockAttempts === 1 ? 'attempt' : 'attempts'} — never waited unboundedly.`,
    );
  }
  try {
    const record = readSession(root, session);
    if (!record) {
      return fail('SESSION_MISSING', `session "${session}" has no record to bind to lane "${lane}".`);
    }
    const bound = { ...record, lane };
    withTransientFsRetry(() => writeJsonAtomic(sessionPath(root, session), bound));
    return { ok: true, session: bound };
  } finally {
    lock.release();
  }
}

/** Remove the binding by OMITTING the key (never lane:null), restoring the unbound shape. */
export function unbindSessionLane(root, sessionId, { lockAttempts = SESSIONS_LOCK_RETRY_ATTEMPTS } = {}) {
  const session = requireId(sessionId, 'session id');
  const lock = acquireSessionsLock(root, lockAttempts);
  if (!lock.acquired) {
    return fail(
      'LOCK_BUSY',
      `session "${session}" unbind could not acquire the sessions lock after ${lockAttempts} bounded ${lockAttempts === 1 ? 'attempt' : 'attempts'} — never waited unboundedly.`,
    );
  }
  try {
    const record = readSession(root, session);
    if (!record) {
      return fail('SESSION_MISSING', `session "${session}" has no record to unbind.`);
    }
    const { lane: _lane, ...unbound } = record;
    withTransientFsRetry(() => writeJsonAtomic(sessionPath(root, session), unbound));
    return { ok: true, session: unbound };
  } finally {
    lock.release();
  }
}

// ─── claims ──────────────────────────────────────────────────────────────────

export function readClaim(root, cellId) {
  const claim = readJson(claimPath(root, cellId), null);
  if (!claim || typeof claim !== 'object') return null;
  return claim;
}

/** TTL semantics mirror reservations.mjs: non-positive/invalid TTL never expires. */
function isClaimExpired(claim, nowMs) {
  const ttl = claim.ttl_seconds;
  if (!Number.isFinite(ttl) || ttl <= 0) return false;
  const claimedMs = Date.parse(claim.claimed_at);
  if (!Number.isFinite(claimedMs)) return false;
  return claimedMs + ttl * 1000 <= nowMs;
}

export function isClaimActive(claim, nowMs = Date.now()) {
  if (!claim || typeof claim !== 'object') return false;
  return !isClaimExpired(claim, nowMs);
}

// Exported for cells.mjs's D4 ownership guard (msh-4): reused verbatim so the
// mutator refusal names owner + expiry in the exact same words claimCellFile
// itself uses below — composing an existing reader, never a new one.
export function claimExpiry(claim) {
  const claimedMs = Date.parse(claim?.claimed_at);
  const ttl = claim?.ttl_seconds;
  if (!Number.isFinite(claimedMs) || !Number.isFinite(ttl) || ttl <= 0) return 'no expiry';
  return `expires ${utcNow(claimedMs + ttl * 1000)}`;
}

function acquireGate(root, cellId, nowMs) {
  try {
    fs.writeFileSync(
      claimGatePath(root, cellId),
      `${JSON.stringify({ pid: process.pid, at: utcNow(nowMs) })}\n`,
      { encoding: 'utf8', flag: 'wx' },
    );
    return true;
  } catch (error) {
    if (error && error.code === 'EEXIST') return false;
    throw error;
  }
}

function releaseGate(root, cellId) {
  // The gate file's own lifetime is sub-millisecond and single-owner (created
  // 'wx' by the same thread that removes it; a losing acquireGate() only ever
  // sees EEXIST on create, never opens the file), so this is lower-risk than
  // the claim/session rmSync/renameSync call sites below — wrapped anyway for
  // uniform coverage of every mutating fs call in this module (rel1710rc-5).
  withTransientFsRetry(() => fs.rmSync(claimGatePath(root, cellId), { force: true }));
}

// hardening-1-7-10 (rel1710rc-3): the gate above is only ever held for the
// handful of synchronous statements inside ONE releaseClaim/adopt/sweep
// critical section — never for an unbounded external operation. Under real
// cross-process contention (observed: scripts/test_claim_race.mjs scenario
// (d), budget-exhausted racers on a 2-core CI runner) one racer's own
// releaseClaim can rmSync the claim FILE (a separate file from the gate) a
// beat before it reaches its own releaseGate; a second racer can win a
// brand-new claimCellFile in that exact window and then call releaseClaim
// itself, colliding on the still-held gate. Before this fix, a single-shot
// GATE_HELD refusal there permanently orphaned that second racer's own claim
// file — nothing else ever revisits an orphaned claim. Because the gate's
// true hold time is always sub-millisecond-to-low-tens-of-milliseconds
// (never a long op, unlike lock.mjs's live-holder case), a short BOUNDED
// retry closes this window without reintroducing an unbounded wait: it only
// ever waits out the tail end of ANOTHER release's own critical section, up
// to GATE_RETRY_ATTEMPTS * GATE_RETRY_DELAY_MS (~300ms worst case) before
// still returning the typed GATE_HELD refusal exactly as before.
const GATE_RETRY_ATTEMPTS = 15;
const GATE_RETRY_DELAY_MS = 20;

// rel180-2 — the 'sessions' store-lock name + bounded-retry budget shared by
// heartbeatSession's write and sweepExpiredClaims's heartbeat re-check +
// reclaim (acquireSessionsLock below). Same worst-case budget as the
// per-claim gate retry above (~300ms) — deliberately generous relative to
// the sub-millisecond critical sections on both sides.
const SESSIONS_LOCK_NAME = 'sessions';
const SESSIONS_LOCK_RETRY_ATTEMPTS = 15;
const SESSIONS_LOCK_RETRY_DELAY_MS = 20;

function sleepSyncMs(ms) {
  Atomics.wait(new Int32Array(new SharedArrayBuffer(4)), 0, 0, ms);
}

function acquireGateWithRetry(root, cellId) {
  for (let attempt = 0; attempt < GATE_RETRY_ATTEMPTS; attempt++) {
    if (acquireGate(root, cellId, Date.now())) return true;
    if (attempt + 1 < GATE_RETRY_ATTEMPTS) sleepSyncMs(GATE_RETRY_DELAY_MS);
  }
  return false;
}

// rel180-2 — the SESSION-scoped counterpart to the per-claim gate above.
// Same lock NAME heartbeatTouch already used ('sessions'), same bounded-
// retry SHAPE as acquireGateWithRetry (worst case ~300ms, never unbounded),
// but reused via lock.mjs's acquireStoreLockOnceSync — the existing sync
// primitive for callers that must stay sync end-to-end (precedent:
// cells.mjs's writeCell) — rather than the per-claim gate's own bespoke
// 'wx'-file mechanism, since this needs to be held by TWO independent call
// sites (heartbeatSession's write, sweepExpiredClaims's heartbeat re-check +
// reclaim) that never nest inside each other's per-claim gate. Whichever
// side acquires first finishes its whole critical section before the other
// can proceed, so a renewal can never land observably between a sweep's
// staleness decision and its reclaim (nor a sweep's decision straddle a
// renewal that is already in flight). No new coordination mechanism: this
// composes two primitives (acquireStoreLockOnceSync + sleepSyncMs) that
// already exist in this file/module for exactly this shape of problem.
function acquireSessionsLock(root, attempts = SESSIONS_LOCK_RETRY_ATTEMPTS) {
  for (let attempt = 0; attempt < attempts; attempt++) {
    const result = acquireStoreLockOnceSync(root, SESSIONS_LOCK_NAME);
    if (result.acquired) return result;
    if (attempt + 1 < attempts) sleepSyncMs(SESSIONS_LOCK_RETRY_DELAY_MS);
  }
  return { acquired: false };
}

// rel1710rc-5 (Windows CI hazard fix): renameSync-onto-existing (the last
// step of fsutil.mjs's writeJsonAtomic) and rmSync-of-a-file can both
// intermittently fail with EBUSY/EPERM/ENOTEMPTY/EMFILE/ENFILE on Windows
// when another thread holds even a brief open handle on that exact path —
// POSIX permits renaming/unlinking a file another process has open outright;
// Windows generally does not. This module's own concurrent race fixtures
// (race_claims_child.mjs's sweep-heartbeat scenario) hammer readSession
// (fs.readFileSync) from several sweeper threads while heartbeatSession
// (writeJsonAtomic -> renameSync) rewrites the SAME session file every ~60ms
// for 500ms straight — real production usage (heartbeatTouch, sweepers) has
// the identical shape, just at lower frequency. fs.rmSync DOES have a
// built-in `maxRetries`/`retryDelay` backoff for exactly this class of
// transient error, but per Node's own docs that option "is ignored if the
// `recursive` option is not true" — every rmSync call in this file targets a
// single file (never recursive), so the built-in backoff never engages here.
// This local bounded retry (same shape/budget as acquireGateWithRetry above:
// worst case ~300ms, never unbounded) plugs that gap the same way the gate
// retry above plugs its own — the losing side of this race is provably
// transient (the other thread's own read/rename is sub-millisecond-to-
// low-tens-of-ms), never a genuine failure to retry away. A non-transient
// error (wrong code, or retries exhausted) still throws, unchanged from
// before this cell.
const TRANSIENT_FS_RETRY_ATTEMPTS = 15;
const TRANSIENT_FS_RETRY_DELAY_MS = 20;
const TRANSIENT_FS_ERROR_CODES = new Set(['EBUSY', 'EPERM', 'ENOTEMPTY', 'EMFILE', 'ENFILE']);

function withTransientFsRetry(fn) {
  for (let attempt = 0; attempt < TRANSIENT_FS_RETRY_ATTEMPTS; attempt++) {
    try {
      return fn();
    } catch (error) {
      const transient = error && TRANSIENT_FS_ERROR_CODES.has(error.code);
      if (!transient || attempt + 1 >= TRANSIENT_FS_RETRY_ATTEMPTS) throw error;
      sleepSyncMs(TRANSIENT_FS_RETRY_DELAY_MS);
    }
  }
}

/**
 * One winner per cell via exclusive create. Losing the race — including to a
 * TTL-expired claim, which only sweepExpiredClaims may reclaim — returns the
 * typed CLAIMED failure naming the holder and expiry.
 *
 * D1 (Δ2-amended): sessionId's requirement is deliberately relaxed — null or
 * undefined is a legal SESSIONLESS claim (single-user use with no
 * CLAUDE_CODE_SESSION_ID and no explicit --session-id): the claim file still
 * gets created (the O_EXCL race-serialization still applies), it simply omits
 * the `session` key entirely rather than writing a placeholder value. A
 * non-null sessionId is still validated the same as before (a plain id, no
 * path separators).
 *
 * hardening-4a: the D1 sessionless relaxation above is single-session-only
 * in spirit. Once isConcurrentMode(root) sees another session's heartbeat
 * live, an unidentified caller (session resolves null) is refused with a
 * typed SESSION_REQUIRED error naming both ways to identify itself
 * (--session-id flag, BEE_SESSION_ID env) BEFORE any claim file is written —
 * a solo caller (nobody else live) is completely unaffected, byte-identical
 * to before this cell.
 *
 * hardening-1-7-10 D5 (Codex session bridge): before refusing, the concurrent
 * case first tries resolveSessionId's durable fallback (this file, above) —
 * exactly ONE fresh live session record anywhere in the checkout means the
 * caller almost certainly IS that session (it just has no env var identifying
 * it), so the claim is auto-adopted under that session's identity instead of
 * refused; the result and the on-disk claim both carry `adopted: true` so
 * callers can audit that the identity was inferred. Two or more fresh live
 * sessions is real ambiguity and still refuses SESSION_REQUIRED exactly as
 * before — this durable fallback never guesses among multiple candidates.
 */
export function claimCellFile(root, sessionId, cellId, ttl = DEFAULT_CLAIM_TTL_SECONDS, { now = Date.now() } = {}) {
  const explicitSession = sessionId == null ? null : requireId(sessionId, 'session id');
  const cell = requireId(cellId, 'cell id');
  let session = explicitSession;
  let adopted = false;
  if (session == null) {
    const audit = {};
    const candidate = resolveSessionId({ root, audit });
    if (candidate && audit.adopted) {
      session = candidate;
      adopted = true;
    } else if (isConcurrentMode(root)) {
      return fail(
        'SESSION_REQUIRED',
        `cell "${cell}" cannot be claimed without identifying the acting session while another session is active — pass --session-id or set BEE_SESSION_ID (CLAUDE_CODE_SESSION_ID is also honored).`,
      );
    }
  }
  ensureDir(claimsDir(root));
  // msn-19 (CONTEXT.md D2/D3): workspace_id is NEVER a caller-supplied
  // argument here — it is auto-looked-up from the ACTING session's own
  // record (stamped there at createSession time, see that function's own
  // comment) and carried onto the claim, mirroring how `session` itself is
  // either explicit or resolved/adopted above. A sessionless claim, or a
  // session whose own record predates msn-19 (no workspace_id stamped),
  // simply omits the field — same "omit rather than write a placeholder"
  // convention as every other optional field in this module.
  let workspaceId;
  if (session) {
    const sessionRecord = readSession(root, session);
    if (sessionRecord && typeof sessionRecord.workspace_id === 'string' && sessionRecord.workspace_id) {
      workspaceId = sessionRecord.workspace_id;
    }
  }
  const claim = {
    cell,
    ...(session ? { session } : {}),
    ...(workspaceId ? { workspace_id: workspaceId } : {}),
    ttl_seconds: Number.isFinite(ttl) && ttl > 0 ? Math.floor(ttl) : DEFAULT_CLAIM_TTL_SECONDS,
    claimed_at: utcNow(now),
    // GH #27.1 (D-GHF-B): stamped ONCE here, at claim creation — the
    // immutable acquisition identity. claimed_at is the mutable expiry
    // clock (renewClaimTTL legitimately rewrites it on every heartbeat);
    // acquired_at never changes for the life of this claim file, so
    // checkCellBudgets can key off it and stay heartbeat-invariant.
    acquired_at: utcNow(now),
    // multisession-native-12 (D4/D9 invariant 10, advisor consult slice 3
    // condition F): a CAS-style fencing token, deliberately named
    // `fence_epoch` rather than bare "epoch" — cells.mjs:2408's
    // checkCellBudgets already uses "epoch" loosely for a different,
    // unrelated concept (a claim/heartbeat acquisition cycle for
    // budget-collapse counting). Stamped 1 here at creation; adoptClaim
    // bumps it +1 on every ownership transfer; renewClaimTTL/releaseClaim
    // below refuse a write typed CLAIM_FENCE_STALE when a caller presents an
    // epoch behind the claim's current one. See adoptClaim's own comment for
    // the compat posture (presenting an epoch is opt-in until slice 4).
    fence_epoch: 1,
    // D5: OMITTED (never false) when the session was explicitly supplied —
    // only a durable-fallback adoption ever sets this, same "omit rather than
    // write a placeholder" convention as `session` itself above.
    ...(adopted ? { adopted: true } : {}),
  };
  try {
    fs.writeFileSync(claimPath(root, cell), `${JSON.stringify(claim, null, 2)}\n`, {
      encoding: 'utf8',
      flag: 'wx',
    });
  } catch (error) {
    if (error && error.code === 'EEXIST') {
      const holder = readClaim(root, cell);
      const owner = holder?.session ?? 'no session (sessionless claim)';
      return fail(
        'CLAIMED',
        `cell "${cell}" is already claimed by session "${owner}" (${claimExpiry(holder)}).`,
        { holder },
      );
    }
    throw error;
  }
  return { ok: true, claim, ...(adopted ? { adopted: true } : {}) };
}

/**
 * Transfer ownership IN PLACE under the exclusive gate: the claim file is
 * atomically rewritten, never deleted, so concurrent 'wx' claimers keep
 * getting EEXIST throughout the adoption. Never release-then-reclaim.
 *
 * multisession-native-12 (D4/D9 invariant 10): every adoption bumps
 * `fence_epoch` by exactly 1, in the SAME atomic write as the ownership
 * change — a stale holder that still presents the pre-adoption epoch to
 * renewClaimTTL/releaseClaim is refused typed CLAIM_FENCE_STALE (see those
 * functions below) even in the edge case where session identity alone would
 * not have caught it (e.g. a stale in-memory copy from before this exact
 * adoption). A legacy claim with no `fence_epoch` on disk (pre-dates this
 * cell) is treated as epoch 1, same default a fresh claimCellFile stamps.
 */
export function adoptClaim(root, cellId, newSessionId, { now = Date.now() } = {}) {
  const cell = requireId(cellId, 'cell id');
  const session = requireId(newSessionId, 'session id');
  ensureDir(claimsDir(root));
  if (!acquireGate(root, cell, now)) {
    return fail('GATE_HELD', `claim "${cell}" is gated by another in-flight adopt/sweep — retry later, never wait on the gate.`);
  }
  try {
    const claim = readClaim(root, cell);
    if (!claim) {
      return fail('NOT_FOUND', `cell "${cell}" has no claim to adopt.`);
    }
    const previous = claim.session;
    const previousEpoch = Number.isFinite(claim.fence_epoch) ? claim.fence_epoch : 1;
    const adopted = {
      ...claim,
      session,
      claimed_at: utcNow(now), // fresh ownership renews the TTL clock
      adopted_from: previous,
      adopted_at: utcNow(now),
      fence_epoch: previousEpoch + 1,
    };
    withTransientFsRetry(() => writeJsonAtomic(claimPath(root, cell), adopted));
    return { ok: true, claim: adopted, previous_owner: previous };
  } finally {
    releaseGate(root, cell);
  }
}

/**
 * Same-session-only TTL renewal (D5): refreshes `claimed_at` — the expiry
 * clock — for every claim file currently owned by sessionId, WITHOUT ever
 * calling adoptClaim and without touching adopted_from/adopted_at. Guarded
 * by the same exclusive per-claim gate as adopt/sweep/release (acquireGate/
 * releaseGate above): a claim whose gate is held by another in-flight
 * adopt/sweep is SKIPPED, never waited on — acquireGate is already a single
 * non-retrying 'wx' attempt, so this can never block. The session match is
 * RE-VERIFIED under the gate (never off the pre-gate listing snapshot), so a
 * claim adopted away between listing and gating is left untouched — a
 * renewal racing an adoption can never revert ownership.
 *
 * multisession-native-12 (D4/D9 invariant 10): `options.presentedEpoch` is
 * OPTIONAL and OFF by default — omitted (the shape every production caller
 * uses today, e.g. heartbeatTouch below), this function is byte-unchanged
 * from before this cell: fencing enforcement engages only when a caller
 * actually presents an epoch (full mandatory presentation arrives with
 * workspace identity in slice 4, per the advisor consult). When presented,
 * it is checked against EACH session-owned claim reached in this pass
 * (after the same gate + re-verified-ownership steps as always); a claim
 * whose current `fence_epoch` is AHEAD of the presented value means a
 * takeover moved ownership forward since the caller last knew about this
 * claim — this whole call refuses immediately, typed CLAIM_FENCE_STALE,
 * naming the stale cell, rather than silently completing a partial renewal
 * of other claims. (In real use a session renews at most the one claim it
 * is actively working, so this short-circuit and a true single-cell
 * semantics coincide.) The gate is always released via `finally` even on
 * this refusal — never leaked.
 */
export function renewClaimTTL(root, sessionId, { now = Date.now(), presentedEpoch } = {}) {
  const session = requireId(sessionId, 'session id');
  const fencing = presentedEpoch !== undefined;
  let entries;
  try {
    entries = fs.readdirSync(claimsDir(root));
  } catch {
    return { ok: true, renewed: [], skipped: [] };
  }
  const renewed = [];
  const skipped = [];
  for (const entry of entries) {
    if (!entry.endsWith('.json')) continue;
    const cell = entry.slice(0, -'.json'.length);
    const preview = readClaim(root, cell);
    if (!preview || preview.session !== session) continue; // not ours (or sessionless): never touched
    if (!acquireGate(root, cell, now)) {
      skipped.push(cell);
      continue;
    }
    try {
      const claim = readClaim(root, cell); // re-verify ownership under the gate
      if (claim && claim.session === session) {
        if (fencing) {
          const currentEpoch = Number.isFinite(claim.fence_epoch) ? claim.fence_epoch : 1;
          if (!Number.isFinite(presentedEpoch) || presentedEpoch < currentEpoch) {
            return fail(
              'CLAIM_FENCE_STALE',
              `cell "${cell}" renew refused: presented epoch ${JSON.stringify(presentedEpoch)} is behind current fence_epoch ${currentEpoch} — a takeover already moved ownership forward; re-adopt before writing again.`,
              { cell, current_epoch: currentEpoch, presented_epoch: presentedEpoch },
            );
          }
        }
        // GH #27.1 (D-GHF-B): the `...claim` spread carries acquired_at
        // forward untouched — only claimed_at (the expiry clock) advances
        // on a heartbeat. Never add an explicit acquired_at key here.
        // Renewal never changes fence_epoch — only adoptClaim bumps it.
        withTransientFsRetry(() => writeJsonAtomic(claimPath(root, cell), { ...claim, claimed_at: utcNow(now) }));
        renewed.push(cell);
      }
    } finally {
      releaseGate(root, cell);
    }
  }
  return { ok: true, renewed, skipped };
}

/**
 * Owner-only removal, under the same exclusive gate as adopt/sweep. D1
 * (Δ2-amended): sessionId is nullable — null/undefined means "release the
 * sessionless claim", and matches only a claim record that itself carries no
 * `session` key (claim.session ?? null === null).
 *
 * multisession-native-12 (D4/D9 invariant 10): `options.presentedEpoch` is
 * OPTIONAL, checked AFTER the existing owner check above (a release must
 * already be from the current owner; epoch fencing is an additional,
 * orthogonal guard, never a substitute for it) — omitted, byte-unchanged
 * legacy behavior (every production caller today: cells.mjs's two
 * releaseClaim call sites). Presented and behind the claim's current
 * `fence_epoch`, the release refuses typed CLAIM_FENCE_STALE and the claim
 * file is left untouched.
 */
export function releaseClaim(root, sessionId, cellId, { presentedEpoch } = {}) {
  const session = sessionId == null ? null : requireId(sessionId, 'session id');
  const cell = requireId(cellId, 'cell id');
  if (!readClaim(root, cell)) {
    return fail('NOT_FOUND', `cell "${cell}" has no claim to release.`);
  }
  if (!acquireGateWithRetry(root, cell)) {
    return fail(
      'GATE_HELD',
      `claim "${cell}" is gated by another in-flight adopt/sweep/release after ${GATE_RETRY_ATTEMPTS} bounded retries — never waited unboundedly.`,
    );
  }
  try {
    const claim = readClaim(root, cell); // re-read under the gate
    if (!claim) {
      return fail('NOT_FOUND', `cell "${cell}" has no claim to release.`);
    }
    const owner = claim.session ?? null;
    if (owner !== session) {
      return fail(
        'NOT_OWNER',
        `cell "${cell}" is owned by session "${owner ?? 'none (sessionless)'}", not "${session ?? 'none (sessionless)'}".`,
      );
    }
    if (presentedEpoch !== undefined) {
      const currentEpoch = Number.isFinite(claim.fence_epoch) ? claim.fence_epoch : 1;
      if (!Number.isFinite(presentedEpoch) || presentedEpoch < currentEpoch) {
        return fail(
          'CLAIM_FENCE_STALE',
          `cell "${cell}" release refused: presented epoch ${JSON.stringify(presentedEpoch)} is behind current fence_epoch ${currentEpoch} — a takeover already moved ownership forward; re-adopt before writing again.`,
          { cell, current_epoch: currentEpoch, presented_epoch: presentedEpoch },
        );
      }
    }
    withTransientFsRetry(() => fs.rmSync(claimPath(root, cell), { force: true }));
    return { ok: true, released: claim };
  } finally {
    releaseGate(root, cell);
  }
}

/**
 * Unconditional claim-file removal for cell-mutator claim-clearing
 * transitions (cap/unclaim/block/drop/reopen — D1 Δ2-amendment): these verbs
 * already own the cell's lifecycle transition and, as of msh-2, do not yet
 * check claim ownership (that check is D4/msh-4's job) — so clearing here is
 * by cell id alone, regardless of which session (if any) holds the claim.
 * Without this, a same-session block -> reopen -> claim round-trip would
 * self-refuse CLAIMED for the claim's full TTL, since nothing ever removed
 * the file the first claim created. No-ops (ok:true, released:null) when
 * there is no claim file — most cells claimed before this change, or cells
 * never claimed at all, have none. Gated the same as adopt/sweep/release so
 * it can never race a concurrent adopt/sweep.
 */
export function clearClaim(root, cellId) {
  const cell = requireId(cellId, 'cell id');
  if (!readClaim(root, cell)) {
    return { ok: true, released: null };
  }
  if (!acquireGate(root, cell, Date.now())) {
    return fail('GATE_HELD', `claim "${cell}" is gated by another in-flight adopt/sweep — retry later, never wait on the gate.`);
  }
  try {
    const claim = readClaim(root, cell); // re-read under the gate
    if (!claim) {
      return { ok: true, released: null };
    }
    withTransientFsRetry(() => fs.rmSync(claimPath(root, cell), { force: true }));
    return { ok: true, released: claim };
  } finally {
    releaseGate(root, cell);
  }
}

// hardening-4b (sweep-reset): a minimal, SELF-CONTAINED read/write of a
// cell's on-disk shape (.bee/cells/<id>.json) for the reset below —
// deliberately NOT importing cells.mjs's readCell/writeCell: cells.mjs
// already imports sweepExpiredClaims FROM this module (see its own import
// comment), so importing back would cycle. This mirrors lock.mjs's own
// documented precedent (its envSessionId duplicates claims.mjs's
// resolveSessionId chain rather than import it) — a small, deliberate
// duplication to keep claims.mjs a dependency-light leaf module. Archived
// cells (moved under .bee/cells/archive/<feature>/<id>.json) are out of
// scope here on purpose: a cell can only be archived once terminal
// (capped/dropped), never while "claimed", so the active-path lookup below
// finding nothing is simply "no live claimed cell to reset" — never an error.
function cellFilePathForSweep(root, cellId) {
  return path.join(root, '.bee', 'cells', `${cellId}.json`);
}

function readCellForSweepReset(root, cellId) {
  return readJson(cellFilePathForSweep(root, cellId), null);
}

/**
 * Reclaim only what is provably abandoned: TTL expired AND owner heartbeat
 * stale (missing/corrupt session record counts as stale), both RE-VERIFIED
 * under the claim's exclusive gate (pattern 20260710 — never steal on a stall
 * signal alone). A held gate means another process is mid-adopt/sweep: skip,
 * never wait.
 *
 * rel180-2: the TTL half of that re-verification (claim.claimed_at) was
 * always fully protected — claimed_at is only ever rewritten under this same
 * per-claim gate (renewClaimTTL). The heartbeat half (session.last_heartbeat)
 * was not: heartbeatSession writes it with no coordination at all, so a
 * renewal landing strictly between this function's fresh session read and
 * its rmSync was invisible to the decision already made. Closed below by
 * acquiring the SAME session-scoped store-lock heartbeatSession itself now
 * holds (acquireSessionsLock) around the read + decide + reclaim as one
 * unit — see that function's comment for the full reasoning. A session that
 * can't be locked this pass is skipped, never stolen; the next sweep pass
 * re-evaluates it.
 *
 * hardening-4b (sweep-reset): once a claim file is actually removed, the
 * CELL it pointed at is very likely still sitting at status "claimed" —
 * before this, nothing ever flipped it back to "open", so a dead session's
 * claim would sweep its claims-store file but silently leave the cell
 * unclaimable-looking-claimed forever (claim-next's own selection only reads
 * "open" cells). The reset below closes that gap: under
 * withStoreLock(`cells:${id}`) — the SAME per-cell lock every other cells.mjs
 * mutator now uses — read the cell fresh, and only reset claimed -> open
 * when BOTH (a) its status is still "claimed" (never touch anything else)
 * and (b) its trace.claim_session matches the JUST-SWEPT claim's session
 * (claim.session ?? null) — i.e. nobody re-claimed it between the claim-file
 * removal above and this reset acquiring the lock. A mismatch means a fresh
 * claim already won the cell; the reset is skipped, never overwriting a live
 * claimant. One audit decision line is logged per actual reset (best-effort:
 * a decision-log failure must never abort an already-decided reset, since
 * the cell write below is the thing that actually matters).
 */
export async function sweepExpiredClaims(
  root,
  { now = Date.now(), staleSeconds = DEFAULT_HEARTBEAT_STALE_SECONDS, _raceSeam } = {},
) {
  const dir = claimsDir(root);
  let entries;
  try {
    entries = fs.readdirSync(dir);
  } catch {
    return { ok: true, swept: [], skipped: [] };
  }
  const swept = [];
  const skipped = [];
  const resetCells = [];
  for (const entry of entries) {
    if (!entry.endsWith('.json')) continue;
    const cell = entry.slice(0, -'.json'.length);
    const preview = readClaim(root, cell);
    if (!preview) continue; // unreadable/corrupt: refuse to touch, never clobber
    if (!isClaimExpired(preview, now)) continue;
    if (!heartbeatStale(readSession(root, preview.session), now, staleSeconds)) continue;
    if (!acquireGate(root, cell, now)) {
      skipped.push(cell);
      continue;
    }
    let sweptClaim = null;
    try {
      const claim = readClaim(root, cell); // re-verify everything under the gate
      if (claim && isClaimExpired(claim, now)) {
        const ownerSession = claim.session ?? null;
        // rel180-2: a sessionless claim has no heartbeat to race against and
        // skips the lock entirely (own-session claims are the only ones
        // heartbeatSession ever writes to).
        const sessionsLock = ownerSession ? acquireSessionsLock(root) : { acquired: true, release() {} };
        if (sessionsLock.acquired) {
          try {
            if (heartbeatStale(readSession(root, ownerSession), now, staleSeconds)) {
              // TEST SEAM (rel180-2, deterministic race proof only — never
              // set by any production caller, undefined is a no-op): fires
              // while still holding the sessions lock, exactly where a
              // concurrent renewal would need to land to reproduce the
              // pre-fix bug — see test_claims.mjs's forced-interleaving row.
              if (typeof _raceSeam === 'function') await _raceSeam({ root, cell, session: ownerSession });
              withTransientFsRetry(() => fs.rmSync(claimPath(root, cell), { force: true }));
              swept.push(cell);
              sweptClaim = claim;
            }
          } finally {
            sessionsLock.release();
          }
        } else {
          // Couldn't get exclusivity with a concurrent renewal this pass —
          // never steal on contention; the next sweep pass re-evaluates.
          skipped.push(cell);
        }
      }
    } finally {
      releaseGate(root, cell);
    }
    if (!sweptClaim) continue;
    const sweptSession = sweptClaim.session ?? null;
    const wasReset = await withStoreLock(root, `cells:${cell}`, () => {
      const cellRecord = readCellForSweepReset(root, cell);
      if (!cellRecord || cellRecord.status !== 'claimed') return false;
      const currentSession = (cellRecord.trace && cellRecord.trace.claim_session) ?? null;
      if (currentSession !== sweptSession) return false; // a fresher claim already owns it — never overwrite
      cellRecord.status = 'open';
      cellRecord.trace = {
        ...(cellRecord.trace || {}),
        worker: null,
        claimed_at: null,
        claim_session: null,
        swept_at: new Date(now).toISOString(),
        swept_from_session: sweptSession,
      };
      withTransientFsRetry(() => writeJsonAtomic(cellFilePathForSweep(root, cell), cellRecord));
      return true;
    });
    if (wasReset) {
      resetCells.push(cell);
      try {
        logDecision(root, {
          decision: `«sweep: cell "${cell}" reset claimed -> open — swept session "${sweptSession ?? 'none (sessionless)'}"'s expired, stale claim»`,
          rationale:
            'sweepExpiredClaims (hardening-4b) removed the abandoned claim file; the cell was still "claimed" by that exact session (trace.claim_session matched), so it is returned to open rather than left claimed-but-unclaimable forever.',
          scope: 'repo',
          source: 'user',
          tags: ['claims', 'sweep'], // jrt-1: an internal caller with no tags throws typed DECISIONS_UNTAGGED_REFUSED once docs/decisions/taxonomy.json exists — census swept, see test_claims.mjs
        });
      } catch {
        // best-effort — the cell reset above already committed; a decision-log
        // failure must never be treated as the reset itself having failed.
      }
    }
  }
  return { ok: true, swept, skipped, reset: resetCells };
}

// ─── heartbeat + lease renewal touch (D5) ───────────────────────────────────

/**
 * D5 — throttled heartbeat + claim-lease renewal ridden by
 * bee-prompt-context.mjs (UserPromptSubmit) and bee-state-sync.mjs
 * (PostToolUse/Stop): a byte-identical no-op unless the session's STORED
 * heartbeat is older than HEARTBEAT_TOUCH_THROTTLE_SECONDS — at most one
 * refresh per session per throttle window, regardless of how many hook
 * events fire in between. When it does refresh: the session's own heartbeat
 * record, then every live claim file this session owns (renewClaimTTL,
 * same-session-only, gated by the existing per-claim .adopting exclusive
 * gate — never adoptClaim). Hold renewal (reservations.mjs's
 * renewHoldsBySession) is deliberately NOT called from here — claims.mjs
 * stays cycle-free w.r.t. reservations.mjs (see the top-of-file import
 * note); hook callers compose it alongside this call using the `touched`
 * flag this returns, exactly the way state.mjs composes both leaf modules
 * without either importing the other.
 *
 * Δ3-amended: hooks never WAIT on a store lock, so the session-heartbeat
 * write below passes lockAttempts: 1 — a LOCK_BUSY collision with a
 * concurrent CLI writer or an in-flight sweep is skipped silently (returned
 * in the result, never thrown); claim renewal is already non-waiting via the
 * per-claim gate, so it needs no separate lock mode. A non-lock error still
 * throws (fail-open is the HOOK's job — see bee-prompt-context.mjs /
 * bee-state-sync.mjs — not something this function fakes by swallowing every
 * error itself).
 *
 * rel180-2: heartbeatSession now owns the 'sessions' store-lock internally
 * (acquireSessionsLock) — this no longer wraps it in a SEPARATE
 * withStoreLock of the same name (nesting the same lock name from the same
 * process would just self-block on its own outer hold). lockAttempts: 1
 * below preserves the exact Δ3 "never wait" contract this always had.
 *
 * Δ6 documented non-goal: this renews blanket for the session regardless of
 * whether it is doing anything bee-relevant right now (a session idling in
 * unrelated chat still renews) — D4's audited force-ownership door and
 * release-on-terminal-transition are the rescue, not a narrower rule here.
 */
export async function heartbeatTouch(root, sessionId, { now = Date.now() } = {}) {
  const session = typeof sessionId === 'string' ? sessionId.trim() : '';
  if (!session) {
    return { ok: true, touched: false, reason: 'no-session' };
  }

  const record = readSession(root, session);
  if (!heartbeatStale(record, now, HEARTBEAT_TOUCH_THROTTLE_SECONDS)) {
    return { ok: true, touched: false, reason: 'throttled' };
  }

  const heartbeat = heartbeatSession(root, session, { now, lockAttempts: 1 });
  const claims = renewClaimTTL(root, session, { now });

  return { ok: true, touched: true, heartbeat, claims };
}
