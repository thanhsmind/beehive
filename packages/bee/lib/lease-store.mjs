// lease-store.mjs — per-resource epoch lease records (multisession-native-11,
// CONTEXT.md D4: "sharded leases replace reservations.json").
//
// A LEASE replaces a row in the single .bee/reservations.json file as the
// unit of coordination for one resource — a cell or a path — so two
// unrelated resources never contend on a shared file at all. Each lease
// lives at its OWN path, keyed by the resource identity:
//
//   .bee/runtime/leases/cells/<cell-id>.json
//   .bee/runtime/leases/paths/<path-hash>.json
//
// record shape: { resource, mode, workflow_id, session_id, workspace_id,
//   epoch, acquired_at, expires_at, kind }. `resource` is the type-prefixed
//   key ("cell:<cell-id>" or "path:<canonical-path>") — carrying the type
//   inline keeps the record to exactly these fields rather than adding a
//   distinct type column. `expires_at` is an ISO timestamp, or `null`
//   meaning "never expires" (TTL semantics matching reservations.mjs's
//   isExpired: a non-positive ttl at acquire/renew time stores
//   `expires_at: null` and sweepExpiredLeases never touches it).
//
// `kind` (multisession-native-13, D4 — advisor consult slice 3 condition D):
// `'intent'|'lease'`, defaults to `'lease'` when the caller omits it — every
// msn-11/12 acquire (this module had no `kind` concept before this cell)
// stays semantically a lease, byte-unchanged in behavior. `'intent'` marks a
// planning-declared broad/glob scope (advisory: warn + a scheduling input,
// never a hard block); `'lease'` marks an exact path a writer is actually
// about to touch (hard). This field is FORWARD GROUNDWORK ONLY in this
// cell — the production write guard (guards.mjs checkWrite) does not read
// from this store yet, only from reservations.mjs's `.bee/reservations.json`
// (which gained the equivalent `kind` field in this same cell); msn-16's
// full shim is what migrates checkWrite onto lease-store.mjs and makes this
// field load-bearing at guard time. Recorded here now so every future lease
// carries the classification from day one instead of a later migration
// having to backfill it.
//
// Placement (condition A, advisor consult slice 3): `.bee/runtime/leases/`
// is deliberate for THIS cell — D2's controlRoot split (shared-across-
// worktrees control plane) has not landed yet, so this module still reads
// and writes under the CALLER's own `root` only. Slice 4 re-roots this
// under controlRoot; that migration is a ONE-LINE change because every
// caller in this module reaches the store exclusively through leasesRoot(root)
// below — never a hand-built `.bee/runtime/leases` path elsewhere in this
// file. Until that re-root lands, this module performs NO cross-workspace
// reads: `root` is always the single checkout the caller is already
// operating in, never another worktree's `.bee/` (cross-worktree visibility
// for the pre-controlRoot world rides worktree-holds.mjs's separate
// mainRoot-anchored ledger, not this module).
//
// Epoch (advisor consult slice 3, item F): `epoch` is stored VERBATIM from
// the caller on every acquire — this module never compares it against a
// prior value at acquire time, and never refuses an acquire because of it
// (there is no forced-takeover primitive here; a genuine takeover is
// release-then-acquire, with the NEW caller deciding the bumped epoch
// value). Fencing ENFORCEMENT is multisession-native-12's addition, on top
// of that verbatim storage: `renewLease`/`releaseLease` below both accept
// an optional `presentedEpoch` and refuse typed LEASE_FENCE_STALE when it
// is behind the resource's currently stored epoch. Omitted (every caller
// today — this module has no production wiring yet, msn-16 adds it), both
// stay byte-unchanged from before this cell; full mandatory presentation
// arrives with workspace identity in slice 4, matching claims.mjs's own
// compat posture for `fence_epoch` (a deliberately distinct name — see that
// module's header — for the equivalent claims-side token).
//
// Locking: acquireLeases needs none of its own — O_EXCL create is the whole
// concurrency primitive (two racers targeting the SAME resource file can
// never both "win" the create; the loser sees EEXIST and is refused with a
// typed LEASE_HELD naming the holder). sweepExpiredLeases and the LEGACY
// (no presentedEpoch) path of releaseLease are a single fs.rmSync per file —
// also lock-free; a delete of one resource's own file cannot race a
// create/delete of a DIFFERENT resource's file, and a racing renew on the
// SAME file is a lost-update at worst (the renewed expires_at simply gets
// overwritten by the delete, which is the same outcome as if the renew had
// lost the race a moment earlier). renewLease (a genuine read-modify-write)
// DOES take a lock — but a lease-scoped one, `lease:<file-hash>`, never a
// store-wide "leases" lock: renewing lease A never contends with acquiring,
// renewing, or releasing lease B. That is the structural difference from
// reservations.mjs (one shared JSON file, hence one shared 'reservations'
// lock for every mutation) this module is built to remove. The FENCED
// (presentedEpoch given) path of releaseLease is likewise a genuine
// read-then-maybe-delete, so multisession-native-12 gives it the SAME
// per-lease `lease:<file-hash>` lock renewLease already uses — never a new
// lock name, per that cell's lock-discipline requirement.
//
// Structural isolation (C4 pattern, workflow-store.mjs precedent): this
// module imports ONLY node builtins, fsutil.mjs, and lock.mjs. It never
// imports claims.mjs, state.mjs, or reservations.mjs, so it can never read
// or touch a session record and can never nest inside (or be nested inside)
// the 'sessions' or 'state' store locks, and it never acquires the
// 'reservations' store lock either — proven by
// test_lease_store.mjs's static source-scan, mirroring
// test_workflow_store.mjs's own C4 proof.
//
// `canonicalizePath` below deliberately DUPLICATES reservations.mjs's
// private `normalizePath` rather than importing it — reservations.mjs
// imports claims.mjs, so importing reservations.mjs here would defeat the
// whole point of this module's session-free isolation. Same deliberate-
// small-duplicate precedent as lock.mjs's `envSessionId` (kept in sync with
// claims.mjs's resolveSessionId by hand) and workflow-store.mjs's
// duplicated `GATE_NAMES`.
//
// Canonical source: this file
// (packages/bee/lib/lease-store.mjs). Vendored verbatim to
// .bee/bin/lib/lease-store.mjs and the plugin mirrors (.agents/, .claude/,
// .claude-plugin/, .codex-plugin/) by render_plugin_skill_trees.mjs +
// onboard_bee.mjs --apply. Edit ONLY this copy; the vendoring scripts
// propagate it.

