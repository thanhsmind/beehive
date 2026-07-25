#!/usr/bin/env node
// test_state_projection.mjs — lib/state-projection.mjs contract tests
// (multisession-native-7, CONTEXT.md D1, advisor consult slice 2 C1/C5/F8).
// Same PASS/FAIL/exit-1 contract as every other suite here — see
// scripts/lib/test-fixture.mjs.
//
// Covers: C1 fallback (zero workflow records is a no-op on the D1-owned
// fields, but bee-state-sync's own cells/last_activity refresh still lands);
// rebuildStateProjection picks the newest ACTIVE workflow and goes
// idle-shaped when none is active; rebuildLaneProjection derives every D1
// field from the live (non-closed) workflow record while passing through
// ad hoc fields (last_scribing_run et al.) and preserving created_at;
// self-heal ("record wins") when a lane file has drifted from its record;
// and the invariant-13/14 proof (delete a projection, rebuild it, get back
// byte-identical content) for both state.json and a lane file.

import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { check, assert, printSummaryAndExit } from '../../../../scripts/lib/test-fixture.mjs';
import { writeJsonAtomic } from '../lib/fsutil.mjs';
import { readState, writeState, readLane, writeLane, statePath, lanePath, GATE_NAMES } from '../lib/state.mjs';
import { createWorkflow, updateWorkflow, listWorkflows } from '../lib/workflow-store.mjs';
import {
  projectionsAuthoritative,
  workflowGatesToApprovedGates,
  pickNewestActiveWorkflow,
  rebuildStateProjection,
  rebuildLaneProjection,
  rebuildAllProjections,
} from '../lib/state-projection.mjs';

function makeRoot() {
  return fs.mkdtempSync(path.join(os.tmpdir(), 'bee-state-projection-'));
}

// ─── workflowGatesToApprovedGates ───────────────────────────────────────────

await check('workflowGatesToApprovedGates: translates the D1 gates map to the legacy boolean shape, in fixed GATE_NAMES order (null approved_for_plan_rev — never rev-scoped, D7 default for context/shape/review)', async () => {
  const gates = {
    review: { approved: true, approved_for_plan_rev: null },
    context: { approved: true, approved_for_plan_rev: null },
    execution: { approved: false, approved_for_plan_rev: null },
    shape: { approved: true, approved_for_plan_rev: null },
  };
  const approved = workflowGatesToApprovedGates(gates, 7); // arbitrary planRev — null revs are immune to it
  assert(
    JSON.stringify(Object.keys(approved)) === JSON.stringify(GATE_NAMES),
    `key order must be GATE_NAMES order regardless of input order, got ${JSON.stringify(Object.keys(approved))}`,
  );
  assert(
    approved.context === true && approved.shape === true && approved.execution === false && approved.review === true,
    `values must reflect each gate's approved flag, got ${JSON.stringify(approved)}`,
  );
});

await check('workflowGatesToApprovedGates: a missing/malformed gates map reads as all-false, never throws', async () => {
  assert(JSON.stringify(workflowGatesToApprovedGates(null)) === JSON.stringify({ context: false, shape: false, execution: false, review: false }));
  assert(JSON.stringify(workflowGatesToApprovedGates({})) === JSON.stringify({ context: false, shape: false, execution: false, review: false }));
});

// ─── C2 (multisession-native-9, CONTEXT.md D7, advisor consult slice 2 —
// binding): the projected boolean is plan-rev-EFFECTIVE, not the bare
// `approved` flag ─────────────────────────────────────────────────────────

await check('workflowGatesToApprovedGates (C2): a gate approved AT the current plan_rev projects true; the SAME gate reads false once planRev moves past its stamped rev', async () => {
  const gates = { execution: { approved: true, approved_for_plan_rev: 1 } };
  assert(workflowGatesToApprovedGates(gates, 1).execution === true, 'matching plan_rev (1 === 1) must stay effective');
  assert(workflowGatesToApprovedGates(gates, 2).execution === false, 'a plan_rev bump past the stamped rev (2 !== 1) must project ineffective');
  assert(workflowGatesToApprovedGates(gates, 0).execution === false, 'a mismatched rev in EITHER direction is ineffective, not just "newer wins"');
});

