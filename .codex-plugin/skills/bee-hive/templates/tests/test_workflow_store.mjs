#!/usr/bin/env node
// test_workflow_store.mjs — lib/workflow-store.mjs contract tests
// (multisession-native-5, CONTEXT.md D1/D6/D7). Same PASS/FAIL/exit-1
// contract as every other suite here — see scripts/lib/test-fixture.mjs.
//
// Covers: create/read/update/list round-trip, workflow_id never equal to the
// feature slug, typed corrupt/missing refusals (never a silent default),
// per-workflow lock isolation (C4: two workflows never contend on the same
// lock file — proven deterministically via an externally-held lock + a
// single-attempt update, no sleeps/real threads needed since
// acquireStoreLockOnceSync gives a REAL held lock synchronously within this
// same process), and a static source-scan proving this module never imports
// session/state code or names the 'sessions'/'state' locks (the structural
// half of C4's "never held together" guarantee).

import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { check, assert, assertThrows, assertRejects, printSummaryAndExit } from '../../../../scripts/lib/test-fixture.mjs';
import { writeJsonAtomic } from '../lib/fsutil.mjs';
import { acquireStoreLockOnceSync, lockFilePath } from '../lib/lock.mjs';
import {
  createWorkflow,
  readWorkflow,
  updateWorkflow,
  updateWorkflowAssumingLock,
  listWorkflows,
  withWorkflowLock,
  workflowStatePath,
  workflowsDir,
  GATE_NAMES,
  STATUS_VALUES,
  WorkflowStoreError,
} from '../lib/workflow-store.mjs';

function makeRoot() {
  return fs.mkdtempSync(path.join(os.tmpdir(), 'bee-workflow-store-'));
}

// Deep-equality via key-sorted JSON — object key INSERTION ORDER differs
// between createWorkflow's literal and readWorkflow's defaults-then-spread
// merge (e.g. `id` lands first vs. last), which is not a meaningful part of
// this module's contract; only the field values are.
function canon(value) {
  if (Array.isArray(value)) return value.map(canon);
  if (value && typeof value === 'object') {
    const out = {};
    for (const key of Object.keys(value).sort()) out[key] = canon(value[key]);
    return out;
  }
  return value;
}
function deepEqual(a, b) {
  return JSON.stringify(canon(a)) === JSON.stringify(canon(b));
}

// ─── create/read/update/list round-trip ─────────────────────────────────────

await check('createWorkflow writes .bee/runtime/workflows/<id>/state.json with the full schema; readWorkflow round-trips it', async () => {
  const root = makeRoot();
  try {
    const created = await createWorkflow(root, { feature: 'demo-feature' });
    assert(typeof created.id === 'string' && created.id.length > 0, 'workflow record carries a generated id');
    assert(created.feature === 'demo-feature', 'workflow record carries the feature');
    assert(created.phase === 'idle', 'default phase is idle');
    assert(created.mode === null, 'default mode is null');
    assert(created.plan_rev === 0, 'default plan_rev is 0');
    assert(created.status === 'active', 'default status is active');
    assert(typeof created.created_at === 'string' && !Number.isNaN(Date.parse(created.created_at)), 'created_at is a timestamp');
    for (const name of GATE_NAMES) {
      assert(created.gates[name] && created.gates[name].approved === false, `gate "${name}" defaults unapproved`);
      assert(created.gates[name].approved_for_plan_rev === null, `gate "${name}" defaults approved_for_plan_rev null`);
    }
    assert(fs.existsSync(workflowStatePath(root, created.id)), 'record exists on disk at the documented path');

    const read = readWorkflow(root, created.id);
    assert(deepEqual(read, created), `readWorkflow round-trips createWorkflow's fields — got ${JSON.stringify(read)} vs ${JSON.stringify(created)}`);
  } finally {
    fs.rmSync(root, { recursive: true, force: true });
  }
});