import fs from 'node:fs';
import path from 'node:path';
import crypto from 'node:crypto';
import { ensureDir, writeJsonAtomic } from './fsutil.mjs';
import { withStoreLock } from './lock.mjs';

const DEFAULT_TTL_SECONDS = 3600;
const RESOURCE_TYPES = ['cell', 'path'];
// multisession-native-13, D4: same two-value vocabulary as
// reservations.mjs's RESERVATION_KINDS — kept as an independent duplicate
// constant (not imported) for the same structural-isolation reason this
// module never imports reservations.mjs at all (see module header, C4
// pattern).
const LEASE_KINDS = ['intent', 'lease'];
const DEFAULT_LEASE_KIND = 'lease';

/** Typed refusal thrown by lease-store verbs — never a silent fallback. */
export class LeaseStoreError extends Error {
  constructor(code, message, details) {
    super(message);
    this.name = 'LeaseStoreError';
    this.type = 'refused';
    this.code = code;
    if (details && typeof details === 'object') {
      if (details.resource !== undefined) this.resource = details.resource;
      if (details.holder !== undefined) this.holder = details.holder;
    }
  }
}

export function runtimeDir(root) {
  return path.join(root, '.bee', 'runtime');
}

/**
 * leasesRoot(root) — the ONE place this module builds the leases directory
 * path from `root`. Every other function reaches cells/paths storage
 * through this (directly or via leaseCellsDir/leasePathsDir), so slice 4's
 * re-root to controlRoot is a one-line change here, not a repo-wide grep.
 */
