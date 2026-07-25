#!/usr/bin/env node
// test_cli_state.mjs — bee.mjs `state` CLI + lanes/handoff/SessionStart wiring
// contract tests (state verbs, worker prune, start-feature, buildStatus lanes
// block, handoff lifecycle, SessionStart rendering, two-session e2e), split
// out of test_lib.mjs (cs-2b) to shrink the monolith. Same PASS/FAIL/exit-1
// contract as every other suite here — see scripts/lib/test-fixture.mjs.

import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { fileURLToPath, pathToFileURL } from 'node:url';
import { runModuleWorker } from '../../../../scripts/lib/run-module-worker.mjs';
import {
  makeTempRepo,
  makeCell,
  check,
  assert,
  assertThrows,
  assertRejects,
  printSummaryAndExit,
} from '../../../../scripts/lib/test-fixture.mjs';
import { readStateStrict, isKnownPhase, startFeature } from '../lib/state.mjs';
import { listWorkflows, createWorkflow, updateWorkflow } from '../lib/workflow-store.mjs';
import { rebuildHandoffProjection } from '../lib/state-projection.mjs';
import { acquireStoreLockOnceSync } from '../lib/lock.mjs';
import { readCell, dropCell, claimCell } from '../lib/cells.mjs';
import { createSession, claimCellFile, readClaim, adoptClaim } from '../lib/claims.mjs';
// fsh-3 (lane store): namespace imports so a not-yet-implemented export fails
// its own row ("… is not a function") instead of crashing the whole module
// graph at import time — the RED-first evidence stays per-row.
import * as laneStore from '../lib/state.mjs';
import * as laneBinding from '../lib/claims.mjs';
import { buildSessionPreamble } from '../lib/inject.mjs';
import { readJson, writeJsonAtomic } from '../lib/fsutil.mjs';
import { reserve } from '../lib/reservations.mjs';
import { renewLease } from '../lib/lease-store.mjs';

const root = makeTempRepo();

// si-3: guard against the exact isolation leak si-1's suite produced once —
// the LIVE checkout's own .bee/state.json `feature` got stamped to a fixture
// value ("other-feature") mid-run, then had to be restored by hand. Every
// fixture in this file uses its own fs.mkdtempSync/makeTempRepo root and
// threads it through runBeeState/runBeeMjs's explicit `cwd`, so nothing here
// has a legitimate reason to ever touch the live repo's own state — this is
// read-only bookkeeping, captured once before the first test runs and
// compared once after the last, never a write. LIVE_REPO_ROOT is computed
// the same way this file already resolves run-module-worker.mjs above
// (relative to this file's own location, canonical-copy-correct — see that
// import) rather than trusting `process.cwd()`, since individual checks
// legitimately chdir away and back via runModuleWorker mid-suite.
const LIVE_REPO_ROOT = fileURLToPath(new URL('../../../../', import.meta.url));
function liveRepoFeature() {
  try {
    return readJson(path.join(LIVE_REPO_ROOT, '.bee', 'state.json'), null)?.feature ?? null;
  } catch {
    return undefined; // unreadable (e.g. no live .bee/ at all) — skip, never false-fail
  }
}
const liveFeatureAtSuiteStart = liveRepoFeature();

// ─── bee.mjs state CLI (cli-mutations-1, decision 0011 primitive) ──────────
// No dedicated lib/state-mutations module backs this CLI (file-bounds forbid
// touching lib/state.mjs semantics), so its verb logic is only exercised at
// the process level — mirroring the existing bee.mjs feedback / commands_detect.mjs
// "CLI entry" tests above. The 9 bee_*.mjs shims are retired (shim-retire
// D1/D5); every call here prepends the "state" group token itself, exactly
// what the retired bee_state.mjs shim used to do internally.

function beeStateModulePath() {
  return fileURLToPath(new URL('../bee.mjs', import.meta.url));
}

function makeStateRepo(prefix) {
  const dir = fs.mkdtempSync(path.join(os.tmpdir(), prefix));
  fs.mkdirSync(path.join(dir, '.bee'), { recursive: true });
  writeJsonAtomic(path.join(dir, '.bee', 'onboarding.json'), {
    schema_version: '1.0',
    bee_version: '0.1.0',
  });
  return dir;
}

function runBeeState(cwd, args) {
  return runModuleWorker(beeStateModulePath(), { args: ['state', ...args], cwd });
}

function readStateFile(repoRoot) {
  return readJson(path.join(repoRoot, '.bee', 'state.json'), null);
}

await check('bee.mjs state with no verb prints a Use: line listing all five verbs and exits non-zero', async () => {
  const dir = makeStateRepo('bee-state-noverb-');
  try {
    const result = await runBeeState(dir, []);
    assert(result.status !== 0, 'no-verb invocation exits non-zero');
    assert(/Use:/.test(result.stderr), `expected a "Use:" line, got stderr="${result.stderr}"`);
    assert(
      /set/.test(result.stderr) &&
        /gate/.test(result.stderr) &&
        /worker/.test(result.stderr) &&
        /scribing-run/.test(result.stderr) &&
        /start-feature/.test(result.stderr),
      `Use: line should list all five verbs, got ${result.stderr}`,
    );
  } finally {
    fs.rmSync(dir, { recursive: true, force: true });
  }
});

await check('P0 (codex-loop-p0): start-feature is the atomic idle->exploring entry; the naive owner=exploring set from idle is refused', async () => {
  const dir = makeStateRepo('bee-state-idle-entry-');
  try {
    // The bug the skill text used to walk into: from a fresh (idle) repo, the
    // hand-written `set --owner exploring --phase exploring` is REFUSED, because
    // --owner must equal the pre-mutation phase (idle). This is the idle->hive
    // bounce the exploring skill's new step 0 exists to prevent.
    const naive = await runBeeState(dir, ['set', '--owner', 'exploring', '--phase', 'exploring', '--feature', 'x']);
    assert(naive.status !== 0, `naive owner=exploring from idle must be refused, got status ${naive.status}`);

    // The documented atomic entry: start-feature moves idle->exploring in one
    // guarded mutation and sets feature+mode.
    const started = await runBeeState(dir, ['start-feature', '--feature', 'demo-p0', '--mode', 'standard']);
    assert(started.status === 0, `start-feature from idle should succeed, got ${started.status}: ${started.stderr}`);
    assert(readStateFile(dir).phase === 'exploring', `start-feature lands exploring, got ${readStateFile(dir).phase}`);

    // Once entered correctly, the documented forward transition flows with no
    // owner-mismatch — owner=exploring now matches the pre-mutation phase.
    const fwd = await runBeeState(dir, ['set', '--owner', 'exploring', '--phase', 'swarming', '--summary', 'onward']);
    assert(fwd.status === 0, `exploring->swarming should succeed after correct entry, got ${fwd.status}: ${fwd.stderr}`);
    assert(readStateFile(dir).phase === 'swarming', 'forward transition landed');
  } finally {
    fs.rmSync(dir, { recursive: true, force: true });
  }
});

await check('codex-loop (advisor #54): start-feature hands off FORWARD — its next_action never orders another hive route', async () => {
  const dir = makeStateRepo('bee-state-nextaction-');
  try {
    // The loop the 1.11.1 fix left open one step later: start-feature IS the
    // routing decision, but it wrote next_action = "Invoke bee-hive ...", which
    // the prompt reminder then replayed on the user's NEXT turn — pulling the
    // session straight back into routing it had just left.
    const started = await runBeeState(dir, ['start-feature', '--feature', 'demo-fwd', '--mode', 'standard']);
    assert(started.status === 0, `start-feature should succeed, got ${started.stderr}`);
    const na = String(readStateFile(dir).next_action || '');
    assert(!/Invoke bee-hive/i.test(na), `start-feature must not order another hive route, got: ${na}`);
    assert(/continue/i.test(na) && /demo-fwd/.test(na), `next_action should hand off forward and name the feature, got: ${na}`);
  } finally {
    fs.rmSync(dir, { recursive: true, force: true });
  }
});

await check('bee.mjs state set writes only the provided fields and creates state.json on a fresh repo', async () => {
  const dir = makeStateRepo('bee-state-set-');
  try {
    const result = await runBeeState(dir, ['set', '--owner', 'idle', '--phase', 'planning', '--summary', 'kickoff']);
    assert(result.status === 0, `set should succeed, got ${result.status}: ${result.stderr}`);
    const state = readStateFile(dir);
    assert(state.phase === 'planning', `phase written, got ${state.phase}`);
    assert(state.summary === 'kickoff', `summary written, got ${state.summary}`);
    assert(state.mode === null, 'mode left at default when its flag is not given');
  } finally {
    fs.rmSync(dir, { recursive: true, force: true });
  }
});

await check('bee.mjs state set rejects an unknown phase (isKnownPhase, not the bare PHASES array) and leaves the file untouched', async () => {
  const dir = makeStateRepo('bee-state-set-badphase-');
  try {
    await runBeeState(dir, ['set', '--owner', 'idle', '--phase', 'swarming', '--summary', 'before']);
    const before = fs.readFileSync(path.join(dir, '.bee', 'state.json'), 'utf8');
    const result = await runBeeState(dir, ['set', '--phase', 'not-a-real-phase']);
    assert(result.status !== 0, 'invalid phase exits non-zero');
    assert(/phase/i.test(result.stderr), `error names the phase, got ${result.stderr}`);
    const after = fs.readFileSync(path.join(dir, '.bee', 'state.json'), 'utf8');
    assert(before === after, 'file untouched after a rejected set');
  } finally {
    fs.rmSync(dir, { recursive: true, force: true });
  }
});

await check('bee.mjs state set refuses to mutate a present-but-corrupt state.json (review P1-1: never clobber to defaults)', async () => {
  const dir = makeStateRepo('bee-state-set-corrupt-');
  try {
    const statePath = path.join(dir, '.bee', 'state.json');
    fs.writeFileSync(statePath, '{ this is not json', 'utf8');
    const before = fs.readFileSync(statePath, 'utf8');
    const result = await runBeeState(dir, ['set', '--summary', 'x']);
    assert(result.status !== 0, `set over a corrupt state.json exits non-zero, got ${result.status}`);
    assert(/state\.json/.test(result.stderr), `error names state.json, got ${result.stderr}`);
    assert(/FIX:/.test(result.stderr), `error carries a FIX:, got ${result.stderr}`);
    const after = fs.readFileSync(statePath, 'utf8');
    assert(before === after, 'corrupt file is byte-identical after the refused mutation — never clobbered to defaults');
  } finally {
    fs.rmSync(dir, { recursive: true, force: true });
  }
});

await check('bee.mjs state set accepts the compounding-complete terminal alias (isKnownPhase, not PHASES) — from compounding, per the tail guard', async () => {
  const dir = makeStateRepo('bee-state-set-terminal-');
  try {
    // chain-integrity: this fixture used to start at the `idle` default and walk
    // straight to the terminal alias — the very transition the tail guard now
    // refuses. The alias is still accepted; it just has to be reached honestly.
    writeJsonAtomic(path.join(dir, '.bee', 'state.json'), { phase: 'compounding' });
    const result = await runBeeState(dir, ['set', '--owner', 'compounding', '--phase', 'compounding-complete']);
    assert(result.status === 0, `terminal alias should be accepted, got ${result.status}: ${result.stderr}`);
    const state = readStateFile(dir);
    assert(state.phase === 'compounding-complete', 'terminal alias written');
  } finally {
    fs.rmSync(dir, { recursive: true, force: true });
  }
});

await check('bee.mjs state set preserves unrelated fields (workers, cells, last_scribing_run) byte-for-byte', async () => {
  const dir = makeStateRepo('bee-state-set-preserve-');
  try {
    const statePath = path.join(dir, '.bee', 'state.json');
    writeJsonAtomic(statePath, {
      schema_version: '1.0',
      phase: 'swarming',
      feature: 'demo',
      mode: 'standard',
      approved_gates: { context: true, shape: true, execution: true, review: false },
      workers: [{ nickname: 'sate', cell: 'demo-1', tier: 'generation', status: 'in-flight' }],
      summary: 'old summary',
      next_action: 'old next action',
      cells: { open: 1, claimed: 0, capped: 2, blocked: 0 },
      last_scribing_run: {
        feature: 'other',
        date: '2026-01-01',
        at: '2026-01-01T00:00:00.000Z',
        areas_synced: ['x'],
        next_action: 'y',
      },
    });
    const result = await runBeeState(dir, ['set', '--owner', 'swarming', '--summary', 'new summary']);
    assert(result.status === 0, `set should succeed, got ${result.status}: ${result.stderr}`);
    const state = readStateFile(dir);
    assert(state.summary === 'new summary', 'summary updated');
    assert(state.phase === 'swarming', 'phase untouched');
    assert(state.feature === 'demo', 'feature untouched');
    assert(state.next_action === 'old next action', 'next_action untouched (flag not given)');
    assert(state.workers.length === 1 && state.workers[0].nickname === 'sate', 'workers array untouched');
    assert(state.cells.capped === 2, 'cells counts untouched');
    assert(state.last_scribing_run.feature === 'other', 'last_scribing_run untouched');
  } finally {
    fs.rmSync(dir, { recursive: true, force: true });
  }
});

await check('bee.mjs state set requires the selected record pre-phase owner and refuses missing/mismatched ownership with zero writes', async () => {
  const fresh = makeStateRepo('bee-state-owner-missing-');
  const existing = makeStateRepo('bee-state-owner-mismatch-');
  try {
    const missing = await runBeeState(fresh, ['set', '--summary', 'must-not-land']);
    assert(missing.status !== 0, 'missing --owner exits non-zero');
    assert(/missing --owner/.test(missing.stderr) && /--owner idle/.test(missing.stderr), `missing-owner refusal carries exact remediation, got ${missing.stderr}`);
    assert(!fs.existsSync(path.join(fresh, '.bee', 'state.json')), 'fresh default record remains absent after missing-owner refusal');

    const statePath = path.join(existing, '.bee', 'state.json');
    writeJsonAtomic(statePath, { phase: 'swarming', summary: 'before', approved_gates: { execution: true } });
    const before = fs.readFileSync(statePath, 'utf8');
    const mismatch = await runBeeState(existing, ['set', '--owner', 'planning', '--summary', 'must-not-land']);
    assert(mismatch.status !== 0, 'mismatched --owner exits non-zero');
    assert(/owner mismatch/.test(mismatch.stderr) && /--owner swarming/.test(mismatch.stderr), `mismatch refusal carries selected pre-phase remediation, got ${mismatch.stderr}`);
    assert(fs.readFileSync(statePath, 'utf8') === before, 'default state is byte-identical after owner mismatch');
  } finally {
    fs.rmSync(fresh, { recursive: true, force: true });
    fs.rmSync(existing, { recursive: true, force: true });
  }
});

await check('bee.mjs state set rolls ownership with phase, never persists owner, and accepts legacy valid records', async () => {
  const dir = makeStateRepo('bee-state-owner-rollover-');
  try {
    const statePath = path.join(dir, '.bee', 'state.json');
    writeJsonAtomic(statePath, { schema_version: '1.0', phase: 'exploring', summary: 'legacy-without-owner' });
    const first = await runBeeState(dir, ['set', '--owner', 'exploring', '--phase', 'planning']);
    assert(first.status === 0, `matching legacy pre-phase owner succeeds: ${first.stderr}`);
    let state = readStateFile(dir);
    assert(state.phase === 'planning', 'successful phase mutation advances the derived owner');
    assert(!Object.prototype.hasOwnProperty.call(state, 'owner'), 'owner is never persisted');

    const beforeStale = fs.readFileSync(statePath, 'utf8');
    const stale = await runBeeState(dir, ['set', '--owner', 'exploring', '--summary', 'stale']);
    assert(stale.status !== 0 && /--owner planning/.test(stale.stderr), `old owner is stale immediately after rollover: ${stale.stderr}`);
    assert(fs.readFileSync(statePath, 'utf8') === beforeStale, 'stale-owner refusal leaves state byte-identical');

    const current = await runBeeState(dir, ['set', '--owner', 'planning', '--summary', 'current']);
    assert(current.status === 0, `rolled owner succeeds: ${current.stderr}`);
    state = readStateFile(dir);
    assert(state.summary === 'current' && !Object.prototype.hasOwnProperty.call(state, 'owner'), 'current owner mutates only requested field and stays derived');
  } finally {
    fs.rmSync(dir, { recursive: true, force: true });
  }
});

await check('bee.mjs state set derives ownership from the selected lane and isolates lane/default bytes', async () => {
  const dir = makeStateRepo('bee-state-owner-lane-');
  try {
    const defaultPath = path.join(dir, '.bee', 'state.json');
    const lanePath = path.join(dir, '.bee', 'lanes', 'alpha.json');
    writeJsonAtomic(defaultPath, { phase: 'validating', feature: 'default', summary: 'untouched' });
    writeJsonAtomic(lanePath, { phase: 'exploring', feature: 'alpha', summary: 'before' });
    const defaultBefore = fs.readFileSync(defaultPath, 'utf8');
    const laneBefore = fs.readFileSync(lanePath, 'utf8');

    const mismatch = await runBeeState(dir, ['set', '--lane', 'alpha', '--owner', 'validating', '--summary', 'wrong']);
    assert(mismatch.status !== 0 && /--owner exploring/.test(mismatch.stderr), `lane owner comes from lane pre-phase, got ${mismatch.stderr}`);
    assert(fs.readFileSync(lanePath, 'utf8') === laneBefore, 'lane mismatch is byte-identical');
    assert(fs.readFileSync(defaultPath, 'utf8') === defaultBefore, 'lane mismatch never touches default state');

    const ok = await runBeeState(dir, ['set', '--lane', 'alpha', '--owner', 'exploring', '--phase', 'planning', '--summary', 'lane-only']);
    assert(ok.status === 0, `matching lane owner succeeds: ${ok.stderr}`);
    const lane = JSON.parse(fs.readFileSync(lanePath, 'utf8'));
    assert(lane.phase === 'planning' && lane.summary === 'lane-only', 'only selected lane fields update');
    assert(!Object.prototype.hasOwnProperty.call(lane, 'owner'), 'lane owner is never persisted');
    assert(fs.readFileSync(defaultPath, 'utf8') === defaultBefore, 'successful lane mutation leaves default bytes untouched');
  } finally {
    fs.rmSync(dir, { recursive: true, force: true });
  }
});

// ─── multisession-native-7 (C5): lane mutations with a LIVE workflow record
// route through the record, never a direct lanes/*.json write ─────────────
// The "derives ownership from the selected lane" check above hand-writes a
// lane file with NO corresponding workflow record — that stays on the C1
// fallback (direct writeLane, unchanged) and is exactly what proves the
// fallback still works. This check instead starts the lane through
// startFeature (creating a real workflow record, msn-6), so the mutation
// below has a live record to route through.

await check('multisession-native-7: bee.mjs state gate --lane (with a live workflow record) updates the workflow record and rebuilds the lane projection from it — the lane file is never the write target directly', async () => {
  const dir = makeStateRepo('bee-state-lane-workflow-');
  try {
    const started = await startFeature(dir, { feature: 'wf-lane-gate', mode: 'standard', lane: true });
    assert(started.feature === 'wf-lane-gate', 'precondition: lane started');
    const wfBefore = listWorkflows(dir).workflows.find((wf) => wf.feature === 'wf-lane-gate');
    assert(wfBefore, 'precondition: a live workflow record exists for this lane (msn-6)');
    assert(wfBefore.gates.execution.approved === false, 'precondition: execution gate starts unapproved on the workflow record');

    const result = await runBeeState(dir, [
      'gate',
      '--lane',
      'wf-lane-gate',
      '--name',
      'execution',
      '--approved',
      'true',
      '--json',
    ]);
    assert(result.status === 0, `gate approval succeeds: ${result.stderr}`);

    const lanePath = path.join(dir, '.bee', 'lanes', 'wf-lane-gate.json');
    const lane = JSON.parse(fs.readFileSync(lanePath, 'utf8'));
    assert(
      lane.approved_gates.execution === true,
      `lane file reflects the approval (via the rebuilt projection), got ${JSON.stringify(lane.approved_gates)}`,
    );

    const wfAfter = listWorkflows(dir).workflows.find((wf) => wf.feature === 'wf-lane-gate');
    assert(
      wfAfter.gates.execution.approved === true,
      'the WORKFLOW RECORD (not merely the lane file) carries the approval — proves the write went through updateWorkflow, not a direct writeLane',
    );
    assert(
      wfAfter.gates.execution.approved_for_plan_rev === 0,
      `multisession-native-9 (D7): approving execution stamps approved_for_plan_rev to the workflow's CURRENT ` +
        `plan_rev (0, freshly started) — got ${JSON.stringify(wfAfter.gates.execution)}`,
    );
    assert(
      wfAfter.gates.context.approved_for_plan_rev === null,
      'context (never approved in this test, and never rev-scoped by D7 default even when approved) stays null',
    );
  } finally {
    fs.rmSync(dir, { recursive: true, force: true });
  }
});