await check('updateWorkflow read-modify-writes under lock; identity fields (id/feature/created_at) survive a patch attempt', async () => {
  const root = makeRoot();
  try {
    const created = await createWorkflow(root, { feature: 'demo-feature' });
    const updated = await updateWorkflow(root, created.id, {
      phase: 'planning',
      plan_rev: 1,
      summary: 'shaping the work',
      // an attacker/bug patch trying to rename identity — must be ignored
      id: 'not-the-real-id',
      feature: 'someone-elses-feature',
      created_at: '1999-01-01T00:00:00.000Z',
    });
    assert(updated.phase === 'planning', 'phase updated');
    assert(updated.plan_rev === 1, 'plan_rev updated');
    assert(updated.summary === 'shaping the work', 'summary updated');
    assert(updated.id === created.id, 'id is immutable across a patch');
    assert(updated.feature === created.feature, 'feature is immutable across a patch');
    assert(updated.created_at === created.created_at, 'created_at is immutable across a patch');

    const persisted = readWorkflow(root, created.id);
    assert(persisted.phase === 'planning' && persisted.plan_rev === 1, 'the write actually persisted to disk');

    // function-shaped updater, reading current state to decide the patch —
    // the shape D7's later gate-approval cell (msn-9) will need.
    const gateFlip = await updateWorkflow(root, created.id, (current) => ({
      gates: { execution: { approved: true, approved_for_plan_rev: current.plan_rev } },
    }));
    assert(gateFlip.gates.execution.approved === true, 'function updater can flip one gate');
    assert(gateFlip.gates.execution.approved_for_plan_rev === 1, 'function updater sees the current record (plan_rev=1)');
    assert(gateFlip.gates.context.approved === false, 'flipping one gate never touches a sibling gate');
  } finally {
    fs.rmSync(root, { recursive: true, force: true });
  }
});

await check('listWorkflows enumerates every created workflow; tolerant of an unreadable entry (skip+report, never throws)', async () => {
  const root = makeRoot();
  try {
    const a = await createWorkflow(root, { feature: 'feature-a' });
    const b = await createWorkflow(root, { feature: 'feature-b' });

    const before = listWorkflows(root);
    assert(before.workflows.length === 2, `expected 2 workflows, got ${before.workflows.length}`);
    assert(before.skipped.length === 0, 'nothing skipped when every record is healthy');
    const ids = before.workflows.map((w) => w.id).sort();
    assert(JSON.stringify(ids) === JSON.stringify([a.id, b.id].sort()), 'listWorkflows returns exactly the created ids');

    // corrupt b's record directly on disk (never through the API) and prove
    // listWorkflows degrades gracefully instead of throwing for the whole call.
    fs.writeFileSync(workflowStatePath(root, b.id), '{not json', 'utf8');
    const after = listWorkflows(root);
    assert(after.workflows.length === 1 && after.workflows[0].id === a.id, 'the healthy workflow is still listed');
    assert(after.skipped.length === 1 && after.skipped[0].id === b.id, 'the corrupt workflow is reported in skipped, not silently dropped');
    assert(typeof after.skipped[0].reason === 'string' && after.skipped[0].reason.length > 0, 'skipped entry carries a reason');
  } finally {
    fs.rmSync(root, { recursive: true, force: true });
  }
});

await check('listWorkflows on an absent .bee/runtime/workflows/ directory returns an empty, non-throwing result', () => {
  const root = makeRoot();
  try {
    const result = listWorkflows(root);
    assert(Array.isArray(result.workflows) && result.workflows.length === 0, 'no workflows dir -> empty workflows array');
    assert(Array.isArray(result.skipped) && result.skipped.length === 0, 'no workflows dir -> empty skipped array');
    assert(!fs.existsSync(workflowsDir(root)), 'listWorkflows never creates the directory as a side effect of listing');
  } finally {
    fs.rmSync(root, { recursive: true, force: true });
  }
});

// ─── workflow_id is never the feature slug (D1) ─────────────────────────────

await check('workflow_id is generated and distinct from the feature slug by default; an explicit id equal to feature is refused', async () => {
  const root = makeRoot();
  try {
    const created = await createWorkflow(root, { feature: 'checkout-flow' });
    assert(created.id !== created.feature, 'generated workflow_id must never equal the feature slug');
    assert(/^wf-/.test(created.id), 'generated workflow_id carries the documented wf- prefix');

    await assertRejects(
      () => createWorkflow(root, { feature: 'checkout-flow', id: 'checkout-flow' }),
      'must not equal the feature slug',
      'createWorkflow refuses an explicit id identical to the feature',
    );
    assert(
      !fs.existsSync(path.join(workflowsDir(root), 'checkout-flow')),
      'the refused create must leave no directory behind',
    );
  } finally {
    fs.rmSync(root, { recursive: true, force: true });
  }
});