export function leasesRoot(root) {
  return path.join(runtimeDir(root), 'leases');
}

export function leaseCellsDir(root) {
  return path.join(leasesRoot(root), 'cells');
}

export function leasePathsDir(root) {
  return path.join(leasesRoot(root), 'paths');
}

// ─── shared id/path canonicalization ────────────────────────────────────────

// Mirrors workflow-store.mjs's requireWorkflowId: a cell id becomes a
// filename under leaseCellsDir(root), so path separators and '..' are bad
// arguments — refused with a typed error rather than allowed to escape the
// leases directory.
function requireCellId(value) {
  if (typeof value !== 'string' || !value.trim()) {
    throw new LeaseStoreError('LEASE_INVALID_REQUEST', 'lease request: cell id is required.');
  }
  const id = value.trim();
  if (/[\\/]/.test(id) || id.includes('..')) {
    throw new LeaseStoreError(
      'LEASE_INVALID_REQUEST',
      `lease request: cell id "${id}" must be a plain id (no path separators) — it becomes a filename under .bee/runtime/leases/cells/.`,
    );
  }
  return id;
}

// Deliberate duplicate of reservations.mjs's private normalizePath — see
// the module header for why this is not imported. Same behavior: strip a
// leading "./", normalize backslashes, drop a trailing slash.
function canonicalizePath(value) {
  return String(value || '')
    .replace(/\\/g, '/')
    .replace(/^\.\/+/, '')
    .replace(/\/+$/, '');
}

function hashString(value) {
  return crypto.createHash('sha256').update(value).digest('hex');
}

// resolveResourceFile(root, { type, id }) — the SINGLE place a {type, id}
// pair becomes a resource key + lock-name hash + on-disk file path, shared
// by acquire/release/renew so all three ever agree on where a given
// resource lives.
function resolveResourceFile(root, { type, id } = {}) {
  if (!RESOURCE_TYPES.includes(type)) {
    throw new LeaseStoreError(
      'LEASE_INVALID_REQUEST',
      `lease request: type must be one of ${RESOURCE_TYPES.join('/')} (got ${JSON.stringify(type)}).`,
    );
  }
  if (type === 'cell') {
    const cellId = requireCellId(id);
    const resourceKey = `cell:${cellId}`;
    return {
      type,
      id: cellId,
      resourceKey,
      hash: hashString(resourceKey),
      file: path.join(leaseCellsDir(root), `${cellId}.json`),
    };
  }
  if (typeof id !== 'string' || !id.trim()) {
    throw new LeaseStoreError('LEASE_INVALID_REQUEST', 'lease request: path id is required.');
  }
  const canonical = canonicalizePath(id);
  const resourceKey = `path:${canonical}`;
  const hash = hashString(resourceKey);
  return {
    type,
    id: canonical,
    resourceKey,
    hash,
    file: path.join(leasePathsDir(root), `${hash}.json`),
  };
}

function lockNameForFile(file) {
  return `lease:${hashString(file)}`;
}

function readLeaseSafe(file) {
  try {
    return JSON.parse(fs.readFileSync(file, 'utf8'));
  } catch {
    return null; // gone, unreadable, or mid-write elsewhere — treat as "no info", never throw
  }
}

function isLeaseExpired(record, nowMs) {
  if (!record || record.expires_at == null) return false; // no record, or "never expires"
  const expiresMs = Date.parse(record.expires_at);
  if (!Number.isFinite(expiresMs)) return false;
  return expiresMs <= nowMs;
}

function computeExpiresAt(ttl, nowMs) {
  return Number.isFinite(ttl) && ttl > 0 ? new Date(nowMs + ttl * 1000).toISOString() : null;
}

