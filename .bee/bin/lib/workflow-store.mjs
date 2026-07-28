// workflow-store.mjs — per-workflow state records (multisession-native-5,
// CONTEXT.md D1/D6/D7: "workflow-first state").
//
// A WORKFLOW replaces the single .bee/state.json / .bee/lanes/<feature>.json
// pipeline record as the unit of coordination state. Each workflow lives at
// its own path, keyed by a GENERATED id — never the feature slug, because a
// feature can reopen or run competing attempts (D1) and identity must not
// collide with a human-chosen name:
//
//   .bee/runtime/workflows/<workflow-id>/state.json
//   { id, feature, phase, mode, plan_rev, gates, summary, next_action,
//     status, created_at }
//
// gates is a closed map of the four bee gate names (context/shape/execution/
// review) to { approved, approved_for_plan_rev } (D7: gate approval is
// scoped to a plan revision, so a plan-rev bump can invalidate exactly one
// gate on exactly one workflow, never a sibling's).
//
// status is one of active|paused|closed.
//
// Locking (C4, advisor consult slice 2, binding condition): every mutating
// verb here takes ONLY the per-workflow lock `workflow:<id>` via lock.mjs's
// withStoreLock — the SAME per-id-lock shape cells.mjs already uses for
// `cells:<id>` (updateCell, capCell, ...). This module is a LEAF, deliberately:
// it imports node builtins, fsutil.mjs, and lock.mjs ONLY. It never imports
// claims.mjs, state.mjs, or reservations.mjs, so it can never read or touch a
// session record and can never nest inside (or be nested inside) the
// 'sessions' or 'state' store locks. That is a structural guarantee, not a
// convention to remember: a caller that wants "start this workflow AND touch
// a session" must take those two locks as two separate transactions
// (advisor C4: sessions and workflow:<id> are never held together — one
// commits and releases before the other acquires). Keeping this module
// import-free of session/state code is what makes that structurally
// impossible to violate by accident, on either side.
//
// lock.mjs's sanitizeLockName hashes the raw lock name into the lock
// filename, so two distinct logical names (e.g. "workflow:aaa" vs
// "workflow:bbb") are GUARANTEED distinct lock files, never merely likely —
// see withWorkflowLock below and test_workflow_store.mjs's lock-isolation
// proof.
//
// Canonical source: this file (packages/bee/lib/workflow-store.mjs).
// Vendored verbatim to .bee/bin/lib/workflow-store.mjs and the plugin
// mirrors (.agents/, .claude/, .claude-plugin/, .codex-plugin/) by
// render_plugin_skill_trees.mjs + onboard_bee.mjs --apply. Edit ONLY this
// copy; the vendoring scripts propagate it.

import fs from 'node:fs';
import path from 'node:path';
import crypto from 'node:crypto';
import { writeJsonAtomic } from './fsutil.mjs';
import { withStoreLock } from './lock.mjs';

/** Typed refusal thrown by readWorkflow/updateWorkflow/createWorkflow — never a silent fallback. */
export class WorkflowStoreError extends Error {
  constructor(code, message) {
    super(message);
    this.name = 'WorkflowStoreError';
    this.type = 'refused';
    this.code = code;
  }
}

// The four bee gate names. Deliberately duplicated from state.mjs's exported
// GATE_NAMES rather than imported — see the module header: importing
// state.mjs here would let this leaf module chain into claims.mjs's session
// code and defeat the whole point of C4's isolation guarantee. Same
// deliberate-small-duplicate precedent as lock.mjs's envSessionId (kept in
// sync with claims.mjs's resolveSessionId by hand) and withTransientFsRetry.
export const GATE_NAMES = ['context', 'shape', 'execution', 'review'];

export const STATUS_VALUES = ['active', 'paused', 'closed'];
const DEFAULT_STATUS = 'active';

export function runtimeDir(root) {
  return path.join(root, '.bee', 'runtime');
}

export function workflowsDir(root) {
  return path.join(runtimeDir(root), 'workflows');
}

