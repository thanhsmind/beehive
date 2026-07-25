// workspace-store.mjs — the workspace registry + single-write-owner lock
// (multisession-native-19, CONTEXT.md D2/D3: "Second write session defaults
// to isolation").
//
// A WORKSPACE record is the control plane's ledger of every physical
// checkout bee knows about — the MAIN checkout plus every worktree that has
// ever been registered — keyed by its workspace id (state.mjs resolveContext's
// `workspaceId`: 'main' for the main checkout, the git-verified worktree id
// for a GRANTED linked worktree). Each record lives at its own path, the same
// one-record-per-file shape workflow-store.mjs and lease-store.mjs already
// use for the same reason (two unrelated workspaces never contend on a shared
// file at all):
//
//   controlRoot/.bee/runtime/workspaces/<id>.json
//   { id, type: 'main'|'worktree', root, branch, base_sha,
//     write_owner_session, fence_epoch, attached_sessions, created_at }
//
// `root` here is always the CALLER's already-resolved controlRoot (this
// module never calls resolveContext/controlRootFor itself — see the
// structural-isolation note below), matching workflow-store.mjs/
// lease-store.mjs/claims.mjs's own "root param, caller resolves it" contract.
//
// ─── registration vs ownership (advisor digest slice4, binding condition 4,
// F5) — the load-bearing distinction this whole module exists to keep
// separate ───────────────────────────────────────────────────────────────
//
// worktree-store.mjs's writeGrant/readGrants/decideWorktreeStore govern STORE
// TOPOLOGY: which physical `.bee/` a write lands in (main's shared control
// plane, or a granted worktree's own local store). That is orthogonal to,
// and never subsumed by, what THIS module governs: WRITE OWNERSHIP — which
// live SESSION currently holds the exclusive right to write inside a given
// workspace, regardless of where that workspace's data physically lives.
// A granted worktree with no live owner is topology-satisfied but
// ownership-open; an owned workspace whose grant has been revoked is
// ownership-satisfied but topology-denied. Neither layer ever asks the other
// a question — a caller that needs BOTH answers (may I physically write here,
// AND do I hold this workspace) composes the two independently, and test
// coverage for all four combinations lives in test_workspace_store.mjs
// (`describe('condition 4 compose')`) precisely to prove neither exists as an
// undocumented side effect of the other ("no double-deny").
//
// ─── ownership mechanics: O_EXCL/fenced, reusing lease/claims patterns ────
//
// Claiming write ownership of a workspace is a read-modify-write of that ONE
// workspace's own record, serialized under a per-workspace lock
// (`workspace:<id>` — the same withStoreLock(root, \`<kind>:<id>\`, ...) shape
// workflow-store.mjs's withWorkflowLock and cells.mjs's `cells:<id>` locks
// already use). There is no separate O_EXCL-create step the way a fresh
// lease/claim file has one (a workspace record, once registered, persists for
// the workspace's whole lifetime — ownership is a FIELD on that record, not a
// separate file) — the lock itself is what makes "read current owner, decide,
// write" atomic against a second session racing the same decision
// (`decideOwnership` below), which is the same guarantee O_EXCL gives a
// fresh-file claim.
//
// `fence_epoch` (claims.mjs `fence_epoch` / lease-store.mjs `epoch` naming
// convention): starts at 0 on registration, bumped by exactly 1 every time
// write ownership changes hands (a fresh claim, or a stale-owner reclaim) —
// never on an idempotent re-claim by the SAME already-owning session. Nothing
// in this cell wires a `presentedEpoch`-style fencing CHECK the way
// claims.mjs's renewClaimTTL/lease-store.mjs's renewLease do (no caller needs
// it yet); the counter is stamped so a future caller can, without a schema
// migration — same "forward groundwork" posture lease-store.mjs's own header
// documents for its own `kind` field.
//
// Staleness (a dead session's ownership is reclaimable): this module does
// NOT import claims.mjs to check a session's heartbeat itself — doing so
// would create exactly the import edge workflow-store.mjs's and
// lease-store.mjs's own headers go out of their way to rule out (the C4
// isolation pattern: a leaf module never reaches into session/state code, so
// it can never nest inside — or be nested inside — the 'sessions' store
// lock). Instead, `claimWriteOwnership`/`attachWorkspace` accept an OPTIONAL
// `isOwnerLive(ownerSessionId, now)` predicate; the caller builds it from
// claims.mjs's own `readSession` + `heartbeatStale` (the exact same
// DEFAULT_HEARTBEAT_STALE_SECONDS window `activeWorkers` uses — see
// test_workspace_store.mjs for the production-shaped predicate). Omitted, the
// default is the SAFE assumption (`() => true`, "the owner is live") — a
// caller that never wires liveness checking gets today's conservative
// behavior (never auto-reclaims) rather than an accidental takeover.
//
// ─── registration: idempotent, never O_EXCL-refuses on an existing record ──
//
// Unlike createWorkflow (which throws WORKFLOW_ALREADY_EXISTS on a second
// call for the same id) or a fresh lease/claim create (O_EXCL refuses on
// EEXIST), `registerWorkspace` is deliberately idempotent: a second
// register for an id that already exists returns the EXISTING record
// unchanged rather than throwing or overwriting. Two reasons, both cell-text
// requirements: (a) "Main checkout auto-registered lazily (first touch)" —
// every session-init hook invocation in the main checkout calls this, so it
// MUST be a safe no-op after the first; (b) createFeatureWorktree calling
// this after `_writeGrant` must never fail the whole worktree-create flow
// just because some earlier, now-abandoned attempt already left a workspace
// record for the same (freshly re-used) id.
//
// ─── prohibition: "no unregistered workspace gains write ownership" ───────
//
// `claimWriteOwnership`/`attachWorkspace` both REFUSE typed
// WORKSPACE_NOT_REGISTERED when no record exists for `id` — they never
// silently register-then-claim. "Main checkout auto-registered lazily" is a
// property of WHEN registerWorkspace gets called (the session-init hook
// calls it before ever touching ownership), never a fallback inside the
// ownership functions themselves — that asymmetry is what keeps a bogus or
// never-created worktree id from ever acquiring ownership by accident.
//
// Structural isolation (C4 pattern, workflow-store.mjs/lease-store.mjs
// precedent): this module imports ONLY node builtins, fsutil.mjs, and
// lock.mjs. It never imports claims.mjs, state.mjs, worktree-store.mjs, or
// reservations.mjs.
//
// Canonical source: this file (packages/bee/lib/workspace-store.mjs).
// Vendored verbatim to .bee/bin/lib/workspace-store.mjs and the plugin
// mirrors (.agents/, .claude/, .claude-plugin/, .codex-plugin/) by
// render_plugin_skill_trees.mjs + onboard_bee.mjs --apply. Edit ONLY this
// copy; the vendoring scripts propagate it.