// ─── multisession-native-9 (CONTEXT.md D7, advisor consult slice 2 C2 —
// binding): gates scoped to plan revision. `state gate` stamps the execution
// gate's approved_for_plan_rev to the workflow's CURRENT plan_rev (proven
// above); `state plan-rev bump` advances that plan_rev by 1, and the
// PROJECTED gate boolean (state-projection.mjs's
// workflowGatesToApprovedGates) is plan-rev-EFFECTIVE: `approved &&
// (approved_for_plan_rev == null || approved_for_plan_rev === plan_rev)`. A
// bump therefore flips the projected execution boolean false for the BUMPED
// workflow only — cells.mjs's claimCell (which reads gateApproved off
// exactly this projected lane file, laneRecordForFeature) refuses a claim
// the moment that happens, with ZERO effect on any other workflow's own
// execution gate (invariant 3). ─────────────────────────────────────────────

function writeOpenCellFixture(root, id, feature) {
  writeJsonAtomic(path.join(root, '.bee', 'cells', `${id}.json`), {
    id,
    feature,
    title: `fixture ${id}`,
    lane: 'small',
    action: 'demo',
    verify: 'node -e "process.exit(0)"',
    status: 'open',
    trace: {
      worker: null,
      outcome: null,
      files_changed: [],
      deviations: [],
      friction: null,
      capped_at: null,
      behavior_change: false,
      verification_evidence: null,
      verify_output: null,
      verify_passed: null,
      claim_session: null,
    },
  });
}

await check('multisession-native-9 (C2, binding, RED-then-GREEN): plan-rev bump flips the PROJECTED execution boolean for the bumped workflow only — a claim refuses on W1; W2 is untouched (invariant 3)', async () => {
  const dir = makeStateRepo('bee-plan-rev-bump-');
  try {
    fs.mkdirSync(path.join(dir, '.bee', 'cells'), { recursive: true });

    // W1 (lane-a) and W2 (lane-b): each its own live workflow, its own
    // execution approval, stamped at its OWN plan_rev (0 for both, freshly
    // started).
    const startedA = await startFeature(dir, { feature: 'lane-a', mode: 'standard', lane: true });
    assert(startedA.feature === 'lane-a', 'precondition: lane-a started');
    const startedB = await startFeature(dir, { feature: 'lane-b', mode: 'standard', lane: true });
    assert(startedB.feature === 'lane-b', 'precondition: lane-b started');

    const gateA = await runBeeState(dir, ['gate', '--lane', 'lane-a', '--name', 'execution', '--approved', 'true', '--json']);
    assert(gateA.status === 0, `lane-a execution approval should succeed: ${gateA.stderr}`);
    const gateB = await runBeeState(dir, ['gate', '--lane', 'lane-b', '--name', 'execution', '--approved', 'true', '--json']);
    assert(gateB.status === 0, `lane-b execution approval should succeed: ${gateB.stderr}`);

    writeOpenCellFixture(dir, 'a-1', 'lane-a');
    writeOpenCellFixture(dir, 'a-2', 'lane-a');
    writeOpenCellFixture(dir, 'b-1', 'lane-b');

    // Pre-bump: claiming a lane-a cell succeeds (execution effective at plan_rev 0).
    const claimed = await claimCell(dir, 'a-1', 'worker-a1');
    assert(claimed.status === 'claimed', 'pre-bump claim on lane-a must succeed');

    // Bump W1 (lane-a) ONLY.
    const bump = await runBeeState(dir, ['plan-rev', 'bump', '--lane', 'lane-a', '--json']);
    assert(bump.status === 0, `plan-rev bump should succeed: ${bump.stderr}`);
    const bumped = JSON.parse(bump.stdout);
    assert(bumped.plan_rev === 1, `expected plan_rev bumped to 1, got ${bump.stdout}`);
    assert(
      bumped.lane.approved_gates.execution === false,
      `PROJECTED execution boolean must flip false on the bumped workflow's lane file, got ${bump.stdout}`,
    );

    // The workflow record itself: execution stays `approved:true` (the
    // APPROVAL is never revoked by a bump — only its plan-rev EFFECTIVENESS
    // changes); its approved_for_plan_rev is unchanged at 0, now stale
    // against plan_rev 1.
    const wfA = listWorkflows(dir).workflows.find((wf) => wf.feature === 'lane-a');
    assert(
      wfA.gates.execution.approved === true && wfA.gates.execution.approved_for_plan_rev === 0,
      `the raw approval record is untouched by a bump, got ${JSON.stringify(wfA.gates.execution)}`,
    );
    assert(wfA.plan_rev === 1, 'the workflow record itself carries the bumped plan_rev');

    // Post-bump: a NEW claim on lane-a refuses, naming the execution gate.
    await assertRejects(
      () => claimCell(dir, 'a-2', 'worker-a2'),
      'execution',
      'post-bump claim on the bumped lane must refuse citing the execution gate',
    );

    // W2 (lane-b) is completely untouched — invariant 3.
    const wfB = listWorkflows(dir).workflows.find((wf) => wf.feature === 'lane-b');
    assert(wfB.plan_rev === 0, "W1's bump must never touch W2's plan_rev");
    assert(
      wfB.gates.execution.approved === true && wfB.gates.execution.approved_for_plan_rev === 0,
      "W2's execution approval is untouched",
    );
    const laneBFile = JSON.parse(fs.readFileSync(path.join(dir, '.bee', 'lanes', 'lane-b.json'), 'utf8'));
    assert(laneBFile.approved_gates.execution === true, 'lane-b.json projected execution boolean must remain true on disk (invariant 3)');
    const claimedB = await claimCell(dir, 'b-1', 'worker-b1');
    assert(claimedB.status === 'claimed', "W2 (lane-b) claim must be unaffected by W1's bump");
  } finally {
    fs.rmSync(dir, { recursive: true, force: true });
  }
});

// ─── multisession-native-10 (D9 invariants 1/2, advisor consult slice 2 C4):
// state set/gate/scribing-run/advisor-ref record now route a workflow-backed
// target's mutation through withMutationLock, locking on `workflow:<id>`
// ONLY — never the blanket 'state' lock every one of these verbs held for
// its WHOLE body through msn-9. The two checks below prove decoupling from
// both angles: the shared 'state' lock, and a DIFFERENT feature's own
// workflow lock — deterministic (real O_EXCL file locks held for the whole
// call, msn-6-style seam proof), never timing-flaky.

await check("multisession-native-10 (invariant 1/2, C4): a lane mutation with a live workflow record never needs the shared 'state' lock — held externally for the whole call, the CLI mutation still completes near-instantly", async () => {
  const dir = makeStateRepo('bee-msn10-no-state-lock-');
  try {
    const started = await startFeature(dir, { feature: 'no-state-lock', mode: 'standard', lane: true });
    assert(started.feature === 'no-state-lock', 'precondition: lane started with a live workflow record');

    const held = acquireStoreLockOnceSync(dir, 'state');
    assert(held.acquired, "precondition: test can hold the 'state' lock directly");
    try {
      const startedAt = Date.now();
      const result = await runBeeState(dir, ['set', '--lane', 'no-state-lock', '--owner', 'exploring', '--phase', 'planning', '--json']);
      const elapsedMs = Date.now() - startedAt;
      assert(result.status === 0, `mutation must succeed while 'state' is held externally: ${result.stderr}`);
      assert(
        elapsedMs < 2000,
        `mutation must complete without waiting out any 'state' lock retry budget (structurally a different lock, single attempt) — took ${elapsedMs}ms`,
      );
      const lane = JSON.parse(result.stdout);
      assert(lane.phase === 'planning', `mutation must have actually applied, got ${result.stdout}`);
    } finally {
      held.release();
    }
  } finally {
    fs.rmSync(dir, { recursive: true, force: true });
  }
});

await check("multisession-native-10 (invariant 1/2, C4): two DIFFERENT features' workflow-routed mutations never contend on a shared lock — holding feature A's workflow lock never blocks feature B's mutation (deterministic seam proof, msn-6-style)", async () => {
  const dir = makeStateRepo('bee-msn10-cross-workflow-');
  try {
    const startedA = await startFeature(dir, { feature: 'lock-a', mode: 'standard', lane: true });
    assert(startedA.feature === 'lock-a', 'precondition: lane A started');
    const startedB = await startFeature(dir, { feature: 'lock-b', mode: 'standard', lane: true });
    assert(startedB.feature === 'lock-b', 'precondition: lane B started');
    const wfA = listWorkflows(dir).workflows.find((wf) => wf.feature === 'lock-a');
    assert(wfA, "precondition: A's workflow record exists");

    const held = acquireStoreLockOnceSync(dir, `workflow:${wfA.id}`);
    assert(held.acquired, "precondition: test can hold A's workflow lock directly");
    try {
      // Sanity/negative control: a single-attempt mutation against A's OWN
      // workflow really is denied while its lock is held — proves the held
      // lock is meaningful, not a no-op (same control shape as msn-6's own
      // cross-workflow-concurrency test above, test_state.mjs).
      await assertRejects(
        () => updateWorkflow(dir, wfA.id, { phase: 'planning' }, { maxAttempts: 1 }),
        'busy',
        "sanity control: A's workflow lock really is held",
      );

      // The real proof: feature B's mutation, run through the FULL CLI path
      // (bee.mjs state set), succeeds without ever contending on A's lock.
      const result = await runBeeState(dir, ['set', '--lane', 'lock-b', '--owner', 'exploring', '--phase', 'planning', '--json']);
      assert(result.status === 0, `B's mutation must succeed while A's workflow lock is held: ${result.stderr}`);
      const lane = JSON.parse(result.stdout);
      assert(lane.phase === 'planning', `B's mutation must have actually applied, got ${result.stdout}`);

      const wfB = listWorkflows(dir).workflows.find((wf) => wf.feature === 'lock-b');
      assert(wfB.phase === 'planning', "B's own workflow record carries the mutation");
      const wfAAfter = listWorkflows(dir).workflows.find((wf) => wf.feature === 'lock-a');
      assert(wfAAfter.phase === 'exploring', "A's workflow record is untouched by B's mutation");
    } finally {
      held.release();
    }
  } finally {
    fs.rmSync(dir, { recursive: true, force: true });
  }
});

// ─── multisession-native-10: DEFAULT-path mutations now route through the
// default feature's OWN live workflow record too (closes the C5 residual
// seam left open by msn-7/9) ─────────────────────────────────────────────

await check('multisession-native-10: state set on the DEFAULT (non-lane) record, with a live workflow, updates the workflow record AND rebuilds state.json from it — no longer a direct writeState', async () => {
  const dir = makeStateRepo('bee-msn10-default-workflow-');
  try {
    const started = await runBeeState(dir, ['start-feature', '--feature', 'wf-default-set', '--mode', 'standard', '--json']);
    assert(started.status === 0, `start-feature should succeed: ${started.stderr}`);
    const wfBefore = listWorkflows(dir).workflows.find((wf) => wf.feature === 'wf-default-set');
    assert(wfBefore, 'precondition: the default feature has a live workflow record (msn-6)');
    assert(wfBefore.phase === 'exploring', `precondition: fresh workflow starts at exploring, got ${wfBefore.phase}`);

    const result = await runBeeState(dir, ['set', '--owner', 'exploring', '--phase', 'planning', '--summary', 'msn-10 default routing', '--json']);
    assert(result.status === 0, `default set should succeed: ${result.stderr}`);

    const wfAfter = listWorkflows(dir).workflows.find((wf) => wf.feature === 'wf-default-set');
    assert(
      wfAfter.phase === 'planning' && wfAfter.summary === 'msn-10 default routing',
      `the WORKFLOW RECORD (not merely state.json) must carry the mutation, got ${JSON.stringify({ phase: wfAfter.phase, summary: wfAfter.summary })}`,
    );

    const onDisk = readStateFile(dir);
    assert(
      onDisk.phase === 'planning' && onDisk.summary === 'msn-10 default routing',
      `state.json must reflect the same mutation (rebuilt from the workflow record), got ${JSON.stringify({ phase: onDisk.phase, summary: onDisk.summary })}`,
    );
  } finally {
    fs.rmSync(dir, { recursive: true, force: true });
  }
});

await check('multisession-native-10 (C1 fallback, unbound/no-records parity): state set on the default record with ZERO workflow records anywhere writes state.json directly, byte-identical to pre-msn-10 behavior', async () => {
  const dir = makeStateRepo('bee-msn10-c1-default-');
  try {
    writeJsonAtomic(path.join(dir, '.bee', 'state.json'), {
      schema_version: '1.0',
      phase: 'exploring',
      feature: 'hand-written-default',
      mode: 'standard',
      approved_gates: { context: false, shape: false, execution: false, review: false },
      workers: [],
      summary: 'hand-written, no workflow record',
      next_action: 'keep going',
    });
    assert(listWorkflows(dir).workflows.length === 0, 'precondition: zero workflow records anywhere');

    const result = await runBeeState(dir, ['set', '--owner', 'exploring', '--phase', 'planning', '--json']);
    assert(result.status === 0, `set should succeed via the C1 fallback: ${result.stderr}`);
    assert(listWorkflows(dir).workflows.length === 0, 'C1 fallback must never create a workflow record as a side effect');
    const onDisk = readStateFile(dir);
    assert(onDisk.phase === 'planning' && onDisk.feature === 'hand-written-default', `state.json must be written directly, got ${JSON.stringify(onDisk)}`);
  } finally {
    fs.rmSync(dir, { recursive: true, force: true });
  }
});

await check("multisession-native-10 (swap carve-out): state set --feature on the default record writes state.json directly and never corrupts the OLD feature's own workflow record", async () => {
  const dir = makeStateRepo('bee-msn10-default-swap-');
  try {
    const started = await runBeeState(dir, ['start-feature', '--feature', 'swap-old', '--mode', 'standard', '--json']);
    assert(started.status === 0, `start-feature should succeed: ${started.stderr}`);
    const wfOldBefore = listWorkflows(dir).workflows.find((wf) => wf.feature === 'swap-old');
    assert(wfOldBefore, "precondition: swap-old has a live workflow record");

    const result = await runBeeState(dir, ['set', '--owner', 'exploring', '--feature', 'swap-new', '--phase', 'exploring', '--json']);
    assert(result.status === 0, `feature swap should succeed: ${result.stderr}`);

    const onDisk = readStateFile(dir);
    assert(onDisk.feature === 'swap-new', `state.json must carry the swap, got ${JSON.stringify(onDisk.feature)}`);

    const wfOldAfter = listWorkflows(dir).workflows.find((wf) => wf.feature === 'swap-old');
    assert(
      wfOldAfter.feature === 'swap-old' && wfOldAfter.phase === wfOldBefore.phase,
      `swap-old's own workflow record must be completely untouched by the swap, got ${JSON.stringify(wfOldAfter)}`,
    );
    assert(
      listWorkflows(dir).workflows.every((wf) => wf.feature !== 'swap-new'),
      'a swap never fabricates a workflow record for the NEW feature name either — it is not startFeature',
    );
  } finally {
    fs.rmSync(dir, { recursive: true, force: true });
  }
});

await check('state plan-rev bump refuses when resolution lands on the default (non-lane) record — C5 residual seam, zero mutation', async () => {
  const dir = makeStateRepo('bee-plan-rev-bump-default-');
  try {
    const result = await runBeeState(dir, ['plan-rev', 'bump', '--no-lane', '--json']);
    assert(result.status !== 0, `expected a refusal, got exit ${result.status}`);
    // --json mode: a refusal's message lands in the JSON error payload on
    // STDOUT (DB3 channel convention), not stderr.
    assert(
      /default \(non-lane\)|multisession-native-10/.test(result.stdout),
      `refusal should name the default/C5 seam, got stdout=${result.stdout}`,
    );
  } finally {
    fs.rmSync(dir, { recursive: true, force: true });
  }
});

await check('state plan-rev bump refuses when the named lane has no live workflow record, zero mutation', async () => {
  const dir = makeStateRepo('bee-plan-rev-bump-nolane-');
  try {
    laneStore.writeLane(dir, {
      schema_version: '1.0',
      feature: 'orphan-lane',
      mode: null,
      phase: 'exploring',
      approved_gates: { context: false, shape: false, execution: false, review: false },
      summary: '',
      next_action: '',
      created_at: new Date().toISOString(),
    });
    const result = await runBeeState(dir, ['plan-rev', 'bump', '--lane', 'orphan-lane', '--json']);
    assert(result.status !== 0, `expected a refusal, got exit ${result.status}`);
    assert(
      /no live workflow record/.test(result.stdout),
      `refusal should name the missing workflow record, got stdout=${result.stdout}`,
    );
  } finally {
    fs.rmSync(dir, { recursive: true, force: true });
  }
});

await check('bee state plan-rev with an unknown subaction reports "Use: bump" (usage-fallback branch)', async () => {
  const dir = makeStateRepo('bee-plan-rev-usage-');
  try {
    const result = await runBeeState(dir, ['plan-rev', 'bogus']);
    assert(result.status !== 0, `expected a refusal, got exit ${result.status}`);
    assert(/Unknown plan-rev action "bogus"\. Use: bump\./.test(result.stderr), `expected the plan-rev usage fallback, got stderr=${result.stderr}`);
  } finally {
    fs.rmSync(dir, { recursive: true, force: true });
  }
});

// ─── i54-closeout-7 (D7): write-path lane auto-resolution ───────────────────
// Writes resolve like reads: explicit --lane > the calling session's bound
// lane > the default record; --no-lane forces the default from a bound
// session. The child process's identity must be fully controlled here: strip
// the harness's own session env, then hand the fixture session id via
// BEE_SESSION_ID (the highest env rung of resolveSessionId's chain — B22).
const D7_BASE_ENV = { ...process.env };
delete D7_BASE_ENV.BEE_SESSION_ID;
delete D7_BASE_ENV.CLAUDE_CODE_SESSION_ID;
function runBeeStateAs(cwd, sessionId, args) {
  return runModuleWorker(beeStateModulePath(), {
    args: ['state', ...args],
    cwd,
    env: sessionId ? { ...D7_BASE_ENV, BEE_SESSION_ID: sessionId } : { ...D7_BASE_ENV },
  });
}

await check('D7: a lane-bound session with --lane omitted writes the LANE record across set/gate/scribing-run, with --owner judged against the lane pre-mutation phase and --feature refused', async () => {
  const dir = makeStateRepo('bee-state-d7-bound-');
  try {
    const defaultPath = path.join(dir, '.bee', 'state.json');
    const lanePath = path.join(dir, '.bee', 'lanes', 'alpha.json');
    writeJsonAtomic(defaultPath, { phase: 'validating', feature: 'default-feat', summary: 'untouched' });
    writeJsonAtomic(lanePath, { phase: 'swarming', feature: 'alpha', summary: 'before' });
    laneBinding.createSession(dir, { id: 'sess-d7' });
    laneBinding.bindSessionLane(dir, 'sess-d7', 'alpha');
    const defaultBefore = fs.readFileSync(defaultPath, 'utf8');

    // --owner comes from the SELECTED record: the default record's phase is
    // refused because the auto-resolved lane's phase is the contract.
    const wrongOwner = await runBeeStateAs(dir, 'sess-d7', ['set', '--owner', 'validating', '--summary', 'wrong']);
    assert(wrongOwner.status !== 0 && /--owner swarming/.test(wrongOwner.stderr), `owner is judged against the auto-resolved lane pre-mutation phase, got ${wrongOwner.stderr}`);
    assert(fs.readFileSync(defaultPath, 'utf8') === defaultBefore, 'owner refusal writes nothing');

    // --feature would rename the lane's identity (writeLane derives the file
    // from record.feature) — refused on the auto-resolved target too, naming
    // the --no-lane escape hatch.
    const feat = await runBeeStateAs(dir, 'sess-d7', ['set', '--owner', 'swarming', '--feature', 'renamed']);
    assert(feat.status !== 0 && /--no-lane/.test(feat.stderr), `--feature on an auto-resolved lane refuses and names --no-lane, got ${feat.stderr}`);

    const set = await runBeeStateAs(dir, 'sess-d7', ['set', '--owner', 'swarming', '--summary', 'lane-auto']);
    assert(set.status === 0, `bound-session set targets the lane, got ${set.status}: ${set.stderr}`);
    assert(/\(lane "alpha"\)/.test(set.stdout), `set output names the auto-resolved lane, got ${set.stdout}`);
    const gate = await runBeeStateAs(dir, 'sess-d7', ['gate', '--name', 'execution', '--approved', 'true']);
    assert(gate.status === 0 && /\(lane "alpha"\)/.test(gate.stdout), `bound-session gate targets the lane, got ${gate.status}: ${gate.stderr} ${gate.stdout}`);
    const scribe = await runBeeStateAs(dir, 'sess-d7', ['scribing-run', '--feature', 'alpha', '--areas', 'auth', '--next-action', 'bee-compounding']);
    assert(scribe.status === 0 && /\(lane "alpha"\)/.test(scribe.stdout), `bound-session scribing-run targets the lane, got ${scribe.status}: ${scribe.stderr} ${scribe.stdout}`);

    const lane = JSON.parse(fs.readFileSync(lanePath, 'utf8'));
    assert(lane.summary === 'lane-auto', 'set landed on the lane record');
    assert(lane.approved_gates && lane.approved_gates.execution === true, 'gate landed on the lane record');
    assert(lane.phase === 'compounding' && lane.last_scribing_run && lane.last_scribing_run.feature === 'alpha', 'scribing-run stamped the lane record and advanced its phase');
    assert(fs.readFileSync(defaultPath, 'utf8') === defaultBefore, 'default state.json bytes are never touched by lane-routed mutations');
  } finally {
    fs.rmSync(dir, { recursive: true, force: true });
  }
});