// Mirrors claims.mjs requireId / state.mjs requireLaneFeature: the id becomes
// a directory name under workflowsDir, so path separators and '..' are bad
// arguments (throw a typed refusal here — this module's own callers never
// see a bare untyped Error), while everything else is an ordinary id.
function requireWorkflowId(value) {
  if (typeof value !== 'string' || !value.trim()) {
    throw new WorkflowStoreError('WORKFLOW_INVALID_ID', 'workflow id is required.');
  }
  const id = value.trim();
  if (/[\\/]/.test(id) || id.includes('..')) {
    throw new WorkflowStoreError(
      'WORKFLOW_INVALID_ID',
      `workflow id "${id}" must be a plain id (no path separators) — it becomes a directory name under .bee/runtime/workflows/.`,
    );
  }
  return id;
}

function requireFeature(value) {
  if (typeof value !== 'string' || !value.trim()) {
    throw new WorkflowStoreError('WORKFLOW_INVALID_FEATURE', 'createWorkflow: feature is required.');
  }
  return value.trim();
}

export function workflowDir(root, id) {
  return path.join(workflowsDir(root), requireWorkflowId(id));
}

export function workflowStatePath(root, id) {
  return path.join(workflowDir(root, id), 'state.json');
}

// generateWorkflowId — a short, prefixed random id (same shape as
// backlog.mjs's generatePbiId: `p-${hex}`; here `wf-${hex}`), collision-free
// by construction (32 bits of crypto randomness — see backlog.mjs's own
// comment on why no lock is needed for id generation itself). The one
// defensive extra: regenerate on the single foreseeable collision — a
// generated id that happens to equal the CALLER'S OWN feature slug (only
// possible if the feature is itself literally shaped like "wf-xxxxxxxx").
// D1's "workflow_id is never the feature slug" is additionally enforced as a
// hard runtime check in createWorkflow below; this loop just keeps the
// default generator from ever needing that fallback to fire in practice.
function generateWorkflowId(feature) {
  const featureTrimmed = typeof feature === 'string' ? feature.trim() : null;
  let id = `wf-${crypto.randomBytes(4).toString('hex')}`;
  while (featureTrimmed !== null && id === featureTrimmed) {
    id = `wf-${crypto.randomBytes(4).toString('hex')}`;
  }
  return id;
}

function defaultGateEntry() {
  return { approved: false, approved_for_plan_rev: null };
}

function defaultGates() {
  const gates = {};
  for (const name of GATE_NAMES) gates[name] = defaultGateEntry();
  return gates;
}

// mergeGates — overlays `overrides` onto `base`, per-gate-name, one level
// deep (never trusts a caller-supplied gate entry to already carry both
// fields). Unknown gate names in overrides are kept (forward-compat: a
// future cell adding a fifth gate must not be silently dropped here), but
// every name in GATE_NAMES is always present with defaults even when
// `overrides` omits it entirely.
function mergeGates(base, overrides) {
  const merged = { ...defaultGates(), ...(base || {}) };
  if (overrides && typeof overrides === 'object' && !Array.isArray(overrides)) {
    for (const [name, value] of Object.entries(overrides)) {
      const baseEntry = merged[name] || defaultGateEntry();
      merged[name] = { ...baseEntry, ...(value && typeof value === 'object' ? value : {}) };
    }
  }
  return merged;
}

function baseWorkflowDefaults() {
  return {
    mode: null,
    phase: 'idle',
    plan_rev: 0,
    summary: '',
    next_action: '',
    status: DEFAULT_STATUS,
    // explicit-triage D1: the validated route record — {class, lane, flags[],
    // product_files, rationale, updated_at} — set via `bee state route --set`.
    // Optional, defaults to null (no route recorded yet); the whole object is
    // always replaced wholesale on update, never merged field-by-field (D4:
    // "one route per feature, updated in place, never a second record" — the
    // generic updateWorkflow/updateWorkflowAssumingLock patch mechanism
    // already gives this for free, same as every other top-level field here).
    route: null,
    // main-verifies D2: the feature-level verify record — {feature, command,
    // output_sha256, result, at} — set via `bee state feature-verify record`.
    // Optional, defaults to null (no verify recorded yet); replaced wholesale
    // on update exactly like `route` above, never merged field-by-field.
    feature_verify: null,
  };
}