await check('createWorkflow accepts an explicit id distinct from feature; refuses to overwrite an existing record at that id', async () => {
  const root = makeRoot();
  try {
    const created = await createWorkflow(root, { feature: 'checkout-flow', id: 'wf-fixed-1' });
    assert(created.id === 'wf-fixed-1', 'explicit id is honored');
    await assertRejects(
      () => createWorkflow(root, { feature: 'checkout-flow', id: 'wf-fixed-1' }),
      'already exists',
      'createWorkflow refuses to overwrite an existing record at the same id',
    );
  } finally {
    fs.rmSync(root, { recursive: true, force: true });
  }
});

// ─── typed refusals — never a silent default over a real record ────────────

await check('readWorkflow: missing record throws typed WORKFLOW_MISSING; corrupt/mismatched record throws typed WORKFLOW_CORRUPT', async () => {
  const root = makeRoot();
  try {
    assertThrows(() => readWorkflow(root, 'wf-ghost'), 'no workflow record', 'missing workflow throws');
    try {
      readWorkflow(root, 'wf-ghost');
      assert(false, 'expected readWorkflow to throw for a missing record');
    } catch (err) {
      assert(err instanceof WorkflowStoreError, 'missing-record error is a WorkflowStoreError');
      assert(err.code === 'WORKFLOW_MISSING', `expected code WORKFLOW_MISSING, got ${err.code}`);
    }

    const created = await createWorkflow(root, { feature: 'demo-feature' });

    fs.writeFileSync(workflowStatePath(root, created.id), '{not valid json', 'utf8');
    try {
      readWorkflow(root, created.id);
      assert(false, 'expected readWorkflow to throw for malformed JSON');
    } catch (err) {
      assert(err instanceof WorkflowStoreError && err.code === 'WORKFLOW_CORRUPT', `malformed JSON expected WORKFLOW_CORRUPT, got ${err && err.code}`);
    }

    writeJsonAtomic(workflowStatePath(root, created.id), ['not', 'an', 'object']);
    try {
      readWorkflow(root, created.id);
      assert(false, 'expected readWorkflow to throw for a non-object record');
    } catch (err) {
      assert(err instanceof WorkflowStoreError && err.code === 'WORKFLOW_CORRUPT', `array record expected WORKFLOW_CORRUPT, got ${err && err.code}`);
    }

    writeJsonAtomic(workflowStatePath(root, created.id), { ...created, id: 'some-other-id' });
    try {
      readWorkflow(root, created.id);
      assert(false, 'expected readWorkflow to throw when the record id does not match the requested id');
    } catch (err) {
      assert(err instanceof WorkflowStoreError && err.code === 'WORKFLOW_CORRUPT', `id-mismatch expected WORKFLOW_CORRUPT, got ${err && err.code}`);
    }
  } finally {
    fs.rmSync(root, { recursive: true, force: true });
  }
});

await check('updateWorkflow propagates the same typed refusals as readWorkflow, leaving a corrupt file untouched', async () => {
  const root = makeRoot();
  try {
    await assertRejects(
      () => updateWorkflow(root, 'wf-ghost', { phase: 'planning' }),
      'no workflow record',
      'updateWorkflow refuses a missing workflow',
    );

    const created = await createWorkflow(root, { feature: 'demo-feature' });
    fs.writeFileSync(workflowStatePath(root, created.id), '{not valid json', 'utf8');
    const before = fs.readFileSync(workflowStatePath(root, created.id), 'utf8');
    await assertRejects(
      () => updateWorkflow(root, created.id, { phase: 'planning' }),
      'not valid json',
      'updateWorkflow refuses a corrupt record instead of clobbering it with a fresh default',
    );
    const after = fs.readFileSync(workflowStatePath(root, created.id), 'utf8');
    assert(before === after, 'a refused update never touches the corrupt file on disk');
  } finally {
    fs.rmSync(root, { recursive: true, force: true });
  }
});

// ─── updateWorkflowAssumingLock (multisession-native-10): the same
// read-modify-write body as updateWorkflow, but for a caller that ALREADY
// holds workflow:<id> itself (bee.mjs's mutation verbs — see this function's
// own doc comment). Same merge/refusal contract, MINUS locking of its own —
// proven below by the exact inverse of updateWorkflow's own lock-isolation
// proof further down this file: updateWorkflow is DENIED while workflow:<id>
// is held externally (it tries to acquire that same lock itself), while
// updateWorkflowAssumingLock succeeds through the SAME held lock, because it
// never asks for it.

