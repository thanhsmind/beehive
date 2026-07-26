// integration-queue.mjs — multisession-native-22 (CONTEXT.md D8 stage 5,
// D9 invariant 12): serializes `bee worktree merge` requests against the
// SAME mainRoot through a durable queue + a processor lease, instead of two
// sessions racing the 'worktree-admin' store lock the way they do today.
//
// THE PROBLEM this replaces: mergeFeatureWorktree's P2 (the verify child)
// deliberately releases 'worktree-admin' (hardening-4c/D10b) so a
// multi-minute verify never holds a filesystem lock. That is correct for
// invariant 12, but it means a SECOND concurrent "bee worktree merge"
// attempt against the same mainRoot no longer waits politely on the lock —
// it slips straight into P1's own isTreeDirty(mainRoot) pre-check and gets
// a hard WORKTREE_MERGE_MAIN_DIRTY refusal, because the first merge's
// staged-but-uncommitted tree genuinely IS dirty. That refusal is honest but
// unfriendly: the second caller has no way to know "try again in a minute"
// versus "something is actually wrong". This module gives the second caller
// a queue position and a bounded wait instead of an opaque dirty-tree error.
//
// DESIGN (advisor consult slice 5, conditions A/B/C — see
// docs/history/multisession-native/reports/advisor-digest-slice5.md):
//
//   A — only `bee worktree merge` (bee.mjs's handleWorktreeMerge) becomes
//   queue-aware. This module never imports or calls dispatch-interlock.mjs
//   or herding.mjs (packages/bee/lib/herding.mjs stays
//   untouched, per its own D4 "never call these from bee automation" rule)
//   — a completely separate coordination concern (the herding cockpit's
//   human-owner enable/disable gesture, not merge serialization).
//
//   B — the drainer (`runThroughQueue` below): every request enqueues a
//   durable record FIRST (queue file transient for the uncontended case —
//   see below), then loops: while this record is the front of the queued
//   line AND the processor lease is free, attempt to become processor and
//   run the real merge; otherwise sleep `pollIntervalMs` and re-check, up to
//   `waitBoundMs` (default a few minutes). On timeout, returns a typed
//   `{ ok: false, code: 'INTEGRATION_QUEUE_TIMEOUT', merged: false, ... }`
//   result whose `message` unambiguously says the merge did NOT run — never
//   a shape a caller could mistake for success (the same discipline
//   mergeFeatureWorktree's own MERGE_CONFLICT/MERGE_VERIFY_RED results use:
//   `ok`/`merged` are always present and always tell the truth). An empty
//   queue resolves this loop on its FIRST iteration with no real sleep at
//   all (front-of-line is trivially this request, the lease is trivially
//   free) — this is the "solo behavior byte-identical" case: the queue
//   record is created and immediately driven to 'done', a transient
//   bookkeeping artifact the merge's own result/output never mentions.
//
//   C — the processor lease is a `path` resource in lease-store.mjs
//   (`.bee/runtime/leases/paths/<hash-of-"path:integration-processor">.json`
//   under `controlRoot`), acquired with a STRICTLY POSITIVE ttl
//   (`tryBecomeProcessor` refuses a non-positive one before ever calling
//   lease-store — see its own comment: lease-store stores `expires_at: null`
//   — never expires — for a non-positive ttl, which would deadlock the
//   queue behind a crashed processor forever). A dead processor's lease is
//   swept (lease-store's own `sweepExpiredLeases`, expiry-only, lock-free)
//   before every acquire attempt, so once its ttl has genuinely passed, the
//   NEXT caller's `tryBecomeProcessor` acquires a fresh record with `epoch`
//   bumped to (previous epoch, if any) + 1 — a real takeover, never a
//   silently-reused epoch. `checkProcessorLeaseEpoch` is the READ side of
//   that fence: given the epoch a processor acquired, it returns a drift
//   string the instant the on-disk record's epoch has moved past it (someone
//   else took over) or the record vanished — wired into
//   `mergeFeatureWorktree`'s P3 (`options.checkProcessorLease`, see
//   worktree-store.mjs) as the FIRST line of defense, re-checked right
//   before the commit, after 'worktree-admin' is re-acquired; the existing
//   `checkMergeFence` staged-tree/HEAD check is the second line (advisor
//   condition C's own framing) — either one aborts the merge untouched.
//   `onVerifyTick` (also wired into mergeFeatureWorktree, fired periodically
//   DURING the — now async, not spawnSync — verify child so a timer can
//   actually run alongside a multi-minute verify) attempts a best-effort
//   renew every tick; a missed renewal is never silently fatal to
//   correctness because `checkProcessorLeaseEpoch` is the authoritative,
//   synchronous re-check right before commit, not this heartbeat.
//
// Zero deps beyond node builtins + fsutil.mjs + lock.mjs + lease-store.mjs —
// deliberately NOT state.mjs/claims.mjs (the C4 structural-isolation pattern
// lease-store.mjs and reservations.mjs both document in their own headers):
// this module takes `controlRoot` as a plain parameter, exactly like
// lease-store.mjs takes `root` — the CALLER (bee.mjs's handleWorktreeMerge)
// resolves `controlRootFor(mainRoot)` and a session id via claims.mjs before
// ever reaching this module, so this module can never cycle back into
// state.mjs (which imports lease-store.mjs, and would cycle straight back
// here if this module imported it).
//
// Canonical source: this file (packages/bee/lib/
// integration-queue.mjs). Vendored verbatim to .bee/bin/lib/
// integration-queue.mjs and the plugin mirrors by render_plugin_skill_trees.mjs
// + onboard_bee.mjs --apply. Edit ONLY this copy.

