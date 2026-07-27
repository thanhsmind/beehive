#!/usr/bin/env node
// test_state_projection_race.mjs — proves that EVERY writer of the shared
// .bee/state.json / .bee/lanes/<f>.json projection serializes on the SAME
// 'state' store lock, and that they all acquire it in the SAME global order
// (workflow:<id> -> 'state'). Feature state-phase-lock-race, cell splr-1,
// GH #70; CONTEXT.md D1-D5.
//
// The defect this suite bites on: .bee/state.json is a bare read-modify-write
// (state-projection.mjs) whose callers did NOT agree on what to serialize
// against — the state-sync hook took 'state', withMutationLock's workflow
// branch took workflow:<id> ONLY, and two more call sites took nothing at
// all. Different lock names do not exclude each other, so a concurrent
// session's write is lost and the write-guard then reads a stale phase and
// falsely denies (the reported symptom).
//
// Why real OS child processes and not Promise.all/setTimeout in one process
// (critical pattern 20260714, "Async assertions under a non-awaiting runner
// pass vacuously"): every write on this path is a SYNCHRONOUS fs call, so an
// in-process "race" never interleaves them and the suite would pass
// vacuously. Every racer below is its own OS process — this same file,
// re-invoked with --role=.
//
// Why the fixture is a temp repo this file creates: no assertion here may be
// satisfiable by the live checkout's ambient state (recorded critical
// pattern: a red-first proof whose oracle can be fed by live-environment
// detection proves nothing). Every scenario builds its own mkdtemp repo and
// bootstraps it through the real CLI, and every oracle reads only from
// inside that temp repo.
//
// Scenarios
//   (a) NEGATIVE CONTROL — the pre-fix arrangement: role A holds 'state',
//       role B holds workflow:<id> ONLY, both driving the same
//       read-modify-write. MUST produce mutual-exclusion violations and/or a
//       lost counter. Without this the suite could pass vacuously — it is
//       what proves the detector actually bites.
//   (b) FIXED ARRANGEMENT — role B nests 'state' inside workflow:<id>.
//       Zero violations AND an exact final count of roles x iters.
//   (c) HELD-LOCK INVARIANT — the real production CLI, run against a temp
//       repo while ANOTHER process holds 'state'. A verb that respects the
//       lock cannot write state.json/lanes during that hold. This is the
//       invariant ("every production write of this record happens under the
//       'state' lock"), applied uniformly to whatever verbs are probed —
//       not a claim about specific source lines.
//   (d) PROJECTION-REBUILD HOLD — start-feature's post-startFeature
//       projection rebuild runs under its OWN 'state' hold (startFeature's
//       internal hold has already been released by then), read off the
//       lock's own contention telemetry.
//   (e) LOCK ORDER — workflow:<id> is always acquired BEFORE 'state', never
//       after. Asserted twice: from acquisition telemetry, and under real
//       contention (a process holding workflow:<id> must never see the
//       'state' lock taken by the blocked verb).
//   (f) STALENESS GUARD — every verb the CLI's own `state --help --json`
//       manifest reports must be classified below, so a newly added writer
//       cannot silently escape this suite.
//
// Scenarios (c)-(f) run against BOTH packages/bee/bee.mjs (canonical) and
// .bee/bin/bee.mjs (vendored) — this repo EXECUTES the vendored copy, so an
// un-vendored fix ships nothing.

import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { spawn, spawnSync } from 'node:child_process';
import { fileURLToPath } from 'node:url';

const __filename = fileURLToPath(import.meta.url);
const REPO_ROOT = path.join(path.dirname(__filename), '..', '..');
const LIB_DIR = path.join(REPO_ROOT, 'packages', 'bee', 'lib');
const STATE_LIB = path.join(LIB_DIR, 'state.mjs');
const PROJECTION_LIB = path.join(LIB_DIR, 'state-projection.mjs');
const LOCK_LIB = path.join(LIB_DIR, 'lock.mjs');
const WORKFLOW_LIB = path.join(LIB_DIR, 'workflow-store.mjs');