await check('updateWorkflowAssumingLock: same merge semantics and typed refusals as updateWorkflow (missing/corrupt propagate, valid patch merges gates per-name)', async () => {
  const root = makeRoot();
  try {
    assertThrows(
      () => updateWorkflowAssumingLock(root, 'wf-ghost', { phase: 'planning' }),
      'no workflow record',
      'updateWorkflowAssumingLock refuses a missing workflow',
    );

    const created = await createWorkflow(root, { feature: 'demo-feature-assuming-lock' });
    fs.writeFileSync(workflowStatePath(root, created.id), '{not valid json', 'utf8');
    const before = fs.readFileSync(workflowStatePath(root, created.id), 'utf8');
    assertThrows(
      () => updateWorkflowAssumingLock(root, created.id, { phase: 'planning' }),
      'not valid json',
      'updateWorkflowAssumingLock refuses a corrupt record instead of clobbering it',
    );
    const after = fs.readFileSync(workflowStatePath(root, created.id), 'utf8');
    assert(before === after, 'a refused update never touches the corrupt file on disk');

    const created2 = await createWorkflow(root, { feature: 'demo-feature-assuming-lock-2', gates: { execution: { approved: true, approved_for_plan_rev: 1 } } });
    const updated = updateWorkflowAssumingLock(root, created2.id, { phase: 'planning', gates: { shape: { approved: true } } });
    assert(updated.phase === 'planning', 'patch applies');
    assert(updated.gates.execution.approved === true && updated.gates.execution.approved_for_plan_rev === 1, 'an untouched gate name is preserved by mergeGates, exactly like updateWorkflow');
    assert(updated.gates.shape.approved === true, 'the patched gate name merges in');
    assert(updated.id === created2.id && updated.feature === created2.feature, 'identity fields (id/feature) are protected, exactly like updateWorkflow');
  } finally {
    fs.rmSync(root, { recursive: true, force: true });
  }
});

await check('updateWorkflowAssumingLock (multisession-native-10, C4): succeeds through an externally-held workflow:<id> lock that DENIES updateWorkflow itself — proves it takes no lock of its own, the exact property that makes it safe for a caller already holding that lock', async () => {
  const root = makeRoot();
  try {
    const created = await createWorkflow(root, { feature: 'assuming-lock-deadlock-proof' });
    const held = acquireStoreLockOnceSync(root, `workflow:${created.id}`);
    assert(held.acquired, 'precondition: test can hold workflow:<id> directly');
    try {
      // Negative control: the SELF-LOCKING form really is denied here — this
      // is exactly the shape that would deadlock if bee.mjs called it from
      // inside its own withWorkflowLock hold instead of the assuming-lock form.
      await assertRejects(
        () => updateWorkflow(root, created.id, { phase: 'planning' }, { maxAttempts: 1 }),
        'busy',
        'updateWorkflow (self-locking) must be denied while the same lock is already held',
      );
      // The real proof: the assuming-lock form succeeds through the SAME held
      // lock, because it never tries to acquire it.
      const updated = updateWorkflowAssumingLock(root, created.id, { phase: 'planning' });
      assert(updated.phase === 'planning', `updateWorkflowAssumingLock must succeed while workflow:<id> is externally held, got ${JSON.stringify(updated)}`);
    } finally {
      held.release();
    }
  } finally {
    fs.rmSync(root, { recursive: true, force: true });
  }
});

await check('status is a closed enum (active|paused|closed); an invalid status is refused on create and on update', async () => {
  const root = makeRoot();
  try {
    assert(STATUS_VALUES.join(',') === 'active,paused,closed', `expected active/paused/closed, got ${STATUS_VALUES.join(',')}`);
    await assertRejects(
      () => createWorkflow(root, { feature: 'demo-feature', status: 'bogus' }),
      'status must be one of',
      'createWorkflow refuses an invalid status',
    );
    const created = await createWorkflow(root, { feature: 'demo-feature' });
    await assertRejects(
      () => updateWorkflow(root, created.id, { status: 'bogus' }),
      'status must be one of',
      'updateWorkflow refuses an invalid status',
    );
    const paused = await updateWorkflow(root, created.id, { status: 'paused' });
    assert(paused.status === 'paused', 'a valid status transition is accepted');
  } finally {
    fs.rmSync(root, { recursive: true, force: true });
  }
});

// ─── C4: per-workflow lock isolation — never a shared/global lock ──────────