await check('workflowGatesToApprovedGates (C2): approved_for_plan_rev === null (or the field absent) is ALWAYS effective, independent of planRev — the D7 default keeps context/shape/review rev-immune', async () => {
  const nullRev = { execution: { approved: true, approved_for_plan_rev: null } };
  assert(workflowGatesToApprovedGates(nullRev, 0).execution === true);
  assert(workflowGatesToApprovedGates(nullRev, 999).execution === true);

  const noRevField = { execution: { approved: true } }; // legacy/hand-written shape, no approved_for_plan_rev key at all
  assert(workflowGatesToApprovedGates(noRevField, 999).execution === true, 'an absent approved_for_plan_rev field must read the same as null — always effective');
});

await check('workflowGatesToApprovedGates (C2): a false `approved` flag stays false regardless of rev match — rev-effectiveness never turns an unapproved gate on', async () => {
  const gates = { execution: { approved: false, approved_for_plan_rev: 3 } };
  assert(workflowGatesToApprovedGates(gates, 3).execution === false, 'approved:false must project false even at a matching plan_rev');
});

// ─── pickNewestActiveWorkflow ───────────────────────────────────────────────

await check('pickNewestActiveWorkflow: only status==="active" is eligible; newest created_at wins; a tie breaks on id descending', async () => {
  const workflows = [
    { id: 'wf-a', status: 'closed', created_at: '2026-01-05T00:00:00.000Z' },
    { id: 'wf-b', status: 'active', created_at: '2026-01-01T00:00:00.000Z' },
    { id: 'wf-c', status: 'active', created_at: '2026-01-03T00:00:00.000Z' },
    { id: 'wf-d', status: 'paused', created_at: '2026-01-09T00:00:00.000Z' },
  ];
  const picked = pickNewestActiveWorkflow(workflows);
  assert(picked && picked.id === 'wf-c', `expected the newest ACTIVE record (wf-c), got ${JSON.stringify(picked)}`);

  const tie = pickNewestActiveWorkflow([
    { id: 'wf-x', status: 'active', created_at: '2026-01-01T00:00:00.000Z' },
    { id: 'wf-y', status: 'active', created_at: '2026-01-01T00:00:00.000Z' },
  ]);
  assert(tie.id === 'wf-y', `a created_at tie must break deterministically (id descending), got ${tie.id}`);

  assert(pickNewestActiveWorkflow([]) === null, 'no workflows -> null');
  assert(pickNewestActiveWorkflow([{ id: 'wf-z', status: 'closed', created_at: '2026-01-01T00:00:00.000Z' }]) === null, 'no ACTIVE workflow -> null even if one exists closed');
});

// ─── C1 fallback: zero workflow records ─────────────────────────────────────

await check('C1: rebuildStateProjection with zero workflow records and no overrides is a pure no-op — the file is never written', async () => {
  const root = makeRoot();
  try {
    writeJsonAtomic(statePath(root), { phase: 'swarming', feature: 'legacy-only', mode: 'standard', approved_gates: { context: true, shape: false, execution: false, review: false }, workers: [], summary: 'hand-written', next_action: 'keep going' });
    const before = fs.readFileSync(statePath(root), 'utf8');
    assert(!projectionsAuthoritative(root), 'precondition: zero workflow records anywhere');

    const result = rebuildStateProjection(root);
    assert(result.authoritative === false, 'authoritative must be false under C1');
    assert(fs.readFileSync(statePath(root), 'utf8') === before, 'C1 fallback: state.json must be byte-identical, never rewritten');
    assert(result.state.feature === 'legacy-only', 'returned state still reflects the untouched legacy record');
  } finally {
    fs.rmSync(root, { recursive: true, force: true });
  }
});