// The two copies of the CLI. The repo executes the vendored one.
const CLI_CANONICAL = path.join(REPO_ROOT, 'packages', 'bee', 'bee.mjs');
const CLI_VENDORED = path.join(REPO_ROOT, '.bee', 'bin', 'bee.mjs');

const RACERS_PER_ROLE = 3;
const ITERS = 6;
const HOLD_MS = 25; // widens the read-modify-write window each racer holds open
const LOCK_HOLD_MS = 2000; // external hold in scenarios (c)/(e)
const OBSERVE_MARGIN_MS = 350; // stop observing before the hold can expire
const POLL_MS = 20;

function argVal(flag) {
  const found = process.argv.find((a) => a.startsWith(`${flag}=`));
  return found ? found.slice(flag.length + 1) : undefined;
}

const role = argVal('--role');

// ─── shared helpers ─────────────────────────────────────────────────────────

function sleep(ms) {
  return new Promise((resolve) => setTimeout(resolve, ms));
}

function readJsonRaw(file, fallback) {
  try {
    return JSON.parse(fs.readFileSync(file, 'utf8'));
  } catch {
    return fallback;
  }
}

// Deliberately NOT atomic: the detector flag must be a naive read-then-write
// so an overlap is observable at all. Exclusivity in scenario (b) comes
// entirely from the locks around the caller, never from this helper.
function writeJsonRaw(file, obj) {
  fs.writeFileSync(file, JSON.stringify(obj));
}

function activeFlagPath(root) {
  return path.join(root, 'race-active.json');
}

function violationsPath(root) {
  return path.join(root, 'race-violations.jsonl');
}

function recordViolation(root, record) {
  try {
    fs.appendFileSync(violationsPath(root), `${JSON.stringify(record)}\n`);
  } catch {
    // never let the detector's own bookkeeping mask the finding
  }
}

function readViolations(root) {
  let raw;
  try {
    raw = fs.readFileSync(violationsPath(root), 'utf8');
  } catch {
    return [];
  }
  return raw
    .split('\n')
    .filter((l) => l.trim())
    .map((l) => {
      try {
        return JSON.parse(l);
      } catch {
        return { kind: 'unparsable', line: l };
      }
    });
}

// ─── worker roles (one OS process each) ─────────────────────────────────────

async function runWorker(kind) {
  if (kind === 'racer') return runRacer();
  if (kind === 'hold-state') return runHolder('state');
  if (kind === 'hold-workflow') return runHolder('workflow');
  process.stderr.write(`unknown --role=${kind}\n`);
  process.exit(2);
}

/**
 * The critical section every racer runs, under whichever lock arrangement
 * its --arrangement selects. Two independent oracles live here:
 *
 *  1. MUTUAL EXCLUSION — mark a shared active flag on entry, hold, then
 *     re-check it was not clobbered. Either an occupied flag on entry or a
 *     foreign holder on the re-check is an overlap, appended to
 *     violations.jsonl. This bites directly on exclusion; a clean final
 *     count alone would not (two racers can each net +1 by luck).
 *  2. LOST UPDATE — a counter read BEFORE the hold and written back AFTER
 *     it, through readState/writeState — the same bare read-modify-write
 *     every production writer of this record performs — followed by the
 *     real rebuildStateProjection so the production projection write is
 *     genuinely exercised, not simulated.
 */
async function racerCriticalSection(root, id) {
  const { readState, writeState } = await import(STATE_LIB);
  const { rebuildStateProjection } = await import(PROJECTION_LIB);
  const flag = activeFlagPath(root);

  const onEntry = readJsonRaw(flag, { active: false, holder: null });
  if (onEntry.active) {
    recordViolation(root, { kind: 'overlap-on-entry', me: id, saw: onEntry.holder, ts: Date.now() });
  }
  writeJsonRaw(flag, { active: true, holder: id });

  const counterBefore = Number(readState(root).race_counter || 0);
  await sleep(HOLD_MS);

  const midHold = readJsonRaw(flag, { active: false, holder: null });
  if (midHold.holder !== id) {
    recordViolation(root, { kind: 'clobbered-during-hold', me: id, saw: midHold.holder, ts: Date.now() });
  }

  const current = readState(root);
  writeState(root, { ...current, race_counter: counterBefore + 1 });
  rebuildStateProjection(root); // the real production projection write

  writeJsonRaw(flag, { active: false, holder: null });
}