// sameLeaseRecord — identity check used only by acquireLeases' own rollback,
// so a rollback can never delete a file that has changed underneath it
// since THIS call created it (e.g. a concurrent releaseLease + a fresh
// acquireLeases racing into the same slot in the tiny window between our
// create and our own rollback). acquired_at carries millisecond-ISO
// granularity, so together with resource/session/workflow/epoch this is
// exact-match proof it is the same acquisition, never merely "looks
// similar" — same discipline as lock.mjs's pid+token+ts holder-identity
// check.
function sameLeaseRecord(a, b) {
  return Boolean(
    a &&
      b &&
      a.resource === b.resource &&
      a.session_id === b.session_id &&
      a.workflow_id === b.workflow_id &&
      a.epoch === b.epoch &&
      a.acquired_at === b.acquired_at,
  );
}

function tryCreateLeaseFile(file, record) {
  ensureDir(path.dirname(file));
  try {
    fs.writeFileSync(file, `${JSON.stringify(record, null, 2)}\n`, { encoding: 'utf8', flag: 'wx' });
    return true;
  } catch (error) {
    if (error && error.code === 'EEXIST') return false;
    throw error;
  }
}

function rollbackOne(file, record) {
  const current = readLeaseSafe(file);
  if (sameLeaseRecord(current, record)) {
    try {
      fs.rmSync(file, { force: true });
    } catch {
      // best-effort — never let rollback cleanup itself throw over the
      // original LEASE_HELD refusal
    }
  }
  // A mismatch means the file changed since we created it (released and
  // re-acquired by someone else) — never touch it; leave it exactly as it
  // is now.
}

function normalizeAcquireRequest(root, req) {
  if (!req || typeof req !== 'object') {
    throw new LeaseStoreError('LEASE_INVALID_REQUEST', 'acquireLeases: each request must be an object.');
  }
  const { type, id, mode, workflow_id, session_id, workspace_id, epoch, ttl = DEFAULT_TTL_SECONDS, kind = DEFAULT_LEASE_KIND } = req;
  const resolved = resolveResourceFile(root, { type, id });
  if (typeof mode !== 'string' || !mode.trim()) {
    throw new LeaseStoreError('LEASE_INVALID_REQUEST', `lease request "${resolved.resourceKey}": mode is required.`);
  }
  if (!LEASE_KINDS.includes(kind)) {
    throw new LeaseStoreError(
      'LEASE_INVALID_REQUEST',
      `lease request "${resolved.resourceKey}": kind must be one of ${LEASE_KINDS.join('/')} (got ${JSON.stringify(kind)}).`,
    );
  }
  for (const [key, value] of [
    ['workflow_id', workflow_id],
    ['session_id', session_id],
    ['workspace_id', workspace_id],
  ]) {
    if (typeof value !== 'string' || !value.trim()) {
      throw new LeaseStoreError('LEASE_INVALID_REQUEST', `lease request "${resolved.resourceKey}": ${key} is required.`);
    }
  }
  if (!Number.isFinite(epoch)) {
    throw new LeaseStoreError('LEASE_INVALID_REQUEST', `lease request "${resolved.resourceKey}": epoch must be a finite number.`);
  }
  return {
    ...resolved,
    mode: mode.trim(),
    workflow_id,
    session_id,
    workspace_id,
    epoch,
    ttl: Number.isFinite(ttl) ? ttl : DEFAULT_TTL_SECONDS,
    kind,
  };
}