await check('C1: rebuildStateProjection with zero workflow records but WITH overrides still refreshes cells/last_activity (bee-state-sync\'s own job is independent of D1 authority)', async () => {
  const root = makeRoot();
  try {
    writeJsonAtomic(statePath(root), { phase: 'swarming', feature: 'legacy-only', mode: 'standard', approved_gates: { context: true, shape: false, execution: false, review: false }, workers: [], summary: 'hand-written', next_action: 'keep going' });
    assert(!projectionsAuthoritative(root), 'precondition: zero workflow records anywhere');

    const counts = { open: 2, claimed: 1, capped: 0, blocked: 0 };
    const result = rebuildStateProjection(root, { cellCounts: counts, lastActivity: '2026-07-24T12:00:00.000Z' });
    assert(result.authoritative === false, 'D1 fields still have no authority under C1');
    const onDisk = readState(root);
    assert(JSON.stringify(onDisk.cells) === JSON.stringify(counts), `cells must be refreshed even under C1, got ${JSON.stringify(onDisk.cells)}`);
    assert(onDisk.last_activity === '2026-07-24T12:00:00.000Z', 'last_activity must be refreshed even under C1');
    assert(onDisk.phase === 'swarming' && onDisk.feature === 'legacy-only', 'D1-owned fields stay exactly as they were under C1');
  } finally {
    fs.rmSync(root, { recursive: true, force: true });
  }
});

await check('C1: rebuildLaneProjection with zero workflow records is a no-op — returns the existing lane untouched', async () => {
  const root = makeRoot();
  try {
    writeJsonAtomic(lanePath(root, 'hand-lane'), { schema_version: '1.0', feature: 'hand-lane', mode: null, phase: 'exploring', approved_gates: { context: false, shape: false, execution: false, review: false }, summary: '', next_action: '', created_at: '2026-01-01T00:00:00.000Z' });
    const before = fs.readFileSync(lanePath(root, 'hand-lane'), 'utf8');
    const result = rebuildLaneProjection(root, 'hand-lane');
    assert(result.authoritative === false, 'authoritative must be false under C1');
    assert(fs.readFileSync(lanePath(root, 'hand-lane'), 'utf8') === before, 'C1 fallback: lane file must be byte-identical, never rewritten');
  } finally {
    fs.rmSync(root, { recursive: true, force: true });
  }
});

// ─── projectionsAuthoritative flips once any record exists ─────────────────

await check('projectionsAuthoritative flips to true the moment ANY workflow record exists anywhere, even for an unrelated feature', async () => {
  const root = makeRoot();
  try {
    assert(!projectionsAuthoritative(root), 'starts false');
    await createWorkflow(root, { feature: 'unrelated' });
    assert(projectionsAuthoritative(root), 'flips true once a single record exists');
  } finally {
    fs.rmSync(root, { recursive: true, force: true });
  }
});

// ─── rebuildStateProjection: authoritative rebuild from records ────────────