import fs from 'node:fs';
import path from 'node:path';
import { writeJsonAtomic } from './fsutil.mjs';
import { withStoreLock } from './lock.mjs';

/** Typed refusal thrown by every mutating/reading verb below — never a silent fallback. */
export class WorkspaceStoreError extends Error {
  constructor(code, message, details) {
    super(message);
    this.name = 'WorkspaceStoreError';
    this.type = 'refused';
    this.code = code;
    if (details && typeof details === 'object') {
      if (details.id !== undefined) this.id = details.id;
      if (details.holder !== undefined) this.holder = details.holder;
    }
  }
}

export const TYPE_VALUES = ['main', 'worktree'];

export function runtimeDir(root) {
  return path.join(root, '.bee', 'runtime');
}

export function workspacesDir(root) {
  return path.join(runtimeDir(root), 'workspaces');
}

// Mirrors workflow-store.mjs's requireWorkflowId / lease-store.mjs's
// requireCellId: an id becomes a filename under workspacesDir(root), so path
// separators and '..' are refused rather than allowed to escape the
// directory.
function requireWorkspaceId(value) {
  if (typeof value !== 'string' || !value.trim()) {
    throw new WorkspaceStoreError('WORKSPACE_INVALID_ID', 'workspace id is required.');
  }
  const id = value.trim();
  if (/[\\/]/.test(id) || id.includes('..')) {
    throw new WorkspaceStoreError(
      'WORKSPACE_INVALID_ID',
      `workspace id "${id}" must be a plain id (no path separators) — it becomes a filename under .bee/runtime/workspaces/.`,
    );
  }
  return id;
}

