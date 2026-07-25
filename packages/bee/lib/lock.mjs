// lock.mjs — withStoreLock(root, name, fn): a tiny cross-process mutual-
// exclusion primitive for bee's own store mutators (CONTEXT.md D2).
//
// Guards a single named critical section under .bee/locks/<name>.lock via an
// O_EXCL ('wx') lockfile: reservation/state logical read-check-write verbs
// run their body inside withStoreLock so two concurrent CLI invocations can
// no longer both pass a conflict check against the same snapshot and have
// the later write silently clobber the earlier one.
//
// hardening-1-7-10 (D2, amended by advisor consult): a live holder may
// LEGITIMATELY hold this lock across a long child spawn — e.g.
// worktree-store.mjs's mergeFeatureWorktree runs the host project's verify
// via a synchronous spawnSync WHILE holding 'worktree-admin', and that verify
// can genuinely run for minutes. Because spawnSync blocks the event loop for
// its own duration, a timer-based heartbeat cannot fire during exactly the
// long holds this needs to protect (locked: no heartbeat — see
// tryStaleTakeover below). So takeover is no longer age-alone: the 30s
// STALE_MS window only ever applies to a CRASHED holder (mtime stale AND the
// recorded owner pid is provably dead per `isPidAlive`). A pid that is
// provably alive is never stolen below the HARD_STALE_MS absolute ceiling —
// past that ceiling (or when liveness is unknowable), takeover proceeds
// regardless, as a pid-reuse guard of last resort.
//
// This module ships the primitive only — msh-1 wires no caller. msh-3/msh-5
// wrap reservations.mjs and state.mjs's logical-RMW verbs in it.

import fs from 'node:fs';
import path from 'node:path';
import crypto from 'node:crypto';
import { ensureDir } from './fsutil.mjs';

const RETRY_DELAY_MS = 50;
const MAX_ATTEMPTS = 100; // ~5s worst-case wait before a typed LOCK_BUSY refusal
const STALE_MS = 30_000; // crashed-holder window: only a candidate once BOTH stale-aged AND pid-dead
// Absolute ceiling: past this age, takeover proceeds regardless of the pid probe result — a
// pid-reuse/unknowable-liveness guard of last resort, set far above any real verify duration.
const HARD_STALE_MS = 3_600_000; // 1h

/** Typed refusal thrown by withStoreLock on timeout — never a silent fall-through. */
export class LockBusyError extends Error {
  constructor(name, holder) {
    const who =
      holder && typeof holder === 'object'
        ? `pid=${holder.pid ?? 'unknown'} session=${holder.session ?? 'unknown'} since ${holder.ts ?? 'unknown'}`
        : 'unknown holder';
    super(`lock "${name}" busy: held by ${who}`);
    this.name = 'LockBusyError';
    this.type = 'refused';
    this.reason = 'LOCK_BUSY';
    this.lockName = name;
    this.holder = holder ?? null;
  }
}

export function locksDir(root) {
  return path.join(root, '.bee', 'locks');
}

// hardening-4a: mirrors claims.mjs's resolveSessionId env-only chain
// (BEE_SESSION_ID wins over the legacy CLAUDE_CODE_SESSION_ID) WITHOUT
// importing claims.mjs — lock.mjs stays a dependency-light leaf module
// (claims.mjs itself imports withStoreLock from here, so importing back
// would cycle). claims.mjs's resolveSessionId is the CANONICAL
// implementation; this is a deliberate small duplicate for the lock-holder
// label only (never used to authorize anything) — keep the two in sync by
// hand if the chain ever changes.
function envSessionId(...values) {
  for (const value of values) {
    if (typeof value === 'string' && value.trim()) return value.trim();
  }
  return null;
}