await check('rebuildStateProjection: sources the newest ACTIVE workflow, resets D1 fields, preserves workers/schema_version pass-through', async () => {
  const root = makeRoot();
  try {
    writeJsonAtomic(statePath(root), { schema_version: '1.0', phase: 'idle', feature: null, mode: null, approved_gates: { context: false, shape: false, execution: false, review: false }, workers: [{ nickname: 'kevin', cell: 'x-1' }], summary: '', next_action: 'No active bee work — awaiting a user request.' });
    const older = await createWorkflow(root, { feature: 'older-feature', phase: 'planning', mode: 'small', summary: 'older summary', next_action: 'older next' });
    await new Promise((resolve) => setTimeout(resolve, 5));
    const newer = await createWorkflow(root, { feature: 'newer-feature', phase: 'swarming', mode: 'standard', plan_rev: 1, gates: { execution: { approved: true, approved_for_plan_rev: 1 } }, summary: 'newer summary', next_action: 'newer next' });

    const result = rebuildStateProjection(root);
    assert(result.authoritative === true && result.source === newer.id, `must source the newer, more-recently-created active workflow, got source=${result.source}`);
    const onDisk = readState(root);
    assert(onDisk.phase === 'swarming' && onDisk.feature === 'newer-feature' && onDisk.mode === 'standard', `D1 fields must come from the newer workflow, got ${JSON.stringify({ phase: onDisk.phase, feature: onDisk.feature, mode: onDisk.mode })}`);
    assert(onDisk.approved_gates.execution === true, 'approved_gates must be translated from the workflow gates map');
    assert(onDisk.summary === 'newer summary' && onDisk.next_action === 'newer next', 'summary/next_action come from the workflow record');
    assert(JSON.stringify(onDisk.workers) === JSON.stringify([{ nickname: 'kevin', cell: 'x-1' }]), 'workers (not yet D1-owned, D6/msn-8) must pass through unchanged');
    assert(onDisk.schema_version === '1.0', 'schema_version passes through unchanged');
    void older;
  } finally {
    fs.rmSync(root, { recursive: true, force: true });
  }
});

await check('rebuildStateProjection: idle-shaped when zero workflows are ACTIVE (all paused/closed)', async () => {
  const root = makeRoot();
  try {
    const wf = await createWorkflow(root, { feature: 'wound-down', phase: 'swarming', mode: 'standard' });
    await updateWorkflow(root, wf.id, { status: 'closed' });
    const result = rebuildStateProjection(root);
    assert(result.authoritative === true && result.source === null, 'authoritative is still true (records exist) but no active source');
    const onDisk = readState(root);
    assert(onDisk.phase === 'idle' && onDisk.feature === null && onDisk.mode === null, `must go idle-shaped, got ${JSON.stringify({ phase: onDisk.phase, feature: onDisk.feature, mode: onDisk.mode })}`);
    assert(Object.values(onDisk.approved_gates).every((v) => v === false), 'approved_gates must be all-false idle-shaped');
  } finally {
    fs.rmSync(root, { recursive: true, force: true });
  }
});

// ─── rebuildLaneProjection: authoritative rebuild, ad hoc field pass-through ─

await check('rebuildLaneProjection: derives every D1 field from the live workflow record; preserves created_at and ad hoc fields (last_scribing_run, gate_revoked_at, advisor_ref) from the existing lane file', async () => {
  const root = makeRoot();
  try {
    const wf = await createWorkflow(root, {
      feature: 'lane-feat',
      phase: 'swarming',
      mode: 'standard',
      plan_rev: 1,
      gates: { shape: { approved: true, approved_for_plan_rev: 1 } },
      summary: 'lane summary',
      next_action: 'lane next',
    });
    writeJsonAtomic(lanePath(root, 'lane-feat'), {
      schema_version: '1.0',
      feature: 'lane-feat',
      mode: null,
      phase: 'exploring',
      approved_gates: { context: false, shape: false, execution: false, review: false },
      summary: 'stale',
      next_action: 'stale',
      created_at: '2020-01-01T00:00:00.000Z',
      last_scribing_run: { feature: 'lane-feat', date: '2026-01-01', at: '2026-01-01T00:00:00.000Z', areas_synced: ['x'], next_action: 'n' },
      gate_revoked_at: { execution: '2026-01-02T00:00:00.000Z' },
    });

    const result = rebuildLaneProjection(root, 'lane-feat');
    assert(result.authoritative === true && result.source === wf.id, 'must be sourced from the live workflow record');
    const lane = readLane(root, 'lane-feat');
    assert(lane.phase === 'swarming' && lane.mode === 'standard', `D1 fields must be re-derived from the record, got ${JSON.stringify({ phase: lane.phase, mode: lane.mode })}`);
    assert(lane.approved_gates.shape === true, 'approved_gates must be re-derived from the record');
    assert(lane.summary === 'lane summary' && lane.next_action === 'lane next', 'summary/next_action must be re-derived from the record, not left stale');
    assert(lane.created_at === '2020-01-01T00:00:00.000Z', 'created_at is preserved from the existing lane file');
    assert(
      JSON.stringify(lane.last_scribing_run) === JSON.stringify({ feature: 'lane-feat', date: '2026-01-01', at: '2026-01-01T00:00:00.000Z', areas_synced: ['x'], next_action: 'n' }),
      `last_scribing_run (not a workflow-record field) must pass through unchanged, got ${JSON.stringify(lane.last_scribing_run)}`,
    );
    assert(
      JSON.stringify(lane.gate_revoked_at) === JSON.stringify({ execution: '2026-01-02T00:00:00.000Z' }),
      `gate_revoked_at must pass through unchanged, got ${JSON.stringify(lane.gate_revoked_at)}`,
    );
  } finally {
    fs.rmSync(root, { recursive: true, force: true });
  }
});