await check('workflow:<id> lock names hash to DISTINCT lock files for distinct ids (sanitizeLockName never collapses them)', () => {
  const root = makeRoot();
  try {
    const fileA = lockFilePath(root, 'workflow:wf-aaaaaaaa');
    const fileB = lockFilePath(root, 'workflow:wf-bbbbbbbb');
    assert(fileA !== fileB, 'two distinct workflow ids must produce two distinct lock file paths');
  } finally {
    fs.rmSync(root, { recursive: true, force: true });
  }
});

await check(
  'two workflows never contend: holding workflow A\'s lock externally does not block a single-attempt update of workflow B (deterministic — no sleeps, real held lock)',
  async () => {
    const root = makeRoot();
    try {
      const a = await createWorkflow(root, { feature: 'feature-a' });
      const b = await createWorkflow(root, { feature: 'feature-b' });

      const held = acquireStoreLockOnceSync(root, `workflow:${a.id}`);
      assert(held.acquired, 'precondition: test can acquire workflow A\'s lock directly');
      try {
        // Negative control first: proves the harness itself is meaningful —
        // a same-id update against an externally-held lock file MUST see
        // contention with a single-attempt (maxAttempts: 1) call, or this
        // whole test would be vacuous (see test_store_lock.mjs's own
        // falsifiability note for the same discipline).
        await assertRejects(
          () => updateWorkflow(root, a.id, { phase: 'planning' }, { maxAttempts: 1 }),
          'busy',
          'same-id update sees LOCK_BUSY while A\'s lock is externally held (sanity control)',
        );

        // The actual C4 proof: workflow B updates successfully in a SINGLE
        // attempt while A's lock is still held — different lock files, zero
        // contention, no retry needed.
        const updatedB = await updateWorkflow(root, b.id, { phase: 'planning' }, { maxAttempts: 1 });
        assert(updatedB.phase === 'planning', 'workflow B\'s update succeeded without retrying, proving it never shared A\'s lock');
      } finally {
        held.release();
      }

      // Lock released — A itself is updatable again.
      const updatedA = await updateWorkflow(root, a.id, { phase: 'planning' });
      assert(updatedA.phase === 'planning', 'workflow A is updatable again once its externally-held lock is released');
    } finally {
      fs.rmSync(root, { recursive: true, force: true });
    }
  },
);

await check('withWorkflowLock is a thin named wrapper: two ids run their bodies without either blocking the other', async () => {
  const root = makeRoot();
  const order = [];
  await Promise.all([
    withWorkflowLock(root, 'wf-p', async () => {
      order.push('p-start');
      await new Promise((resolve) => setTimeout(resolve, 10));
      order.push('p-end');
    }),
    withWorkflowLock(root, 'wf-q', async () => {
      order.push('q-start');
      order.push('q-end');
    }),
  ]);
  fs.rmSync(root, { recursive: true, force: true });
  // 'q' (no artificial delay) must finish BEFORE 'p' (10ms delay) if the two
  // locks are truly independent; if they shared one lock, 'q' would queue
  // behind 'p' and 'q-start' would land after 'p-end'.
  const qEndIndex = order.indexOf('q-end');
  const pEndIndex = order.indexOf('p-end');
  assert(qEndIndex < pEndIndex, `expected q to finish before p under independent locks, got order ${JSON.stringify(order)}`);
});

// ─── C4 structural guarantee: this module is a session/state-free leaf ─────

await check('workflow-store.mjs never imports claims.mjs/state.mjs and never names the sessions/state locks (C4 structural isolation)', () => {
  const modulePath = fileURLToPath(new URL('../lib/workflow-store.mjs', import.meta.url));
  const source = fs.readFileSync(modulePath, 'utf8');
  assert(!/from ['"]\.\/claims\.mjs['"]/.test(source), 'must never import claims.mjs (session records)');
  assert(!/from ['"]\.\/state\.mjs['"]/.test(source), 'must never import state.mjs (would risk a claims.mjs transitive import, and a future cycle once state.mjs imports this module)');
  assert(!/withStoreLock\(\s*root\s*,\s*['"]sessions['"]/.test(source), 'must never acquire the global "sessions" lock');
  assert(!/withStoreLock\(\s*root\s*,\s*['"]state['"]/.test(source), 'must never acquire the global "state" lock (must_haves prohibition)');
});

printSummaryAndExit();