import fs from 'node:fs';
import path from 'node:path';
import { ensureDir, writeJsonAtomic, readJson } from './fsutil.mjs';
import { withStoreLock } from './lock.mjs';
import { acquireLeases, releaseLease, renewLease, listLeases, sweepExpiredLeases, LeaseStoreError } from './lease-store.mjs';

/** Typed refusal thrown by integration-queue verbs — never a silent fallback. */
export class IntegrationQueueError extends Error {
  constructor(code, message, details) {
    super(message);
    this.name = 'IntegrationQueueError';
    this.type = 'refused';
    this.code = code;
    if (details && typeof details === 'object') Object.assign(this, details);
  }
}

const QUEUE_LOCK_NAME = 'integration-queue';
const PROCESSOR_RESOURCE_ID = 'integration-processor';
const PROCESSOR_WORKFLOW_ID = 'integration-queue'; // descriptive only — see module header, C4
const SEQ_WIDTH = 6;

export const DEFAULT_PROCESSOR_TTL_SECONDS = 120;
export const DEFAULT_RENEW_INTERVAL_MS = 30_000;
export const DEFAULT_WAIT_BOUND_MS = 180_000; // "a few minutes" (advisor condition B)
export const DEFAULT_POLL_INTERVAL_MS = 500;

// ─── queue directory / records ──────────────────────────────────────────────

export function integrationQueueDir(controlRoot) {
  return path.join(controlRoot, '.bee', 'runtime', 'integration', 'queue');
}

function queueRecordPath(controlRoot, seq) {
  return path.join(integrationQueueDir(controlRoot), `${String(seq).padStart(SEQ_WIDTH, '0')}.json`);
}

function listQueueRecords(controlRoot) {
  let entries;
  try {
    entries = fs.readdirSync(integrationQueueDir(controlRoot));
  } catch {
    return []; // directory not created yet — no requests ever enqueued
  }
  const records = [];
  for (const name of entries) {
    if (!name.endsWith('.json')) continue;
    const seq = Number.parseInt(name.slice(0, -'.json'.length), 10);
    if (!Number.isFinite(seq)) continue;
    const record = readJson(path.join(integrationQueueDir(controlRoot), name), null);
    if (record) records.push({ ...record, seq });
  }
  records.sort((a, b) => a.seq - b.seq);
  return records;
}

// nextQueueSeq — the highest existing seq + 1 (1 when empty). Only ever
// called from inside withStoreLock(controlRoot, QUEUE_LOCK_NAME, ...), same
// discipline as state.mjs's nextHandoffSeq.
function nextQueueSeq(controlRoot) {
  const records = listQueueRecords(controlRoot);
  return records.length === 0 ? 1 : records[records.length - 1].seq + 1;
}

/**
 * enqueueMergeRequest(controlRoot, { worktreeId, feature, requestedBySession })
 * — append one durable queue record: `{seq, worktree_id, feature,
 * requested_by_session, requested_at, status: 'queued'}`. Runs under
 * `withStoreLock(controlRoot, 'integration-queue')` so two concurrent
 * enqueuers can never generate the same seq — a brief read-a-directory-then-
 * write critical section, never held across a merge itself (invariant 12).
 */