async function runRacer() {
  const root = argVal('--root');
  const id = argVal('--id');
  const iters = Number(argVal('--iters'));
  const arrangement = argVal('--arrangement');
  const wfId = argVal('--wf');

  const { withStoreLock } = await import(LOCK_LIB);
  const { withWorkflowLock } = await import(WORKFLOW_LIB);
  const { controlRootFor } = await import(STATE_LIB);
  const ctrlRoot = controlRootFor(root);

  for (let i = 0; i < iters; i++) {
    const body = () => racerCriticalSection(root, id);
    if (arrangement === 'state') {
      // The state-sync hook shape (bee-state-sync.mjs:127).
      await withStoreLock(root, 'state', body);
    } else if (arrangement === 'workflow') {
      // The PRE-FIX CLI shape (withMutationLock's workflow branch):
      // workflow:<id> only, which does not exclude 'state'.
      await withWorkflowLock(ctrlRoot, wfId, body);
    } else if (arrangement === 'nested') {
      // The POST-FIX CLI shape: workflow:<id> -> 'state', in that order.
      await withWorkflowLock(ctrlRoot, wfId, () => withStoreLock(root, 'state', body));
    } else {
      throw new Error(`racer: unknown --arrangement=${arrangement}`);
    }
  }
  process.exit(0);
}

// Holds one named lock for --ms, touching --ready the instant it is held so
// the orchestrator never starts observing before the hold is real.
async function runHolder(which) {
  const root = argVal('--root');
  const ms = Number(argVal('--ms'));
  const ready = argVal('--ready');
  const { withStoreLock } = await import(LOCK_LIB);
  const { withWorkflowLock } = await import(WORKFLOW_LIB);
  const { controlRootFor } = await import(STATE_LIB);

  const body = async () => {
    fs.writeFileSync(ready, String(Date.now()));
    await sleep(ms);
  };
  if (which === 'state') {
    await withStoreLock(root, 'state', body, { maxAttempts: 200 });
  } else {
    await withWorkflowLock(controlRootFor(root), argVal('--wf'), body, { maxAttempts: 200 });
  }
  process.exit(0);
}

// ─── orchestrator ───────────────────────────────────────────────────────────

function spawnChild(args, opts = {}) {
  return new Promise((resolve) => {
    const child = spawn(process.execPath, args, { stdio: ['ignore', 'pipe', 'pipe'], ...opts });
    let stdout = '';
    let stderr = '';
    child.stdout.on('data', (c) => {
      stdout += c.toString();
    });
    child.stderr.on('data', (c) => {
      stderr += c.toString();
    });
    child.on('exit', (code) => resolve({ code, stdout, stderr }));
    child.on('error', (err) => resolve({ code: null, stdout, stderr: String(err) }));
  });
}

function runCli(cli, root, args) {
  return spawnSync(process.execPath, [cli, ...args], { cwd: root, encoding: 'utf8' });
}

/** A temp repo this test owns end to end — nothing is read from the live checkout. */
function makeRepo(prefix) {
  const root = fs.realpathSync(fs.mkdtempSync(path.join(os.tmpdir(), prefix)));
  fs.mkdirSync(path.join(root, '.bee'), { recursive: true });
  // The root marker findRepoRoot walks up for. Deliberately minimal: this
  // fixture carries no cells, no config, no history — only what the state
  // verbs themselves create.
  fs.writeFileSync(path.join(root, '.bee', 'onboarding.json'), JSON.stringify({ schema_version: '1.0' }));
  return root;
}

function bootstrapFeature(cli, root, feature, { lane = false } = {}) {
  const args = ['state', 'start-feature', '--feature', feature, '--mode', 'tiny'];
  if (lane) args.push('--as-lane');
  const res = runCli(cli, root, args);
  if (res.status !== 0) {
    throw new Error(`fixture bootstrap failed (${feature}): ${res.stdout}${res.stderr}`);
  }
  return res;
}