/**
 * withWorkflowLock(root, id, fn, options) — run fn() holding the per-workflow
 * lock `workflow:<id>`. A thin named wrapper over lock.mjs's withStoreLock so
 * every mutating verb below (and any future caller) acquires the SAME lock
 * name for the SAME id, and so that name never accidentally drifts from
 * `workflow:<id>` at a call site. See the module header for the C4 isolation
 * guarantee this depends on: this file never acquires any OTHER named lock.
 */
export function withWorkflowLock(root, id, fn, options) {
  return withStoreLock(root, `workflow:${requireWorkflowId(id)}`, fn, options);
}

// readWorkflowRecord — the shared read/parse/validate body for readWorkflow
// and updateWorkflow's read-modify-write. `workflowId` is assumed already
// validated by the caller (both callers validate it themselves, for lock
// naming or path building, before this ever runs).
function readWorkflowRecord(root, workflowId) {
  const file = workflowStatePath(root, workflowId);
  let text;
  try {
    text = fs.readFileSync(file, 'utf8');
  } catch (err) {
    if (err && err.code === 'ENOENT') {
      throw new WorkflowStoreError(
        'WORKFLOW_MISSING',
        `readWorkflow: no workflow record at "${file}". FIX: createWorkflow first, or check the id.`,
      );
    }
    throw new WorkflowStoreError(
      'WORKFLOW_CORRUPT',
      `readWorkflow: could not read "${file}" (${err && err.code ? err.code : err}). The bee CLI refuses to guess ` +
        `at a workflow record it cannot read — that could silently clobber real state (gates, phase). ` +
        `FIX: inspect/restore the file (e.g. "git checkout -- ${path.relative(root, file)}").`,
    );
  }
  let parsed;
  try {
    parsed = JSON.parse(text);
  } catch (err) {
    throw new WorkflowStoreError(
      'WORKFLOW_CORRUPT',
      `readWorkflow: "${file}" exists but is not valid JSON (${err.message}). The bee CLI refuses to rebuild a ` +
        `workflow from defaults over a present-but-corrupt file — that would silently clobber real state (gates, ` +
        `phase) while reporting success. FIX: inspect/restore the file (e.g. "git checkout -- ${path.relative(root, file)}").`,
    );
  }
  if (!parsed || typeof parsed !== 'object' || Array.isArray(parsed)) {
    throw new WorkflowStoreError(
      'WORKFLOW_CORRUPT',
      `readWorkflow: "${file}" exists but is not a JSON object (found ${Array.isArray(parsed) ? 'an array' : typeof parsed}).`,
    );
  }
  if (parsed.id !== workflowId) {
    throw new WorkflowStoreError(
      'WORKFLOW_CORRUPT',
      `readWorkflow: "${file}" exists but its id field ("${parsed.id}") does not match the requested workflow ` +
        `"${workflowId}" — never trusted. FIX: inspect/restore the file (e.g. "git checkout -- ${path.relative(root, file)}").`,
    );
  }
  return {
    ...baseWorkflowDefaults(),
    ...parsed,
    id: workflowId,
    gates: mergeGates(defaultGates(), parsed.gates),
  };
}

/**
 * readWorkflow(root, id) — read a single workflow record. Typed refusal on
 * every failure mode, mirroring state.mjs's lane LANE_MISSING/LANE_CORRUPT
 * discipline: a missing record throws WORKFLOW_MISSING, an unreadable/
 * unparseable/shape-mismatched record throws WORKFLOW_CORRUPT. Never a
 * silent default — a workflow record carries gates, so guessing at one would
 * risk reporting an unapproved gate as approved (or vice versa).
 */