await check('rebuildLaneProjection: no-op when the only workflow record for this feature is CLOSED (never guesses, never deletes)', async () => {
  const root = makeRoot();
  try {
    const wf = await createWorkflow(root, { feature: 'closed-feat', phase: 'compounding-complete' });
    await updateWorkflow(root, wf.id, { status: 'closed' });
    writeJsonAtomic(lanePath(root, 'closed-feat'), { schema_version: '1.0', feature: 'closed-feat', mode: null, phase: 'compounding-complete', approved_gates: { context: true, shape: true, execution: true, review: true }, summary: 'final', next_action: '', created_at: '2020-01-01T00:00:00.000Z' });
    const before = fs.readFileSync(lanePath(root, 'closed-feat'), 'utf8');

    const result = rebuildLaneProjection(root, 'closed-feat');
    assert(result.authoritative === false, 'a closed workflow record is not a live source');
    assert(fs.readFileSync(lanePath(root, 'closed-feat'), 'utf8') === before, 'a closed feature\'s lane file must be left exactly as it was');
  } finally {
    fs.rmSync(root, { recursive: true, force: true });
  }
});

// ─── self-heal: "record wins" over a diverged lane file ─────────────────────

await check('self-heal (F8): a lane file that has drifted from its workflow record is corrected on the next rebuild — the RECORD wins', async () => {
  const root = makeRoot();
  try {
    const wf = await createWorkflow(root, { feature: 'drift-feat', phase: 'swarming', mode: 'standard', summary: 'record summary', next_action: 'record next' });
    rebuildLaneProjection(root, 'drift-feat'); // first honest projection

    // Simulate the crash-window divergence: the lane file gets hand-edited
    // (or left stale by a crash between "record written" and "projection
    // rebuilt") so it disagrees with the record.
    const lane = readLane(root, 'drift-feat');
    writeLane(root, { ...lane, phase: 'validating', summary: 'DRIFTED — must not survive a rebuild' });
    assert(readLane(root, 'drift-feat').phase === 'validating', 'precondition: the lane file is now diverged from its record');

    const result = rebuildLaneProjection(root, 'drift-feat');
    assert(result.authoritative === true && result.source === wf.id);
    const healed = readLane(root, 'drift-feat');
    assert(healed.phase === 'swarming', `record must win over the drifted lane file, got phase=${healed.phase}`);
    assert(healed.summary === 'record summary', `record must win, got summary=${JSON.stringify(healed.summary)}`);
  } finally {
    fs.rmSync(root, { recursive: true, force: true });
  }
});

// ─── invariant 13/14: delete a projection, rebuild it, get identical content ─