function requireType(value) {
  if (!TYPE_VALUES.includes(value)) {
    throw new WorkspaceStoreError(
      'WORKSPACE_INVALID_TYPE',
      `workspace type must be one of ${TYPE_VALUES.join('/')} (got ${JSON.stringify(value)}).`,
    );
  }
  return value;
}

function requireWorkspaceRoot(value) {
  if (typeof value !== 'string' || !value.trim()) {
    throw new WorkspaceStoreError('WORKSPACE_INVALID_ROOT', 'workspace root (physical checkout path) is required.');
  }
  return value;
}

export function workspacePath(root, id) {
  return path.join(workspacesDir(root), `${requireWorkspaceId(id)}.json`);
}

/**
 * withWorkspaceLock(root, id, fn, options) — run fn() holding the
 * per-workspace lock `workspace:<id>`. Same named-wrapper shape as
 * workflow-store.mjs's withWorkflowLock: every mutating verb below acquires
 * this SAME lock name for the SAME id, so it can never accidentally drift at
 * a call site, and lock.mjs's sanitizeLockName hashes it into a distinct
 * lock file per id (never merely likely-distinct).
 */
export function withWorkspaceLock(root, id, fn, options) {
  return withStoreLock(root, `workspace:${requireWorkspaceId(id)}`, fn, options);
}

function readWorkspaceRecord(root, workspaceId) {
  const file = workspacePath(root, workspaceId);
  let text;
  try {
    text = fs.readFileSync(file, 'utf8');
  } catch (err) {
    if (err && err.code === 'ENOENT') {
      throw new WorkspaceStoreError(
        'WORKSPACE_MISSING',
        `readWorkspace: no workspace record at "${file}". FIX: registerWorkspace first, or check the id.`,
        { id: workspaceId },
      );
    }
    throw new WorkspaceStoreError(
      'WORKSPACE_CORRUPT',
      `readWorkspace: could not read "${file}" (${err && err.code ? err.code : err}). The bee CLI refuses to guess ` +
        `at a workspace record it cannot read — that could silently misreport write ownership. ` +
        `FIX: inspect/restore the file (e.g. "git checkout -- ${path.relative(root, file)}").`,
      { id: workspaceId },
    );
  }
  let parsed;
  try {
    parsed = JSON.parse(text);
  } catch (err) {
    throw new WorkspaceStoreError(
      'WORKSPACE_CORRUPT',
      `readWorkspace: "${file}" exists but is not valid JSON (${err.message}).`,
      { id: workspaceId },
    );
  }
  if (!parsed || typeof parsed !== 'object' || Array.isArray(parsed)) {
    throw new WorkspaceStoreError(
      'WORKSPACE_CORRUPT',
      `readWorkspace: "${file}" exists but is not a JSON object (found ${Array.isArray(parsed) ? 'an array' : typeof parsed}).`,
      { id: workspaceId },
    );
  }
  if (parsed.id !== workspaceId) {
    throw new WorkspaceStoreError(
      'WORKSPACE_CORRUPT',
      `readWorkspace: "${file}" exists but its id field ("${parsed.id}") does not match the requested workspace "${workspaceId}" — never trusted.`,
      { id: workspaceId },
    );
  }
  return {
    write_owner_session: null,
    fence_epoch: 0,
    attached_sessions: [],
    branch: null,
    base_sha: null,
    ...parsed,
    id: workspaceId,
  };
}