export function readWorkflow(root, id) {
  return readWorkflowRecord(root, requireWorkflowId(id));
}

/**
 * createWorkflow(root, { feature, phase, mode, plan_rev, gates, summary,
 * next_action, status, id }) — create a new workflow record. `id` defaults to
 * a fresh generateWorkflowId(feature) (mirrors claims.mjs createSession's
 * `{ id = randomUUID(), ... }` default-param shape) but may be passed
 * explicitly (tests, or a future resumption path) — EXCEPT equal to
 * `feature`, which is refused outright (WORKFLOW_INVALID_ID): D1 requires
 * workflow_id to never be the feature slug, enforced here as a hard
 * invariant rather than left as an incidental property of the default
 * generator. Runs under withWorkflowLock so two callers racing the same
 * explicit id can never both "win" a create.
 */
export function createWorkflow(
  root,
  {
    feature,
    phase = 'idle',
    mode = null,
    plan_rev = 0,
    gates,
    summary = '',
    next_action = '',
    status = DEFAULT_STATUS,
    id = generateWorkflowId(feature),
  } = {},
  options,
) {
  const workflowId = requireWorkflowId(id);
  const featureName = requireFeature(feature);
  if (workflowId === featureName) {
    throw new WorkflowStoreError(
      'WORKFLOW_INVALID_ID',
      `createWorkflow: workflow id "${workflowId}" must not equal the feature slug "${featureName}" — ids are ` +
        `generated identifiers, never feature slugs (CONTEXT.md D1: a feature can reopen or run competing ` +
        `attempts, so identity must never collide with the human-chosen name). FIX: pass an explicit id distinct ` +
        `from the feature, or omit id to let one be generated.`,
    );
  }
  if (!STATUS_VALUES.includes(status)) {
    throw new WorkflowStoreError(
      'WORKFLOW_INVALID_STATUS',
      `createWorkflow: status must be one of ${STATUS_VALUES.join('/')} (got ${JSON.stringify(status)}).`,
    );
  }
  return withWorkflowLock(
    root,
    workflowId,
    () => {
      const file = workflowStatePath(root, workflowId);
      if (fs.existsSync(file)) {
        throw new WorkflowStoreError(
          'WORKFLOW_ALREADY_EXISTS',
          `createWorkflow: a workflow record already exists at "${file}" — createWorkflow never overwrites an ` +
            `existing record. FIX: use updateWorkflow, or generate a fresh id.`,
        );
      }
      const record = {
        // main-verifies mv-4: spread baseWorkflowDefaults() FIRST so every
        // field readWorkflowRecord defaults on read (route, feature_verify,
        // and any future addition) is also present on the record written
        // here — created and read must be byte-symmetric. The explicit
        // fields below override the spread with the caller's actual values;
        // route/feature_verify have no create-time param (yet) so they keep
        // their baseWorkflowDefaults() null.
        ...baseWorkflowDefaults(),
        id: workflowId,
        feature: featureName,
        phase,
        mode,
        plan_rev,
        gates: mergeGates(defaultGates(), gates),
        summary,
        next_action,
        status,
        created_at: new Date().toISOString(),
      };
      writeJsonAtomic(file, record);
      return record;
    },
    options,
  );
}

/**
 * updateWorkflowAssumingLock(root, id, updater) — the SAME read-modify-write
 * body updateWorkflow uses, but WITHOUT acquiring `workflow:<id>` itself
 * (multisession-native-10). For a caller that ALREADY holds that lock —
 * bee.mjs's mutation verbs (state set/gate/scribing-run/advisor-ref record)
 * hold `workflow:<id>` for their WHOLE read-validate-write body via
 * withWorkflowLock so a concurrent CLI call on the SAME workflow can't race
 * a lost update, exactly the shape createWorkflow/updateWorkflow's own lock
 * would otherwise NEST inside. lock.mjs's file lock is a non-reentrant O_EXCL
 * primitive: a second acquire of the SAME lock name from the SAME process
 * would just contend against its own outer hold and eventually throw
 * LockBusyError — a self-deadlock-shaped bug, not a race. This function is
 * the escape hatch: identical merge semantics (mergeGates, protected identity
 * fields), zero locking of its own. NEVER call this without already holding
 * `workflow:<id>` for this exact `id` — every other caller must use
 * updateWorkflow below, which still self-locks as before.
 */