await check('invariant 13/14: deleting .bee/lanes/<feature>.json and rebuilding it reproduces byte-identical content — "deleting a projection loses nothing"', async () => {
  const root = makeRoot();
  try {
    await createWorkflow(root, { feature: 'invariant-lane', phase: 'validating', mode: 'high-risk', plan_rev: 1, gates: { context: { approved: true, approved_for_plan_rev: 1 }, shape: { approved: true, approved_for_plan_rev: 1 } }, summary: 'invariant summary', next_action: 'invariant next' });

    const first = rebuildLaneProjection(root, 'invariant-lane');
    assert(first.authoritative === true);
    const beforeDelete = fs.readFileSync(lanePath(root, 'invariant-lane'), 'utf8');

    fs.rmSync(lanePath(root, 'invariant-lane'), { force: true });
    assert(!fs.existsSync(lanePath(root, 'invariant-lane')), 'precondition: the projection file is genuinely gone');

    const second = rebuildLaneProjection(root, 'invariant-lane');
    assert(second.authoritative === true);
    const afterRebuild = fs.readFileSync(lanePath(root, 'invariant-lane'), 'utf8');

    assert(afterRebuild === beforeDelete, `delete + rebuild must reproduce byte-identical content — the workflow record is the sole source of truth for every field this lane carries.\nbefore: ${beforeDelete}\nafter:  ${afterRebuild}`);
  } finally {
    fs.rmSync(root, { recursive: true, force: true });
  }
});

await check('invariant 13/14: deleting .bee/state.json and rebuilding it (no overrides) reproduces byte-identical D1-owned content — "overview rebuilds fully from records"', async () => {
  const root = makeRoot();
  try {
    await createWorkflow(root, { feature: 'invariant-state', phase: 'swarming', mode: 'standard', plan_rev: 2, gates: { execution: { approved: true, approved_for_plan_rev: 2 } }, summary: 'overview summary', next_action: 'overview next' });

    const first = rebuildStateProjection(root);
    assert(first.authoritative === true);
    const beforeDelete = fs.readFileSync(statePath(root), 'utf8');

    fs.rmSync(statePath(root), { force: true });
    assert(!fs.existsSync(statePath(root)), 'precondition: the projection file is genuinely gone');

    const second = rebuildStateProjection(root);
    assert(second.authoritative === true);
    const afterRebuild = fs.readFileSync(statePath(root), 'utf8');

    assert(afterRebuild === beforeDelete, `delete + rebuild (no overrides) must reproduce byte-identical content for a freshly-defaulted file.\nbefore: ${beforeDelete}\nafter:  ${afterRebuild}`);
  } finally {
    fs.rmSync(root, { recursive: true, force: true });
  }
});

// ─── rebuildAllProjections: the recovery entry point ────────────────────────

await check('rebuildAllProjections: rebuilds state.json from the newest active workflow AND one lane file PER active workflow', async () => {
  const root = makeRoot();
  try {
    const a = await createWorkflow(root, { feature: 'multi-a', phase: 'planning', mode: 'small', summary: 'a-summary', next_action: 'a-next' });
    await new Promise((resolve) => setTimeout(resolve, 5));
    const b = await createWorkflow(root, { feature: 'multi-b', phase: 'swarming', mode: 'standard', summary: 'b-summary', next_action: 'b-next' });
    const c = await createWorkflow(root, { feature: 'multi-c-closed', phase: 'compounding-complete' });
    await updateWorkflow(root, c.id, { status: 'closed' });

    const result = rebuildAllProjections(root);
    assert(result.state.authoritative === true && result.state.source === b.id, `state.json must source the newest active workflow (b), got ${result.state.source}`);
    assert(result.lanes.length === 2, `exactly one lane result per ACTIVE workflow (a, b) — closed (c) excluded, got ${result.lanes.length}`);

    const laneA = readLane(root, 'multi-a');
    const laneB = readLane(root, 'multi-b');
    assert(laneA && laneA.summary === 'a-summary', 'lane a must be projected from its own record');
    assert(laneB && laneB.summary === 'b-summary', 'lane b must be projected from its own record');
    assert(!fs.existsSync(lanePath(root, 'multi-c-closed')), 'a closed workflow must never get a lane projection created for it');
    void a;
  } finally {
    fs.rmSync(root, { recursive: true, force: true });
  }
});

printSummaryAndExit();