/** readWorkspace(root, id) — typed refusal on every failure mode (WORKSPACE_MISSING/WORKSPACE_CORRUPT), never a silent default (a workspace record carries write ownership — guessing at one could misreport who may write). */
export function readWorkspace(root, id) {
  return readWorkspaceRecord(root, requireWorkspaceId(id));
}

function writeWorkspaceFileAtomic(root, id, record) {
  fs.mkdirSync(workspacesDir(root), { recursive: true });
  writeJsonAtomic(workspacePath(root, id), record);
  return record;
}

/**
 * registerWorkspace(root, { id, type, root: workspaceRoot, branch, base_sha }, options)
 * — idempotent create. If a record already exists for `id`, it is returned
 * UNCHANGED (never overwritten, never an error) — see the module header for
 * why this must be idempotent rather than O_EXCL-refuse-on-exists like
 * createWorkflow/a fresh lease. A fresh registration starts with
 * `write_owner_session: null`, `fence_epoch: 0`, `attached_sessions: []`.
 */
export async function registerWorkspace(root, { id, type, root: workspaceRoot, branch = null, base_sha = null } = {}, options) {
  const workspaceId = requireWorkspaceId(id);
  const workspaceType = requireType(type);
  const resolvedRoot = requireWorkspaceRoot(workspaceRoot);
  return withWorkspaceLock(
    root,
    workspaceId,
    () => {
      const file = workspacePath(root, workspaceId);
      if (fs.existsSync(file)) {
        return readWorkspaceRecord(root, workspaceId);
      }
      const record = {
        id: workspaceId,
        type: workspaceType,
        root: resolvedRoot,
        branch: typeof branch === 'string' && branch ? branch : null,
        base_sha: typeof base_sha === 'string' && base_sha ? base_sha : null,
        write_owner_session: null,
        fence_epoch: 0,
        attached_sessions: [],
        created_at: new Date().toISOString(),
      };
      return writeWorkspaceFileAtomic(root, workspaceId, record);
    },
    options,
  );
}

/**
 * unregisterWorkspace(root, id, options) — delete the workspace record.
 * Idempotent: unregistering an absent workspace is `{ ok: true, removed:
 * false }`, never an error (mirrors lease-store.mjs's releaseLease
 * tolerance of "nothing on disk").
 */
export async function unregisterWorkspace(root, id, options) {
  const workspaceId = requireWorkspaceId(id);
  return withWorkspaceLock(
    root,
    workspaceId,
    () => {
      const file = workspacePath(root, workspaceId);
      try {
        fs.rmSync(file, { force: false });
        return { ok: true, removed: true };
      } catch (err) {
        if (err && err.code === 'ENOENT') return { ok: true, removed: false };
        throw err;
      }
    },
    options,
  );
}

/**
 * listWorkspaces(root) — fail-open enumeration, the workspace-store sibling
 * of workflow-store.mjs's listWorkflows / lease-store.mjs's listLeases: a
 * corrupt or unreadable entry is SKIPPED (never guessed at) and reported in
 * `skipped`. Returns `{ workspaces, skipped }`.
 */
export function listWorkspaces(root) {
  let entries;
  try {
    entries = fs.readdirSync(workspacesDir(root), { withFileTypes: true });
  } catch {
    return { workspaces: [], skipped: [] };
  }
  const workspaces = [];
  const skipped = [];
  for (const entry of entries) {
    if (!entry.isFile() || !entry.name.endsWith('.json')) continue;
    const id = entry.name.slice(0, -'.json'.length);
    try {
      workspaces.push(readWorkspaceRecord(root, id));
    } catch (err) {
      const reason = err instanceof WorkspaceStoreError ? err.message : String((err && err.message) || err);
      skipped.push({ id, reason });
      console.warn(`listWorkspaces: skipping unreadable workspace "${id}" — ${reason}`);
    }
  }
  return { workspaces, skipped };
}