// Windows-invalid filename characters (< > : " / \ | ? *) plus control chars.
// eslint-disable-next-line no-control-regex
const UNSAFE_LOCK_NAME_CHARS = /[<>:"/\\|?*\x00-\x1f]/g;

/**
 * Maps a logical lock name (e.g. "cells:some-id") to a filesystem-safe
 * basename. Runtime lock names contain ':' (cells.mjs's `cells:${id}`),
 * which Windows rejects in filenames — plain substitution alone risks two
 * DISTINCT logical names colliding after sanitization (e.g. "cells:a" and
 * "cells/a" both -> "cells_a"), so a short deterministic hash of the
 * ORIGINAL name is always appended: same logical name -> same file (pure
 * function, safe across processes), distinct logical names -> distinct
 * files, guaranteed rather than merely likely.
 */
function sanitizeLockName(name) {
  const raw = String(name);
  const sanitized = raw.replace(UNSAFE_LOCK_NAME_CHARS, '_');
  const hash = crypto.createHash('sha256').update(raw).digest('hex').slice(0, 8);
  return `${sanitized}-${hash}`;
}

export function lockFilePath(root, name) {
  return path.join(locksDir(root), `${sanitizeLockName(name)}.lock`);
}

function readHolder(lockPath) {
  try {
    return JSON.parse(fs.readFileSync(lockPath, 'utf8'));
  } catch {
    return null; // gone, unreadable, or mid-write elsewhere — treat as "no info", never throw
  }
}

function sleep(ms) {
  return new Promise((resolve) => setTimeout(resolve, ms));
}

// multisession-native-3 (C3, advisor consult stage 0-1): lock-contention
// telemetry — one JSON line per store-lock acquire OUTCOME (success with 0
// retries, retried success, or LOCK_BUSY exhaustion) appended to
// .bee/logs/contention.jsonl. Deliberately mirrors bee.mjs's OWN
// recordTiming append for .bee/logs/timings.jsonl byte-for-byte in spirit:
// plain fs.mkdirSync + fs.appendFileSync, wrapped in a try/catch that
// swallows EVERYTHING (including EBUSY) — a telemetry write must never
// break a lock acquisition or a hook heartbeat. This is called from INSIDE
// withStoreLock/acquireStoreLockOnceSync, so it is intentionally lock-free:
// routing it back through either of those primitives would recurse forever
// (this module's whole reason to exist is guarding critical sections, not
// being one). No fsync — best-effort, hot-path-safe, matching the same
// no-fsync posture as timings.jsonl's own append.
function appendContentionTelemetry(root, record) {
  try {
    const logsDir = path.join(root, '.bee', 'logs');
    fs.mkdirSync(logsDir, { recursive: true });
    fs.appendFileSync(path.join(logsDir, 'contention.jsonl'), `${JSON.stringify(record)}\n`);
  } catch {
    // fail-open: telemetry must never change a lock acquisition's outcome
  }
}

// Builds one contention.jsonl record. workflow_id/workspace_id/resource are
// not knowable at this layer yet (no caller currently threads them down to
// lock.mjs) and are always null — schema fields reserved for a future cell,
// per C3. holder_session is whatever contending holder session was
// observed while waiting (null when the acquire never contended at all).
function buildContentionRecord({ name, session, waitStartMs, holderSession, result }) {
  return {
    ts: new Date().toISOString(),
    lock_name: name,
    lock_wait_ms: Date.now() - waitStartMs,
    holder_session: holderSession ?? null,
    caller_session: session ?? null,
    workflow_id: null,
    workspace_id: null,
    resource: null,
    result,
  };
}

// rel180-4 (Windows CI hazard fix, same class as claims.mjs's rel1710rc-5):
// lock.mjs's own open ('wx' writeFileSync)/rename/unlink calls can
// intermittently fail with EBUSY/EPERM/ENOTEMPTY/EMFILE/ENFILE on Windows
// when another thread/process holds even a brief open handle on that exact
// path — POSIX permits renaming/unlinking a file another process has open
// outright; Windows generally does not. dp-3's new decisions store lock
// (which goes through THIS module) exposed the exact gap claims.mjs already
// closed for its own claim/session files (rel1710rc-5). lock.mjs stays a
// dependency-light leaf module (claims.mjs imports FROM here, so importing
// back would cycle — same precedent as envSessionId's duplicated chain
// above), so this is a deliberate small duplicate of claims.mjs's
// withTransientFsRetry: same shape, same budget — keep the two in sync by
// hand if the retry policy ever changes.
const TRANSIENT_FS_RETRY_ATTEMPTS = 15;
const TRANSIENT_FS_RETRY_DELAY_MS = 20;
const TRANSIENT_FS_ERROR_CODES = new Set(['EBUSY', 'EPERM', 'ENOTEMPTY', 'EMFILE', 'ENFILE']);

function sleepSyncMs(ms) {
  Atomics.wait(new Int32Array(new SharedArrayBuffer(4)), 0, 0, ms);
}

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
 * isPidAlive(pid) — synchronous liveness probe via the null-signal trick
 * (`process.kill(pid, 0)` never actually signals anything; it only reports
 * whether the kernel would let this process signal that pid). Same-host by
 * construction (locks live under the per-checkout `.bee/locks/`), so a pid
 * probe is meaningful here.
 *
 *   - missing/unparsable pid (not a positive integer)         -> dead
 *   - process.kill(pid, 0) succeeds (pid exists, ours or not)  -> alive
 *   - ESRCH (no such process)                                  -> dead
 *   - EPERM (pid exists, we just can't signal it)              -> alive
 *   - any other errno                                          -> alive
 *     (liveness genuinely unknowable; treated conservatively as alive so a
 *     live holder is never falsely stolen below HARD_STALE_MS — the ceiling
 *     is exactly the guard for this "unknowable" case)
 */
export function isPidAlive(pid) {
  const n = Number(pid);
  if (!Number.isInteger(n) || n <= 0) return false;
  try {
    process.kill(n, 0);
    return true;
  } catch (error) {
    if (error && error.code === 'ESRCH') return false;
    return true;
  }
}

function tryAcquire(lockPath, body) {
  try {
    withTransientFsRetry(() =>
      fs.writeFileSync(lockPath, `${JSON.stringify(body)}\n`, { encoding: 'utf8', flag: 'wx' }),
    );
    return true;
  } catch (error) {
    if (error && error.code === 'EEXIST') return false;
    throw error;
  }
}

// rel180-4 (fix-first, pre-existing mutual-exclusion violation, backlog P2):
// judges whether lockPath is CURRENTLY eligible for stale takeover (mtime +
// pid-liveness rule, D2/hardening-1-7-10 — unchanged from before this cell)
// and, when eligible, returns a SNAPSHOT of the holder body observed at this
// exact moment. That snapshot is the anchor performTakeoverClaim below
// re-verifies against BEFORE AND AFTER the rename (rel190-2 hardened the
// before-check — see that function's own comment for the full failure mode
// this closes). Returns null when not eligible (fresh/live holder, or the
// lock vanished — either way, normal retry, never an error).
function judgeStaleTakeoverEligibility(lockPath, nowMs) {
  let stat;
  try {
    stat = fs.statSync(lockPath);
  } catch {
    return null; // lock vanished between our EEXIST and this stat — normal retry
  }
  const ageMs = nowMs - stat.mtimeMs;
  if (ageMs <= STALE_MS) return null;
  const holderBefore = readHolder(lockPath);
  if (ageMs <= HARD_STALE_MS) {
    const pid = holderBefore && typeof holderBefore === 'object' ? holderBefore.pid : undefined;
    if (isPidAlive(pid)) return null; // live holder — legitimately long-running, never stolen
  }
  return { holderBefore };
}

// Attempt exactly one stale takeover of lockPath by ATOMIC RENAME. Returns
// true only when THIS call performed the rename AND the file it renamed away
// is STILL the exact holder body judgeStaleTakeoverEligibility judged stale
// (never an unconditional unlink, which lets a waiter delete a fresh
// holder's lock out from under it — spike negative control: naive unlink
// reproduced 7-8 simultaneous "winners").
//
// rel180-4 (fix-first, pre-existing mutual-exclusion violation, backlog P2):
// rename() only guarantees ONE racer consumes a given PATH at a time — it
// does NOT guarantee the CONTENT at that path is still what an earlier
// stat/readHolder observed. Under real cross-process contention, a racer
// that judged lockPath stale can be descheduled between that judgment and
// its own renameSync; in that window a FASTER racer can legitimately win the
// same original takeover, recreate a fresh lock, and even be mid-critical-
// section by the time the slower racer's renameSync finally runs. That
// renameSync still "succeeds" (rename doesn't care whose content is at the
// path) — pre-fix, the slower racer then unconditionally deleted the corpse
// and reported a win, so it and the fast racer's still-live holder both
// believed they held the lock at once (reproduced as the observed
// mutual-exclusion violation; the forced-interleaving test in
// scripts/test_store_lock.mjs reproduces this deterministically). The fix:
// after the rename, compare the renamed-away body against the snapshot
// judged stale (pid+token+ts — token is a fresh random value per
// acquisition, so any mismatch means the content changed underneath us). A
// match means it truly was the same stale lock — safe to delete, takeover
// wins. A mismatch means we accidentally displaced a DIFFERENT, fresher
// lock: put it back (unless something has since re-occupied lockPath — never
// clobber a newer legitimate acquisition that raced in during our own
// verification window) and report that we did NOT win. Two racers renaming
// the same source path is still safe even though each targets a different
// pid-unique destination: rename() consumes its source, so the loser's
// rename sees the source already gone and fails ENOENT — that is a normal
// loss, not an error, so it just backs off into the retry loop.
// Compares two holder snapshots by pid+token+ts — a fresh random token per
// acquisition means an exact match on all three is proof the content is the
// SAME acquisition instance, never merely "looks similar". Shared by the
// pre-rename re-verification and the post-rename verification below.
function sameHolderIdentity(a, b) {
  return Boolean(a && b && a.pid === b.pid && a.token === b.token && a.ts === b.ts);
}

// Attempt exactly one takeover rename of lockPath, moving WHATEVER currently
// occupies it to a fresh, pid+random-unique staging path. Returns
// { stalePath, holderAfter } on success (holderAfter is whatever content the
// rename actually captured — never assumed to still be holderBefore), or
// null if lockPath had already been consumed by another racer (ENOENT is a
// normal loss — rename() only guarantees ONE racer consumes a given PATH;
// the loser's rename sees the source already gone, a normal retry, never an
// error).
function renameForTakeover(lockPath, nowMs) {
  const stalePath = `${lockPath}.stale-${process.pid}-${nowMs}-${crypto.randomBytes(4).toString('hex')}`;
  try {
    withTransientFsRetry(() => fs.renameSync(lockPath, stalePath));
  } catch (error) {
    if (error && error.code === 'ENOENT') return null; // another racer already renamed it away
    throw error;
  }
  return { stalePath, holderAfter: readHolder(stalePath) };
}

// Given what a takeover rename actually captured, decide whether it was
// truly the lock judged stale (holderBefore) — deleting the corpse and
// reporting a win — or a MISMATCH, in which case put it back (unless a
// third racer has since legitimately re-occupied lockPath, which must never
// be clobbered) and report a loss. Shared by both the sync and async
// takeover paths below.
function settleTakeover(lockPath, stalePath, holderBefore, holderAfter) {
  if (sameHolderIdentity(holderAfter, holderBefore)) {
    withTransientFsRetry(() => fs.rmSync(stalePath, { force: true }));
    return true;
  }
  // We renamed away a DIFFERENT, fresher lock than the one judged stale —
  // put it back unless lockPath has since been re-occupied by yet another
  // legitimate acquisition (never clobber that); either way this call did
  // NOT win the takeover.
  let occupied = false;
  try {
    fs.statSync(lockPath);
    occupied = true;
  } catch {
    // not occupied — safe to restore
  }
  if (!occupied) {
    try {
      withTransientFsRetry(() => fs.renameSync(stalePath, lockPath));
      return false;
    } catch {
      // lost the restore race too (lockPath got recreated in the sliver
      // between our check and this rename) — fall through and drop the
      // corpse below rather than leave two files behind.
    }
  }
  withTransientFsRetry(() => fs.rmSync(stalePath, { force: true }));
  return false;
}

// rel180-4 (fix-first, pre-existing mutual-exclusion violation, backlog P2):
// rename() only guarantees ONE racer consumes a given PATH at a time — it
// does NOT guarantee the CONTENT at that path is still what an earlier
// stat/readHolder observed. Under real cross-process contention, a racer
// that judged lockPath stale can be descheduled between that judgment and
// its own renameSync; in that window a FASTER racer can legitimately win the
// same original takeover, recreate a fresh lock, and even be mid-critical-
// section by the time the slower racer's renameSync finally runs. That
// renameSync still "succeeds" (rename doesn't care whose content is at the
// path) — pre-fix, the slower racer then unconditionally deleted the corpse
// and reported a win, so it and the fast racer's still-live holder both
// believed they held the lock at once.
//
// rel190-2 (fix-first, class-level hardening — same bug shape resurfacing:
// docs/history/learnings/20260722-lock-takeover-verifies-what-it-deletes.md):
// verifying identity AFTER the rename (settleTakeover above) prevents
// PERMANENTLY losing a live holder's lock, but the rename still runs FIRST,
// unconditionally, against whatever currently occupies lockPath — even a
// lock that was refreshed and is ACTIVELY held by another racer's
// still-running critical section. For the instant between that rename and
// the mismatch decision, lockPath is genuinely EMPTY on disk: a completely
// unrelated THIRD racer's ordinary tryAcquire (no takeover involved) can win
// that vacancy and enter its own critical section while the original,
// still-active holder has no idea its lock ever moved — two simultaneous
// holders. Restoring the displaced lock afterward only prevents losing the
// FILE; it cannot retroactively undo a double-entry a third racer already
// committed during the gap. This is exactly the CI-observed scenario (f)
// failure (a live-but-past-ceiling holder's lock, mid-critical-section,
// briefly vacated by a slower racer's takeover attempt) — the forced
// two-racer test (g) never reproduces it because there is no third racer to
// exploit the vacancy; scripts/test_store_lock.mjs's (g2) scenario adds one.
function performTakeoverClaim(lockPath, nowMs, holderBefore) {
  if (!sameHolderIdentity(readHolder(lockPath), holderBefore)) {
    return false; // already changed since we judged it stale — never touch it
  }
  const renamed = renameForTakeover(lockPath, nowMs);
  if (!renamed) return false;
  return settleTakeover(lockPath, renamed.stalePath, holderBefore, renamed.holderAfter);
}

// Synchronous, no-seam entry point (acquireStoreLockOnceSync's sibling —
// must stay sync end-to-end, see that function's own doc comment).
function tryStaleTakeover(lockPath, nowMs) {
  const eligibility = judgeStaleTakeoverEligibility(lockPath, nowMs);
  if (!eligibility) return false;
  return performTakeoverClaim(lockPath, nowMs, eligibility.holderBefore);
}

// Async sibling used only by withStoreLock: identical logic to
// tryStaleTakeover above, but accepts TWO optional test-only async hooks —
// never set by any production caller, undefined is a no-op for both.
// `seam` (rel180-4) pauses AFTER judging eligibility and BEFORE the takeover
// rename — this is what lets scripts/test_store_lock.mjs's forced-
// interleaving test (g) hold one racer suspended at exactly the TOCTOU
// window settleTakeover's post-rename verification closes.
// `postRenameSeam` (rel190-2) pauses AFTER a successful rename and BEFORE
// the mismatch decision — this is what lets (g2) force a third racer into
// the vacancy window deterministically instead of by load.
async function tryStaleTakeoverAsync(lockPath, nowMs, seam, postRenameSeam) {
  const eligibility = judgeStaleTakeoverEligibility(lockPath, nowMs);
  if (!eligibility) return false;
  if (typeof seam === 'function') await seam();
  if (!sameHolderIdentity(readHolder(lockPath), eligibility.holderBefore)) {
    return false; // already changed since we judged it stale — never touch it
  }
  const renamed = renameForTakeover(lockPath, nowMs);
  if (!renamed) return false;
  if (typeof postRenameSeam === 'function') await postRenameSeam();
  return settleTakeover(lockPath, renamed.stalePath, eligibility.holderBefore, renamed.holderAfter);
}

/**
 * withStoreLock(root, name, fn, options) — run fn() with .bee/locks/<name>.lock
 * held exclusively across processes. fn's return value/throw propagates
 * unchanged. Always releases in `finally`, and release only ever removes a
 * lock THIS acquisition created (matched by pid + a per-call token) — never
 * someone else's, including one that took over after this call's own lock
 * somehow went stale (should never happen for a millisecond-scale section,
 * but the token match makes it structurally impossible to unlink the wrong
 * lock either way).
 *
 * Session id is self-derived (BEE_SESSION_ID, falling back to the legacy
 * CLAUDE_CODE_SESSION_ID), never a parameter — matching D3's "never handed
 * down" posture even though full session-id resolution (explicit flag ->
 * env -> hook payload) is msh-2's helper (claims.mjs resolveSessionId).
 *
 * options.maxAttempts (default MAX_ATTEMPTS, ~100) lets a caller opt into a
 * SINGLE attempt (msh-5, D5 Δ3-amended: "hooks never WAIT on the lock" —
 * every store write on the hook-driven heartbeat/lease-renewal touch path
 * passes {maxAttempts: 1} here instead of the CLI's normal ~5s retry
 * budget). The retry/backoff SHAPE is otherwise byte-identical to before —
 * a caller that omits options gets exactly the original ~100-try, ~5s-worst-
 * case wait. The one deliberate behavior tweak (bug fix, not a race risk):
 * the inter-attempt sleep only runs when another attempt will follow, so the
 * final failing attempt no longer wastes one extra RETRY_DELAY_MS before
 * throwing — shaving ~50ms off the existing timeout path, never adding any.
 *
 * Timeout after ~maxAttempts * retryDelayMs throws LockBusyError naming the
 * current holder parsed from the lock body — never a fall-through unlocked
 * write.
 */
/**
 * acquireStoreLockOnceSync(root, name) — the SYNCHRONOUS, single-attempt
 * sibling of withStoreLock, for callers that must stay sync end-to-end
 * (hardening-1-7-10 D4: writeCell — the single cell-write funnel called
 * synchronously from addCells' `.map()` at cells.mjs — cannot become async
 * without cascading `await` through every caller up to that call site).
 *
 * Applies the SAME stale-takeover rule as withStoreLock's retry loop
 * (tryStaleTakeover: mtime > STALE_MS AND owner pid dead, or past the
 * HARD_STALE_MS absolute ceiling regardless of liveness) but with NO retry
 * loop and NO sleep: exactly one acquire attempt, and — only if that first
 * attempt found the lock stale-eligible and won the takeover race — exactly
 * one follow-up acquire attempt. Anything else (a live holder, or losing the
 * takeover race to another racer) is reported back as `{ acquired: false }`
 * rather than waited out; the caller decides how to surface that (cells.mjs
 * throws a typed CELLS_ARCHIVE_BUSY).
 *
 * Returns `{ acquired: true, release }` on success. `release()` is
 * idempotent and safe to call from a `finally`; it removes the lock file
 * only if it still matches THIS acquisition's pid + token (same anti-
 * clobber discipline as withStoreLock's own finally block below — a caller
 * can never unlink a lock some other holder has since taken over).
 * Returns `{ acquired: false, holder }` on contention, `holder` being
 * whatever readHolder could parse from the lock file (possibly null).
 */
export function acquireStoreLockOnceSync(root, name) {
  ensureDir(locksDir(root));
  const lockPath = lockFilePath(root, name);
  const token = crypto.randomBytes(8).toString('hex');
  const session = envSessionId(process.env.BEE_SESSION_ID, process.env.CLAUDE_CODE_SESSION_ID);
  const nowMs = Date.now();
  const waitStartMs = nowMs;
  const body = { pid: process.pid, session, ts: new Date(nowMs).toISOString(), token };

  let acquired = tryAcquire(lockPath, body);
  let contendedHolderSession = null;
  if (!acquired) {
    const contending = readHolder(lockPath);
    if (contending && typeof contending === 'object' && contending.session) {
      contendedHolderSession = contending.session;
    }
    if (tryStaleTakeover(lockPath, nowMs)) {
      acquired = tryAcquire(lockPath, { ...body, ts: new Date(Date.now()).toISOString() });
    }
  }
  if (!acquired) {
    const holder = readHolder(lockPath);
    appendContentionTelemetry(
      root,
      buildContentionRecord({
        name,
        session,
        waitStartMs,
        holderSession: (holder && holder.session) || contendedHolderSession,
        result: 'busy',
      }),
    );
    return { acquired: false, holder };
  }
  appendContentionTelemetry(
    root,
    buildContentionRecord({ name, session, waitStartMs, holderSession: contendedHolderSession, result: 'acquired' }),
  );
  let released = false;
  return {
    acquired: true,
    release: () => {
      if (released) return;
      released = true;
      const holder = readHolder(lockPath);
      if (holder && holder.token === token && holder.pid === process.pid) {
        withTransientFsRetry(() => fs.rmSync(lockPath, { force: true }));
      }
    },
  };
}

// options._takeoverSeam / options._postRenameSeam (rel180-4 / rel190-2):
// test-only async hooks forwarded to tryStaleTakeoverAsync — see that
// function's doc comment. Never set by any production caller; undefined is
// a complete no-op for both.
export async function withStoreLock(
  root,
  name,
  fn,
  { maxAttempts = MAX_ATTEMPTS, retryDelayMs = RETRY_DELAY_MS, _takeoverSeam, _postRenameSeam } = {},
) {
  ensureDir(locksDir(root));
  const lockPath = lockFilePath(root, name);
  const token = crypto.randomBytes(8).toString('hex');
  const session = envSessionId(process.env.BEE_SESSION_ID, process.env.CLAUDE_CODE_SESSION_ID);
  let acquired = false;
  const waitStartMs = Date.now();
  let contendedHolderSession = null;

  for (let attempt = 0; attempt < maxAttempts && !acquired; attempt++) {
    const nowMs = Date.now();
    const body = { pid: process.pid, session, ts: new Date(nowMs).toISOString(), token };
    if (tryAcquire(lockPath, body)) {
      acquired = true;
      break;
    }
    const contending = readHolder(lockPath);
    if (contending && typeof contending === 'object' && contending.session) {
      contendedHolderSession = contending.session;
    }
    // Staleness is re-verified at THIS retry, on the real filesystem mtime —
    // never cached from an earlier check.
    if (await tryStaleTakeoverAsync(lockPath, nowMs, _takeoverSeam, _postRenameSeam)) {
      // We just freed the slot ourselves; race for it immediately rather
      // than waiting a full retry interval behind everyone else.
      if (tryAcquire(lockPath, { ...body, ts: new Date(Date.now()).toISOString() })) {
        acquired = true;
        break;
      }
    }
    if (attempt + 1 < maxAttempts) {
      await sleep(retryDelayMs);
    }
  }

  if (!acquired) {
    const holder = readHolder(lockPath);
    appendContentionTelemetry(
      root,
      buildContentionRecord({
        name,
        session,
        waitStartMs,
        holderSession: (holder && holder.session) || contendedHolderSession,
        result: 'busy',
      }),
    );
    throw new LockBusyError(name, holder);
  }

  appendContentionTelemetry(
    root,
    buildContentionRecord({ name, session, waitStartMs, holderSession: contendedHolderSession, result: 'acquired' }),
  );

  try {
    return await fn();
  } finally {
    const holder = readHolder(lockPath);
    if (holder && holder.token === token && holder.pid === process.pid) {
      withTransientFsRetry(() => fs.rmSync(lockPath, { force: true }));
    }
  }
}