export function enqueueMergeRequest(controlRoot, { worktreeId, feature = null, requestedBySession, now = new Date().toISOString() } = {}) {
  if (typeof worktreeId !== 'string' || !worktreeId.trim()) {
    throw new IntegrationQueueError('QUEUE_INVALID_REQUEST', 'enqueueMergeRequest: worktreeId is required.');
  }
  if (typeof requestedBySession !== 'string' || !requestedBySession.trim()) {
    throw new IntegrationQueueError('QUEUE_INVALID_REQUEST', 'enqueueMergeRequest: requestedBySession is required.');
  }
  return withStoreLock(controlRoot, QUEUE_LOCK_NAME, () => {
    const seq = nextQueueSeq(controlRoot);
    const record = {
      seq,
      worktree_id: worktreeId.trim(),
      feature,
      requested_by_session: requestedBySession.trim(),
      requested_at: now,
      status: 'queued',
    };
    ensureDir(integrationQueueDir(controlRoot));
    writeJsonAtomic(queueRecordPath(controlRoot, seq), record);
    return record;
  });
}

function writeQueueStatus(controlRoot, seq, status, extra = {}) {
  return withStoreLock(controlRoot, QUEUE_LOCK_NAME, () => {
    const file = queueRecordPath(controlRoot, seq);
    const current = readJson(file, null);
    if (!current) return null; // record pruned/gone — nothing to update
    const next = { ...current, status, ...extra };
    writeJsonAtomic(file, next);
    return next;
  });
}

/** queueStatusFor(controlRoot, seq) — the current record, or null if absent. */
export function queueStatusFor(controlRoot, seq) {
  return readJson(queueRecordPath(controlRoot, seq), null);
}

/**
 * queuePosition(controlRoot, seq) — 1-based position of `seq` among every
 * still-OPEN ('queued' or 'processing') record, oldest first. `null` when
 * `seq` is not currently open (done/failed/never existed) — a caller that
 * gets `null` back should read `queueStatusFor` for the terminal status
 * instead of treating this as "still waiting".
 */
export function queuePosition(controlRoot, seq) {
  const open = listQueueRecords(controlRoot).filter((r) => r.status === 'queued' || r.status === 'processing');
  const idx = open.findIndex((r) => r.seq === seq);
  if (idx === -1) return null;
  return { position: idx + 1, ahead: idx, total_open: open.length };
}

// frontOfLine — the oldest still-'queued' record, or null when none are
// queued (either empty, or every open record is already 'processing').
function frontOfLine(controlRoot) {
  const queued = listQueueRecords(controlRoot).filter((r) => r.status === 'queued');
  return queued.length ? queued[0] : null;
}

// ─── processor lease (advisor condition C) ─────────────────────────────────

function readProcessorLeaseRaw(controlRoot) {
  const { leases } = listLeases(controlRoot);
  return leases.find((l) => l.resource === `path:${PROCESSOR_RESOURCE_ID}`) || null;
}

/**
 * tryBecomeProcessor(controlRoot, { sessionId, workspaceId, ttlSeconds, now })
 * — attempt to acquire the SINGLE processor lease for this controlRoot's
 * integration queue.
 *
 * `ttlSeconds` MUST be strictly positive (condition C): lease-store stores
 * `expires_at: null` ("never expires") for a non-positive ttl, which would
 * deadlock the queue behind a crashed processor forever — refused with a
 * typed `QUEUE_INVALID_TTL` before ever touching lease-store.
 *
 * Sweeps expired leases first (`sweepExpiredLeases`, lock-free, expiry-only
 * — a LIVE processor's still-valid lease is untouched) so a genuinely dead
 * processor's slot is free to re-acquire. The new lease's `epoch` is
 * (currently-stored epoch, if any, else 0) + 1 — a real takeover, never a
 * silently-reused epoch, so `checkProcessorLeaseEpoch` can always tell a
 * stale processor's epoch apart from the current one.
 *
 * Returns `{ ok: true, lease, tookOver }` on success, or `{ ok: false,
 * holder }` when a live processor already holds it (the ordinary contention
 * case — never throws for this). Any other lease-store error propagates.
 */