// decideOwnership — PURE decision core shared by claimWriteOwnership and
// attachWorkspace, run under the SAME withWorkspaceLock hold both acquire.
// `isOwnerLive` defaults to the SAFE assumption ("the current owner is
// live") when the caller supplies none — see module header.
function decideOwnership(record, sessionId, { now, isOwnerLive }) {
  const owner = record.write_owner_session;
  if (!owner || owner === sessionId) {
    return { outcome: owner === sessionId ? 'already_owner' : 'become_owner' };
  }
  const live = typeof isOwnerLive === 'function' ? Boolean(isOwnerLive(owner, now)) : true;
  if (!live) return { outcome: 'reclaim' };
  return { outcome: 'blocked', owner };
}

function applyOwnershipTakeover(record, sessionId, now) {
  const attached = Array.isArray(record.attached_sessions) ? record.attached_sessions.filter((s) => s !== sessionId) : [];
  return {
    ...record,
    write_owner_session: sessionId,
    fence_epoch: (Number.isFinite(record.fence_epoch) ? record.fence_epoch : 0) + 1,
    owner_claimed_at: new Date(now).toISOString(),
    attached_sessions: attached,
  };
}

/**
 * claimWriteOwnership(root, id, sessionId, { now, isOwnerLive }, options) —
 * the STRICT primitive: become the workspace's write owner, or throw a typed
 * refusal. Never falls back to a read-only attach — see attachWorkspace for
 * the forgiving wrapper. Refuses WORKSPACE_NOT_REGISTERED when no record
 * exists (the prohibition: "no unregistered workspace gains write
 * ownership"). Refuses typed WORKSPACE_OWNED naming the current holder when
 * a DIFFERENT session already owns it and `isOwnerLive` says that owner is
 * still live (or no predicate was given at all — the safe default).
 * Idempotent for the SAME session re-claiming its own ownership (no epoch
 * bump, `reclaimed: false`).
 */
export async function claimWriteOwnership(root, id, sessionId, { now = Date.now(), isOwnerLive } = {}, options) {
  const workspaceId = requireWorkspaceId(id);
  const session = typeof sessionId === 'string' && sessionId.trim() ? sessionId.trim() : null;
  if (!session) {
    throw new WorkspaceStoreError('WORKSPACE_INVALID_SESSION', 'claimWriteOwnership: sessionId is required.');
  }
  return withWorkspaceLock(
    root,
    workspaceId,
    () => {
      let record;
      try {
        record = readWorkspaceRecord(root, workspaceId);
      } catch (err) {
        if (err instanceof WorkspaceStoreError && err.code === 'WORKSPACE_MISSING') {
          throw new WorkspaceStoreError(
            'WORKSPACE_NOT_REGISTERED',
            `claimWriteOwnership: workspace "${workspaceId}" has never been registered — no unregistered workspace may gain write ownership. FIX: registerWorkspace first (worktree create/merge and the main-checkout first-touch path both call this automatically).`,
            { id: workspaceId },
          );
        }
        throw err;
      }
      const decision = decideOwnership(record, session, { now, isOwnerLive });
      if (decision.outcome === 'already_owner') {
        return { ok: true, owner: true, reclaimed: false, record };
      }
      if (decision.outcome === 'blocked') {
        throw new WorkspaceStoreError(
          'WORKSPACE_OWNED',
          `claimWriteOwnership: workspace "${workspaceId}" is already write-owned by session "${decision.owner}".`,
          { id: workspaceId, holder: decision.owner },
        );
      }
      const next = applyOwnershipTakeover(record, session, now);
      writeWorkspaceFileAtomic(root, workspaceId, next);
      return { ok: true, owner: true, reclaimed: decision.outcome === 'reclaim', record: next };
    },
    options,
  );
}