async function workflowIdFor(root, feature) {
  const { listWorkflows } = await import(WORKFLOW_LIB);
  const { controlRootFor } = await import(STATE_LIB);
  const wf = listWorkflows(controlRootFor(root)).workflows.find((w) => w.feature === feature && w.status !== 'closed');
  if (!wf) throw new Error(`fixture: no live workflow record for "${feature}"`);
  return wf.id;
}

/** Fingerprint of the shared projection record(s) this feature protects. */
function projectionFingerprint(root) {
  const parts = [];
  parts.push(`state:${readFileOrEmpty(path.join(root, '.bee', 'state.json'))}`);
  const lanesDir = path.join(root, '.bee', 'lanes');
  let lanes = [];
  try {
    lanes = fs.readdirSync(lanesDir).filter((f) => f.endsWith('.json')).sort();
  } catch {
    lanes = [];
  }
  for (const l of lanes) parts.push(`lane:${l}:${readFileOrEmpty(path.join(lanesDir, l))}`);
  return parts.join(' ');
}

function readFileOrEmpty(file) {
  try {
    return fs.readFileSync(file, 'utf8');
  } catch {
    return '';
  }
}

function contentionPath(root) {
  return path.join(root, '.bee', 'logs', 'contention.jsonl');
}

function resetContention(root) {
  const file = contentionPath(root);
  fs.mkdirSync(path.dirname(file), { recursive: true });
  fs.writeFileSync(file, '');
}

/** Every lock acquisition this repo's lock primitive recorded, in order. */
function acquisitions(root) {
  return readFileOrEmpty(contentionPath(root))
    .split('\n')
    .filter((l) => l.trim())
    .map((l) => {
      try {
        return JSON.parse(l);
      } catch {
        return null;
      }
    })
    .filter((r) => r && r.result === 'acquired');
}

async function waitForFile(file, timeoutMs) {
  const deadline = Date.now() + timeoutMs;
  while (Date.now() < deadline) {
    if (fs.existsSync(file)) return true;
    await sleep(10);
  }
  return false;
}

// ─── scenario (a)/(b): multi-process arrangement race ───────────────────────

async function runArrangementScenario(label, arrangementB) {
  const root = makeRepo(`bee-splr-${label}-`);
  const feature = 'race-feat';
  bootstrapFeature(CLI_CANONICAL, root, feature);
  const wfId = await workflowIdFor(root, feature);
  writeJsonRaw(activeFlagPath(root), { active: false, holder: null });
  fs.writeFileSync(violationsPath(root), '');

  const children = [];
  for (let i = 0; i < RACERS_PER_ROLE; i++) {
    children.push(
      spawnChild([
        __filename,
        '--role=racer',
        `--root=${root}`,
        `--id=A${i}`,
        `--iters=${ITERS}`,
        '--arrangement=state',
        `--wf=${wfId}`,
      ]),
    );
    children.push(
      spawnChild([
        __filename,
        '--role=racer',
        `--root=${root}`,
        `--id=B${i}`,
        `--iters=${ITERS}`,
        `--arrangement=${arrangementB}`,
        `--wf=${wfId}`,
      ]),
    );
  }
  const results = await Promise.all(children);
  const failedChildren = results.filter((r) => r.code !== 0);

  const { readState } = await import(STATE_LIB);
  const counter = Number(readState(root).race_counter || 0);
  const expected = RACERS_PER_ROLE * 2 * ITERS;
  const violations = readViolations(root);

  fs.rmSync(root, { recursive: true, force: true });
  return { counter, expected, violations, failedChildren };
}

// ─── scenario (c): the held-lock invariant, against the real CLI ────────────

/**
 * Runs one production CLI invocation against a temp repo while a SEPARATE
 * process holds the 'state' lock, and reports whether the shared projection
 * changed during that hold. A writer that serializes on 'state' cannot
 * write; a writer that takes a different lock (or none) writes immediately.
 */