export function updateWorkflowAssumingLock(root, id, updater) {
  const workflowId = requireWorkflowId(id);
  const current = readWorkflowRecord(root, workflowId); // throws WORKFLOW_MISSING/WORKFLOW_CORRUPT, file untouched
  const patch = typeof updater === 'function' ? updater({ ...current }) : updater;
  if (!patch || typeof patch !== 'object' || Array.isArray(patch)) {
    throw new WorkflowStoreError(
      'WORKFLOW_INVALID_PATCH',
      'updateWorkflowAssumingLock: updater must be (or return) a plain object patch.',
    );
  }
  if (patch.status !== undefined && !STATUS_VALUES.includes(patch.status)) {
    throw new WorkflowStoreError(
      'WORKFLOW_INVALID_STATUS',
      `updateWorkflowAssumingLock: status must be one of ${STATUS_VALUES.join('/')} (got ${JSON.stringify(patch.status)}).`,
    );
  }
  const next = {
    ...current,
    ...patch,
    id: current.id,
    feature: current.feature,
    created_at: current.created_at,
    gates: mergeGates(current.gates, patch.gates),
  };
  writeJsonAtomic(workflowStatePath(root, workflowId), next);
  return next;
}

/**
 * updateWorkflow(root, id, updater, options) — read-modify-write a workflow
 * record under withWorkflowLock (the workflow-scoped sibling of cells.mjs's
 * `withStoreLock(root, \`cells:${id}\`, ...)` RMW verbs). `updater` is either
 * a plain object patch (shallow-merged onto the current record, with `gates`
 * merged per-gate-name via mergeGates) or a function `(current) => patch`
 * for callers that need to read before deciding what to write (e.g. bumping
 * plan_rev, or flipping one gate's `approved_for_plan_rev` to the CURRENT
 * plan_rev). `id`, `feature`, and `created_at` are identity fields and are
 * never touched by a patch, even if the patch happens to name them — the
 * same "never let a write silently corrupt identity" posture as readWorkflow
 * refusing to guess at a corrupt record. Thin lock-then-delegate wrapper
 * (multisession-native-10) over updateWorkflowAssumingLock above — same
 * behavior as before this cell for every existing caller.
 */
export function updateWorkflow(root, id, updater, options) {
  const workflowId = requireWorkflowId(id);
  return withWorkflowLock(root, workflowId, () => updateWorkflowAssumingLock(root, workflowId, updater), options);
}

/**
 * listWorkflows(root) — fail-open enumeration for display, the workflow
 * sibling of state.mjs's listLanes/readLane: a corrupt or unreadable entry is
 * SKIPPED (never guessed at, never thrown for the whole list) and reported
 * back in `skipped` (plus a console.warn, matching readLane's own convention)
 * so the caller can surface it without the whole listing failing. Returns
 * `{ workflows, skipped }` — `workflows` in directory-read order (no
 * particular sort; callers needing an order sort themselves).
 */
export function listWorkflows(root) {
  let entries;
  try {
    entries = fs.readdirSync(workflowsDir(root), { withFileTypes: true });
  } catch {
    return { workflows: [], skipped: [] };
  }
  const workflows = [];
  const skipped = [];
  for (const entry of entries) {
    if (!entry.isDirectory()) continue;
    const id = entry.name;
    try {
      workflows.push(readWorkflowRecord(root, id));
    } catch (err) {
      const reason = err instanceof WorkflowStoreError ? err.message : String((err && err.message) || err);
      skipped.push({ id, reason });
      console.warn(`listWorkflows: skipping unreadable workflow "${id}" — ${reason}`);
    }
  }
  return { workflows, skipped };
}