/**
 * attachWorkspace(root, id, sessionId, { now, isOwnerLive }, options) — the
 * FORGIVING wrapper a session actually joining a workspace should call:
 * becomes owner exactly like claimWriteOwnership when ownership is free (or
 * reclaimable), but when a DIFFERENT session already holds live ownership,
 * this records a READ-ONLY ATTACH (sessionId appended to
 * `attached_sessions`, deduplicated, idempotent) instead of throwing.
 * Minimal recorded shape, documented here per the cell's own "pick minimal
 * shape, document" instruction: `attached_sessions` is a plain array of
 * session ids on the workspace record itself — no separate per-attach file,
 * since attach is a lightweight, frequently-changing fact, not an
 * independent resource with its own lifecycle. Still refuses typed
 * WORKSPACE_NOT_REGISTERED on an unregistered id (the prohibition applies
 * here too — attach never registers).
 */
export async function attachWorkspace(root, id, sessionId, { now = Date.now(), isOwnerLive } = {}, options) {
  const workspaceId = requireWorkspaceId(id);
  const session = typeof sessionId === 'string' && sessionId.trim() ? sessionId.trim() : null;
  if (!session) {
    throw new WorkspaceStoreError('WORKSPACE_INVALID_SESSION', 'attachWorkspace: sessionId is required.');
  }
  return withWorkspaceLock(
    root,
    workspaceId,
    () => {
      let record;
      try {
        record = readWorkspaceRecord(root, workspaceId);
      } catch (err) {
        if (err instanceof WorkspaceStoreError && err.code === 'WORKSPACE_MISSING') {
          throw new WorkspaceStoreError(
            'WORKSPACE_NOT_REGISTERED',
            `attachWorkspace: workspace "${workspaceId}" has never been registered — attach never auto-registers. FIX: registerWorkspace first.`,
            { id: workspaceId },
          );
        }
        throw err;
      }
      const decision = decideOwnership(record, session, { now, isOwnerLive });
      if (decision.outcome === 'already_owner') {
        return { ok: true, role: 'owner', reclaimed: false, record };
      }
      if (decision.outcome === 'become_owner' || decision.outcome === 'reclaim') {
        const next = applyOwnershipTakeover(record, session, now);
        writeWorkspaceFileAtomic(root, workspaceId, next);
        return { ok: true, role: 'owner', reclaimed: decision.outcome === 'reclaim', record: next };
      }
      // blocked: record a read-only attach rather than refuse.
      const existing = Array.isArray(record.attached_sessions) ? record.attached_sessions : [];
      const attached_sessions = existing.includes(session) ? existing : [...existing, session];
      const next = { ...record, attached_sessions };
      writeWorkspaceFileAtomic(root, workspaceId, next);
      return { ok: true, role: 'read-only', write_owner_session: decision.owner, record: next };
    },
    options,
  );
}

/**
 * releaseWriteOwnership(root, id, sessionId, options) — same-session-only
 * release: clears `write_owner_session` (and bumps `fence_epoch`, matching
 * every other ownership-change in this module) only when `sessionId`
 * currently holds it; a non-owner calling this is a no-op (`released:
 * false`), never an error — mirrors lease-store.mjs's releaseLease
 * tolerance. Not exercised by the cell's own must-haves directly, but the
 * natural symmetric counterpart to claimWriteOwnership/attachWorkspace: a
 * session that finishes its work releases explicitly rather than always
 * waiting out the heartbeat-staleness window for the next owner.
 */
export async function releaseWriteOwnership(root, id, sessionId, options) {
  const workspaceId = requireWorkspaceId(id);
  const session = typeof sessionId === 'string' && sessionId.trim() ? sessionId.trim() : null;
  if (!session) {
    throw new WorkspaceStoreError('WORKSPACE_INVALID_SESSION', 'releaseWriteOwnership: sessionId is required.');
  }
  return withWorkspaceLock(
    root,
    workspaceId,
    () => {
      const record = readWorkspaceRecord(root, workspaceId);
      if (record.write_owner_session !== session) {
        return { ok: true, released: false, record };
      }
      const next = {
        ...record,
        write_owner_session: null,
        fence_epoch: (Number.isFinite(record.fence_epoch) ? record.fence_epoch : 0) + 1,
      };
      writeWorkspaceFileAtomic(root, workspaceId, next);
      return { ok: true, released: true, record: next };
    },
    options,
  );
}