async function probeHeldLock(cli, label, buildFixture, argsFor) {
  const root = makeRepo(`bee-splr-held-${label}-`);
  const ctx = buildFixture(cli, root);
  const readyFile = path.join(root, 'holder.ready');
  resetContention(root);

  const holder = spawnChild([
    __filename,
    '--role=hold-state',
    `--root=${root}`,
    `--ms=${LOCK_HOLD_MS}`,
    `--ready=${readyFile}`,
  ]);
  if (!(await waitForFile(readyFile, 5000))) {
    fs.rmSync(root, { recursive: true, force: true });
    return { label, error: 'holder never acquired the state lock' };
  }
  const holdStart = Date.now();
  const before = projectionFingerprint(root);

  const verb = spawnChild([cli, ...argsFor(ctx)], { cwd: root });

  let wroteDuringHold = false;
  let firstChangeAtMs = null;
  while (Date.now() - holdStart < LOCK_HOLD_MS - OBSERVE_MARGIN_MS) {
    if (projectionFingerprint(root) !== before) {
      wroteDuringHold = true;
      firstChangeAtMs = Date.now() - holdStart;
      break;
    }
    await sleep(POLL_MS);
  }

  await holder;
  const verbResult = await verb;
  const after = projectionFingerprint(root);
  fs.rmSync(root, { recursive: true, force: true });
  return {
    label,
    wroteDuringHold,
    firstChangeAtMs,
    changedEventually: after !== before,
    verbCode: verbResult.code,
    verbOut: `${verbResult.stdout}${verbResult.stderr}`.trim().split('\n').slice(-2).join(' | '),
  };
}

// ─── scenario (d): the projection rebuild has its own 'state' hold ─────────

/**
 * start-feature writes twice: the legacy record inside startFeature's own
 * 'state' hold, and then the projection rebuild AFTER that hold has already
 * been released (state.mjs releases before createWorkflow). An external
 * holder therefore cannot separate them — but the lock primitive's own
 * contention telemetry can: the rebuild running under the lock means a
 * SECOND 'state' acquisition is recorded for the same invocation.
 */
async function probeStartFeatureRebuild(cli, { lane }) {
  const root = makeRepo(`bee-splr-sf-${lane ? 'lane' : 'default'}-`);
  resetContention(root);
  const args = ['state', 'start-feature', '--feature', 'probe-feat', '--mode', 'tiny'];
  if (lane) args.push('--as-lane');
  const res = runCli(cli, root, args);
  const acqs = acquisitions(root);
  const stateAcqs = acqs.filter((a) => a.lock_name === 'state').length;
  fs.rmSync(root, { recursive: true, force: true });
  return {
    label: `state start-feature${lane ? ' --as-lane' : ''}`,
    ok: res.status === 0,
    stateAcqs,
    out: `${res.stdout}${res.stderr}`.trim().split('\n').slice(-2).join(' | '),
  };
}

// ─── scenario (e): one global lock order, workflow:<id> -> 'state' ─────────

/** Acquisition ORDER, read off the lock primitive's own telemetry. */
async function probeLockOrderTelemetry(cli) {
  const root = makeRepo('bee-splr-order-');
  const feature = 'lane-feat';
  bootstrapFeature(cli, root, feature, { lane: true });
  resetContention(root);
  const res = runCli(cli, root, ['state', 'plan-rev', 'bump', '--lane', feature]);
  const acqs = acquisitions(root);
  const names = acqs.map((a) => a.lock_name);
  const firstState = names.indexOf('state');
  const firstWorkflow = names.findIndex((n) => n.startsWith('workflow:'));
  fs.rmSync(root, { recursive: true, force: true });
  return {
    ok: res.status === 0,
    names,
    firstState,
    firstWorkflow,
    out: `${res.stdout}${res.stderr}`.trim().split('\n').slice(-2).join(' | '),
  };
}

/**
 * The same inversion under real contention: while a process holds
 * workflow:<id>, a blocked `plan-rev bump` must not be sitting on the
 * 'state' lock. If the 'state' lock file ever appears during that hold, the
 * two locks are being taken in opposite orders by two sessions — the
 * deterministic dual-LockBusyError the advisor consult flagged.
 */