export async function tryBecomeProcessor(controlRoot, { sessionId, workspaceId = 'main', ttlSeconds = DEFAULT_PROCESSOR_TTL_SECONDS, now = Date.now() } = {}) {
  if (typeof sessionId !== 'string' || !sessionId.trim()) {
    throw new IntegrationQueueError('QUEUE_INVALID_REQUEST', 'tryBecomeProcessor: sessionId is required.');
  }
  if (!Number.isFinite(ttlSeconds) || ttlSeconds <= 0) {
    throw new IntegrationQueueError(
      'QUEUE_INVALID_TTL',
      `tryBecomeProcessor: ttlSeconds must be strictly positive (got ${JSON.stringify(ttlSeconds)}) — a non-positive ttl never expires (lease-store expires_at: null) and would deadlock the queue behind a crashed processor.`,
    );
  }
  const before = readProcessorLeaseRaw(controlRoot);
  sweepExpiredLeases(controlRoot, { now });
  const nextEpoch = (before && Number.isFinite(before.epoch) ? before.epoch : 0) + 1;
  try {
    const [lease] = await acquireLeases(
      controlRoot,
      [
        {
          type: 'path',
          id: PROCESSOR_RESOURCE_ID,
          mode: 'processor',
          workflow_id: PROCESSOR_WORKFLOW_ID,
          session_id: sessionId.trim(),
          workspace_id: workspaceId,
          epoch: nextEpoch,
          ttl: ttlSeconds,
        },
      ],
      { now },
    );
    return { ok: true, lease, tookOver: before != null };
  } catch (error) {
    if (error instanceof LeaseStoreError && error.code === 'LEASE_HELD') {
      return { ok: false, holder: error.holder ?? readProcessorLeaseRaw(controlRoot) };
    }
    throw error;
  }
}

/** renewProcessorLease — heartbeat-renew the processor lease this caller
 * holds, fenced on the epoch it originally acquired (refuses typed
 * LEASE_FENCE_STALE if a takeover already moved the epoch forward). */
export async function renewProcessorLease(controlRoot, { ttlSeconds = DEFAULT_PROCESSOR_TTL_SECONDS, now = Date.now(), presentedEpoch } = {}) {
  return renewLease(controlRoot, { type: 'path', id: PROCESSOR_RESOURCE_ID }, { ttl: ttlSeconds, now, presentedEpoch });
}

/** releaseProcessorLease — give up the processor lease, fenced on the
 * caller's own epoch so a zombie can never delete a takeover's fresh lease. */
export async function releaseProcessorLease(controlRoot, { presentedEpoch } = {}) {
  return releaseLease(controlRoot, { type: 'path', id: PROCESSOR_RESOURCE_ID }, { presentedEpoch });
}

/**
 * checkProcessorLeaseEpoch(controlRoot, expectedEpoch) — P3's primary fence
 * (advisor condition C, "first line"; worktree-store.mjs's existing
 * checkMergeFence staged-tree/HEAD check is the second line). Returns a
 * drift description string the instant the on-disk processor lease no
 * longer carries `expectedEpoch` (a takeover already bumped it, or the
 * record vanished — released or swept out from under this caller), or
 * `null` when it is still exactly the lease this caller acquired. Same
 * "string | null" contract checkMergeFence already uses, so the two compose
 * with a single `||` in mergeFeatureWorktreeFinish.
 */
export function checkProcessorLeaseEpoch(controlRoot, expectedEpoch) {
  const current = readProcessorLeaseRaw(controlRoot);
  if (!current) {
    return `the integration-queue processor lease disappeared (this processor acquired epoch ${expectedEpoch}) while its verify was running — released or swept out from under it`;
  }
  if (current.epoch !== expectedEpoch) {
    return `the integration-queue processor lease was taken over while this processor's verify was running (epoch ${expectedEpoch} -> ${current.epoch}, now held by session "${current.session_id}") — this processor is a zombie`;
  }
  return null;
}

// ─── the drainer (advisor condition B) ─────────────────────────────────────

/**
 * runThroughQueue(controlRoot, options) — the ONE entrypoint `bee worktree
 * merge` calls (condition A: no other CLI verb becomes queue-aware).
 *
 * `options.runMerge(hooks)` is the caller-supplied function that actually
 * performs the merge (bee.mjs wires this to `mergeFeatureWorktree`). `hooks`
 * passed to it: `{ checkProcessorLease, onVerifyTick }` — see
 * worktree-store.mjs's `mergeFeatureWorktree` for how those two wire into
 * its P2/P3.
 *
 * Every request enqueues a durable record FIRST (`enqueueMergeRequest`),
 * then loops: while this record is the front of the queued line AND the
 * processor lease is free, attempt to become processor and run; otherwise
 * sleep `pollIntervalMs` and re-check, up to `waitBoundMs`. An empty queue
 * resolves on the FIRST iteration with no sleep at all — the "solo merge
 * byte-identical" case (the queue record is created and immediately driven
 * to 'done'/'failed', a transient bookkeeping artifact the merge's own
 * result/output never mentions).
 *
 * On timeout (advisor condition B — "a position-returned merge NEVER reads
 * as success"): returns `{ ok: false, code: 'INTEGRATION_QUEUE_TIMEOUT',
 * merged: false, queue: {...}, message }` — the merge did NOT run, and
 * `message` says so unambiguously. NEVER blocks past `waitBoundMs`
 * (prohibition: "no unbounded wait").
 */