/**
 * acquireLeases(root, requests, options) — acquire one or more leases as a
 * single all-or-nothing batch. Each request:
 *   { type: 'cell'|'path', id, mode, workflow_id, session_id, workspace_id,
 *     epoch, ttl?, kind? }
 *
 * `kind` (multisession-native-13, D4): optional, `'intent'|'lease'`,
 * defaults to `'lease'` — see the module header for the full split. Invalid
 * values throw LEASE_INVALID_REQUEST before any file is touched, same as
 * every other malformed field here.
 *
 * Deadlock-free regardless of caller order (D4): requests are sorted by the
 * sha256 hash of their resource key BEFORE any file is created, so two
 * callers racing to acquire the same two resources in opposite request
 * order still attempt creation in the SAME globally-determined order — the
 * classic "always lock in a fixed global order" deadlock-avoidance
 * discipline, here applied to O_EXCL creates instead of mutex acquisition.
 *
 * On the first collision (EEXIST), every lease already created by THIS
 * call is rolled back (deleted) before the typed LEASE_HELD refusal is
 * thrown — a partial acquire is never left standing. The refusal names the
 * conflicting holder (whatever the colliding file's current content parses
 * to) via `.resource` / `.holder` on the thrown error.
 *
 * options._orderSeam(resourceKey) — test-only hook invoked once per
 * resource, in the actual (post-sort) processing order, immediately before
 * that resource's create attempt. Never set by any production caller;
 * undefined is a no-op. Lets tests observe the real acquisition order
 * without depending on filesystem mtimes.
 */
export function acquireLeases(root, requests, { now = Date.now(), _orderSeam } = {}) {
  if (!Array.isArray(requests) || requests.length === 0) {
    throw new LeaseStoreError('LEASE_INVALID_REQUEST', 'acquireLeases: requests must be a non-empty array.');
  }
  const normalized = requests.map((req) => normalizeAcquireRequest(root, req));

  const seenFiles = new Set();
  for (const item of normalized) {
    if (seenFiles.has(item.file)) {
      throw new LeaseStoreError(
        'LEASE_INVALID_REQUEST',
        `acquireLeases: resource "${item.resourceKey}" was requested more than once in the same call.`,
      );
    }
    seenFiles.add(item.file);
  }

  const sorted = [...normalized].sort((a, b) => (a.hash < b.hash ? -1 : a.hash > b.hash ? 1 : 0));

  const acquired = [];
  for (const item of sorted) {
    if (typeof _orderSeam === 'function') _orderSeam(item.resourceKey);
    const record = {
      resource: item.resourceKey,
      mode: item.mode,
      workflow_id: item.workflow_id,
      session_id: item.session_id,
      workspace_id: item.workspace_id,
      epoch: item.epoch,
      acquired_at: new Date(now).toISOString(),
      expires_at: computeExpiresAt(item.ttl, now),
      kind: item.kind,
    };
    if (tryCreateLeaseFile(item.file, record)) {
      acquired.push({ file: item.file, record });
      continue;
    }
    const holder = readLeaseSafe(item.file);
    for (const won of acquired) rollbackOne(won.file, won.record);
    throw new LeaseStoreError(
      'LEASE_HELD',
      `acquireLeases: resource "${item.resourceKey}" is already leased${
        holder && holder.session_id ? ` by session "${holder.session_id}"` : ''
      }.`,
      { resource: item.resourceKey, holder },
    );
  }
  return acquired.map((item) => item.record);
}

/**
 * releaseLease(root, { type, id }, { presentedEpoch }) — delete the lease
 * file for one resource. Idempotent: releasing an absent lease is `{ ok:
 * true, released: false }`, never an error (mirrors reservations.mjs's
 * release() tolerance of "nothing matched") — true whether or not a
 * presentedEpoch was given, since there is nothing on disk to fence
 * against.
 *
 * multisession-native-12 (D4/D9 invariant 10): `presentedEpoch` is OPTIONAL.
 * Omitted, this is BYTE-UNCHANGED from before this cell — a single
 * lock-free fs.rmSync, exactly as documented in the module header. Given
 * (finite number), the lease record is read first, under the SAME
 * per-resource `lease:<file-hash>` lock renewLease uses (a genuine
 * read-then-maybe-delete is no longer safely lock-free); if its stored
 * `epoch` is ahead of `presentedEpoch`, the release refuses typed
 * LEASE_FENCE_STALE and the file is left untouched — never deleted on a
 * stale presentation.
 */