async function probeLockOrderContention(cli) {
  const root = makeRepo('bee-splr-invert-');
  const feature = 'lane-feat';
  bootstrapFeature(cli, root, feature, { lane: true });
  const wfId = await workflowIdFor(root, feature);
  const { lockFilePath } = await import(LOCK_LIB);
  const stateLockFile = lockFilePath(root, 'state');
  const readyFile = path.join(root, 'holder.ready');

  const holder = spawnChild([
    __filename,
    '--role=hold-workflow',
    `--root=${root}`,
    `--wf=${wfId}`,
    `--ms=${LOCK_HOLD_MS}`,
    `--ready=${readyFile}`,
  ]);
  if (!(await waitForFile(readyFile, 5000))) {
    fs.rmSync(root, { recursive: true, force: true });
    return { error: 'holder never acquired the workflow lock' };
  }
  const holdStart = Date.now();
  const bump = spawnChild([cli, 'state', 'plan-rev', 'bump', '--lane', feature], { cwd: root });

  let stateHeldWhileBlocked = false;
  while (Date.now() - holdStart < LOCK_HOLD_MS - OBSERVE_MARGIN_MS) {
    if (fs.existsSync(stateLockFile)) {
      stateHeldWhileBlocked = true;
      break;
    }
    await sleep(POLL_MS);
  }

  await holder;
  const bumpResult = await bump;
  fs.rmSync(root, { recursive: true, force: true });
  return {
    stateHeldWhileBlocked,
    bumpCode: bumpResult.code,
    bumpOut: `${bumpResult.stdout}${bumpResult.stderr}`.trim().split('\n').slice(-2).join(' | '),
  };
}

// ─── scenario (f): staleness guard over the CLI's own manifest ─────────────

// Verbs that write .bee/state.json or a lane projection. Each must serialize
// on 'state'; the ones marked probed are exercised by scenario (c)/(d)/(e).
const STATE_WRITING_VERBS = [
  'state.set',
  'state.gate',
  'state.plan-rev.bump',
  'state.worker.add',
  'state.worker.update',
  'state.worker.remove',
  'state.worker.clear',
  'state.worker.prune',
  'state.scribing-run',
  'state.start-feature',
  'state.rebuild-projections',
  'state.advisor-ref.record',
];

// Verbs that never write the shared projection record (read-only listings,
// the handoff mailbox, compaction bookkeeping).
const NON_PROJECTION_VERBS = [
  'state.lanes',
  'state.session.list',
  'state.session.bind',
  'state.session.unbind',
  'state.handoff.write',
  'state.handoff.adopt',
  'state.handoff.show',
  'state.advisor-ref.show',
  'state.compact-log',
  'state.compact-check',
  'state.compact-capsule',
];

function probeManifestCoverage(cli) {
  const root = makeRepo('bee-splr-manifest-');
  const res = runCli(cli, root, ['state', '--help', '--json']);
  let names = [];
  let error = null;
  try {
    names = JSON.parse(res.stdout).commands.map((c) => c.name);
  } catch (err) {
    error = `could not parse the state manifest: ${String(err)}`;
  }
  fs.rmSync(root, { recursive: true, force: true });
  const classified = new Set([...STATE_WRITING_VERBS, ...NON_PROJECTION_VERBS]);
  const unclassified = names.filter((n) => !classified.has(n));
  const missing = [...classified].filter((n) => !names.includes(n));
  return { error, unclassified, missing, count: names.length };
}

// ─── run ────────────────────────────────────────────────────────────────────