await check('D7: an unbound session (or an identity with no session record) with --lane omitted writes the default record exactly as before', async () => {
  const dir = makeStateRepo('bee-state-d7-unbound-');
  try {
    const defaultPath = path.join(dir, '.bee', 'state.json');
    const lanePath = path.join(dir, '.bee', 'lanes', 'alpha.json');
    writeJsonAtomic(defaultPath, { phase: 'validating', feature: 'default-feat', summary: 'untouched' });
    writeJsonAtomic(lanePath, { phase: 'swarming', feature: 'alpha', summary: 'before' });
    laneBinding.createSession(dir, { id: 'sess-free' });
    const laneBefore = fs.readFileSync(lanePath, 'utf8');

    const set = await runBeeStateAs(dir, 'sess-free', ['set', '--owner', 'validating', '--summary', 'default-write']);
    assert(set.status === 0, `unbound session writes the default record, got ${set.status}: ${set.stderr}`);
    assert(!/\(lane/.test(set.stdout), `unbound session output never names a lane, got ${set.stdout}`);
    assert(readStateFile(dir).summary === 'default-write', 'default record updated');

    // An identity with no session record in this repo is also "unbound".
    const ghost = await runBeeStateAs(dir, 'ghost-session', ['gate', '--name', 'context', '--approved', 'true']);
    assert(ghost.status === 0, `no-session identity writes the default record, got ${ghost.status}: ${ghost.stderr}`);
    assert(readStateFile(dir).approved_gates.context === true, 'gate landed on the default record');
    assert(fs.readFileSync(lanePath, 'utf8') === laneBefore, 'lane bytes never touched by unbound-session mutations');
  } finally {
    fs.rmSync(dir, { recursive: true, force: true });
  }
});

await check('D7: explicit --lane always wins over the session binding, and --no-lane forces the default record from a bound session', async () => {
  const dir = makeStateRepo('bee-state-d7-explicit-');
  try {
    const defaultPath = path.join(dir, '.bee', 'state.json');
    const alphaPath = path.join(dir, '.bee', 'lanes', 'alpha.json');
    const betaPath = path.join(dir, '.bee', 'lanes', 'beta.json');
    writeJsonAtomic(defaultPath, { phase: 'validating', feature: 'default-feat', summary: 'untouched' });
    writeJsonAtomic(alphaPath, { phase: 'swarming', feature: 'alpha', summary: 'before' });
    writeJsonAtomic(betaPath, { phase: 'exploring', feature: 'beta', summary: 'before' });
    laneBinding.createSession(dir, { id: 'sess-d7x' });
    laneBinding.bindSessionLane(dir, 'sess-d7x', 'alpha');
    const alphaBefore = fs.readFileSync(alphaPath, 'utf8');
    const defaultBefore = fs.readFileSync(defaultPath, 'utf8');

    const explicit = await runBeeStateAs(dir, 'sess-d7x', ['set', '--lane', 'beta', '--owner', 'exploring', '--summary', 'explicit-wins']);
    assert(explicit.status === 0 && /\(lane "beta"\)/.test(explicit.stdout), `explicit --lane beats the session binding, got ${explicit.status}: ${explicit.stderr} ${explicit.stdout}`);
    assert(JSON.parse(fs.readFileSync(betaPath, 'utf8')).summary === 'explicit-wins', 'explicit lane record updated');
    assert(fs.readFileSync(alphaPath, 'utf8') === alphaBefore, 'bound lane untouched when another lane is explicit');
    assert(fs.readFileSync(defaultPath, 'utf8') === defaultBefore, 'default untouched by explicit lane write');

    const forced = await runBeeStateAs(dir, 'sess-d7x', ['set', '--no-lane', '--owner', 'validating', '--summary', 'forced-default']);
    assert(forced.status === 0 && !/\(lane/.test(forced.stdout), `--no-lane forces the default record, got ${forced.status}: ${forced.stderr} ${forced.stdout}`);
    assert(readStateFile(dir).summary === 'forced-default', '--no-lane wrote the default record');
    assert(fs.readFileSync(alphaPath, 'utf8') === alphaBefore, '--no-lane leaves the bound lane untouched');

    const both = await runBeeStateAs(dir, 'sess-d7x', ['set', '--no-lane', '--lane', 'alpha', '--owner', 'swarming', '--summary', 'contradiction']);
    assert(both.status !== 0 && /--no-lane cannot be combined with --lane/.test(both.stderr), `--no-lane + --lane is refused, got ${both.stderr}`);
    assert(fs.readFileSync(alphaPath, 'utf8') === alphaBefore, 'contradictory selector writes nothing');
  } finally {
    fs.rmSync(dir, { recursive: true, force: true });
  }
});

await check('D7: a bound session whose lane record is corrupt or missing refuses loudly with zero writes — never a silent fall-back to the default record', async () => {
  const dir = makeStateRepo('bee-state-d7-corrupt-');
  try {
    const defaultPath = path.join(dir, '.bee', 'state.json');
    const lanePath = path.join(dir, '.bee', 'lanes', 'alpha.json');
    writeJsonAtomic(defaultPath, { phase: 'validating', feature: 'default-feat', summary: 'untouched' });
    fs.mkdirSync(path.dirname(lanePath), { recursive: true });
    fs.writeFileSync(lanePath, '{corrupt', 'utf8');
    laneBinding.createSession(dir, { id: 'sess-d7c' });
    laneBinding.bindSessionLane(dir, 'sess-d7c', 'alpha');
    const defaultBefore = fs.readFileSync(defaultPath, 'utf8');
    const corruptBefore = fs.readFileSync(lanePath, 'utf8');

    const corrupt = await runBeeStateAs(dir, 'sess-d7c', ['set', '--owner', 'validating', '--summary', 'must-not-land']);
    assert(corrupt.status !== 0 && /corrupt|could not read/.test(corrupt.stderr), `corrupt bound lane refuses loudly, got ${corrupt.status}: ${corrupt.stderr}`);
    assert(fs.readFileSync(defaultPath, 'utf8') === defaultBefore, 'corrupt-lane refusal never falls back to writing the default record');
    assert(fs.readFileSync(lanePath, 'utf8') === corruptBefore, 'corrupt lane bytes untouched');

    // Missing lane record: bound to a lane that was never started.
    laneBinding.createSession(dir, { id: 'sess-d7m' });
    laneBinding.bindSessionLane(dir, 'sess-d7m', 'ghost-lane');
    const missing = await runBeeStateAs(dir, 'sess-d7m', ['gate', '--name', 'context', '--approved', 'true']);
    assert(missing.status !== 0 && /ghost-lane/.test(missing.stderr) && /--no-lane/.test(missing.stderr), `missing bound lane refuses and names the --no-lane escape hatch, got ${missing.stderr}`);
    assert(fs.readFileSync(defaultPath, 'utf8') === defaultBefore, 'missing-lane refusal writes nothing');
  } finally {
    fs.rmSync(dir, { recursive: true, force: true });
  }
});

await check('D7: advisor-ref record follows the same write-path resolution — a bound session with --lane omitted records onto the LANE record', async () => {
  const dir = makeStateRepo('bee-state-d7-advisor-');
  try {
    const defaultPath = path.join(dir, '.bee', 'state.json');
    const lanePath = path.join(dir, '.bee', 'lanes', 'alpha.json');
    writeJsonAtomic(defaultPath, { phase: 'validating', feature: 'default-feat', summary: 'untouched' });
    writeJsonAtomic(lanePath, { phase: 'planning', feature: 'alpha', summary: 'before' });
    laneBinding.createSession(dir, { id: 'sess-d7a' });
    laneBinding.bindSessionLane(dir, 'sess-d7a', 'alpha');
    const defaultBefore = fs.readFileSync(defaultPath, 'utf8');
    const digestPath = path.join(dir, 'consult.txt');
    fs.writeFileSync(digestPath, 'advisor digest for the D7 row', 'utf8');

    const rec = await runBeeStateAs(dir, 'sess-d7a', ['advisor-ref', 'record', '--advisor', 'test-advisor', '--digest-file', digestPath]);
    assert(rec.status === 0 && /\(lane "alpha"\)/.test(rec.stdout), `bound-session advisor-ref record targets the lane, got ${rec.status}: ${rec.stderr} ${rec.stdout}`);
    const lane = JSON.parse(fs.readFileSync(lanePath, 'utf8'));
    assert(lane.advisor_ref && lane.advisor_ref.advisor === 'test-advisor' && lane.advisor_ref.feature === 'alpha', 'advisor_ref landed on the lane record with lane-bound anchors');
    assert(fs.readFileSync(defaultPath, 'utf8') === defaultBefore, 'default record untouched by the lane-routed advisor_ref');
  } finally {
    fs.rmSync(dir, { recursive: true, force: true });
  }
});

await check('bee.mjs state set treats a missing/invalid stored phase as corrupt before ownership and leaves bytes untouched', async () => {
  const dir = makeStateRepo('bee-state-owner-corrupt-phase-');
  try {
    const statePath = path.join(dir, '.bee', 'state.json');
    writeJsonAtomic(statePath, { phase: 'not-a-real-phase', summary: 'preserve' });
    const before = fs.readFileSync(statePath, 'utf8');
    const result = await runBeeState(dir, ['set', '--owner', 'not-a-real-phase', '--summary', 'must-not-land']);
    assert(result.status !== 0, 'invalid stored phase refuses mutation');
    assert(/invalid pre-mutation phase/.test(result.stderr) && /nothing was written/.test(result.stderr), `corrupt-phase refusal is explicit, got ${result.stderr}`);
    assert(fs.readFileSync(statePath, 'utf8') === before, 'invalid stored phase is byte-identical after refusal');
  } finally {
    fs.rmSync(dir, { recursive: true, force: true });
  }
});

await check('bee.mjs state gate approves a named gate and is idempotent (same call twice = identical file)', async () => {
  const dir = makeStateRepo('bee-state-gate-');
  try {
    const first = await runBeeState(dir, ['gate', '--name', 'execution', '--approved', 'true']);
    assert(first.status === 0, `gate should succeed, got ${first.status}: ${first.stderr}`);
    const state = readStateFile(dir);
    assert(state.approved_gates.execution === true, 'execution gate approved');
    assert(state.approved_gates.review === false, 'other gates untouched');
    const afterFirst = fs.readFileSync(path.join(dir, '.bee', 'state.json'), 'utf8');
    const second = await runBeeState(dir, ['gate', '--name', 'execution', '--approved', 'true']);
    assert(second.status === 0, 'second identical gate call also succeeds');
    const afterSecond = fs.readFileSync(path.join(dir, '.bee', 'state.json'), 'utf8');
    assert(afterFirst === afterSecond, 'gate --approved true run twice yields an identical file (idempotent)');
    const ownerRejected = await runBeeState(dir, ['gate', '--owner', 'idle', '--name', 'execution', '--approved', 'false']);
    assert(ownerRejected.status !== 0 && /--owner is not accepted/.test(ownerRejected.stderr), `dedicated gate rejects routing ownership, got ${ownerRejected.stderr}`);
    assert(fs.readFileSync(path.join(dir, '.bee', 'state.json'), 'utf8') === afterSecond, 'rejected gate --owner leaves gate state byte-identical');
  } finally {
    fs.rmSync(dir, { recursive: true, force: true });
  }
});

await check('bee.mjs state gate rejects an unknown gate name and a non-boolean --approved', async () => {
  const dir = makeStateRepo('bee-state-gate-bad-');
  try {
    const badName = await runBeeState(dir, ['gate', '--name', 'launch', '--approved', 'true']);
    assert(badName.status !== 0, 'unknown gate name rejected');
    // Pre-existing drift found (unrelated to si-1): the ce-1 refactor (see the
    // comment above handleStateGate) moved --name validation into the shared
    // requireFlags enum batching, which names the flag and the bad value
    // ("--name \"launch\" (must be one of ...)") rather than the bespoke
    // "gate name" wording this assertion still expected. The CLI behavior is
    // correct and unchanged by this cell; only this stale regex needed fixing
    // to match it.
    assert(/--name/i.test(badName.stderr) && /launch/.test(badName.stderr), `error names the bad --name value, got ${badName.stderr}`);
    const badBool = await runBeeState(dir, ['gate', '--name', 'context', '--approved', 'yes']);
    assert(badBool.status !== 0, 'non-boolean --approved rejected');
  } finally {
    fs.rmSync(dir, { recursive: true, force: true });
  }
});

await check('shipped routing callers declare their pre-phase owner and independent review has no generic state-set caller', async () => {
  const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '../../../..');
  const routingSkills = [
    'skills/bee-exploring/SKILL.md',
    'skills/bee-planning/SKILL.md',
    'skills/bee-validating/SKILL.md',
    'skills/bee-compounding/SKILL.md',
  ];
  const calls = routingSkills.flatMap((relative) => {
    const text = fs.readFileSync(path.join(repoRoot, relative), 'utf8');
    return [...text.matchAll(/node \.bee\/bin\/bee\.mjs state set\b[^`\n]*/g)].map((match) => ({ relative, call: match[0] }));
  });
  assert(calls.length === 5, `expected five shipped routing state-set calls, got ${JSON.stringify(calls)}`);
  for (const { relative, call } of calls) {
    assert(/\s--owner\s+\S+/.test(call), `${relative} has a state-set caller without explicit ownership: ${call}`);
  }
  const reviewing = fs.readFileSync(path.join(repoRoot, 'skills/bee-reviewing/SKILL.md'), 'utf8');
  assert(
    !/node \.bee\/bin\/bee\.mjs state set\b/.test(reviewing),
    'independent review must keep outcomes session-local and own no generic state mutation',
  );
});

await check('bee.mjs state worker add -> update -> remove -> clear round-trips and preserves unrelated fields', async () => {
  const dir = makeStateRepo('bee-state-worker-');
  try {
    const statePath = path.join(dir, '.bee', 'state.json');
    writeJsonAtomic(statePath, { schema_version: '1.0', phase: 'swarming', feature: 'demo', summary: 'keep-me' });

    const add = await runBeeState(dir, [
      'worker',
      'add',
      '--nickname',
      'sate',
      '--cell',
      'demo-1',
      '--tier',
      'generation',
      '--status',
      'in-flight',
    ]);
    assert(add.status === 0, `worker add should succeed, got ${add.status}: ${add.stderr}`);
    let state = readStateFile(dir);
    assert(state.workers.length === 1, 'one worker added');
    assert(
      state.workers[0].nickname === 'sate' &&
        state.workers[0].cell === 'demo-1' &&
        state.workers[0].tier === 'generation' &&
        state.workers[0].status === 'in-flight',
      'worker fields recorded',
    );
    assert(state.summary === 'keep-me', 'unrelated field untouched by worker add');

    const update = await runBeeState(dir, ['worker', 'update', '--nickname', 'sate', '--status', 'done']);
    assert(update.status === 0, `worker update should succeed, got ${update.status}: ${update.stderr}`);
    state = readStateFile(dir);
    assert(
      state.workers.length === 1 && state.workers[0].status === 'done' && state.workers[0].cell === 'demo-1',
      'update merges only the given field',
    );

    const badUpdate = await runBeeState(dir, ['worker', 'update', '--nickname', 'ghost', '--status', 'done']);
    assert(badUpdate.status !== 0, 'update on a missing nickname is rejected');

    const remove = await runBeeState(dir, ['worker', 'remove', '--nickname', 'sate']);
    assert(remove.status === 0, `worker remove should succeed, got ${remove.status}: ${remove.stderr}`);
    state = readStateFile(dir);
    assert(state.workers.length === 0, 'worker removed');

    const badRemove = await runBeeState(dir, ['worker', 'remove', '--nickname', 'sate']);
    assert(badRemove.status !== 0, 'removing an already-absent nickname is rejected');

    await runBeeState(dir, ['worker', 'add', '--nickname', 'a', '--cell', 'c1']);
    await runBeeState(dir, ['worker', 'add', '--nickname', 'b', '--cell', 'c2']);
    state = readStateFile(dir);
    assert(state.workers.length === 2, 'two workers present before clear');
    const clear = await runBeeState(dir, ['worker', 'clear']);
    assert(clear.status === 0, `worker clear should succeed, got ${clear.status}: ${clear.stderr}`);
    state = readStateFile(dir);
    assert(Array.isArray(state.workers) && state.workers.length === 0, 'clear empties the array');
    assert(state.summary === 'keep-me', 'unrelated field survives the full round-trip');
  } finally {
    fs.rmSync(dir, { recursive: true, force: true });
  }
});

await check('bee.mjs state worker add rejects an unknown tier', async () => {
  const dir = makeStateRepo('bee-state-worker-badtier-');
  try {
    const result = await runBeeState(dir, ['worker', 'add', '--nickname', 'x', '--cell', 'c1', '--tier', 'super-strong']);
    assert(result.status !== 0, 'unknown tier rejected');
    assert(/tier/i.test(result.stderr), `error names the tier, got ${result.stderr}`);
  } finally {
    fs.rmSync(dir, { recursive: true, force: true });
  }
});

// ─── bee.mjs state worker prune (workers-prune-1) ────────────────────────────

function makePruneRepo(prefix) {
  const dir = makeStateRepo(prefix);
  fs.mkdirSync(path.join(dir, '.bee', 'workers'), { recursive: true });
  fs.mkdirSync(path.join(dir, '.bee', 'cells'), { recursive: true });
  writeJsonAtomic(path.join(dir, '.bee', 'state.json'), {
    schema_version: '1.0',
    phase: 'swarming',
    workers: [
      { nickname: 'kevin', cell: 'live-1', tier: 'generation', status: 'in-flight' },
      { nickname: 'bob', cell: 'alpha.out10', tier: 'generation', status: 'in-flight' },
    ],
  });
  writeJsonAtomic(path.join(dir, '.bee', 'cells', 'done-1.json'), { id: 'done-1', status: 'capped' });
  writeJsonAtomic(path.join(dir, '.bee', 'cells', 'open-1.json'), { id: 'open-1', status: 'open' });
  const w = (name) => fs.writeFileSync(path.join(dir, '.bee', 'workers', name), 'x', 'utf8');
  w('done-1.prompt.md'); // capped cell -> prunable
  w('done-1.out.log'); // capped cell -> prunable
  w('done-1.out2.log'); // .outN.log belongs to the same cell id -> prunable
  w('done-1.result.json'); // capped cell -> prunable
  w('open-1.prompt.md'); // open cell -> kept
  w('live-1.result.md'); // active worker's cell (no cell file) -> kept
  w('alpha.out10.log'); // dotted active cell id: suffix regex must not mis-stem it -> kept
  w('review-arch.log'); // no cell, no active worker -> prunable
  w('evidence-pre.json'); // bare .json outside the suffix set -> never touched
  w('.log'); // empty stem -> not a dispatch transient, never touched
  w('.out10.log'); // empty stem -> never touched
  fs.mkdirSync(path.join(dir, '.bee', 'workers', 'nested'), { recursive: true });
  fs.writeFileSync(path.join(dir, '.bee', 'workers', 'nested', 'sub.prompt.md'), 'x', 'utf8'); // subdir contents -> never touched
  return dir;
}

const PRUNE_EXPECTED = ['done-1.out.log', 'done-1.out2.log', 'done-1.prompt.md', 'done-1.result.json', 'review-arch.log'];
const PRUNE_SURVIVORS = ['.log', '.out10.log', 'alpha.out10.log', 'evidence-pre.json', 'live-1.result.md', 'nested', 'open-1.prompt.md'];

function workerFiles(dir) {
  return fs.readdirSync(path.join(dir, '.bee', 'workers')).sort();
}

await check('bee.mjs state worker prune deletes only capped/orphan transients and keeps open-cell, active-worker (dotted ids included), subdir, and non-transient files', async () => {
  const dir = makePruneRepo('bee-state-prune-');
  try {
    const stateBefore = fs.readFileSync(path.join(dir, '.bee', 'state.json'), 'utf8');
    const result = await runBeeState(dir, ['worker', 'prune', '--json']);
    assert(result.status === 0, `prune should succeed, got ${result.status}: ${result.stderr}`);
    const out = JSON.parse(result.stdout);
    assert(
      JSON.stringify(out.pruned) === JSON.stringify(PRUNE_EXPECTED),
      `pruned set, got ${JSON.stringify(out.pruned)}`,
    );
    assert(
      JSON.stringify(workerFiles(dir)) === JSON.stringify(PRUNE_SURVIVORS),
      `survivors, got ${JSON.stringify(workerFiles(dir))}`,
    );
    assert(
      fs.existsSync(path.join(dir, '.bee', 'workers', 'nested', 'sub.prompt.md')),
      'subdirectory contents untouched',
    );
    const stateAfter = fs.readFileSync(path.join(dir, '.bee', 'state.json'), 'utf8');
    assert(stateBefore === stateAfter, 'prune never writes state.json');
  } finally {
    fs.rmSync(dir, { recursive: true, force: true });
  }
});

await check('bee.mjs state worker prune --dry-run reports the exact same candidate set and deletes nothing', async () => {
  const dir = makePruneRepo('bee-state-prune-dry-');
  try {
    const before = workerFiles(dir);
    const result = await runBeeState(dir, ['worker', 'prune', '--dry-run', '--json']);
    assert(result.status === 0, `dry-run should succeed, got ${result.status}: ${result.stderr}`);
    const out = JSON.parse(result.stdout);
    assert(out.dry_run === true, 'dry_run flagged in output');
    assert(
      JSON.stringify(out.pruned) === JSON.stringify(PRUNE_EXPECTED),
      `dry-run candidate set is exactly the real prune set, got ${JSON.stringify(out.pruned)}`,
    );
    assert(JSON.stringify(workerFiles(dir)) === JSON.stringify(before), 'no file deleted under --dry-run');
  } finally {
    fs.rmSync(dir, { recursive: true, force: true });
  }
});

await check('bee.mjs state worker prune rejects unknown flags (a --dryrun typo must never delete) and non-prune verbs reject --dry-run', async () => {
  const dir = makePruneRepo('bee-state-prune-strictflags-');
  try {
    const before = workerFiles(dir);
    const typo = await runBeeState(dir, ['worker', 'prune', '--dryrun', '--json']);
    assert(typo.status !== 0, `--dryrun typo exits non-zero, got ${typo.status}`);
    assert(/dryrun/.test(typo.stderr), `error names the unknown flag, got ${typo.stderr}`);
    assert(JSON.stringify(workerFiles(dir)) === JSON.stringify(before), 'zero deletions on an unknown flag');
    const stateBefore = fs.readFileSync(path.join(dir, '.bee', 'state.json'), 'utf8');
    const clearDry = await runBeeState(dir, ['worker', 'clear', '--dry-run']);
    assert(clearDry.status !== 0, `worker clear --dry-run exits non-zero, got ${clearDry.status}`);
    assert(/dry-run/.test(clearDry.stderr), `error names --dry-run, got ${clearDry.stderr}`);
    const stateAfter = fs.readFileSync(path.join(dir, '.bee', 'state.json'), 'utf8');
    assert(stateBefore === stateAfter, 'a refused dry-run mutation leaves state.json untouched');
  } finally {
    fs.rmSync(dir, { recursive: true, force: true });
  }
});

await check('bee.mjs state worker prune fails closed when state.workers is not an array (semantic corruption, valid JSON)', async () => {
  const dir = makePruneRepo('bee-state-prune-badworkers-');
  try {
    writeJsonAtomic(path.join(dir, '.bee', 'state.json'), {
      schema_version: '1.0',
      phase: 'swarming',
      workers: { nickname: 'kevin', cell: 'live-1' },
    });
    const before = workerFiles(dir);
    const result = await runBeeState(dir, ['worker', 'prune']);
    assert(result.status !== 0, `malformed workers exits non-zero, got ${result.status}`);
    assert(/workers/.test(result.stderr), `error names state.workers, got ${result.stderr}`);
    assert(JSON.stringify(workerFiles(dir)) === JSON.stringify(before), 'zero deletions when the keep set is malformed');
  } finally {
    fs.rmSync(dir, { recursive: true, force: true });
  }
});

await check('bee.mjs state worker prune over a corrupt state.json exits non-zero and deletes nothing (readStateStrict before any rm)', async () => {
  const dir = makePruneRepo('bee-state-prune-corrupt-');
  try {
    fs.writeFileSync(path.join(dir, '.bee', 'state.json'), '{ not json', 'utf8');
    const before = workerFiles(dir);
    const result = await runBeeState(dir, ['worker', 'prune']);
    assert(result.status !== 0, `corrupt state exits non-zero, got ${result.status}`);
    assert(JSON.stringify(workerFiles(dir)) === JSON.stringify(before), 'zero deletions on a corrupt state');
  } finally {
    fs.rmSync(dir, { recursive: true, force: true });
  }
});

await check('bee.mjs state worker prune with no .bee/workers dir succeeds with 0 pruned, and the unknown-action Use: line lists prune', async () => {
  const dir = makeStateRepo('bee-state-prune-nodir-');
  try {
    const result = await runBeeState(dir, ['worker', 'prune', '--json']);
    assert(result.status === 0, `missing dir is success, got ${result.status}: ${result.stderr}`);
    const out = JSON.parse(result.stdout);
    assert(out.pruned.length === 0, 'nothing pruned when the dir is absent');
    const bad = await runBeeState(dir, ['worker', 'shave']);
    assert(bad.status !== 0, 'unknown worker action exits non-zero');
    assert(/prune/.test(bad.stderr), `Use: line lists prune, got ${bad.stderr}`);
  } finally {
    fs.rmSync(dir, { recursive: true, force: true });
  }
});

await check('bee.mjs state scribing-run stamps the exact key set from bee-scribing SKILL.md:112 including an ISO-precise at', async () => {
  const dir = makeStateRepo('bee-state-scribing-');
  try {
    writeJsonAtomic(path.join(dir, '.bee', 'state.json'), {
      schema_version: '1.0',
      phase: 'scribing',
      feature: 'demo',
    });
    const result = await runBeeState(dir, [
      'scribing-run',
      '--feature',
      'demo',
      '--areas',
      'auth,billing',
      '--next-action',
      'bee-compounding',
    ]);
    assert(result.status === 0, `scribing-run should succeed, got ${result.status}: ${result.stderr}`);
    const state = readStateFile(dir);
    const run = state.last_scribing_run;
    assert(run && run.feature === 'demo', 'feature stamped');
    assert(typeof run.date === 'string' && /^\d{4}-\d{2}-\d{2}$/.test(run.date), `date is day-precise, got ${run.date}`);
    assert(
      typeof run.at === 'string' && !Number.isNaN(Date.parse(run.at)) && run.at.length > run.date.length,
      `at is ISO-precise, got ${run.at}`,
    );
    assert(
      Array.isArray(run.areas_synced) && run.areas_synced.join(',') === 'auth,billing',
      `areas_synced parsed from the comma list, got ${JSON.stringify(run.areas_synced)}`,
    );
    assert(run.next_action === 'bee-compounding', 'next_action stamped in last_scribing_run');
    assert(
      state.next_action === 'bee-compounding',
      'top-level next_action mirrors the flag (SKILL.md:112 "plus top-level phase/next_action")',
    );
    assert(
      state.phase === 'compounding',
      'top-level phase advances to compounding, the fixed next chain node after bee-scribing',
    );
  } finally {
    fs.rmSync(dir, { recursive: true, force: true });
  }
});

await check('bee.mjs state scribing-run accepts a single descriptive area with no comma (real-world shape)', async () => {
  const dir = makeStateRepo('bee-state-scribing-single-');
  try {
    // chain-integrity D3: scribing-run now demands a phase where execution
    // actually happened — it used to advance to `compounding` from anywhere,
    // including the `idle` default this fixture relied on.
    writeJsonAtomic(path.join(dir, '.bee', 'state.json'), { phase: 'swarming' });
    const result = await runBeeState(dir, [
      'scribing-run',
      '--feature',
      'demo',
      '--areas',
      'no docs/specs area sync needed — hooks-as-source convention',
      '--next-action',
      'bee-compounding',
    ]);
    assert(result.status === 0, `scribing-run should succeed, got ${result.status}: ${result.stderr}`);
    const state = readStateFile(dir);
    assert(state.last_scribing_run.areas_synced.length === 1, 'a single descriptive sentence stays one array element');
  } finally {
    fs.rmSync(dir, { recursive: true, force: true });
  }
});

await check('bee.mjs state rejects an unknown verb with a Use: line, exit non-zero', async () => {
  const dir = makeStateRepo('bee-state-unknown-');
  try {
    const result = await runBeeState(dir, ['launch']);
    assert(result.status !== 0, 'unknown verb exits non-zero');
    assert(/Use:/.test(result.stderr), `error names the Use: line, got ${result.stderr}`);
  } finally {
    fs.rmSync(dir, { recursive: true, force: true });
  }
});

// ─── bee.mjs state start-feature (codex-parity-5, decision D2, plan.md test ─
// matrix row 5 "state transitions") — the guarded atomic feature-start verb.
// Every refusal test asserts BOTH non-zero exit AND byte-identical state.json
// before/after (zero mutations on refusal), matching the file's established
// "leaves the file untouched" idiom.

function makeCellFile(dir, id, extra = {}) {
  fs.mkdirSync(path.join(dir, '.bee', 'cells'), { recursive: true });
  const cell = {
    id,
    feature: 'old-feature',
    title: `Cell ${id}`,
    lane: 'tiny',
    status: 'open',
    deps: [],
    action: 'do it',
    verify: 'node -e "process.exit(0)"',
    trace: {},
    ...extra,
  };
  writeJsonAtomic(path.join(dir, '.bee', 'cells', `${id}.json`), cell);
  return cell;
}

await check('start-feature (lib): succeeds from idle with no leftover work, resets all four gates and writes feature/mode/phase in one call', async () => {
  const dir = makeStateRepo('bee-state-start-lib-ok-');
  try {
    writeJsonAtomic(path.join(dir, '.bee', 'state.json'), {
      schema_version: '1.0',
      phase: 'idle',
      feature: null,
      mode: null,
      approved_gates: { context: false, shape: false, execution: false, review: false },
      workers: [],
      summary: 'prior',
      next_action: 'prior next',
    });
    const state = await startFeature(dir, { feature: 'new-feat', mode: 'standard', phase: 'exploring' });
    assert(state.feature === 'new-feat', `feature written, got ${state.feature}`);
    assert(state.mode === 'standard', `mode written, got ${state.mode}`);
    assert(state.phase === 'exploring', `phase written, got ${state.phase}`);
    assert(
      state.approved_gates.context === false &&
        state.approved_gates.shape === false &&
        state.approved_gates.execution === false &&
        state.approved_gates.review === false,
      `all four gates reset false, got ${JSON.stringify(state.approved_gates)}`,
    );
  } finally {
    fs.rmSync(dir, { recursive: true, force: true });
  }
});

await check('start-feature (lib): a prior feature carrying approved gates never lets the new feature inherit them', async () => {
  const dir = makeStateRepo('bee-state-start-lib-inherit-');
  try {
    writeJsonAtomic(path.join(dir, '.bee', 'state.json'), {
      schema_version: '1.0',
      phase: 'compounding-complete',
      feature: 'old-feature',
      mode: 'standard',
      approved_gates: { context: true, shape: true, execution: true, review: true },
      workers: [],
    });
    const state = await startFeature(dir, { feature: 'next-feat' });
    assert(
      Object.values(state.approved_gates).every((v) => v === false),
      `no gate carried across features, got ${JSON.stringify(state.approved_gates)}`,
    );
  } finally {
    fs.rmSync(dir, { recursive: true, force: true });
  }
});

await check('bee.mjs state start-feature requires --feature', async () => {
  const dir = makeStateRepo('bee-state-start-nofeat-');
  try {
    const result = await runBeeState(dir, ['start-feature']);
    assert(result.status !== 0, 'missing --feature exits non-zero');
    assert(/feature/i.test(result.stderr), `error names the missing flag, got ${result.stderr}`);
  } finally {
    fs.rmSync(dir, { recursive: true, force: true });
  }
});

await check('bee.mjs state start-feature rejects a phase outside the closed vocabulary, zero mutations', async () => {
  const dir = makeStateRepo('bee-state-start-badphase-');
  try {
    const statePath = path.join(dir, '.bee', 'state.json');
    writeJsonAtomic(statePath, { schema_version: '1.0', phase: 'idle', workers: [] });
    const before = fs.readFileSync(statePath, 'utf8');
    const result = await runBeeState(dir, ['start-feature', '--feature', 'f1', '--phase', 'launched']);
    assert(result.status !== 0, 'invented phase exits non-zero');
    assert(/phase/i.test(result.stderr), `error names the phase, got ${result.stderr}`);
    const after = fs.readFileSync(statePath, 'utf8');
    assert(before === after, 'file untouched after a rejected phase');
  } finally {
    fs.rmSync(dir, { recursive: true, force: true });
  }
});

await check('bee.mjs state start-feature refuses when the current phase is not idle/terminal, zero mutations', async () => {
  const dir = makeStateRepo('bee-state-start-midflight-');
  try {
    const statePath = path.join(dir, '.bee', 'state.json');
    writeJsonAtomic(statePath, { schema_version: '1.0', phase: 'swarming', feature: 'old-feature', workers: [] });
    const before = fs.readFileSync(statePath, 'utf8');
    const result = await runBeeState(dir, ['start-feature', '--feature', 'new-feat']);
    assert(result.status !== 0, 'mid-flight phase refuses');
    assert(/phase/i.test(result.stderr), `error names the phase problem, got ${result.stderr}`);
    const after = fs.readFileSync(statePath, 'utf8');
    assert(before === after, 'file untouched after a mid-flight refusal');
  } finally {
    fs.rmSync(dir, { recursive: true, force: true });
  }
});

await check('bee.mjs state start-feature refuses while .bee/HANDOFF.json names THIS feature, zero mutations (F5: per-feature scoped)', async () => {
  const dir = makeStateRepo('bee-state-start-handoff-');
  try {
    const statePath = path.join(dir, '.bee', 'state.json');
    writeJsonAtomic(statePath, { schema_version: '1.0', phase: 'idle', workers: [] });
    writeJsonAtomic(path.join(dir, '.bee', 'HANDOFF.json'), { feature: 'new-feat', cell: 'x', done: [], remaining: [] });
    const before = fs.readFileSync(statePath, 'utf8');
    const result = await runBeeState(dir, ['start-feature', '--feature', 'new-feat']);
    assert(result.status !== 0, 'active HANDOFF naming this feature refuses');
    assert(/HANDOFF/.test(result.stderr), `error names HANDOFF.json, got ${result.stderr}`);
    const after = fs.readFileSync(statePath, 'utf8');
    assert(before === after, 'file untouched after a HANDOFF refusal');
  } finally {
    fs.rmSync(dir, { recursive: true, force: true });
  }
});

// F5 (advisor consult slice 2, binding, multisession-native-6): the default
// path's HANDOFF precondition is now scoped per-feature exactly like the
// lane path (state.mjs startFeature header comment) — a handoff naming a
// DIFFERENT feature (or carrying no feature field at all, the pre-scoping
// legacy shape) no longer blocks an unrelated start.
await check('bee.mjs state start-feature succeeds despite .bee/HANDOFF.json naming a DIFFERENT feature (F5)', async () => {
  const dir = makeStateRepo('bee-state-start-handoff-unrelated-');
  try {
    const statePath = path.join(dir, '.bee', 'state.json');
    writeJsonAtomic(statePath, { schema_version: '1.0', phase: 'idle', workers: [] });
    writeJsonAtomic(path.join(dir, '.bee', 'HANDOFF.json'), { feature: 'other-feat', cell: 'x', done: [], remaining: [] });
    const result = await runBeeState(dir, ['start-feature', '--feature', 'new-feat']);
    assert(result.status === 0, `unrelated-feature HANDOFF must not block, got ${result.status}: ${result.stderr}`);
    const state = readStateFile(dir);
    assert(state.feature === 'new-feat', 'new feature recorded despite the unrelated handoff');
  } finally {
    fs.rmSync(dir, { recursive: true, force: true });
  }
});

// multisession-native-8 (D6): the workers precondition no longer trusts
// state.json's hand-mutated `workers` array — it now reads claims.mjs's
// activeWorkers derived view (live-heartbeat sessions x claims). A
// hand-written entry with nobody actually heartbeating is display data
// only and must never block anymore.
await check('bee.mjs state start-feature: a hand-written state.workers entry alone no longer blocks (D6 demotion, msn-8) — the array is display-only', async () => {
  const dir = makeStateRepo('bee-state-start-worker-ghost-');
  try {
    const statePath = path.join(dir, '.bee', 'state.json');
    writeJsonAtomic(statePath, {
      schema_version: '1.0',
      phase: 'idle',
      workers: [{ nickname: 'bob', cell: 'x-1', tier: 'generation', status: 'in-flight' }],
    });
    const result = await runBeeState(dir, ['start-feature', '--feature', 'new-feat']);
    assert(result.status === 0, `a hand-written workers entry with no live session must never block, got ${result.status}: ${result.stderr}`);
  } finally {
    fs.rmSync(dir, { recursive: true, force: true });
  }
});

await check('bee.mjs state start-feature refuses while ANOTHER session has a live heartbeat + an active claim, zero mutations', async () => {
  const dir = makeStateRepo('bee-state-start-worker-live-');
  try {
    const statePath = path.join(dir, '.bee', 'state.json');
    writeJsonAtomic(statePath, { schema_version: '1.0', phase: 'idle', workers: [] });
    createSession(dir, { id: 'other-live' });
    claimCellFile(dir, 'other-live', 'x-1');
    const before = fs.readFileSync(statePath, 'utf8');
    const result = await runBeeState(dir, ['start-feature', '--feature', 'new-feat', '--session-id', 'caller']);
    assert(result.status !== 0, 'a live other session with an active claim refuses');
    assert(/worker/i.test(result.stderr), `error names the active worker, got ${result.stderr}`);
    assert(/other-live/.test(result.stderr), `error names the live session, got ${result.stderr}`);
    const after = fs.readFileSync(statePath, 'utf8');
    assert(before === after, 'file untouched after a worker refusal');
  } finally {
    fs.rmSync(dir, { recursive: true, force: true });
  }
});

// C3 (advisor consult slice 2, binding — RED-first against pre-msn-8
// startFeature: without excludeSessionId wired through, a solo session's own
// live heartbeat would be counted as "an active worker" by the naive derived
// view and incorrectly refuse its own start).
await check('bee.mjs state start-feature: the calling session\'s own live heartbeat never blocks its own start (C3 — excludeSessionId pattern, msn-8)', async () => {
  const dir = makeStateRepo('bee-state-start-worker-solo-');
  try {
    createSession(dir, { id: 'solo' });
    const result = await runBeeState(dir, ['start-feature', '--feature', 'solo-feat', '--session-id', 'solo']);
    assert(result.status === 0, `a solo session must never be blocked by its own heartbeat, got ${result.status}: ${result.stderr}`);
  } finally {
    fs.rmSync(dir, { recursive: true, force: true });
  }
});

await check('bee.mjs state start-feature refuses while an active reservation remains, zero mutations; an expired one does not block', async () => {
  const dir = makeStateRepo('bee-state-start-reservation-');
  try {
    const statePath = path.join(dir, '.bee', 'state.json');
    writeJsonAtomic(statePath, { schema_version: '1.0', phase: 'idle', workers: [] });
    // multisession-native-16: `.bee/reservations.json` is a rebuildable
    // PROJECTION over lease-store.mjs now — startFeature's precondition
    // (state.mjs's listActiveReservationsForStart) reads THROUGH the shim
    // (listReservations) rather than this file (advisor consult slice 3
    // condition C), so a fixture reservation must be a REAL lease, seeded via
    // reserve() itself.
    const made = await reserve(dir, { agent: 'bob', cell: 'x-1', path: 'src/app.ts' });
    assert(made.ok === true, 'precondition: the reservation was actually acquired');
    const before = fs.readFileSync(statePath, 'utf8');
    const result = await runBeeState(dir, ['start-feature', '--feature', 'new-feat']);
    assert(result.status !== 0, 'active reservation refuses');
    assert(/reservation/i.test(result.stderr), `error names the reservation, got ${result.stderr}`);
    const after = fs.readFileSync(statePath, 'utf8');
    assert(before === after, 'file untouched after a reservation refusal');

    // an EXPIRED reservation (reserved long before its own ttl) is not
    // "active" — backdate the SAME underlying lease directly (renewLease)
    // rather than writing a second, colliding reserve() at the same path.
    await renewLease(dir, { type: 'path', id: 'src/app.ts' }, { ttl: 60, now: Date.now() - 7200 * 1000 });
    const retry = await runBeeState(dir, ['start-feature', '--feature', 'new-feat']);
    assert(retry.status === 0, `expired reservation must not block start-feature, got ${retry.status}: ${retry.stderr}`);
  } finally {
    fs.rmSync(dir, { recursive: true, force: true });
  }
});

await check('bee.mjs state start-feature refuses while ANY cell anywhere is claimed, zero mutations', async () => {
  const dir = makeStateRepo('bee-state-start-claimed-');
  try {
    const statePath = path.join(dir, '.bee', 'state.json');
    writeJsonAtomic(statePath, { schema_version: '1.0', phase: 'idle', feature: null, workers: [] });
    makeCellFile(dir, 'unrelated-1', { feature: 'some-other-feature', status: 'claimed' });
    const before = fs.readFileSync(statePath, 'utf8');
    const result = await runBeeState(dir, ['start-feature', '--feature', 'new-feat']);
    assert(result.status !== 0, 'a claimed cell anywhere refuses, even for an unrelated feature');
    assert(/claimed/i.test(result.stderr), `error names the claimed cell, got ${result.stderr}`);
    const after = fs.readFileSync(statePath, 'utf8');
    assert(before === after, 'file untouched after a claimed-cell refusal');
  } finally {
    fs.rmSync(dir, { recursive: true, force: true });
  }
});

await check('bee.mjs state start-feature refuses while the PRIOR feature has a nonterminal (open/blocked) cell, and succeeds once each is dropped via the existing drop verb (P1 repair: no auto-clear cleanup)', async () => {
  const dir = makeStateRepo('bee-state-start-nonterminal-');
  try {
    const statePath = path.join(dir, '.bee', 'state.json');
    writeJsonAtomic(statePath, {
      schema_version: '1.0',
      phase: 'compounding-complete',
      feature: 'old-feature',
      workers: [],
    });
    makeCellFile(dir, 'old-1', { status: 'open' });
    makeCellFile(dir, 'old-2', { status: 'blocked' });
    const before = fs.readFileSync(statePath, 'utf8');

    const result = await runBeeState(dir, ['start-feature', '--feature', 'new-feat']);
    assert(result.status !== 0, 'nonterminal prior-feature cells refuse');
    assert(/old-feature/.test(result.stderr), `error names the prior feature, got ${result.stderr}`);
    assert(/old-1/.test(result.stderr) && /old-2/.test(result.stderr), `error lists both nonterminal cells, got ${result.stderr}`);
    const after = fs.readFileSync(statePath, 'utf8');
    assert(before === after, 'file untouched while nonterminal cells remain');

    // Resolve through the EXISTING drop verb (lib/cells.mjs dropCell) — never
    // an auto-clear inside startFeature itself.
    await dropCell(dir, 'old-1', 'abandoned, superseded by new-feat');
    await dropCell(dir, 'old-2', 'abandoned, superseded by new-feat');
    assert(readCell(dir, 'old-1').status === 'dropped', 'old-1 dropped');
    assert(readCell(dir, 'old-2').status === 'dropped', 'old-2 dropped');

    const retry = await runBeeState(dir, ['start-feature', '--feature', 'new-feat', '--phase', 'exploring']);
    assert(retry.status === 0, `start-feature succeeds once every nonterminal cell is dropped, got ${retry.status}: ${retry.stderr}`);
    const state = readStateFile(dir);
    assert(state.feature === 'new-feat', 'new feature recorded');
    assert(state.phase === 'exploring', 'phase advanced to exploring');
  } finally {
    fs.rmSync(dir, { recursive: true, force: true });
  }
});

await check('bee.mjs state start-feature: a CAPPED prior-feature cell is terminal and never blocks (only open/claimed/blocked do)', async () => {
  const dir = makeStateRepo('bee-state-start-capped-ok-');
  try {
    writeJsonAtomic(path.join(dir, '.bee', 'state.json'), {
      schema_version: '1.0',
      phase: 'compounding-complete',
      feature: 'old-feature',
      workers: [],
    });
    makeCellFile(dir, 'old-done', { status: 'capped' });
    const result = await runBeeState(dir, ['start-feature', '--feature', 'new-feat']);
    assert(result.status === 0, `a fully capped prior feature never blocks a new start, got ${result.status}: ${result.stderr}`);
  } finally {
    fs.rmSync(dir, { recursive: true, force: true });
  }
});

await check('bee.mjs state start-feature defaults --phase to "exploring" and --mode to null when omitted', async () => {
  const dir = makeStateRepo('bee-state-start-defaults-');
  try {
    writeJsonAtomic(path.join(dir, '.bee', 'state.json'), { schema_version: '1.0', phase: 'idle', workers: [] });
    const result = await runBeeState(dir, ['start-feature', '--feature', 'defaulted-feat']);
    assert(result.status === 0, `default start succeeds, got ${result.status}: ${result.stderr}`);
    const state = readStateFile(dir);
    assert(state.phase === 'exploring', `phase defaults to exploring, got ${state.phase}`);
    assert(state.mode === null, `mode defaults to null, got ${state.mode}`);
  } finally {
    fs.rmSync(dir, { recursive: true, force: true });
  }
});

await check('bee.mjs state start-feature rejects --dry-run (a mutating verb, same generic guard as every non-prune verb)', async () => {
  const dir = makeStateRepo('bee-state-start-dryrun-');
  try {
    writeJsonAtomic(path.join(dir, '.bee', 'state.json'), { schema_version: '1.0', phase: 'idle', workers: [] });
    const result = await runBeeState(dir, ['start-feature', '--feature', 'f1', '--dry-run']);
    assert(result.status !== 0, '--dry-run on start-feature is rejected');
  } finally {
    fs.rmSync(dir, { recursive: true, force: true });
  }
});

// ─── bee.mjs buildStatus/renderStatusText: lanes block (fsh-6, D4) ──────────

function beeMjsModulePath() {
  return fileURLToPath(new URL('../bee.mjs', import.meta.url));
}

function runBeeMjs(cwd, args, opts = {}) {
  return runModuleWorker(beeMjsModulePath(), { args, cwd, ...opts });
}

// lpsp-2 (P2, payload-size): the `lanes` block was measured at 58% of a full
// `status --json` payload on this repo — a per-session context tax paid at
// every session start/compaction (AGENTS.md step 3). `lanes` now summarizes
// by DEFAULT (active lane in full + counts/ids for the rest); `--lanes-full`
// restores the pre-cell full per-lane array unchanged. This whole worker
// process may itself be running inside a bee session (BEE_SESSION_ID /
// CLAUDE_CODE_SESSION_ID set in the ambient env that runModuleWorker inherits
// by default), which would otherwise leak in as the "acting session" for the
// spawned `bee status` call and shadow the fixture's own `sess-lx` identity —
// CLEAN_ENV strips both so resolveSessionId falls through to root-inference
// (exactly one live session on disk -> adopts it), deterministic either way.
const CLEAN_ENV = { ...process.env };
delete CLEAN_ENV.BEE_SESSION_ID;
delete CLEAN_ENV.CLAUDE_CODE_SESSION_ID;

await check('bee.mjs status --json summarizes `lanes` by default (active lane in full + counts/ids for the rest); --lanes-full restores the full per-lane array; zero lanes on disk renders the empty shape either way; every pre-existing status field keeps its exact shape', async () => {
  const dir = makeStateRepo('bee-status-lanes-');
  try {
    const zero = await runBeeMjs(dir, ['status', '--json'], { env: CLEAN_ENV });
    assert(zero.status === 0, `bee.mjs status --json exited ${zero.status} :: ${zero.stderr}`);
    const zeroPayload = JSON.parse(zero.stdout);
    assert(
      zeroPayload.lanes && typeof zeroPayload.lanes === 'object' && !Array.isArray(zeroPayload.lanes)
        && zeroPayload.lanes.active === null && Object.keys(zeroPayload.lanes.counts).length === 0 && zeroPayload.lanes.ids.length === 0,
      `zero lanes on disk renders the empty summary shape {active:null,counts:{},ids:[]}, got ${JSON.stringify(zeroPayload.lanes)}`,
    );
    assert(zeroPayload.phase === 'idle', 'pre-existing top-level phase field keeps its exact zero-lane shape');
    assert(!/Lanes:/.test((await runBeeMjs(dir, ['status'], { env: CLEAN_ENV })).stdout), 'text render carries no Lanes line when no lanes exist (zero-lane byte parity)');

    const zeroFull = await runBeeMjs(dir, ['status', '--lanes-full', '--json'], { env: CLEAN_ENV });
    assert(Array.isArray(JSON.parse(zeroFull.stdout).lanes) && JSON.parse(zeroFull.stdout).lanes.length === 0, '--lanes-full on zero lanes still renders the pre-cell empty ARRAY shape, unchanged');

    laneStore.writeLane(dir, {
      schema_version: '1.0',
      feature: 'lane-x',
      mode: 'standard',
      phase: 'swarming',
      approved_gates: { context: true, shape: true, execution: true, review: false },
      summary: '',
      next_action: '',
      created_at: new Date().toISOString(),
    });
    laneBinding.createSession(dir, { id: 'sess-lx' });
    laneBinding.bindSessionLane(dir, 'sess-lx', 'lane-x');

    // --lanes-full: today's exact pre-cell shape, byte-unchanged.
    const withLaneFull = await runBeeMjs(dir, ['status', '--lanes-full', '--json'], { env: CLEAN_ENV });
    const fullPayload = JSON.parse(withLaneFull.stdout);
    assert(Array.isArray(fullPayload.lanes) && fullPayload.lanes.length === 1, `--lanes-full lists the one lane record, got ${JSON.stringify(fullPayload.lanes)}`);
    const row = fullPayload.lanes[0];
    assert(row.feature === 'lane-x' && row.phase === 'swarming', `lane row carries feature/phase, got ${JSON.stringify(row)}`);
    assert(row.approved_gates && row.approved_gates.execution === true, 'lane row carries its own approved_gates');
    assert(Array.isArray(row.bound_sessions) && row.bound_sessions.includes('sess-lx'), `lane row names the bound session, got ${JSON.stringify(row.bound_sessions)}`);
    assert(fullPayload.phase === 'idle', 'the pre-existing top-level phase field is untouched by --lanes-full (it stays the default pipeline)');

    const textFull = (await runBeeMjs(dir, ['status', '--lanes-full'], { env: CLEAN_ENV })).stdout;
    assert(/Lanes: lane-x \[swarming\]/.test(textFull), `--lanes-full text render carries a Lanes line once a lane exists, got:\n${textFull}`);
    assert(/sessions=sess-lx/.test(textFull), `--lanes-full text Lanes line names the bound session, got:\n${textFull}`);

    // default: summarized — the session-bound lane is THE active lane, in
    // full; with only one lane total, counts/ids for "the rest" are empty.
    const withLane = await runBeeMjs(dir, ['status', '--json'], { env: CLEAN_ENV });
    const payload = JSON.parse(withLane.stdout);
    assert(!Array.isArray(payload.lanes), `default lanes must summarize, not the full array, got ${JSON.stringify(payload.lanes)}`);
    assert(payload.lanes.active && payload.lanes.active.feature === 'lane-x' && payload.lanes.active.phase === 'swarming', `default lanes.active should be the session-bound lane in full, got ${JSON.stringify(payload.lanes.active)}`);
    assert(payload.lanes.active.approved_gates && payload.lanes.active.approved_gates.execution === true, 'default active lane keeps its own full approved_gates');
    assert(Array.isArray(payload.lanes.active.bound_sessions) && payload.lanes.active.bound_sessions.includes('sess-lx'), `default active lane still names the bound session, got ${JSON.stringify(payload.lanes.active)}`);
    assert(Object.keys(payload.lanes.counts).length === 0 && payload.lanes.ids.length === 0, `with only one (active) lane, counts/ids for "the rest" stay empty, got ${JSON.stringify(payload.lanes)}`);
    assert(payload.phase === 'idle', 'the pre-existing top-level phase field is untouched by the default lanes summary (it stays the default pipeline)');

    const text = (await runBeeMjs(dir, ['status'], { env: CLEAN_ENV })).stdout;
    assert(/Lanes: active: lane-x \[swarming\]/.test(text), `default text render carries an "active:" Lanes line once a lane exists, got:\n${text}`);
    assert(/sessions=sess-lx/.test(text), `default text Lanes line names the bound session, got:\n${text}`);
  } finally {
    fs.rmSync(dir, { recursive: true, force: true });
  }
});

// ─── fsh-9 (fresh-session-handoff S4, D1): two-kind handoff lifecycle — the
// guarded writer/adopter over the free-form HANDOFF.json file. readHandoff
// stays fail-open for DISPLAY but normalizes kind (missing/unknown -> 'pause',
// fail-safe for every legacy record); writeHandoff is the strict CLI-owned
// writer (mirrors readStateStrict/readLaneStrict's throw-on-refusal
// convention in this same module); adoptHandoff wraps claims.mjs's
// adoptClaim and returns typed refusals, never throws (mirrors claims.mjs's
// own contract for the primitive it wraps). Namespace import (laneStore, an
// existing fsh-3 alias) keeps this RED-first: a not-yet-implemented export
// fails its own row instead of crashing the whole module graph at import
// time. ───────────────────────────────────────────────────────────────────

function writeCappedCellFixture(root, id, { verifyPassed = true } = {}) {
  writeJsonAtomic(path.join(root, '.bee', 'cells', `${id}.json`), {
    id,
    feature: 'fresh-session-handoff',
    title: 'fixture',
    lane: 'small',
    status: 'capped',
    trace: { verify_passed: verifyPassed },
  });
}

await check('readHandoff: no file -> null; missing/unknown kind normalizes to "pause" (fail-safe); an explicit planned-next kind is preserved', async () => {
  const dir = makeStateRepo('bee-handoff-read-');
  try {
    assert(laneStore.readHandoff(dir) === null, 'no HANDOFF.json reads as null');

    writeJsonAtomic(path.join(dir, '.bee', 'HANDOFF.json'), { cell: 'x', done: [], remaining: [] });
    assert(laneStore.readHandoff(dir).kind === 'pause', 'a legacy handoff with no kind field normalizes to pause');

    writeJsonAtomic(path.join(dir, '.bee', 'HANDOFF.json'), { kind: 'something-else', cell: 'x' });
    assert(laneStore.readHandoff(dir).kind === 'pause', 'an unknown kind value normalizes to pause (fail-safe)');

    writeJsonAtomic(path.join(dir, '.bee', 'HANDOFF.json'), {
      kind: 'planned-next',
      next_cell: 'n',
      writer_session: 'w',
    });
    assert(laneStore.readHandoff(dir).kind === 'planned-next', 'an explicit planned-next kind is preserved');
  } finally {
    fs.rmSync(dir, { recursive: true, force: true });
  }
});

await check("writeHandoff: --kind pause keeps today's free-form fields, adds kind + written_at, no preconditions", async () => {
  const dir = makeStateRepo('bee-handoff-write-pause-');
  try {
    const record = laneStore.writeHandoff(dir, {
      kind: 'pause',
      cell: 'wip-1',
      files: ['a.js', 'b.js'],
      done: ['step1'],
      remaining: ['step2'],
      next_action: 'resume wip-1',
    });
    assert(record.kind === 'pause' && record.cell === 'wip-1', `expected a pause record, got ${JSON.stringify(record)}`);
    assert(typeof record.written_at === 'string', 'written_at stamped');
    const onDisk = readJson(path.join(dir, '.bee', 'HANDOFF.json'), null);
    assert(onDisk && onDisk.kind === 'pause', 'HANDOFF.json on disk carries the pause kind');
  } finally {
    fs.rmSync(dir, { recursive: true, force: true });
  }
});

await check('writeHandoff: refuses a missing/invalid --kind, zero mutation', async () => {
  const dir = makeStateRepo('bee-handoff-write-badkind-');
  try {
    assertThrows(() => laneStore.writeHandoff(dir, {}), 'kind', 'missing kind refuses');
    assertThrows(() => laneStore.writeHandoff(dir, { kind: 'nope' }), 'kind', 'invalid kind refuses');
    assert(!fs.existsSync(path.join(dir, '.bee', 'HANDOFF.json')), 'no partial file on a bad-kind refusal');
  } finally {
    fs.rmSync(dir, { recursive: true, force: true });
  }
});

await check("writeHandoff: planned-next succeeds only when the previous cell is capped with verify_passed true AND the next cell's claim is owned by writer_session; stores writer_session/previous_cell/next_cell (must-have truth)", async () => {
  const dir = makeStateRepo('bee-handoff-write-planned-');
  try {
    writeCappedCellFixture(dir, 'prev-1');
    claimCellFile(dir, 'sess-writer', 'next-1');
    const record = laneStore.writeHandoff(dir, {
      kind: 'planned-next',
      writer_session: 'sess-writer',
      previous_cell: 'prev-1',
      next_cell: 'next-1',
      next_action: 'start next-1',
    });
    assert(record.kind === 'planned-next', `expected planned-next, got ${JSON.stringify(record)}`);
    assert(
      record.writer_session === 'sess-writer' && record.previous_cell === 'prev-1' && record.next_cell === 'next-1',
      `expected the carried identifiers, got ${JSON.stringify(record)}`,
    );
  } finally {
    fs.rmSync(dir, { recursive: true, force: true });
  }
});

await check('writeHandoff: planned-next refuses (typed, zero mutation) when the previous cell is not capped, or capped without verify_passed true (must-have truth)', async () => {
  const dir = makeStateRepo('bee-handoff-write-planned-refuse-cap-');
  try {
    claimCellFile(dir, 'sess-writer', 'next-1');

    assertThrows(
      () =>
        laneStore.writeHandoff(dir, {
          kind: 'planned-next',
          writer_session: 'sess-writer',
          previous_cell: 'ghost',
          next_cell: 'next-1',
        }),
      'capped',
      'a missing previous cell refuses',
    );

    writeJsonAtomic(path.join(dir, '.bee', 'cells', 'prev-open.json'), {
      id: 'prev-open',
      status: 'open',
      trace: { verify_passed: null },
    });
    assertThrows(
      () =>
        laneStore.writeHandoff(dir, {
          kind: 'planned-next',
          writer_session: 'sess-writer',
          previous_cell: 'prev-open',
          next_cell: 'next-1',
        }),
      'capped',
      'an open (uncapped) previous cell refuses',
    );

    writeJsonAtomic(path.join(dir, '.bee', 'cells', 'prev-nogreen.json'), {
      id: 'prev-nogreen',
      status: 'capped',
      trace: { verify_passed: false },
    });
    assertThrows(
      () =>
        laneStore.writeHandoff(dir, {
          kind: 'planned-next',
          writer_session: 'sess-writer',
          previous_cell: 'prev-nogreen',
          next_cell: 'next-1',
        }),
      'capped',
      'a capped cell without verify_passed true refuses',
    );

    assert(!fs.existsSync(path.join(dir, '.bee', 'HANDOFF.json')), 'no partial handoff file after any of the refusals above');
  } finally {
    fs.rmSync(dir, { recursive: true, force: true });
  }
});

await check("writeHandoff: planned-next refuses (typed, zero mutation) when the next cell has no claim, or a claim owned by a DIFFERENT session (must-have truth)", async () => {
  const dir = makeStateRepo('bee-handoff-write-planned-refuse-claim-');
  try {
    writeCappedCellFixture(dir, 'prev-2');

    assertThrows(
      () =>
        laneStore.writeHandoff(dir, {
          kind: 'planned-next',
          writer_session: 'sess-writer',
          previous_cell: 'prev-2',
          next_cell: 'ghost-cell',
        }),
      'claim',
      'a next cell with no claim at all refuses',
    );

    claimCellFile(dir, 'sess-someone-else', 'next-2');
    assertThrows(
      () =>
        laneStore.writeHandoff(dir, {
          kind: 'planned-next',
          writer_session: 'sess-writer',
          previous_cell: 'prev-2',
          next_cell: 'next-2',
        }),
      'claim',
      'a next cell claimed by a different session refuses',
    );

    assert(!fs.existsSync(path.join(dir, '.bee', 'HANDOFF.json')), 'no partial handoff file after either claim refusal');
  } finally {
    fs.rmSync(dir, { recursive: true, force: true });
  }
});

await check('adoptHandoff: transfers the carried claim to the adopting session then clears the handoff (success path)', async () => {
  const dir = makeStateRepo('bee-handoff-adopt-ok-');
  try {
    writeCappedCellFixture(dir, 'prev-3');
    claimCellFile(dir, 'sess-old', 'next-3');
    laneStore.writeHandoff(dir, {
      kind: 'planned-next',
      writer_session: 'sess-old',
      previous_cell: 'prev-3',
      next_cell: 'next-3',
    });

    const result = laneStore.adoptHandoff(dir, 'sess-new');
    assert(result.ok === true, `expected adoption to succeed, got ${JSON.stringify(result)}`);
    const claim = readClaim(dir, 'next-3');
    assert(claim.session === 'sess-new', `expected the claim transferred to sess-new, got ${JSON.stringify(claim)}`);
    assert(!fs.existsSync(path.join(dir, '.bee', 'HANDOFF.json')), 'handoff cleared after a successful adopt');
  } finally {
    fs.rmSync(dir, { recursive: true, force: true });
  }
});

await check('adoptHandoff: refuses (typed, never throws) with no handoff present', async () => {
  const dir = makeStateRepo('bee-handoff-adopt-none-');
  try {
    const result = laneStore.adoptHandoff(dir, 'sess-new');
    assert(result.ok === false && typeof result.code === 'string', `expected a typed refusal, got ${JSON.stringify(result)}`);
  } finally {
    fs.rmSync(dir, { recursive: true, force: true });
  }
});

await check('adoptHandoff: a pause handoff is NEVER adopted — typed refusal, handoff left intact (D1 auto-resume boundary)', async () => {
  const dir = makeStateRepo('bee-handoff-adopt-pause-');
  try {
    laneStore.writeHandoff(dir, { kind: 'pause', cell: 'wip-2' });
    const before = fs.readFileSync(path.join(dir, '.bee', 'HANDOFF.json'), 'utf8');
    const result = laneStore.adoptHandoff(dir, 'sess-new');
    assert(result.ok === false, `expected a typed refusal for a pause handoff, got ${JSON.stringify(result)}`);
    const after = fs.readFileSync(path.join(dir, '.bee', 'HANDOFF.json'), 'utf8');
    assert(before === after, 'the pause handoff stays byte-untouched after a refused adopt');
  } finally {
    fs.rmSync(dir, { recursive: true, force: true });
  }
});

await check('adoptHandoff: a failed claim adopt (e.g. the claim vanished underneath the handoff) is a typed refusal that leaves the handoff intact, never a throw', async () => {
  const dir = makeStateRepo('bee-handoff-adopt-claimgone-');
  try {
    writeCappedCellFixture(dir, 'prev-4');
    claimCellFile(dir, 'sess-old', 'next-4');
    laneStore.writeHandoff(dir, {
      kind: 'planned-next',
      writer_session: 'sess-old',
      previous_cell: 'prev-4',
      next_cell: 'next-4',
    });
    fs.rmSync(path.join(dir, '.bee', 'claims', 'next-4.json'), { force: true });

    const before = fs.readFileSync(path.join(dir, '.bee', 'HANDOFF.json'), 'utf8');
    const result = laneStore.adoptHandoff(dir, 'sess-new');
    assert(result.ok === false, `expected a typed refusal, got ${JSON.stringify(result)}`);
    const after = fs.readFileSync(path.join(dir, '.bee', 'HANDOFF.json'), 'utf8');
    assert(before === after, 'the handoff stays untouched after a failed underlying claim adopt');
  } finally {
    fs.rmSync(dir, { recursive: true, force: true });
  }
});

await check('adoptHandoff: idempotent recovery — a crash between claim-adopt and handoff-clear self-heals on the next call (benign self-adopt then clear), never orphaning the claim', async () => {
  const dir = makeStateRepo('bee-handoff-adopt-crash-recover-');
  try {
    writeCappedCellFixture(dir, 'prev-5');
    claimCellFile(dir, 'sess-old', 'next-5');
    laneStore.writeHandoff(dir, {
      kind: 'planned-next',
      writer_session: 'sess-old',
      previous_cell: 'prev-5',
      next_cell: 'next-5',
    });

    // Simulate a crash landing exactly between the two steps: the claim was
    // already adopted by the new session, but the handoff never got cleared.
    const midCrash = adoptClaim(dir, 'next-5', 'sess-new');
    assert(midCrash.ok === true, 'the simulated first-step adopt succeeds');
    assert(fs.existsSync(path.join(dir, '.bee', 'HANDOFF.json')), 'the handoff is still present, exactly as it would be right after a mid-flight crash');

    const recovered = laneStore.adoptHandoff(dir, 'sess-new');
    assert(recovered.ok === true, `expected the recovery call to succeed, got ${JSON.stringify(recovered)}`);
    assert(!fs.existsSync(path.join(dir, '.bee', 'HANDOFF.json')), 'the handoff is cleared once recovery completes');
    const claim = readClaim(dir, 'next-5');
    assert(claim.session === 'sess-new', 'the claim stays owned by sess-new through the recovery');
  } finally {
    fs.rmSync(dir, { recursive: true, force: true });
  }
});

// ─── multisession-native-15 (D5, issue #56 3.7/mục 10): handoff mailboxes
// per workflow. Legacy writeHandoff/adoptHandoff above stay byte-identical
// (proven by every fsh-9 row above, all still green with zero mailbox
// involvement) — this section proves the NEW per-workflow mailbox path:
// distinct seq per workflow, target_role scoping (no clobber), the
// fence_epoch-bumping adopt with idempotent self-heal, the legacy-fallback
// boundary (C1, zero workflow records), and invariant 9 (one paused
// workflow never blocks or is visible to another's own adopt). ───────────

await check('writeMailboxHandoff/listHandoffMailbox: two handoffs for one workflow keep distinct seq; the second (same default role) auto-clears the first; adopt picks the newest OPEN record', async () => {
  const dir = makeStateRepo('bee-mailbox-two-seq-');
  try {
    const wf = await createWorkflow(dir, { feature: 'msn15-two-seq' });
    const first = await laneStore.writeMailboxHandoff(dir, wf.id, { kind: 'pause', cell: 'wip-a' });
    assert(first.seq === 1, `expected the first record to be seq 1, got ${JSON.stringify(first)}`);

    writeCappedCellFixture(dir, 'msn15-prev-1');
    claimCellFile(dir, 'sess-writer-1', 'msn15-next-1');
    const second = await laneStore.writeMailboxHandoff(dir, wf.id, {
      kind: 'planned-next',
      writer_session: 'sess-writer-1',
      previous_cell: 'msn15-prev-1',
      next_cell: 'msn15-next-1',
    });
    assert(second.seq === 2, `expected the second record to be seq 2 (distinct from the first), got ${JSON.stringify(second)}`);

    const records = laneStore.listHandoffMailbox(dir, wf.id);
    assert(records.length === 2, `expected both records to remain on disk, got ${records.length}`);
    assert(records[0].seq === 1 && records[0].status === 'cleared', `expected seq 1 auto-cleared by the same-role rewrite, got ${JSON.stringify(records[0])}`);
    assert(records[1].seq === 2 && records[1].status === 'open', `expected seq 2 still open, got ${JSON.stringify(records[1])}`);

    const adopted = await laneStore.adoptMailboxHandoff(dir, wf.id, 'sess-adopter-1');
    assert(adopted.ok === true && adopted.seq === 2, `expected adopt to pick the newest OPEN record (seq 2), got ${JSON.stringify(adopted)}`);
    const claim = readClaim(dir, 'msn15-next-1');
    assert(claim.session === 'sess-adopter-1', `expected the claim transferred to the adopter, got ${JSON.stringify(claim)}`);
  } finally {
    fs.rmSync(dir, { recursive: true, force: true });
  }
});

await check("writeMailboxHandoff target_role scoping: a reviewer-role handoff does not clobber an implementer(default)-role handoff — distinct seq records, both stay independently open", async () => {
  const dir = makeStateRepo('bee-mailbox-role-scope-');
  try {
    const wf = await createWorkflow(dir, { feature: 'msn15-role-scope' });
    const implementerHandoff = await laneStore.writeMailboxHandoff(dir, wf.id, { kind: 'pause', cell: 'wip-impl', next_action: 'resume implementer work' });
    const reviewerHandoff = await laneStore.writeMailboxHandoff(dir, wf.id, {
      kind: 'pause',
      cell: 'wip-review',
      next_action: 'resume review',
      target_role: 'reviewer',
    });
    assert(implementerHandoff.seq !== reviewerHandoff.seq, 'the two roles get distinct seq records, never sharing one');

    const stillOpenImplementer = laneStore.newestOpenHandoffMailboxRecord(dir, wf.id, null);
    assert(
      stillOpenImplementer && stillOpenImplementer.seq === implementerHandoff.seq && stillOpenImplementer.status === 'open',
      `expected the implementer(default)-role handoff to remain open and untouched, got ${JSON.stringify(stillOpenImplementer)}`,
    );
    const openReviewer = laneStore.newestOpenHandoffMailboxRecord(dir, wf.id, 'reviewer');
    assert(
      openReviewer && openReviewer.seq === reviewerHandoff.seq && openReviewer.status === 'open',
      `expected the reviewer-role handoff to be independently open, got ${JSON.stringify(openReviewer)}`,
    );

    // Writing a SECOND reviewer-role handoff auto-clears only the FIRST
    // reviewer-role one — the implementer(default)-role handoff is still
    // completely untouched by either reviewer write.
    const reviewerHandoff2 = await laneStore.writeMailboxHandoff(dir, wf.id, { kind: 'pause', cell: 'wip-review-2', target_role: 'reviewer' });
    const records = laneStore.listHandoffMailbox(dir, wf.id);
    const clearedReviewer1 = records.find((r) => r.seq === reviewerHandoff.seq);
    assert(clearedReviewer1.status === 'cleared', 'the FIRST reviewer-role record is auto-cleared by the second reviewer-role write');
    const stillOpenImplementer2 = records.find((r) => r.seq === implementerHandoff.seq);
    assert(stillOpenImplementer2.status === 'open', 'the implementer(default)-role record is untouched by a reviewer-role write (no clobber)');
    assert(reviewerHandoff2.status === 'open', 'the newest reviewer-role record is open');
  } finally {
    fs.rmSync(dir, { recursive: true, force: true });
  }
});

await check('adoptMailboxHandoff: transfers the claim with a bumped fence_epoch (msn-12), clears the record, and a re-run is a typed NO_HANDOFF no-op (never re-adopts, never re-bumps)', async () => {
  const dir = makeStateRepo('bee-mailbox-adopt-epoch-');
  try {
    const wf = await createWorkflow(dir, { feature: 'msn15-adopt-epoch' });
    writeCappedCellFixture(dir, 'msn15-prev-2');
    claimCellFile(dir, 'sess-old-2', 'msn15-next-2');
    const before = readClaim(dir, 'msn15-next-2');
    assert(before.fence_epoch === 1, `expected a freshly claimed cell to start at fence_epoch 1, got ${JSON.stringify(before)}`);

    const written = await laneStore.writeMailboxHandoff(dir, wf.id, {
      kind: 'planned-next',
      writer_session: 'sess-old-2',
      previous_cell: 'msn15-prev-2',
      next_cell: 'msn15-next-2',
    });
    assert(written.claim_epoch === 1, `expected claim_epoch snapshotted at write time (still 1, pre-adopt), got ${JSON.stringify(written)}`);

    const adopted = await laneStore.adoptMailboxHandoff(dir, wf.id, 'sess-new-2');
    assert(adopted.ok === true, `expected adoption to succeed, got ${JSON.stringify(adopted)}`);
    assert(adopted.claim.fence_epoch === 2, `expected fence_epoch bumped by exactly 1 on adoption, got ${JSON.stringify(adopted.claim)}`);
    const afterClaim = readClaim(dir, 'msn15-next-2');
    assert(afterClaim.session === 'sess-new-2' && afterClaim.fence_epoch === 2, `expected the on-disk claim to carry the transferred owner and bumped epoch, got ${JSON.stringify(afterClaim)}`);

    const records = laneStore.listHandoffMailbox(dir, wf.id);
    assert(records[0].status === 'cleared', `expected the record cleared after adoption, got ${JSON.stringify(records[0])}`);

    // Idempotent re-run: no OPEN or ADOPTED record remains for this
    // (workflow, role), so this is a typed no-op refusal — never a throw,
    // never a second claim mutation.
    const again = await laneStore.adoptMailboxHandoff(dir, wf.id, 'sess-new-2');
    assert(again.ok === false && again.code === 'NO_HANDOFF', `expected a typed NO_HANDOFF no-op on re-run, got ${JSON.stringify(again)}`);
    const claimAfterRerun = readClaim(dir, 'msn15-next-2');
    assert(
      claimAfterRerun.session === 'sess-new-2' && claimAfterRerun.fence_epoch === 2,
      `expected the claim byte-unchanged by the no-op re-run, got ${JSON.stringify(claimAfterRerun)}`,
    );
  } finally {
    fs.rmSync(dir, { recursive: true, force: true });
  }
});

await check('adoptMailboxHandoff: idempotent recovery — a crash between claim-adopt and record-clear self-heals on the next call without re-bumping fence_epoch', async () => {
  const dir = makeStateRepo('bee-mailbox-adopt-crash-recover-');
  try {
    const wf = await createWorkflow(dir, { feature: 'msn15-adopt-crash' });
    writeCappedCellFixture(dir, 'msn15-prev-3');
    claimCellFile(dir, 'sess-old-3', 'msn15-next-3');
    await laneStore.writeMailboxHandoff(dir, wf.id, {
      kind: 'planned-next',
      writer_session: 'sess-old-3',
      previous_cell: 'msn15-prev-3',
      next_cell: 'msn15-next-3',
    });

    // Simulate a crash landing exactly between the two steps: the claim was
    // already transferred (and its epoch bumped) by the FIRST step, but the
    // record was never marked 'cleared' — hand-construct that intermediate
    // state the way adoptMailboxHandoff's own 'adopted' status represents it.
    const midCrash = adoptClaim(dir, 'msn15-next-3', 'sess-new-3');
    assert(midCrash.ok === true && midCrash.claim.fence_epoch === 2, 'the simulated first-step transfer succeeds and bumps the epoch once');
    const mailboxDir = laneStore.handoffMailboxDir(dir, wf.id);
    const recordFile = path.join(mailboxDir, '0001.json');
    const onDisk = readJson(recordFile, null);
    writeJsonAtomic(recordFile, { ...onDisk, status: 'adopted', claim_epoch: 2, adopted_by: 'sess-new-3' });

    const recovered = await laneStore.adoptMailboxHandoff(dir, wf.id, 'sess-new-3');
    assert(recovered.ok === true, `expected the recovery call to succeed, got ${JSON.stringify(recovered)}`);
    const claim = readClaim(dir, 'msn15-next-3');
    assert(
      claim.session === 'sess-new-3' && claim.fence_epoch === 2,
      `expected the claim to stay at epoch 2 (never re-bumped by the self-heal), got ${JSON.stringify(claim)}`,
    );
    const record = readJson(recordFile, null);
    assert(record.status === 'cleared', `expected the record cleared once recovery completes, got ${JSON.stringify(record)}`);
  } finally {
    fs.rmSync(dir, { recursive: true, force: true });
  }
});

await check('state handoff write/adopt (CLI, C1): a repo with ZERO workflow records anywhere keeps writing the single legacy .bee/HANDOFF.json — no .bee/runtime/handoffs/ directory is ever created', async () => {
  const dir = makeStateRepo('bee-mailbox-c1-legacy-');
  try {
    const result = await runBeeState(dir, ['handoff', 'write', '--kind', 'pause', '--cell', 'wip-legacy', '--json']);
    assert(result.status === 0, `expected the legacy write to succeed, got status=${result.status} stderr=${result.stderr}`);
    assert(fs.existsSync(path.join(dir, '.bee', 'HANDOFF.json')), 'the legacy file exists');
    assert(!fs.existsSync(path.join(dir, '.bee', 'runtime', 'handoffs')), 'no mailbox directory is created when zero workflow records exist anywhere (C1)');
    const parsed = JSON.parse(result.stdout);
    assert(parsed.kind === 'pause' && !('workflow_id' in parsed), `expected a plain legacy record with no mailbox envelope fields, got ${result.stdout}`);
  } finally {
    fs.rmSync(dir, { recursive: true, force: true });
  }
});

await check('rebuildHandoffProjection: the legacy .bee/HANDOFF.json mirrors the newest OPEN mailbox record across every workflow, in the legacy field shape (writer_session, not from_session/id/workflow_id/status/target_role/seq); removed once every mailbox handoff is cleared', async () => {
  const dir = makeStateRepo('bee-mailbox-projection-');
  try {
    const wf = await createWorkflow(dir, { feature: 'msn15-projection' });
    writeCappedCellFixture(dir, 'msn15-prev-4');
    claimCellFile(dir, 'sess-writer-4', 'msn15-next-4');
    await laneStore.writeMailboxHandoff(dir, wf.id, {
      kind: 'planned-next',
      writer_session: 'sess-writer-4',
      previous_cell: 'msn15-prev-4',
      next_cell: 'msn15-next-4',
    });
    const rebuilt = rebuildHandoffProjection(dir);
    assert(rebuilt.authoritative === true && rebuilt.source === wf.id, `expected an authoritative rebuild sourced from the mailbox's own workflow, got ${JSON.stringify(rebuilt)}`);

    const projected = readJson(path.join(dir, '.bee', 'HANDOFF.json'), null);
    assert(projected && projected.kind === 'planned-next' && projected.writer_session === 'sess-writer-4', `expected the projected file to carry writer_session, got ${JSON.stringify(projected)}`);
    for (const mailboxOnlyField of ['id', 'workflow_id', 'status', 'target_role', 'seq', 'from_session']) {
      assert(!(mailboxOnlyField in projected), `expected the projected legacy file to drop the mailbox-only field "${mailboxOnlyField}", got ${JSON.stringify(projected)}`);
    }
    // readHandoff (the byte-identical readers' own entry point) sees exactly
    // this projected file — proving hooks/bee-session-close.mjs,
    // lib/compaction.mjs, and lib/inject.mjs need zero code changes.
    assert(laneStore.readHandoff(dir).kind === 'planned-next', 'readHandoff sees the projected record unchanged');

    await laneStore.adoptMailboxHandoff(dir, wf.id, 'sess-adopter-4');
    const rebuiltAfterClear = rebuildHandoffProjection(dir);
    assert(rebuiltAfterClear.authoritative === true && rebuiltAfterClear.source === null, `expected a no-open-handoff rebuild once cleared, got ${JSON.stringify(rebuiltAfterClear)}`);
    assert(!fs.existsSync(path.join(dir, '.bee', 'HANDOFF.json')), 'the projected legacy file is removed once no workflow has an open mailbox handoff');
  } finally {
    fs.rmSync(dir, { recursive: true, force: true });
  }
});

await check('invariant 9 (D9, must-have truth): workflow A paused with an open handoff never blocks workflow B from starting, working, or adopting its OWN handoff — and A\'s own handoff is completely untouched by any of B\'s mailbox activity', async () => {
  const dir = makeStateRepo('bee-mailbox-invariant9-');
  try {
    // Workflow A: the DEFAULT pipeline, paused (an open, never-adopted
    // 'pause' handoff — real mid-flight-interruption shape).
    const startA = await runBeeState(dir, ['start-feature', '--feature', 'msn15-inv9-a', '--json']);
    assert(startA.status === 0, `expected feature A to start, got stderr=${startA.stderr}`);
    const pauseA = await runBeeState(dir, ['handoff', 'write', '--kind', 'pause', '--cell', 'wip-a', '--next-action', 'resume A', '--json']);
    assert(pauseA.status === 0, `expected A's pause handoff to write, got stderr=${pauseA.stderr}`);
    const workflowsAfterA = listWorkflows(dir).workflows;
    const wfA = workflowsAfterA.find((w) => w.feature === 'msn15-inv9-a');
    assert(wfA, 'workflow A exists');
    assert(laneStore.newestOpenHandoffMailboxRecord(dir, wfA.id, null)?.status === 'open', "A's own handoff resolved through the mailbox, open");

    // Workflow B: a LANE, started WHILE A is paused — never blocked (F5
    // scoping: a handoff naming a DIFFERENT feature never blocks a start).
    const startB = await runBeeState(dir, ['start-feature', '--feature', 'msn15-inv9-b', '--as-lane', '--json']);
    assert(startB.status === 0, `expected feature B to start despite A's open handoff, got stderr=${startB.stderr}`);
    const wfB = listWorkflows(dir).workflows.find((w) => w.feature === 'msn15-inv9-b');
    assert(wfB, 'workflow B exists');

    // B works and writes its OWN planned-next handoff, targeted at its own
    // lane via --lane (never touching A's mailbox).
    writeCappedCellFixture(dir, 'msn15-inv9-prev-b');
    claimCellFile(dir, 'sess-writer-b', 'msn15-inv9-next-b');
    const writeB = await runBeeState(dir, [
      'handoff', 'write',
      '--kind', 'planned-next',
      '--lane', 'msn15-inv9-b',
      '--writer-session', 'sess-writer-b',
      '--previous-cell', 'msn15-inv9-prev-b',
      '--next-cell', 'msn15-inv9-next-b',
      '--json',
    ]);
    assert(writeB.status === 0, `expected B's planned-next handoff to write, got stderr=${writeB.stderr}`);

    // B adopts its own handoff freely — succeeds regardless of A's open pause.
    const adoptB = await runBeeState(dir, ['handoff', 'adopt', '--lane', 'msn15-inv9-b', '--session-id', 'sess-adopter-b', '--json']);
    assert(adoptB.status === 0, `expected B's adopt to succeed while A is still paused, got stdout=${adoptB.stdout} stderr=${adoptB.stderr}`);
    const adoptedB = JSON.parse(adoptB.stdout);
    assert(adoptedB.ok === true, `expected B's adoption to report ok:true, got ${adoptB.stdout}`);
    const claimB = readClaim(dir, 'msn15-inv9-next-b');
    assert(claimB.session === 'sess-adopter-b', `expected B's claim transferred to its own adopter, got ${JSON.stringify(claimB)}`);

    // A's own handoff is COMPLETELY untouched: still the same open pause
    // record it started as, at its original seq — B's start/write/adopt
    // never wrote a single byte into A's mailbox.
    const finalA = laneStore.newestOpenHandoffMailboxRecord(dir, wfA.id, null);
    assert(finalA && finalA.kind === 'pause' && finalA.status === 'open' && finalA.seq === 1, `expected A's pause handoff untouched by any of B's activity, got ${JSON.stringify(finalA)}`);

    // B's mailbox, symmetrically, carries none of A's records.
    const bRecords = laneStore.listHandoffMailbox(dir, wfB.id);
    assert(bRecords.every((r) => r.workflow_id === wfB.id), 'every record in B\'s mailbox belongs to B, never A');

    // `state handoff show` with no --lane resolves to the DEFAULT record's
    // own workflow (A) and still shows A's open pause — while `--lane
    // msn15-inv9-b` shows B's own (now null, cleared) state. Each session's
    // view is scoped to the workflow it actually resolves to.
    const showDefault = await runBeeState(dir, ['handoff', 'show', '--json']);
    assert(showDefault.status === 0, `expected the default show to succeed, got stderr=${showDefault.stderr}`);
    const shownDefault = JSON.parse(showDefault.stdout);
    assert(shownDefault && shownDefault.kind === 'pause', `expected the default (workflow A) show to still see A's pause, got ${showDefault.stdout}`);
    const showB = await runBeeState(dir, ['handoff', 'show', '--lane', 'msn15-inv9-b', '--json']);
    assert(JSON.parse(showB.stdout) === null, `expected B's own mailbox to show null once cleared, got ${showB.stdout}`);
  } finally {
    fs.rmSync(dir, { recursive: true, force: true });
  }
});

// ─── fsh-10 (fresh-session-handoff S4, D1): SessionStart wiring — the render
// side. PURITY PIN (validation-s4 panel W2): buildSessionPreamble stays a
// PURE builder — it never adopts anything itself. The hook
// (hooks/bee-session-init.mjs) performs the source-gated adoption and passes
// the typed outcome in as `handoffOutcome`; these direct-lib rows exercise
// only the rendering contract: null (no attempt), ok:true (start-now),
// ok:false (wait block + one reason line). The through-the-real-hook rows
// (source gating, claim transfer, byte-parity) live in
// hooks/test_hook_contracts.mjs. ────────────────────────────────────────────

function writeNextCellFixture(root, id, { lane = 'standard', verify = 'node test.mjs', title = 'next task' } = {}) {
  writeJsonAtomic(path.join(root, '.bee', 'cells', `${id}.json`), {
    id,
    feature: 'fresh-session-handoff',
    title,
    lane,
    status: 'open',
    verify,
  });
}

await check('buildSessionPreamble: handoffOutcome omitted (null) renders a pause handoff identically whether or not a sessionId is bound — no start-now, no reason line ever fabricated', async () => {
  const dir = makeStateRepo('bee-preamble-handoff-pause-');
  try {
    laneStore.writeHandoff(dir, { kind: 'pause', cell: 'wip-h1', next_action: 'resume wip-h1' });
    const bare = buildSessionPreamble(dir);
    laneBinding.createSession(dir, { id: 'sess-bound-pause' });
    const bound = buildSessionPreamble(dir, { sessionId: 'sess-bound-pause' });
    assert(bare === bound, 'a bound sessionId with no handoffOutcome renders the identical pause block');
    assert(/HANDOFF present — present it and WAIT/.test(bare), 'the classic wait heading is present');
    assert(!/Adoption not applied/.test(bare), 'a pause handoff never carries an adoption reason line');
    assert(!/PLANNED-NEXT ADOPTED/.test(bare), 'a pause handoff never renders the start-now heading');
  } finally {
    fs.rmSync(dir, { recursive: true, force: true });
  }
});

await check('buildSessionPreamble: a planned-next handoff with no handoffOutcome (e.g. no session_id at all) renders the plain wait block — no start-now, no reason line — this is the fsh-10 no-session_id byte-parity contract', async () => {
  const dir = makeStateRepo('bee-preamble-handoff-planned-no-outcome-');
  try {
    writeCappedCellFixture(dir, 'prev-h2');
    claimCellFile(dir, 'sess-writer-h2', 'next-h2');
    writeNextCellFixture(dir, 'next-h2', { lane: 'high-risk', verify: 'node verify-h2.mjs' });
    laneStore.writeHandoff(dir, {
      kind: 'planned-next',
      writer_session: 'sess-writer-h2',
      previous_cell: 'prev-h2',
      next_cell: 'next-h2',
      next_action: 'start next-h2',
    });
    const noSession = buildSessionPreamble(dir);
    assert(
      /HANDOFF present — present it and WAIT/.test(noSession),
      'a planned-next handoff with no outcome renders the classic wait heading',
    );
    assert(!/PLANNED-NEXT ADOPTED/.test(noSession), 'no start-now block is fabricated without an outcome');
    assert(
      !/Adoption not applied/.test(noSession),
      'no reason line is fabricated without an outcome — this is the exact pre-fsh-10 rendering',
    );
  } finally {
    fs.rmSync(dir, { recursive: true, force: true });
  }
});

await check("buildSessionPreamble: handoffOutcome.ok===true replaces the wait block with a start-now block naming the adopted cell, its lane, and its verify command (must-have truth)", async () => {
  const dir = makeStateRepo('bee-preamble-handoff-adopted-');
  try {
    writeCappedCellFixture(dir, 'prev-h3');
    claimCellFile(dir, 'sess-writer-h3', 'next-h3');
    writeNextCellFixture(dir, 'next-h3', {
      lane: 'high-risk',
      verify: 'node verify-h3.mjs && echo ok',
      title: 'wire the thing',
    });
    laneStore.writeHandoff(dir, {
      kind: 'planned-next',
      writer_session: 'sess-writer-h3',
      previous_cell: 'prev-h3',
      next_cell: 'next-h3',
      next_action: 'start next-h3',
    });
    const outcome = {
      ok: true,
      next_cell: 'next-h3',
      claim: { session: 'sess-new-h3' },
      previous_owner: 'sess-writer-h3',
    };
    const rendered = buildSessionPreamble(dir, { sessionId: 'sess-new-h3', handoffOutcome: outcome });
    assert(
      /PLANNED-NEXT ADOPTED — starting now, no confirmation needed \(D1\)/.test(rendered),
      `expected the start-now heading, got:\n${rendered}`,
    );
    assert(/- Cell: next-h3 — wire the thing/.test(rendered), `expected the adopted cell named with its title, got:\n${rendered}`);
    assert(/- Lane: high-risk/.test(rendered), `expected the adopted cell's lane, got:\n${rendered}`);
    assert(
      /- Verify: `node verify-h3\.mjs && echo ok`/.test(rendered),
      `expected the adopted cell's verify command, got:\n${rendered}`,
    );
    assert(!/HANDOFF present — present it and WAIT/.test(rendered), 'the wait heading is fully replaced, never both shown');
  } finally {
    fs.rmSync(dir, { recursive: true, force: true });
  }
});

await check("buildSessionPreamble: handoffOutcome.ok===false renders the wait block plus one reason line — never a fabricated start-now (must-have truth)", async () => {
  const dir = makeStateRepo('bee-preamble-handoff-refused-');
  try {
    writeCappedCellFixture(dir, 'prev-h4');
    claimCellFile(dir, 'sess-writer-h4', 'next-h4');
    writeNextCellFixture(dir, 'next-h4');
    laneStore.writeHandoff(dir, {
      kind: 'planned-next',
      writer_session: 'sess-writer-h4',
      previous_cell: 'prev-h4',
      next_cell: 'next-h4',
    });
    const outcome = {
      ok: false,
      code: 'WRONG_SOURCE',
      reason: 'a planned-next handoff never auto-adopts on source "resume"',
    };
    const rendered = buildSessionPreamble(dir, { sessionId: 'sess-resuming', handoffOutcome: outcome });
    assert(/HANDOFF present — present it and WAIT/.test(rendered), `expected the classic wait heading, got:\n${rendered}`);
    assert(!/PLANNED-NEXT ADOPTED/.test(rendered), 'a refused outcome never renders the start-now heading');
    assert(
      /- Adoption not applied: a planned-next handoff never auto-adopts on source "resume"/.test(rendered),
      `expected the refusal reason line, got:\n${rendered}`,
    );
  } finally {
    fs.rmSync(dir, { recursive: true, force: true });
  }
});

// ─── fsh-10: two-session end-to-end fixture (D1, D2 epic-map E4 proof) ──────
// Direct-lib proof, not through the hook: session A caps its cell and claims
// the next one, writes a planned-next handoff carrying that claim; session B
// "crosses the /clear boundary" by calling adoptHandoff; a THIRD session's
// CONCURRENT claimCellFile steal attempt on the same cell must lose with the
// typed CLAIMED failure — riding fsh-2's Worker/barrier-file race pattern
// (race_claims_child.mjs). A small self-contained orchestrator is generated
// into a throwaway temp path and re-execs itself as concurrent Worker racers;
// its outer module entrypoint runs through the shared serialized runner.

function fsh10HandoffRaceScript() {
  const libDir = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '../lib');
  const stateUrl = pathToFileURL(path.join(libDir, 'state.mjs')).href;
  const claimsUrl = pathToFileURL(path.join(libDir, 'claims.mjs')).href;
  const fsutilUrl = pathToFileURL(path.join(libDir, 'fsutil.mjs')).href;
  const scriptDir = fs.mkdtempSync(path.join(os.tmpdir(), 'bee-fsh10-handoff-race-'));
  const scriptPath = path.join(scriptDir, 'orchestrator.mjs');
  const lines = [
    "import fs from 'node:fs';",
    "import os from 'node:os';",
    "import path from 'node:path';",
    "import { fileURLToPath } from 'node:url';",
    "import { Worker, workerData } from 'node:worker_threads';",
    `import { writeHandoff, adoptHandoff } from ${JSON.stringify(stateUrl)};`,
    `import { createSession, claimCellFile, readClaim } from ${JSON.stringify(claimsUrl)};`,
    `import { writeJsonAtomic } from ${JSON.stringify(fsutilUrl)};`,
    '',
    'const self = fileURLToPath(import.meta.url);',
    '',
    'if (workerData?.fsh10Role) {',
    '  runRole(workerData.fsh10Role);',
    '} else {',
    '  main();',
    '}',
    '',
    'function spinUntil(goFile) {',
    '  while (!fs.existsSync(goFile)) { /* spin */ }',
    '}',
    '',
    'function runRole(role) {',
    '  spinUntil(role.goFile);',
    "  if (role.kind === 'adopt-handoff') {",
    '    const result = adoptHandoff(role.root, role.sessionId);',
    '    process.exit(result.ok === true ? 0 : 2);',
    '  } else {',
    '    const result = claimCellFile(role.root, role.sessionId, role.cellId, role.ttl || 60);',
    "    if (result.ok === false && result.code === 'CLAIMED') process.exit(1);", // expected: steal denied
    '    process.exit(result.ok === true ? 3 : 2);', // 3 = BUG (steal succeeded)
    '  }',
    '}',
    '',
    'function startRole(role) {',
    '  return new Worker(self, { workerData: { fsh10Role: role } });',
    '}',
    '',
    'function waitExit(child) {',
    '  return new Promise((resolve) => child.on("exit", (code) => resolve(code)));',
    '}',
    '',
    'function sleep(ms) {',
    '  return new Promise((resolve) => setTimeout(resolve, ms));',
    '}',
    '',
    'async function main() {',
    '  const root = fs.mkdtempSync(path.join(os.tmpdir(), "bee-fsh10-handoff-race-root-"));',
    '  fs.mkdirSync(path.join(root, ".bee", "cells"), { recursive: true });',
    '  try {',
    '    createSession(root, { id: "sess-A" });',
    '    createSession(root, { id: "sess-B" });',
    '    createSession(root, { id: "sess-thief-1" });',
    '    createSession(root, { id: "sess-thief-2" });',
    '',
    '    writeJsonAtomic(path.join(root, ".bee", "cells", "prev-race.json"), {',
    '      id: "prev-race",',
    '      status: "capped",',
    '      trace: { verify_passed: true },',
    '    });',
    '    const claimed = claimCellFile(root, "sess-A", "next-race", 3600);',
    '    if (!claimed.ok) {',
    '      console.log("FAIL  two-session-handoff-race: setup claim failed");',
    '      process.exitCode = 1;',
    '      return;',
    '    }',
    '    writeHandoff(root, {',
    '      kind: "planned-next",',
    '      writer_session: "sess-A",',
    '      previous_cell: "prev-race",',
    '      next_cell: "next-race",',
    '      next_action: "start next-race",',
    '    });',
    '',
    '    const goFile = path.join(root, "go");',
    '    const children = [',
    '      startRole({ kind: "adopt-handoff", root, sessionId: "sess-B", goFile }),',
    '      startRole({ kind: "steal", root, sessionId: "sess-thief-1", cellId: "next-race", goFile, ttl: 60 }),',
    '      startRole({ kind: "steal", root, sessionId: "sess-thief-2", cellId: "next-race", goFile, ttl: 60 }),',
    '    ];',
    '    const exits = Promise.all(children.map(waitExit));',
    '    await sleep(150);',
    '    fs.writeFileSync(goFile, "1");',
    '    const codes = await exits;',
    '    const adoptCode = codes[0];',
    '    const thiefCodes = codes.slice(1);',
    '    const bugSteals = thiefCodes.filter((c) => c === 3).length;',
    '    const unexpectedThieves = thiefCodes.filter((c) => c !== 1 && c !== 3).length;',
    '    const finalClaim = readClaim(root, "next-race");',
    '    const handoffGone = !fs.existsSync(path.join(root, ".bee", "HANDOFF.json"));',
    '',
    '    const ok =',
    '      adoptCode === 0 &&',
    '      bugSteals === 0 &&',
    '      unexpectedThieves === 0 &&',
    '      finalClaim &&',
    '      finalClaim.session === "sess-B" &&',
    '      handoffGone;',
    '',
    '    if (!ok) {',
    '      console.log(',
    '        "FAIL  two-session-handoff-race: adoptCode=" + adoptCode + " bugSteals=" + bugSteals +',
    '          " unexpected=" + unexpectedThieves + " finalOwner=" + (finalClaim ? finalClaim.session : null) +',
    '          " handoffGone=" + handoffGone,',
    '      );',
    '      process.exitCode = 1;',
    '      return;',
    '    }',
    '    console.log(',
    '      "PASS  two-session-handoff-race: session A capped+claimed+handed off, session B adopted across the /clear boundary, both concurrent thieves lost with typed CLAIMED",',
    '    );',
    '    process.exitCode = 0;',
    '  } finally {',
    '    fs.rmSync(root, { recursive: true, force: true });',
    '  }',
    '}',
    '',
  ];
  fs.writeFileSync(scriptPath, lines.join('\n'));
  return scriptPath;
}

await check(
  "race: two-session handoff — session A caps+claims+hands off, session B adopts across the simulated /clear boundary, and a concurrent third-session steal loses with typed CLAIMED (epic-map E4, riding fsh-2's race harness pattern)",
  async () => {
    const scriptPath = fsh10HandoffRaceScript();
    try {
      const result = await runModuleWorker(scriptPath, { timeout: 60000 });
      assert(result.status === 0, `two-session-handoff race failed (status ${result.status}): ${result.stdout}${result.stderr}`);
      assert(/^PASS +two-session-handoff-race/m.test(result.stdout), `expected a PASS summary line, got: ${result.stdout}`);
    } finally {
      fs.rmSync(path.dirname(scriptPath), { recursive: true, force: true });
    }
  },
);

// ─── scribing-integrity si-1: debt wall on every door ───────────────────────
// The post-mortem's three structural holes: (1) scribingDebt only ever fires
// as a WALL at a feature's own explicit compounding-complete — swapping the
// default record to a different feature over unpaid debt has the identical
// silent-abandonment effect and nothing catches it; (2) lane closes never
// compute debt at all, since scribingDebt reads only the default record; (3)
// scribing-run now writes a durable ledger line on every call, including a
// repair stamp for a feature that is not the currently active one.

function makeSwapDebtRepo(prefix) {
  const dir = fs.mkdtempSync(path.join(os.tmpdir(), prefix));
  fs.mkdirSync(path.join(dir, '.bee', 'cells'), { recursive: true });
  writeJsonAtomic(path.join(dir, '.bee', 'onboarding.json'), { schema_version: '1.0', bee_version: '0.1.0' });
  writeJsonAtomic(path.join(dir, '.bee', 'state.json'), { phase: 'swarming', feature: 'demo' });
  for (const id of ['sw-1', 'sw-2']) {
    writeJsonAtomic(path.join(dir, '.bee', 'cells', `${id}.json`), {
      id,
      feature: 'demo',
      status: 'capped',
      trace: { behavior_change: true, capped_at: new Date().toISOString() },
    });
  }
  return dir;
}

await check('si-1 (D1): state set --feature <different> is REFUSED while the CURRENT feature carries unpaid scribing debt — a swap has the same silent-abandonment effect as a close', async () => {
  const dir = makeSwapDebtRepo('bee-swap-debt-');
  try {
    const refused = await runBeeState(dir, ['set', '--owner', 'swarming', '--feature', 'other-feature', '--json']);
    assert(refused.status !== 0, 'swapping away from a feature with unpaid debt must be refused');
    const out = refused.stdout + refused.stderr;
    assert(/sw-1/.test(out) && /sw-2/.test(out), `refusal must name every unscribed cell, got: ${out}`);
    assert(/waive-scribing-debt/.test(out), `refusal must disclose the waiver door, got: ${out}`);
    assert(
      JSON.parse(fs.readFileSync(path.join(dir, '.bee', 'state.json'), 'utf8')).feature === 'demo',
      'a refused swap must leave feature untouched — no partial write',
    );
  } finally {
    fs.rmSync(dir, { recursive: true, force: true });
  }
});

await check('si-1: --waive-scribing-debt permits a feature swap over unpaid debt and logs the waiver, same as a close', async () => {
  const dir = makeSwapDebtRepo('bee-swap-debt-waive-');
  try {
    const ok = await runBeeState(dir, ['set', '--owner', 'swarming', '--feature', 'other-feature', '--waive-scribing-debt', '--json']);
    assert(ok.status === 0, `the waiver must permit the swap, got: ${ok.stdout}${ok.stderr}`);
    assert(
      JSON.parse(fs.readFileSync(path.join(dir, '.bee', 'state.json'), 'utf8')).feature === 'other-feature',
      'the waived swap must actually write the new feature',
    );
    const log = fs.readFileSync(path.join(dir, '.bee', 'decisions.jsonl'), 'utf8');
    assert(/sw-1/.test(log) && /sw-2/.test(log), `the waiver decision must name every waived cell, got: ${log}`);
  } finally {
    fs.rmSync(dir, { recursive: true, force: true });
  }
});

await check('si-1: state set --feature <same value> is never treated as a swap — no debt check fires even with unpaid debt standing', async () => {
  const dir = makeSwapDebtRepo('bee-swap-debt-noop-');
  try {
    const ok = await runBeeState(dir, ['set', '--owner', 'swarming', '--feature', 'demo', '--summary', 'noop-rewrite', '--json']);
    assert(ok.status === 0, `setting --feature to its own current value is not a swap, got: ${ok.stdout}${ok.stderr}`);
  } finally {
    fs.rmSync(dir, { recursive: true, force: true });
  }
});

function makeLaneDebtRepo(prefix) {
  const dir = fs.mkdtempSync(path.join(os.tmpdir(), prefix));
  fs.mkdirSync(path.join(dir, '.bee', 'cells'), { recursive: true });
  fs.mkdirSync(path.join(dir, '.bee', 'lanes'), { recursive: true });
  writeJsonAtomic(path.join(dir, '.bee', 'onboarding.json'), { schema_version: '1.0', bee_version: '0.1.0' });
  // The default record sits on a totally different, clean feature — proof
  // the lane close computes the LANE's own feature debt, not the default's.
  writeJsonAtomic(path.join(dir, '.bee', 'state.json'), { phase: 'idle', feature: null });
  writeJsonAtomic(path.join(dir, '.bee', 'lanes', 'lane-feat.json'), {
    schema_version: '1.0',
    feature: 'lane-feat',
    phase: 'compounding',
    approved_gates: { context: true, shape: true, execution: true, review: false },
  });
  for (const id of ['ln-1', 'ln-2']) {
    writeJsonAtomic(path.join(dir, '.bee', 'cells', `${id}.json`), {
      id,
      feature: 'lane-feat',
      status: 'capped',
      trace: { behavior_change: true, capped_at: new Date().toISOString() },
    });
  }
  return dir;
}

await check('si-1 (D2): a LANE close (state set --lane X --phase compounding-complete) checks the LANE feature\'s own debt — an idle/clean default record does not let a debt-carrying lane through', async () => {
  const dir = makeLaneDebtRepo('bee-lane-debt-');
  try {
    const refused = await runBeeState(dir, ['set', '--lane', 'lane-feat', '--owner', 'compounding', '--phase', 'compounding-complete', '--json']);
    assert(refused.status !== 0, 'a lane close over unpaid lane debt must be refused, even with an idle/clean default record');
    const out = refused.stdout + refused.stderr;
    assert(/ln-1/.test(out) && /ln-2/.test(out), `refusal must name every unscribed lane cell, got: ${out}`);
    assert(/waive-scribing-debt/.test(out), `refusal must disclose the waiver door, got: ${out}`);
    assert(
      JSON.parse(fs.readFileSync(path.join(dir, '.bee', 'lanes', 'lane-feat.json'), 'utf8')).phase === 'compounding',
      'a refused lane close must leave the lane phase untouched',
    );
  } finally {
    fs.rmSync(dir, { recursive: true, force: true });
  }
});

await check('si-1: --waive-scribing-debt permits a lane close over unpaid lane debt and logs the waiver', async () => {
  const dir = makeLaneDebtRepo('bee-lane-debt-waive-');
  try {
    const ok = await runBeeState(dir, ['set', '--lane', 'lane-feat', '--owner', 'compounding', '--phase', 'compounding-complete', '--waive-scribing-debt', '--json']);
    assert(ok.status === 0, `the waiver must permit the lane close, got: ${ok.stdout}${ok.stderr}`);
    assert(
      JSON.parse(fs.readFileSync(path.join(dir, '.bee', 'lanes', 'lane-feat.json'), 'utf8')).phase === 'compounding-complete',
      'the waived lane close must write the terminal phase',
    );
    const log = fs.readFileSync(path.join(dir, '.bee', 'decisions.jsonl'), 'utf8');
    assert(/ln-1/.test(log) && /ln-2/.test(log), `the waiver decision must name every waived lane cell, got: ${log}`);
  } finally {
    fs.rmSync(dir, { recursive: true, force: true });
  }
});

await check('si-1: a lane close with a scribing run stamped AFTER the lane cells were capped (the lane\'s own last_scribing_run) passes cleanly with no waiver', async () => {
  const dir = makeLaneDebtRepo('bee-lane-debt-synced-');
  try {
    const laneFile = path.join(dir, '.bee', 'lanes', 'lane-feat.json');
    const lane = JSON.parse(fs.readFileSync(laneFile, 'utf8'));
    lane.last_scribing_run = { feature: 'lane-feat', at: new Date(Date.now() + 60000).toISOString() };
    writeJsonAtomic(laneFile, lane);
    const ok = await runBeeState(dir, ['set', '--lane', 'lane-feat', '--owner', 'compounding', '--phase', 'compounding-complete', '--json']);
    assert(ok.status === 0, `a lane synced after its caps must close cleanly with no waiver needed, got: ${ok.stdout}${ok.stderr}`);
  } finally {
    fs.rmSync(dir, { recursive: true, force: true });
  }
});

await check('si-1: state scribing-run appends a parseable line to .bee/logs/scribing-runs.jsonl on every call', async () => {
  const dir = makeStateRepo('bee-scribing-ledger-');
  try {
    writeJsonAtomic(path.join(dir, '.bee', 'state.json'), { phase: 'swarming', feature: 'demo' });
    const result = await runBeeState(dir, [
      'scribing-run', '--feature', 'demo', '--areas', 'demo-area', '--next-action', 'bee-compounding', '--json',
    ]);
    assert(result.status === 0, `scribing-run should succeed, got ${result.status}: ${result.stderr}`);
    const ledgerPath = path.join(dir, '.bee', 'logs', 'scribing-runs.jsonl');
    assert(fs.existsSync(ledgerPath), 'ledger file must exist after a scribing-run call');
    const lines = fs.readFileSync(ledgerPath, 'utf8').trim().split('\n');
    assert(lines.length === 1, `exactly one ledger line for one call, got ${lines.length}`);
    const entry = JSON.parse(lines[0]);
    assert(entry.feature === 'demo', `ledger entry must name the feature, got ${JSON.stringify(entry)}`);
    assert(typeof entry.ts === 'string' && Number.isFinite(Date.parse(entry.ts)), 'ledger entry carries a parseable ts');
    assert(Array.isArray(entry.areas) && entry.areas.includes('demo-area'), 'ledger entry carries the synced areas');
  } finally {
    fs.rmSync(dir, { recursive: true, force: true });
  }
});

await check('si-1: state scribing-run can stamp a NON-active feature — writes the ledger line but leaves the default record feature/phase untouched', async () => {
  const dir = makeStateRepo('bee-scribing-nonactive-');
  try {
    writeJsonAtomic(path.join(dir, '.bee', 'state.json'), { phase: 'swarming', feature: 'demo' });
    const result = await runBeeState(dir, [
      'scribing-run', '--feature', 'orphan-feature', '--areas', 'ghost', '--next-action', 'repair', '--json',
    ]);
    assert(result.status === 0, `scribing-run for a non-active feature should still succeed (repair path), got ${result.status}: ${result.stderr}`);
    const after = JSON.parse(fs.readFileSync(path.join(dir, '.bee', 'state.json'), 'utf8'));
    assert(after.feature === 'demo', 'the default record feature must NOT be corrupted by stamping a different feature');
    assert(after.phase === 'swarming', 'the default record phase must NOT advance when the stamped feature is not the active one');
    const ledgerPath = path.join(dir, '.bee', 'logs', 'scribing-runs.jsonl');
    const lines = fs.readFileSync(ledgerPath, 'utf8').trim().split('\n');
    const entry = JSON.parse(lines[lines.length - 1]);
    assert(entry.feature === 'orphan-feature', `the ledger line must name the stamped feature, got ${JSON.stringify(entry)}`);
  } finally {
    fs.rmSync(dir, { recursive: true, force: true });
  }
});

await check('tst-1: state scribing-run for a NON-active feature succeeds from phase "compounding-complete" — the phase gate never applies to a repair-path call that leaves the default record untouched', async () => {
  const dir = makeStateRepo('bee-scribing-nonactive-terminal-');
  try {
    const before = { phase: 'compounding-complete', feature: 'demo' };
    writeJsonAtomic(path.join(dir, '.bee', 'state.json'), before);
    const beforeBytes = fs.readFileSync(path.join(dir, '.bee', 'state.json'));
    const result = await runBeeState(dir, [
      'scribing-run', '--feature', 'cli-ergonomics', '--areas', 'workflow-state,doctrine-layer', '--next-action', '-', '--json',
    ]);
    assert(
      result.status === 0,
      `a non-active-feature scribing stamp must succeed from any phase (including terminal), got ${result.status}: ${result.stdout}${result.stderr}`,
    );
    const afterBytes = fs.readFileSync(path.join(dir, '.bee', 'state.json'));
    assert(afterBytes.equals(beforeBytes), 'the default record must stay byte-unchanged when the stamped feature is not the active one');
    const ledgerPath = path.join(dir, '.bee', 'logs', 'scribing-runs.jsonl');
    const lines = fs.readFileSync(ledgerPath, 'utf8').trim().split('\n');
    const entry = JSON.parse(lines[lines.length - 1]);
    assert(entry.feature === 'cli-ergonomics', `the ledger line must name the stamped feature, got ${JSON.stringify(entry)}`);
  } finally {
    fs.rmSync(dir, { recursive: true, force: true });
  }
});

await check('tst-1: state scribing-run for the ACTIVE feature still refuses from phase "compounding-complete" — the phase gate is unchanged for the run that actually advances the default record', async () => {
  const dir = makeStateRepo('bee-scribing-active-terminal-');
  try {
    writeJsonAtomic(path.join(dir, '.bee', 'state.json'), { phase: 'compounding-complete', feature: 'demo' });
    const result = await runBeeState(dir, [
      'scribing-run', '--feature', 'demo', '--areas', 'workflow-state', '--next-action', '-', '--json',
    ]);
    assert(result.status !== 0, 'an active-feature scribing-run from the terminal phase must still be refused');
    const out = result.stdout + result.stderr;
    assert(
      /scribing-run: refused from phase \\?"compounding-complete\\?"/.test(out),
      `refusal must name the phase-gate reason, got: ${out}`,
    );
  } finally {
    fs.rmSync(dir, { recursive: true, force: true });
  }
});

await check('si-1 (D5): status --json scribing_debt gains an ADDITIVE orphaned block {count, features} from the global sweep, independent of the active feature', async () => {
  const dir = makeStateRepo('bee-status-orphan-');
  fs.mkdirSync(path.join(dir, '.bee', 'cells'), { recursive: true });
  writeJsonAtomic(path.join(dir, '.bee', 'state.json'), { phase: 'idle', feature: null });
  writeJsonAtomic(path.join(dir, '.bee', 'cells', 'st-orphan-1.json'), {
    id: 'st-orphan-1',
    feature: 'orphan-feat',
    status: 'capped',
    trace: { behavior_change: true, capped_at: new Date().toISOString() },
  });
  try {
    const zero = await runBeeMjs(dir, ['status', '--json'], { env: CLEAN_ENV });
    assert(zero.status === 0, `bee.mjs status --json exited ${zero.status} :: ${zero.stderr}`);
    const payload = JSON.parse(zero.stdout);
    assert(payload.scribing_debt && typeof payload.scribing_debt === 'object', 'scribing_debt object present');
    assert(payload.scribing_debt.count === 0, 'no active feature → the direct debt count is still 0 (unchanged field)');
    assert(payload.scribing_debt.orphaned && typeof payload.scribing_debt.orphaned === 'object', 'orphaned block present');
    assert(payload.scribing_debt.orphaned.count === 1, `orphan sweep must count the orphaned cell, got ${JSON.stringify(payload.scribing_debt.orphaned)}`);
    assert(
      payload.scribing_debt.orphaned.features.some((f) => f.feature === 'orphan-feat' && f.cells.includes('st-orphan-1')),
      `orphaned.features must name the feature and cell, got ${JSON.stringify(payload.scribing_debt.orphaned.features)}`,
    );
  } finally {
    fs.rmSync(dir, { recursive: true, force: true });
  }
});

await check('sqs-b3: state scribing-run --show returns the most-recent stamp overall, across every feature in the ledger', async () => {
  const dir = makeStateRepo('bee-scribing-show-overall-');
  try {
    writeJsonAtomic(path.join(dir, '.bee', 'state.json'), { phase: 'swarming', feature: 'demo' });
    const first = await runBeeState(dir, [
      'scribing-run', '--feature', 'demo', '--areas', 'demo-area', '--next-action', 'bee-compounding', '--json',
    ]);
    assert(first.status === 0, `setup scribing-run should succeed, got ${first.status}: ${first.stderr}`);
    // A second, later stamp for a DIFFERENT feature must win as "overall".
    const second = await runBeeState(dir, [
      'scribing-run', '--feature', 'other-feat', '--areas', 'other-area', '--next-action', '-', '--json',
    ]);
    assert(second.status === 0, `second scribing-run should succeed, got ${second.status}: ${second.stderr}`);
    const show = await runBeeState(dir, ['scribing-run', '--show', '--json']);
    assert(show.status === 0, `--show should succeed, got ${show.status}: ${show.stdout}${show.stderr}`);
    const payload = JSON.parse(show.stdout);
    assert(payload.feature === null, `overall --show carries no feature filter, got ${JSON.stringify(payload)}`);
    const secondEntry = JSON.parse(
      fs.readFileSync(path.join(dir, '.bee', 'logs', 'scribing-runs.jsonl'), 'utf8').trim().split('\n').pop(),
    );
    assert(
      payload.stamp === secondEntry.ts,
      `overall --show must return the LATEST stamp across every feature (${secondEntry.ts}), got ${JSON.stringify(payload)}`,
    );
  } finally {
    fs.rmSync(dir, { recursive: true, force: true });
  }
});

await check('sqs-b3: state scribing-run --show --feature <slug> returns that feature\'s own last stamp, not a different feature\'s later one', async () => {
  const dir = makeStateRepo('bee-scribing-show-feature-');
  try {
    writeJsonAtomic(path.join(dir, '.bee', 'state.json'), { phase: 'swarming', feature: 'demo' });
    const first = await runBeeState(dir, [
      'scribing-run', '--feature', 'demo', '--areas', 'demo-area', '--next-action', 'bee-compounding', '--json',
    ]);
    assert(first.status === 0, `setup scribing-run should succeed, got ${first.status}: ${first.stderr}`);
    const firstEntry = JSON.parse(
      fs.readFileSync(path.join(dir, '.bee', 'logs', 'scribing-runs.jsonl'), 'utf8').trim().split('\n').pop(),
    );
    // A LATER stamp for a different feature must not leak into "demo"'s answer.
    const second = await runBeeState(dir, [
      'scribing-run', '--feature', 'other-feat', '--areas', 'other-area', '--next-action', '-', '--json',
    ]);
    assert(second.status === 0, `second scribing-run should succeed, got ${second.status}: ${second.stderr}`);
    const show = await runBeeState(dir, ['scribing-run', '--show', '--feature', 'demo', '--json']);
    assert(show.status === 0, `--show --feature should succeed, got ${show.status}: ${show.stdout}${show.stderr}`);
    const payload = JSON.parse(show.stdout);
    assert(payload.feature === 'demo', `--show --feature must echo the requested feature, got ${JSON.stringify(payload)}`);
    assert(
      payload.stamp === firstEntry.ts,
      `--show --feature demo must return demo's OWN last stamp (${firstEntry.ts}), not the later other-feat one, got ${JSON.stringify(payload)}`,
    );
  } finally {
    fs.rmSync(dir, { recursive: true, force: true });
  }
});

await check('sqs-b3: state scribing-run --show is read-only — it does not append to the ledger and does not advance phase', async () => {
  const dir = makeStateRepo('bee-scribing-show-readonly-');
  try {
    writeJsonAtomic(path.join(dir, '.bee', 'state.json'), { phase: 'swarming', feature: 'demo' });
    const setup = await runBeeState(dir, [
      'scribing-run', '--feature', 'demo', '--areas', 'demo-area', '--next-action', 'bee-compounding', '--json',
    ]);
    assert(setup.status === 0, `setup scribing-run should succeed, got ${setup.status}: ${setup.stderr}`);
    const ledgerPath = path.join(dir, '.bee', 'logs', 'scribing-runs.jsonl');
    const beforeLedger = fs.readFileSync(ledgerPath, 'utf8');
    const beforeState = fs.readFileSync(path.join(dir, '.bee', 'state.json'));
    const show = await runBeeState(dir, ['scribing-run', '--show', '--feature', 'demo', '--json']);
    assert(show.status === 0, `--show should succeed, got ${show.status}: ${show.stdout}${show.stderr}`);
    const afterLedger = fs.readFileSync(ledgerPath, 'utf8');
    const afterState = fs.readFileSync(path.join(dir, '.bee', 'state.json'));
    assert(afterLedger === beforeLedger, '--show must NOT append a line to the scribing ledger');
    assert(afterState.equals(beforeState), '--show must NOT mutate state.json (no phase advance, no field write)');
    // The setup call above already advanced phase to "compounding" (the
    // normal write-mode side effect) — --show's own contract is that it
    // changes NOTHING further, which the byte-equality assert above already
    // proves; this just names the specific field for a clearer failure.
    const before = JSON.parse(beforeState.toString());
    const after = JSON.parse(afterState.toString());
    assert(after.phase === before.phase, `--show must not advance phase, was ${before.phase} now ${after.phase}`);
  } finally {
    fs.rmSync(dir, { recursive: true, force: true });
  }
});

await check('sqs-b3: state scribing-run --show works with NO --areas/--next-action (a bare read never needs write-only flags)', async () => {
  const dir = makeStateRepo('bee-scribing-show-bare-');
  try {
    writeJsonAtomic(path.join(dir, '.bee', 'state.json'), { phase: 'swarming', feature: null });
    const show = await runBeeState(dir, ['scribing-run', '--show', '--json']);
    assert(
      show.status === 0,
      `--show alone (no --areas/--next-action) must succeed since it never writes, got ${show.status}: ${show.stdout}${show.stderr}`,
    );
    const payload = JSON.parse(show.stdout);
    assert(payload.stamp === null, `an empty ledger must yield a null stamp, got ${JSON.stringify(payload)}`);
  } finally {
    fs.rmSync(dir, { recursive: true, force: true });
  }
});

await check('sqs-b3: bee.mjs --help --json advertises scribing-run --show', async () => {
  const dir = makeStateRepo('bee-scribing-show-registry-');
  try {
    const result = await runModuleWorker(beeStateModulePath(), { args: ['--help', '--json'], cwd: dir });
    assert(result.status === 0, `--help --json should succeed, got ${result.status}: ${result.stderr}`);
    const manifest = JSON.parse(result.stdout);
    const entry = manifest.commands.find((c) => c.name === 'state.scribing-run');
    assert(entry, 'state.scribing-run must be registered in the command manifest');
    assert(
      entry.parameters.properties.show,
      `state.scribing-run's registry entry must declare a --show flag, got ${JSON.stringify(Object.keys(entry.parameters.properties))}`,
    );
  } finally {
    fs.rmSync(dir, { recursive: true, force: true });
  }
});

// ─── msn-18c TOPOLOGY (must_have, red-first per the cell contract): a
// session bound from a linked worktree's `bee state session bind` lands the
// UPDATED record in MAIN's .bee/sessions/, never the worktree's own (pre-cell:
// nonexistent/empty) local sessions dir. Same manual worktree-fixture
// pattern test_cells.mjs's makeLinkedWorktreeCellFixture / test_reservations
// .mjs's makeLinkedWorktree use — this file's own minimal variant, onboarded
// on both sides so bee.mjs's CLI dispatch (subprocess, via runBeeState)
// resolves a real root on both.

function makeLinkedWorktreeStateFixture(mainPrefix, workPrefix, id) {
  const main = fs.mkdtempSync(path.join(os.tmpdir(), mainPrefix));
  const work = fs.mkdtempSync(path.join(os.tmpdir(), workPrefix));
  fs.mkdirSync(path.join(main, '.git'));
  const gitdir = path.join(main, '.git', 'worktrees', id);
  fs.mkdirSync(gitdir, { recursive: true });
  fs.writeFileSync(path.join(work, '.git'), `gitdir: ${gitdir}\n`);
  fs.writeFileSync(path.join(gitdir, 'gitdir'), path.join(work, '.git') + '\n');
  for (const dir of [main, work]) {
    fs.mkdirSync(path.join(dir, '.bee'), { recursive: true });
    writeJsonAtomic(path.join(dir, '.bee', 'onboarding.json'), { schema_version: '1.0', bee_version: '0.1.0' });
  }
  // GRANTED worktree (worktree-feature-parallelism): resolveRoots' own P40
  // default already resolves storeRoot to MAIN for an UNGRANTED worktree —
  // that path never touches the control-plane gap this cell fixes. Only a
  // GRANTED worktree (its own local store for workspace-local state) gets
  // `root` === workRoot at the CLI layer (findRepoRoot -> resolveRoots(cwd)
  // .storeRoot), which is exactly the population where session/workflow/
  // handoff-mailbox stores must STILL resolve to main (control-plane) even
  // though cells/backlog/decisions now resolve to the worktree itself —
  // advisor-digest-slice4 binding condition F4's "granted worktrees have
  // fully worktree-local stores today" population.
  fs.mkdirSync(path.join(main, '.bee', 'runtime'), { recursive: true });
  writeJsonAtomic(path.join(main, '.bee', 'runtime', 'worktree-grants.json'), { [id]: true });
  return { main, work };
}

await check(
  'TOPOLOGY (red-first): `bee state session bind` invoked from a linked worktree updates the session record in MAIN\'s .bee/sessions — pre-cell it looked for (and failed to find) the record in the worktree\'s own, always-empty local sessions dir',
  async () => {
    const { main, work } = makeLinkedWorktreeStateFixture('bee-topo-sess-main-', 'bee-topo-sess-work-', 'topo-sess-fixture');
    try {
      // Seed the session record where a live session's session-init hook
      // would actually create it post-cell: MAIN's control store. This is
      // the record the bind call below must find and update.
      const created = createSession(main, { id: 'topo-sess-1' });
      assert(created.ok === true, `precondition: session record must be creatable at main, got ${JSON.stringify(created)}`);

      const bound = await runBeeState(work, ['session', 'bind', '--session-id', 'topo-sess-1', '--lane', 'topo-feat', '--json']);
      assert(bound.status === 0, `session bind run from the worktree must succeed (the session lives at main), got ${bound.status}: ${bound.stderr}`);

      const atMain = readJson(path.join(main, '.bee', 'sessions', 'topo-sess-1.json'), null);
      assert(atMain && atMain.lane === 'topo-feat', `expected the bound record at MAIN to carry lane "topo-feat", got ${JSON.stringify(atMain)}`);

      // The worktree's own local sessions dir must never have received a
      // copy — pre-cell baseline was the exact opposite (bindSessionLane
      // looked ONLY at the worktree's own .bee/sessions/, found nothing,
      // and refused SESSION_MISSING even though a live record existed at
      // main); this is the red-first flip this cell fixes.
      const atWork = path.join(work, '.bee', 'sessions', 'topo-sess-1.json');
      assert(!fs.existsSync(atWork), `the worktree's own sessions dir must NOT hold a copy, found a file at ${atWork}`);
    } finally {
      fs.rmSync(main, { recursive: true, force: true });
      fs.rmSync(work, { recursive: true, force: true });
    }
  },
);

await check('TOPOLOGY: solo/main repos byte-identical — `bee state session bind` behaves exactly as before this cell (no linked worktree involved)', async () => {
  const dir = makeStateRepo('bee-topo-sess-solo-');
  try {
    const created = createSession(dir, { id: 'topo-sess-solo-1' });
    assert(created.ok === true, `precondition: session record must be creatable, got ${JSON.stringify(created)}`);
    const bound = await runBeeState(dir, ['session', 'bind', '--session-id', 'topo-sess-solo-1', '--lane', 'topo-feat-solo', '--json']);
    assert(bound.status === 0, `session bind should succeed, got ${bound.status}: ${bound.stderr}`);
    const record = readJson(path.join(dir, '.bee', 'sessions', 'topo-sess-solo-1.json'), null);
    assert(record && record.lane === 'topo-feat-solo', `expected the bound record to carry the lane, got ${JSON.stringify(record)}`);
  } finally {
    fs.rmSync(dir, { recursive: true, force: true });
  }
});

// si-3: the isolation guard itself — run last, over every check above.
// liveFeatureAtSuiteStart was captured once before the first check ran; this
// re-reads the SAME live path and asserts byte-identical `feature`. A
// mismatch names exactly what si-1's suite once did silently: some check in
// this file mutated the live repo's own .bee/state.json instead of its own
// fixture root.
await check(
  'si-3: this suite never mutates the LIVE repo .bee/state.json — its `feature` captured at suite start is unchanged at suite end (every fixture here uses its own scratch root; the isolation leak si-1 left behind, restored by hand, must never recur silently)',
  async () => {
    const liveFeatureAtSuiteEnd = liveRepoFeature();
    assert(
      liveFeatureAtSuiteStart === undefined || liveFeatureAtSuiteEnd === liveFeatureAtSuiteStart,
      `LIVE .bee/state.json feature drifted during this suite: was ${JSON.stringify(liveFeatureAtSuiteStart)}, is now ${JSON.stringify(liveFeatureAtSuiteEnd)} — some check above wrote through the live repo root instead of its own scratch fixture root.`,
    );
  },
);

printSummaryAndExit();