export async function releaseLease(root, request, { presentedEpoch } = {}) {
  const resolved = resolveResourceFile(root, request || {});
  if (presentedEpoch === undefined) {
    try {
      fs.rmSync(resolved.file, { force: false });
      return { ok: true, released: true };
    } catch (error) {
      if (error && error.code === 'ENOENT') return { ok: true, released: false };
      throw error;
    }
  }
  return withStoreLock(root, lockNameForFile(resolved.file), () => {
    const current = readLeaseSafe(resolved.file);
    if (!current) return { ok: true, released: false }; // nothing on disk — nothing to fence against
    if (!Number.isFinite(presentedEpoch) || presentedEpoch < current.epoch) {
      throw new LeaseStoreError(
        'LEASE_FENCE_STALE',
        `releaseLease: presented epoch ${JSON.stringify(presentedEpoch)} is behind resource "${current.resource}"'s current epoch ${current.epoch} — a takeover already moved ownership forward.`,
        { resource: current.resource, holder: current },
      );
    }
    try {
      fs.rmSync(resolved.file, { force: false });
      return { ok: true, released: true };
    } catch (error) {
      if (error && error.code === 'ENOENT') return { ok: true, released: false };
      throw error;
    }
  });
}

// Shared renew body: read-modify-write one lease file under its OWN
// per-file lock (never a store-wide lock — see module header). `ttl`
// resets the never-expires/expires-at-N clock exactly like a fresh
// acquire's ttl would (non-positive -> expires_at: null).
//
// multisession-native-12 (D4/D9 invariant 10): `presentedEpoch` is
// OPTIONAL. Omitted, the fencing check below never runs — byte-unchanged
// from before this cell. Given, it is compared against the just-read
// record's stored `epoch` BEFORE the write: behind it, this throws typed
// LEASE_FENCE_STALE instead of renewing (the record on disk is left
// exactly as read — no partial write). Renewal never changes the stored
// `epoch` itself, same as claims.mjs's renewClaimTTL never changing
// fence_epoch — only a fresh acquire (a caller-decided takeover) does.
function renewLeaseFile(root, file, { ttl, now, lockOptions, presentedEpoch }) {
  return withStoreLock(
    root,
    lockNameForFile(file),
    () => {
      let current;
      try {
        current = JSON.parse(fs.readFileSync(file, 'utf8'));
      } catch (error) {
        if (error && error.code === 'ENOENT') {
          throw new LeaseStoreError('LEASE_MISSING', `renewLease: no lease record at "${file}" — acquire first.`);
        }
        throw new LeaseStoreError(
          'LEASE_CORRUPT',
          `renewLease: could not read/parse lease record at "${file}" (${error && error.message ? error.message : error}).`,
        );
      }
      if (presentedEpoch !== undefined && (!Number.isFinite(presentedEpoch) || presentedEpoch < current.epoch)) {
        throw new LeaseStoreError(
          'LEASE_FENCE_STALE',
          `renewLease: presented epoch ${JSON.stringify(presentedEpoch)} is behind resource "${current.resource}"'s current epoch ${current.epoch} — a takeover already moved ownership forward.`,
          { resource: current.resource, holder: current },
        );
      }
      const next = { ...current, expires_at: computeExpiresAt(ttl, now) };
      writeJsonAtomic(file, next);
      return next;
    },
    lockOptions,
  );
}

/**
 * renewLease(root, { type, id }, options) — extend one lease's expiry.
 * `options.ttl` (default DEFAULT_TTL_SECONDS, 3600) is the new TTL measured
 * from `options.now` (default Date.now()); non-positive means "never
 * expires" from now on, matching acquire's own TTL semantics. Throws
 * typed LEASE_MISSING/LEASE_CORRUPT, never guesses at a record it cannot
 * read. `options.presentedEpoch` — see renewLeaseFile above (multisession-
 * native-12): optional, refuses typed LEASE_FENCE_STALE when stale.
 */
export async function renewLease(root, request, { ttl = DEFAULT_TTL_SECONDS, now = Date.now(), lockOptions, presentedEpoch } = {}) {
  const resolved = resolveResourceFile(root, request || {});
  return renewLeaseFile(root, resolved.file, { ttl, now, lockOptions, presentedEpoch });
}