async function runOrchestrator() {
  const failures = [];
  const note = (line) => process.stdout.write(`${line}\n`);

  note('test_state_projection_race — every state.json writer holds the same lock');
  note('');

  // (a) negative control
  const neg = await runArrangementScenario('negctl', 'workflow');
  note(
    `(a) NEGATIVE CONTROL  pre-fix arrangement (role B: workflow:<id> only): ` +
      `violations=${neg.violations.length} counter=${neg.counter}/${neg.expected}`,
  );
  if (neg.failedChildren.length) {
    failures.push(`(a) ${neg.failedChildren.length} racer process(es) exited non-zero: ${neg.failedChildren[0].stderr.slice(0, 400)}`);
  }
  if (neg.violations.length === 0) {
    failures.push(
      '(a) NEGATIVE CONTROL produced ZERO mutual-exclusion violations — the detector does not bite, so every ' +
        'other scenario in this suite could be passing vacuously. FIX the detector, never the expectation.',
    );
  }

  // (b) fixed arrangement
  const fixed = await runArrangementScenario('fixed', 'nested');
  note(
    `(b) FIXED ARRANGEMENT workflow:<id> -> 'state' (role B nested):        ` +
      `violations=${fixed.violations.length} counter=${fixed.counter}/${fixed.expected}`,
  );
  if (fixed.failedChildren.length) {
    failures.push(`(b) ${fixed.failedChildren.length} racer process(es) exited non-zero: ${fixed.failedChildren[0].stderr.slice(0, 400)}`);
  }
  if (fixed.violations.length !== 0) {
    failures.push(
      `(b) ${fixed.violations.length} mutual-exclusion violation(s) with 'state' nested inside workflow:<id> — ` +
        `first: ${JSON.stringify(fixed.violations[0])}`,
    );
  }
  if (fixed.counter !== fixed.expected) {
    failures.push(
      `(b) lost update: counter reached ${fixed.counter}, expected exactly ${fixed.expected} ` +
        '(roles x iters) through the shared read-modify-write.',
    );
  }

  for (const [cliLabel, cli] of [
    ['canonical packages/bee/bee.mjs', CLI_CANONICAL],
    ['vendored .bee/bin/bee.mjs', CLI_VENDORED],
  ]) {
    note('');
    note(`── production CLI: ${cliLabel}`);

    // (c) held-lock invariant
    const probes = [
      await probeHeldLock(
        cli,
        'state-set',
        (c, root) => {
          bootstrapFeature(c, root, 'race-feat');
          return { root };
        },
        () => ['state', 'set', '--phase', 'planning', '--owner', 'exploring'],
      ),
      await probeHeldLock(
        cli,
        'state-gate',
        (c, root) => {
          bootstrapFeature(c, root, 'race-feat');
          return { root };
        },
        () => ['state', 'gate', '--name', 'context', '--approved', 'true'],
      ),
      await probeHeldLock(
        cli,
        'rebuild-projections',
        (c, root) => {
          bootstrapFeature(c, root, 'race-feat');
          return { root };
        },
        () => ['state', 'rebuild-projections'],
      ),
      await probeHeldLock(
        cli,
        'worker-add',
        (c, root) => {
          bootstrapFeature(c, root, 'race-feat');
          return { root };
        },
        () => ['state', 'worker', 'add', '--nickname', 'w1', '--cell', 'c-1'],
      ),
    ];
    for (const p of probes) {
      if (p.error) {
        failures.push(`(c) ${cliLabel} ${p.label}: ${p.error}`);
        continue;
      }
      note(
        `(c) HELD-LOCK  ${p.label.padEnd(20)} wrote-during-hold=${String(p.wroteDuringHold).padEnd(5)} ` +
          `at=${p.firstChangeAtMs === null ? '-' : `${p.firstChangeAtMs}ms`} verb-exit=${p.verbCode}`,
      );
      if (p.wroteDuringHold) {
        failures.push(
          `(c) ${cliLabel}: "${p.label}" wrote the shared projection ${p.firstChangeAtMs}ms into another ` +
            "process's 'state' lock hold — it does not serialize on 'state', so a concurrent writer's update " +
            'is lost (GH #70).',
        );
      }
      if (p.verbCode !== 0) {
        failures.push(`(c) ${cliLabel}: "${p.label}" exited ${p.verbCode} after the lock was released: ${p.verbOut}`);
      } else if (!p.changedEventually) {
        failures.push(
          `(c) ${cliLabel}: "${p.label}" never wrote the projection at all — the probe proves nothing. ` +
            'FIX the fixture so the verb genuinely writes.',
        );
      }
    }

    // (d) the post-startFeature rebuild holds 'state' itself
    for (const lane of [false, true]) {
      const sf = await probeStartFeatureRebuild(cli, { lane });
      note(`(d) REBUILD-HOLD ${sf.label.padEnd(28)} 'state' acquisitions=${sf.stateAcqs}`);
      if (!sf.ok) {
        failures.push(`(d) ${cliLabel}: "${sf.label}" failed: ${sf.out}`);
      } else if (sf.stateAcqs < 2) {
        failures.push(
          `(d) ${cliLabel}: "${sf.label}" recorded ${sf.stateAcqs} 'state' acquisition(s). startFeature's own ` +
            'hold accounts for one and is released before the projection rebuild runs, so a second acquisition ' +
            'is what proves the rebuild is itself under the lock. It is currently unlocked.',
        );
      }
    }

    // (e) one global lock order
    const order = await probeLockOrderTelemetry(cli);
    note(`(e) LOCK-ORDER  plan-rev bump acquisitions: ${order.names.join(' -> ') || '(none)'}`);
    if (!order.ok) {
      failures.push(`(e) ${cliLabel}: "state plan-rev bump" failed: ${order.out}`);
    } else if (order.firstState === -1) {
      failures.push(`(e) ${cliLabel}: "state plan-rev bump" rebuilt a lane projection without ever holding 'state'.`);
    } else if (order.firstWorkflow === -1) {
      failures.push(`(e) ${cliLabel}: "state plan-rev bump" never acquired workflow:<id> — fixture is wrong.`);
    } else if (order.firstState < order.firstWorkflow) {
      failures.push(
        `(e) ${cliLabel}: "state plan-rev bump" acquires 'state' BEFORE workflow:<id> (${order.names.join(' -> ')}). ` +
          "The rest of the repo takes workflow:<id> -> 'state', so this is a lock-order inversion: two sessions " +
          'deadlock into a dual LockBusyError.',
      );
    }

    const inversion = await probeLockOrderContention(cli);
    if (inversion.error) {
      failures.push(`(e) ${cliLabel}: ${inversion.error}`);
    } else {
      note(
        `(e) LOCK-ORDER  under contention: 'state' taken while blocked on workflow:<id> = ` +
          `${inversion.stateHeldWhileBlocked}, bump exit=${inversion.bumpCode}`,
      );
      if (inversion.stateHeldWhileBlocked) {
        failures.push(
          `(e) ${cliLabel}: while another process held workflow:<id>, "state plan-rev bump" was blocked while ` +
            "HOLDING 'state' — the exact inverse of the order every other writer uses.",
        );
      }
      if (inversion.bumpCode !== 0) {
        failures.push(`(e) ${cliLabel}: "state plan-rev bump" exited ${inversion.bumpCode}: ${inversion.bumpOut}`);
      }
    }

    // (f) staleness guard
    const coverage = probeManifestCoverage(cli);
    note(`(f) COVERAGE    ${coverage.count} state verbs in the CLI manifest, all classified`);
    if (coverage.error) {
      failures.push(`(f) ${cliLabel}: ${coverage.error}`);
    }
    if (coverage.unclassified.length) {
      failures.push(
        `(f) ${cliLabel}: unclassified state verb(s) ${coverage.unclassified.join(', ')} — a new writer of ` +
          '.bee/state.json could be escaping this suite. FIX: add each to STATE_WRITING_VERBS (and probe it) ' +
          'or to NON_PROJECTION_VERBS.',
      );
    }
    if (coverage.missing.length) {
      failures.push(
        `(f) ${cliLabel}: classified verb(s) ${coverage.missing.join(', ')} no longer exist in the manifest — ` +
          'remove them from the classification lists.',
      );
    }
  }

  note('');
  if (failures.length) {
    note(`FAIL — ${failures.length} finding(s):`);
    for (const f of failures) note(`  ✗ ${f}`);
    process.exit(1);
  }
  note('PASS — every production writer of .bee/state.json serializes on the "state" lock,');
  note('       in one global order (workflow:<id> -> state), in both the canonical and vendored CLI.');
  process.exit(0);
}

// Dispatch last: the scenario tables above are `const`, so entering the
// orchestrator before they initialize would hit the temporal dead zone.
if (role) {
  await runWorker(role);
} else {
  await runOrchestrator();
}