export async function runThroughQueue(
  controlRoot,
  {
    worktreeId,
    feature = null,
    sessionId,
    workspaceId = 'main',
    runMerge,
    ttlSeconds = DEFAULT_PROCESSOR_TTL_SECONDS,
    renewIntervalMs = DEFAULT_RENEW_INTERVAL_MS,
    waitBoundMs = DEFAULT_WAIT_BOUND_MS,
    pollIntervalMs = DEFAULT_POLL_INTERVAL_MS,
    sleep = (ms) => new Promise((resolve) => setTimeout(resolve, ms)),
    _seams,
  } = {},
) {
  if (typeof runMerge !== 'function') {
    throw new IntegrationQueueError('QUEUE_INVALID_REQUEST', 'runThroughQueue: options.runMerge must be a function.');
  }

  const record = await enqueueMergeRequest(controlRoot, { worktreeId, feature, requestedBySession: sessionId });
  if (_seams?.onEnqueued) await _seams.onEnqueued(record);

  const deadline = Date.now() + waitBoundMs;
  for (;;) {
    if (_seams?.onPollTick) await _seams.onPollTick(record);
    const front = frontOfLine(controlRoot);
    if (front && front.seq === record.seq) {
      const attempt = await tryBecomeProcessor(controlRoot, { sessionId, workspaceId, ttlSeconds });
      if (attempt.ok) {
        return processAsOwner(controlRoot, { record, lease: attempt.lease, runMerge, ttlSeconds, renewIntervalMs });
      }
      // Lease still held by a live processor from a PRIOR era (e.g. still
      // finishing a merge queued before this record ever existed) — keep
      // waiting; do not fall through to the timeout check on a lost race
      // alone, only once waitBoundMs has genuinely elapsed.
    }
    if (Date.now() >= deadline) {
      const position = queuePosition(controlRoot, record.seq);
      const status = queueStatusFor(controlRoot, record.seq)?.status ?? 'queued';
      return {
        ok: false,
        code: 'INTEGRATION_QUEUE_TIMEOUT',
        merged: false,
        queue: { seq: record.seq, status, ...(position || {}) },
        message:
          `the integration queue's wait bound (${waitBoundMs}ms) elapsed before this merge request reached the front of the line — ` +
          `the merge did NOT run and nothing was committed. This request is still ${status} at position ${
            position ? position.position : '?'
          } (${position ? position.ahead : '?'} ahead of it) — retry "bee worktree merge" to wait again, or check "bee worktree list" once the queue drains.`,
      };
    }
    // eslint-disable-next-line no-await-in-loop
    await sleep(pollIntervalMs);
  }
}

// processAsOwner — this caller just became the processor for `record`
// (or, in a future variant, could own no record at all — every current
// caller always has one, per runThroughQueue's "enqueue first" design).
// Renews the lease from a periodic tick (wired into mergeFeatureWorktree's
// P2 verify child, condition C) and always releases the lease + marks the
// record terminal in a `finally`, so a throwing `runMerge` never leaves the
// queue or the lease stuck.
async function processAsOwner(controlRoot, { record, lease, runMerge, ttlSeconds, renewIntervalMs }) {
  await writeQueueStatus(controlRoot, record.seq, 'processing', { processing_started_at: new Date().toISOString() });
  const onVerifyTick = async () => {
    try {
      await renewProcessorLease(controlRoot, { ttlSeconds, presentedEpoch: lease.epoch });
    } catch {
      // Best-effort (module header: a missed renewal is a liveness concern,
      // never a correctness one) — mergeFeatureWorktreeFinish's
      // checkProcessorLease re-check right before commit is authoritative.
    }
  };
  try {
    const result = await runMerge({
      checkProcessorLease: () => checkProcessorLeaseEpoch(controlRoot, lease.epoch),
      onVerifyTick,
      verifyTickIntervalMs: renewIntervalMs,
    });
    await writeQueueStatus(controlRoot, record.seq, result && result.ok === false ? 'failed' : 'done', {
      finished_at: new Date().toISOString(),
    });
    return result;
  } catch (error) {
    await writeQueueStatus(controlRoot, record.seq, 'failed', { finished_at: new Date().toISOString(), error: error instanceof Error ? error.message : String(error) });
    throw error;
  } finally {
    await releaseProcessorLease(controlRoot, { presentedEpoch: lease.epoch }).catch(() => {});
  }
}