function listAllLeaseFiles(root) {
  const dirs = [leaseCellsDir(root), leasePathsDir(root)];
  const files = [];
  for (const dir of dirs) {
    let entries;
    try {
      entries = fs.readdirSync(dir, { withFileTypes: true });
    } catch {
      continue; // directory not created yet — no leases of that kind
    }
    for (const entry of entries) {
      if (entry.isFile() && entry.name.endsWith('.json')) files.push(path.join(dir, entry.name));
    }
  }
  return files;
}

/**
 * renewLeasesBySession(root, sessionId, options) — condition E groundwork
 * (advisor consult slice 3): renews every still-present lease owned by
 * `sessionId`, one per-file lock at a time, so msn-16 can wire this into
 * bee-state-sync.mjs's heartbeat with the SAME {maxAttempts:1} try-once
 * posture reservations.mjs's renewHoldsBySession already uses (threaded
 * through here as `options.lockOptions`, unchanged shape). Deliberately NO
 * store-wide lock — each renewal only ever contends with a concurrent
 * acquire/release/renew/sweep of that SAME resource, never a sibling one. A
 * lease swept or released by someone else between the listing and this
 * call's own renew attempt is treated as already-gone (skipped, not an
 * error) — renewal can never resurrect a lease that stopped existing.
 */
export async function renewLeasesBySession(root, sessionId, { ttl = DEFAULT_TTL_SECONDS, now = Date.now(), lockOptions } = {}) {
  const session = typeof sessionId === 'string' ? sessionId.trim() : '';
  if (!session) return { ok: true, renewed: 0 };
  const files = listAllLeaseFiles(root);
  let renewed = 0;
  for (const file of files) {
    const current = readLeaseSafe(file);
    if (!current || current.session_id !== session) continue;
    try {
      await renewLeaseFile(root, file, { ttl, now, lockOptions });
      renewed += 1;
    } catch (error) {
      if (error instanceof LeaseStoreError && error.code === 'LEASE_MISSING') continue; // swept concurrently — fine, skip
      throw error;
    }
  }
  return { ok: true, renewed };
}

/**
 * sweepExpiredLeases(root, options) — delete every lease file whose
 * expires_at has passed. Per-record (each is its own fs.rmSync), never a
 * store-wide lock — matches reservations.mjs's sweepExpired in spirit
 * (TTL-only expiry) but needs none of its whole-file-rewrite machinery
 * because each lease already lives at its own path. Best-effort: a delete
 * racing a concurrent release of the same (already-expired) lease is a
 * harmless double-delete, swallowed rather than thrown, mirroring
 * reservations.mjs's own sweep tolerance.
 */
export function sweepExpiredLeases(root, { now = Date.now() } = {}) {
  const files = listAllLeaseFiles(root);
  let swept = 0;
  for (const file of files) {
    const record = readLeaseSafe(file);
    if (!record) continue; // corrupt/unreadable — never guess, skip
    if (!isLeaseExpired(record, now)) continue;
    try {
      fs.rmSync(file, { force: true });
      swept += 1;
    } catch {
      // best-effort — a delete racing a concurrent release of the same
      // already-expired lease is a harmless no-op, never a sweep failure
    }
  }
  return swept;
}

/**
 * listLeases(root) — fail-open enumeration for display/debugging, the
 * lease-store sibling of workflow-store.mjs's listWorkflows: a corrupt or
 * unreadable entry is SKIPPED (never guessed at) and reported back in
 * `skipped`. Returns `{ leases, skipped }` in directory-read order.
 */
export function listLeases(root) {
  const leases = [];
  const skipped = [];
  for (const file of listAllLeaseFiles(root)) {
    const record = readLeaseSafe(file);
    if (record) {
      leases.push(record);
    } else {
      skipped.push({ file, reason: 'unreadable or corrupt JSON' });
    }
  }
  return { leases, skipped };
}
