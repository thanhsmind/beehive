#!/usr/bin/env node
// test_scratch.mjs — self-contained contract tests (no framework) for the
// tree-hygiene D1/D2 scratch home + broom: lib/scratch.mjs (computeSweepPlan/
// runSweep/containedRoot/isLiveFeature) plus the `bee tmp sweep` CLI wiring.
//
// SAFETY IS THE POINT (th-4): the sweep may ONLY EVER delete inside a real
// repo's <repo>/.bee/tmp/ or <repo>/.bee/spikes/ — every candidate is
// canonically resolved and re-checked to be contained in one of those two
// roots immediately before removal, and a symlink escape is refused, never
// followed. Every fixture here is a fresh fs.mkdtempSync() root standing in
// for a whole repo — NEVER the real checkout this file lives in.
//
// RED-FIRST (cell th-4): this file is written and run red (lib/scratch.mjs
// does not exist yet) BEFORE any implementation, per docs/history/
// tree-hygiene/CONTEXT.md D1/D2 and AGENTS.md critical rule 2.

import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

import { runModuleWorker } from '../../../scripts/lib/run-module-worker.mjs';
import { canSymlink, envSkipLine, SYMLINK_SKIP_REASON } from '../../../scripts/lib/env-capabilities.mjs';
import { writeJsonAtomic, readJson } from '../lib/fsutil.mjs';
import {
  scratchRoots,
  containedRoot,
  isLiveFeature,
  computeSweepPlan,
  runSweep,
} from '../lib/scratch.mjs';

const TESTS_DIR = path.dirname(fileURLToPath(import.meta.url));
const TEMPLATES_DIR = path.dirname(TESTS_DIR);
const BEE_MJS = path.join(TEMPLATES_DIR, 'bee.mjs');

let passed = 0;
let failed = 0;

async function check(name, fn) {
  try {
    await fn();
    passed += 1;
    console.log(`PASS  ${name}`);
  } catch (error) {
    failed += 1;
    console.log(`FAIL  ${name}`);
    console.log(`      ${error instanceof Error ? (error.stack || error.message) : error}`);
  }
}

function assert(condition, message) {
  if (!condition) throw new Error(message);
}

// A check whose FIXTURE needs symlink creation (denied without elevation /
// Developer Mode on win32). The symlink-escape REFUSAL contracts these
// checks pin cannot be exercised where the escape itself cannot be built —
// skip loudly, never weaken, never fail on a machine limit.
async function checkNeedsSymlink(name, fn) {
  if (!canSymlink()) {
    console.log(envSkipLine(SYMLINK_SKIP_REASON, name));
    return;
  }
  await check(name, fn);
}

// ─── fixture builders (mkdtempSync fixture roots only — never the real repo tree) ───

function makeFixtureRepo() {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), 'bee-scratch-test-'));
  fs.mkdirSync(path.join(root, '.bee'), { recursive: true });
  writeJsonAtomic(path.join(root, '.bee', 'onboarding.json'), {
    schema_version: '1.0',
    bee_version: '0.1.0',
  });
  return root;
}

function writeDefaultState(root, { feature = null, phase = 'idle' } = {}) {
  writeJsonAtomic(path.join(root, '.bee', 'state.json'), { feature, phase });
}

function writeLaneFixture(root, feature, phase) {
  fs.mkdirSync(path.join(root, '.bee', 'lanes'), { recursive: true });
  writeJsonAtomic(path.join(root, '.bee', 'lanes', `${feature}.json`), {
    schema_version: '1.0',
    feature,
    mode: null,
    phase,
    approved_gates: { context: false, shape: false, execution: false, review: false },
  });
}

// files: { "relative/path.txt": "content", ... }
function makeScratchDir(root, scratchRootRel, name, files = { 'a.txt': 'hello world' }) {
  const dir = path.join(root, scratchRootRel, name);
  for (const [rel, content] of Object.entries(files)) {
    const filePath = path.join(dir, rel);
    fs.mkdirSync(path.dirname(filePath), { recursive: true });
    fs.writeFileSync(filePath, content);
  }
  return dir;
}

function expectedBytesAndFiles(files) {
  let bytes = 0;
  let count = 0;
  for (const content of Object.values(files)) {
    bytes += Buffer.byteLength(content, 'utf8');
    count += 1;
  }
  return { bytes, count };
}

// ─── lib-level: containment / escape refusal (the safety bar) ─────────────

await check('scratchRoots resolves .bee/tmp and .bee/spikes when present, skips a missing one', async () => {
  const root = makeFixtureRepo();
  makeScratchDir(root, '.bee/tmp', 'only-tmp-exists');
  const roots = scratchRoots(root);
  const rels = roots.map((r) => r.rel).sort();
  assert(rels.length === 1 && rels[0] === '.bee/tmp', `expected only .bee/tmp, got ${JSON.stringify(rels)}`);
});

await check('containedRoot proves a plain in-root candidate contained', async () => {
  const root = makeFixtureRepo();
  const dir = makeScratchDir(root, '.bee/tmp', 'good-feat');
  const roots = scratchRoots(root);
  const proof = containedRoot(dir, roots);
  assert(proof && proof.rel === '.bee/tmp', `expected containment proof for a plain in-root dir, got ${JSON.stringify(proof)}`);
});

await checkNeedsSymlink('containedRoot REFUSES a symlink escaping both allowed roots — never follows it', async () => {
  const root = makeFixtureRepo();
  fs.mkdirSync(path.join(root, '.bee', 'tmp'), { recursive: true });
  const outsideDir = fs.mkdtempSync(path.join(os.tmpdir(), 'bee-scratch-outside-'));
  fs.writeFileSync(path.join(outsideDir, 'secret.txt'), 'do not delete me');
  const evilLink = path.join(root, '.bee', 'tmp', 'evil-link');
  fs.symlinkSync(outsideDir, evilLink, 'dir');

  const roots = scratchRoots(root);
  const proof = containedRoot(evilLink, roots);
  assert(proof === null, `a symlink escaping both roots must be refused (null), got ${JSON.stringify(proof)}`);
  // The escape must never even be followed to read its target's contents.
  assert(fs.existsSync(path.join(outsideDir, 'secret.txt')), 'outside target must be untouched by the mere containment check');
});

await checkNeedsSymlink('computeSweepPlan --all refuses the symlink escape (refused_escapes), never includes it, and dirSize/removal never dereference it', async () => {
  const root = makeFixtureRepo();
  makeScratchDir(root, '.bee/tmp', 'closed-feat', { 'x.txt': 'closed scratch' });
  const outsideDir = fs.mkdtempSync(path.join(os.tmpdir(), 'bee-scratch-outside2-'));
  fs.writeFileSync(path.join(outsideDir, 'secret.txt'), 'do not delete me either');
  fs.symlinkSync(outsideDir, path.join(root, '.bee', 'tmp', 'evil-link'), 'dir');

  const plan = computeSweepPlan(root, { all: true });
  const includedNames = plan.included.map((c) => c.name);
  assert(!includedNames.includes('evil-link'), `escape must never be included in the plan, got ${JSON.stringify(includedNames)}`);
  const refusedNames = plan.refusedEscapes.map((c) => c.name);
  assert(refusedNames.includes('evil-link'), `escape must be reported in refusedEscapes, got ${JSON.stringify(refusedNames)}`);

  const result = runSweep(root, { all: true, dryRun: false });
  assert(fs.existsSync(path.join(outsideDir, 'secret.txt')), 'a real --all sweep must never delete anything outside the two allowed roots');
  assert(fs.lstatSync(path.join(root, '.bee', 'tmp', 'evil-link')).isSymbolicLink(), 'the refused symlink itself must be left untouched, never removed');
  assert(!fs.existsSync(path.join(root, '.bee', 'tmp', 'closed-feat')), 'the legitimate contained candidate must still be removed by the same --all sweep');
  assert(result.refused_escapes.some((r) => r.name === 'evil-link'), 'runSweep result must report the refused escape');
});

// ─── no-flag refusal (CLI level — no default purge) ────────────────────────

await check('`bee tmp sweep` with NO flags refuses (typed), no default purge', async () => {
  const root = makeFixtureRepo();
  writeDefaultState(root, { feature: null, phase: 'idle' });
  makeScratchDir(root, '.bee/tmp', 'would-be-swept');
  const result = await runModuleWorker(BEE_MJS, { args: ['tmp', 'sweep'], cwd: root });
  assert(result.status !== 0, `no-flag call must refuse (non-zero exit), got status=${result.status} stdout=${result.stdout} stderr=${result.stderr}`);
  assert(/--feature|--before|--all|--dry-run/.test(result.stderr), `refusal must name the available flags, got stderr=${result.stderr}`);
  assert(fs.existsSync(path.join(root, '.bee', 'tmp', 'would-be-swept')), 'a refused no-flag call must delete nothing');
});

await check('`bee tmp sweep --json` (json alone, no real flag) still refuses with no default purge', async () => {
  const root = makeFixtureRepo();
  writeDefaultState(root, { feature: null, phase: 'idle' });
  const result = await runModuleWorker(BEE_MJS, { args: ['tmp', 'sweep', '--json'], cwd: root });
  assert(result.status !== 0, `--json alone must still refuse, got status=${result.status} stdout=${result.stdout} stderr=${result.stderr}`);
});

await check('`bee tmp sweep --dry-run` (a real flag) does not refuse and reports zero candidates with nothing on disk', async () => {
  const root = makeFixtureRepo();
  writeDefaultState(root, { feature: null, phase: 'idle' });
  const result = await runModuleWorker(BEE_MJS, { args: ['tmp', 'sweep', '--dry-run', '--json'], cwd: root });
  assert(result.status === 0, `--dry-run must be accepted as a real flag, got status=${result.status} stderr=${result.stderr}`);
  const payload = JSON.parse(result.stdout);
  assert(payload.dry_run === true, `expected dry_run:true, got ${result.stdout}`);
});

// ─── --dry-run removes nothing, and its listing matches a real run ─────────

await check('--dry-run removes nothing on disk and its removed[] matches a real run byte-for-byte', async () => {
  const root = makeFixtureRepo();
  const files = { 'a.txt': 'hello world', 'nested/b.txt': 'second file, more bytes here' };
  makeScratchDir(root, '.bee/tmp', 'closed-feat', files);
  writeLaneFixture(root, 'closed-feat', 'compounding-complete');
  writeDefaultState(root, { feature: null, phase: 'idle' });

  const dryPlan = runSweep(root, { all: true, dryRun: true });
  assert(dryPlan.dry_run === true, 'dry_run flag must be true');
  assert(fs.existsSync(path.join(root, '.bee', 'tmp', 'closed-feat')), '--dry-run must delete nothing');
  const expected = expectedBytesAndFiles(files);
  assert(dryPlan.bytes_freed === expected.bytes, `dry-run bytes_freed mismatch: expected ${expected.bytes}, got ${dryPlan.bytes_freed}`);
  assert(dryPlan.files_freed === expected.count, `dry-run files_freed mismatch: expected ${expected.count}, got ${dryPlan.files_freed}`);

  const realResult = runSweep(root, { all: true, dryRun: false });
  assert(realResult.bytes_freed === dryPlan.bytes_freed, 'a real run must free exactly the bytes the dry-run reported');
  assert(realResult.files_freed === dryPlan.files_freed, 'a real run must free exactly the files the dry-run reported');
  assert(!fs.existsSync(path.join(root, '.bee', 'tmp', 'closed-feat')), 'the real run must actually remove the directory');
});

// ─── closed / absent / live feature scratch (D2's default target set) ─────

await check('isLiveFeature: true for the default pipeline\'s active feature at a non-terminal phase', async () => {
  const root = makeFixtureRepo();
  writeDefaultState(root, { feature: 'live-feat', phase: 'planning' });
  assert(isLiveFeature(root, 'live-feat') === true, 'active non-terminal default feature must be live');
});

await check('isLiveFeature: false once the default pipeline feature reaches a terminal phase', async () => {
  const root = makeFixtureRepo();
  writeDefaultState(root, { feature: 'closed-feat', phase: 'compounding-complete' });
  assert(isLiveFeature(root, 'closed-feat') === false, 'a terminal-phase feature must not be live');
});

await check('isLiveFeature: true for a lane record at a non-terminal phase; false once compounding-complete', async () => {
  const root = makeFixtureRepo();
  writeLaneFixture(root, 'live-lane', 'swarming');
  writeLaneFixture(root, 'closed-lane', 'compounding-complete');
  assert(isLiveFeature(root, 'live-lane') === true, 'a swarming-phase lane must be live');
  assert(isLiveFeature(root, 'closed-lane') === false, 'a compounding-complete lane must not be live');
});

await check('isLiveFeature: false for a name with no record anywhere (absent)', async () => {
  const root = makeFixtureRepo();
  writeDefaultState(root, { feature: null, phase: 'idle' });
  assert(isLiveFeature(root, 'never-heard-of-it') === false, 'an unregistered name must never read as live');
});

await check('default sweep (no --feature/--all): closed + absent-with-before scratch is swept, live feature scratch survives', async () => {
  const root = makeFixtureRepo();
  writeDefaultState(root, { feature: 'live-feat', phase: 'swarming' }); // live
  writeLaneFixture(root, 'closed-feat', 'compounding-complete'); // closed
  // 'absent-feat' has no record anywhere.

  makeScratchDir(root, '.bee/tmp', 'live-feat', { 'a.txt': 'live scratch' });
  makeScratchDir(root, '.bee/tmp', 'closed-feat', { 'a.txt': 'closed scratch' });
  makeScratchDir(root, '.bee/tmp', 'absent-feat', { 'a.txt': 'orphan scratch' });

  const farFutureIso = new Date(Date.now() + 24 * 60 * 60 * 1000).toISOString();
  const result = runSweep(root, { before: farFutureIso, dryRun: false });

  assert(fs.existsSync(path.join(root, '.bee', 'tmp', 'live-feat')), 'a LIVE feature\'s scratch must survive an unnamed default sweep');
  assert(!fs.existsSync(path.join(root, '.bee', 'tmp', 'closed-feat')), 'a closed feature\'s scratch must be swept by default');
  assert(!fs.existsSync(path.join(root, '.bee', 'tmp', 'absent-feat')), 'absent (no-record) scratch older than --before must be swept by default');
});

await check('default sweep with NO --before never sweeps absent (no-record) scratch — only structurally closed scratch', async () => {
  const root = makeFixtureRepo();
  writeDefaultState(root, { feature: null, phase: 'idle' });
  writeLaneFixture(root, 'closed-feat', 'compounding-complete');
  makeScratchDir(root, '.bee/tmp', 'closed-feat', { 'a.txt': 'closed scratch' });
  makeScratchDir(root, '.bee/tmp', 'absent-feat', { 'a.txt': 'orphan scratch' });

  // computeSweepPlan called directly (lib level) — the CLI's own no-flag
  // refusal is a separate, already-covered concern; here we're proving the
  // "no age window for absent scratch without --before" default-set rule.
  const plan = computeSweepPlan(root, {});
  const names = plan.included.map((c) => c.name);
  assert(names.includes('closed-feat'), 'closed scratch must qualify even with no --before');
  assert(!names.includes('absent-feat'), 'absent (no-record) scratch must NOT qualify without an explicit --before cutoff');
});

await check('--feature explicitly sweeps a LIVE feature\'s scratch (the one documented override)', async () => {
  const root = makeFixtureRepo();
  writeDefaultState(root, { feature: 'live-feat', phase: 'swarming' });
  makeScratchDir(root, '.bee/tmp', 'live-feat', { 'a.txt': 'live scratch, explicitly named' });

  assert(isLiveFeature(root, 'live-feat') === true, 'fixture sanity: live-feat must be live');
  const result = runSweep(root, { feature: 'live-feat', dryRun: false });
  assert(result.removed.some((r) => r.name === 'live-feat'), `--feature must sweep the named live feature, got removed=${JSON.stringify(result.removed)}`);
  assert(!fs.existsSync(path.join(root, '.bee', 'tmp', 'live-feat')), '--feature must actually remove the named live feature\'s scratch dir');
});

await check('--all sweeps every entry, live or closed or absent alike ("clears the lot")', async () => {
  const root = makeFixtureRepo();
  writeDefaultState(root, { feature: 'live-feat', phase: 'swarming' });
  writeLaneFixture(root, 'closed-feat', 'compounding-complete');
  makeScratchDir(root, '.bee/tmp', 'live-feat');
  makeScratchDir(root, '.bee/tmp', 'closed-feat');
  makeScratchDir(root, '.bee/spikes', 'absent-feat');

  const result = runSweep(root, { all: true, dryRun: false });
  assert(result.removed.length === 3, `--all must sweep every entry, got ${JSON.stringify(result.removed)}`);
  assert(!fs.existsSync(path.join(root, '.bee', 'tmp', 'live-feat')), '--all must remove even a live feature\'s scratch');
});

// ─── deliverables are never touched, under every flag combination ─────────

await check('docs/**, .bee/cells/, .bee/decisions.jsonl are NEVER touched under any flag combination', async () => {
  const root = makeFixtureRepo();
  writeDefaultState(root, { feature: null, phase: 'idle' });
  writeLaneFixture(root, 'closed-feat', 'compounding-complete');

  fs.mkdirSync(path.join(root, 'docs', 'history', 'demo'), { recursive: true });
  fs.writeFileSync(path.join(root, 'docs', 'history', 'demo', 'CONTEXT.md'), '# real deliverable');
  fs.mkdirSync(path.join(root, '.bee', 'cells'), { recursive: true });
  fs.writeFileSync(path.join(root, '.bee', 'cells', 'demo-1.json'), '{"id":"demo-1"}');
  fs.writeFileSync(path.join(root, '.bee', 'decisions.jsonl'), '{"id":"dec-1"}\n');
  const docsHash = fs.readFileSync(path.join(root, 'docs', 'history', 'demo', 'CONTEXT.md'), 'utf8');
  const cellsHash = fs.readFileSync(path.join(root, '.bee', 'cells', 'demo-1.json'), 'utf8');
  const decisionsHash = fs.readFileSync(path.join(root, '.bee', 'decisions.jsonl'), 'utf8');

  const combos = [
    { all: true, dryRun: true },
    { all: true, dryRun: false },
    { feature: 'closed-feat', dryRun: false },
    { before: new Date(Date.now() + 86400000).toISOString(), dryRun: false },
  ];
  for (const flags of combos) {
    makeScratchDir(root, '.bee/tmp', 'closed-feat', { 'a.txt': 'reseeded scratch' });
    runSweep(root, flags);
    assert(fs.readFileSync(path.join(root, 'docs', 'history', 'demo', 'CONTEXT.md'), 'utf8') === docsHash, `docs/** must survive flags=${JSON.stringify(flags)}`);
    assert(fs.readFileSync(path.join(root, '.bee', 'cells', 'demo-1.json'), 'utf8') === cellsHash, `.bee/cells/ must survive flags=${JSON.stringify(flags)}`);
    assert(fs.readFileSync(path.join(root, '.bee', 'decisions.jsonl'), 'utf8') === decisionsHash, `.bee/decisions.jsonl must survive flags=${JSON.stringify(flags)}`);
  }
});

// ─── byte/file counts in the JSON match reality ────────────────────────────

await check('reported bytes_freed/files_freed match a manual walk of the real files removed', async () => {
  const root = makeFixtureRepo();
  const files = {
    'a.txt': 'x'.repeat(37),
    'sub/b.txt': 'y'.repeat(101),
    'sub/deeper/c.txt': 'z'.repeat(5),
  };
  makeScratchDir(root, '.bee/tmp', 'byte-check-feat', files);
  const expected = expectedBytesAndFiles(files);

  const result = runSweep(root, { feature: 'byte-check-feat', dryRun: false });
  assert(result.bytes_freed === expected.bytes, `expected bytes_freed ${expected.bytes}, got ${result.bytes_freed}`);
  assert(result.files_freed === expected.count, `expected files_freed ${expected.count}, got ${result.files_freed}`);
  assert(result.removed[0].bytes === expected.bytes, 'per-entry bytes must match the total for a single-feature sweep');
  assert(result.removed[0].files === expected.count, 'per-entry files must match the total for a single-feature sweep');
});

// ─── CLI wiring smoke test (registry entry actually dispatches) ───────────

await check('`bee tmp sweep --all --dry-run --json` (CLI, real fixture) reports the correct shape and deletes nothing', async () => {
  const root = makeFixtureRepo();
  writeDefaultState(root, { feature: null, phase: 'idle' });
  makeScratchDir(root, '.bee/tmp', 'cli-check-feat', { 'a.txt': 'cli wiring check' });
  const result = await runModuleWorker(BEE_MJS, { args: ['tmp', 'sweep', '--all', '--dry-run', '--json'], cwd: root });
  assert(result.status === 0, `CLI dry-run --all must succeed, got status=${result.status} stderr=${result.stderr}`);
  const payload = JSON.parse(result.stdout);
  assert(payload.dry_run === true, `expected dry_run:true in CLI output, got ${result.stdout}`);
  assert(Array.isArray(payload.removed) && payload.removed.some((r) => r.name === 'cli-check-feat'), `expected cli-check-feat listed, got ${result.stdout}`);
  assert(fs.existsSync(path.join(root, '.bee', 'tmp', 'cli-check-feat')), 'CLI --dry-run must not delete anything');
});

// ─── REWORK (judge th4-scratch-root-symlink-escape): a symlinked ROOT itself
// must never become the authority — only the LITERAL <repo>/.bee/tmp or
// <repo>/.bee/spikes prefix is ever trusted. Reproduces the judge's real
// data-loss repro through the same lib entry points a real caller uses. ────

await checkNeedsSymlink('scratchRoots REFUSES a symlinked .bee/spikes ROOT — never relocates the authority root to the symlink target', async () => {
  const root = makeFixtureRepo();
  fs.mkdirSync(path.join(root, '.bee', 'cells'), { recursive: true });
  fs.writeFileSync(path.join(root, '.bee', 'cells', 'demo-1.json'), '{"id":"demo-1"}');
  fs.writeFileSync(path.join(root, '.bee', 'decisions.jsonl'), '{"id":"dec-1"}\n');
  // .bee/spikes IS the repo root, symlinked — the exact judge repro.
  fs.symlinkSync(root, path.join(root, '.bee', 'spikes'), 'dir');

  const roots = scratchRoots(root);
  assert(!roots.some((r) => r.rel === '.bee/spikes'), `a symlinked .bee/spikes root must be refused entirely, got roots=${JSON.stringify(roots)}`);
});

await checkNeedsSymlink('REPRO (judge): .bee/spikes -> repo root, `tmp sweep --all` must NOT delete .bee/cells/ or .bee/decisions.jsonl', async () => {
  const root = makeFixtureRepo();
  writeDefaultState(root, { feature: null, phase: 'idle' });
  fs.mkdirSync(path.join(root, '.bee', 'cells'), { recursive: true });
  fs.writeFileSync(path.join(root, '.bee', 'cells', 'demo-1.json'), '{"id":"demo-1"}');
  fs.writeFileSync(path.join(root, '.bee', 'decisions.jsonl'), '{"id":"dec-1"}\n');
  fs.symlinkSync(root, path.join(root, '.bee', 'spikes'), 'dir');

  runSweep(root, { all: true, dryRun: false });

  assert(fs.existsSync(path.join(root, '.bee', 'cells', 'demo-1.json')), '.bee/cells/ must survive a symlinked .bee/spikes root under --all');
  assert(fs.existsSync(path.join(root, '.bee', 'decisions.jsonl')), '.bee/decisions.jsonl must survive a symlinked .bee/spikes root under --all');
  assert(fs.existsSync(path.join(root, '.bee', 'onboarding.json')), '.bee/onboarding.json must survive a symlinked .bee/spikes root under --all');
});

await checkNeedsSymlink('REPRO (judge): .bee/tmp -> docs/, `tmp sweep --all` must NOT delete docs/history', async () => {
  const root = makeFixtureRepo();
  writeDefaultState(root, { feature: null, phase: 'idle' });
  fs.mkdirSync(path.join(root, 'docs', 'history', 'demo'), { recursive: true });
  fs.writeFileSync(path.join(root, 'docs', 'history', 'demo', 'CONTEXT.md'), '# real deliverable');
  fs.rmSync(path.join(root, '.bee', 'tmp'), { recursive: true, force: true });
  fs.symlinkSync(path.join(root, 'docs'), path.join(root, '.bee', 'tmp'), 'dir');

  runSweep(root, { all: true, dryRun: false });

  assert(fs.existsSync(path.join(root, 'docs', 'history', 'demo', 'CONTEXT.md')), 'docs/history must survive a symlinked .bee/tmp root under --all');
});

await checkNeedsSymlink('dry-run/real-run parity: a symlinked .bee/tmp root is refused THE SAME WAY under --dry-run as under a real run', async () => {
  const root = makeFixtureRepo();
  writeDefaultState(root, { feature: null, phase: 'idle' });
  fs.mkdirSync(path.join(root, 'docs', 'history', 'demo'), { recursive: true });
  fs.writeFileSync(path.join(root, 'docs', 'history', 'demo', 'CONTEXT.md'), '# real deliverable');
  fs.rmSync(path.join(root, '.bee', 'tmp'), { recursive: true, force: true });
  fs.symlinkSync(path.join(root, 'docs'), path.join(root, '.bee', 'tmp'), 'dir');

  const dry = runSweep(root, { all: true, dryRun: true });
  const dryNames = dry.removed.map((r) => r.name);
  assert(!dryNames.includes('history'), `dry-run must not advertise 'docs' contents as sweepable via a symlinked root, got removed=${JSON.stringify(dry.removed)}`);
  assert(fs.existsSync(path.join(root, 'docs', 'history', 'demo', 'CONTEXT.md')), 'dry-run must delete nothing regardless');
});

await checkNeedsSymlink('plan-then-swap: a plan computed while .bee/tmp is real does not license a later run after the root is swapped to a symlink', async () => {
  const root = makeFixtureRepo();
  makeScratchDir(root, '.bee/tmp', 'closed-feat', { 'a.txt': 'closed scratch' });
  writeLaneFixture(root, 'closed-feat', 'compounding-complete');
  writeDefaultState(root, { feature: null, phase: 'idle' });
  fs.mkdirSync(path.join(root, '.bee', 'cells'), { recursive: true });
  fs.writeFileSync(path.join(root, '.bee', 'cells', 'demo-1.json'), '{"id":"demo-1"}');

  const plan = computeSweepPlan(root, { all: true });
  assert(plan.included.some((c) => c.name === 'closed-feat'), 'sanity: the plan must legitimately include closed-feat while the root is real');

  // Swap the ROOT itself to a symlink pointing at the repo root, between
  // planning and the actual run — the run must recompute and refuse, not
  // trust the earlier plan or the moved root.
  fs.rmSync(path.join(root, '.bee', 'tmp'), { recursive: true, force: true });
  fs.symlinkSync(root, path.join(root, '.bee', 'tmp'), 'dir');

  runSweep(root, { all: true, dryRun: false });
  assert(fs.existsSync(path.join(root, '.bee', 'cells', 'demo-1.json')), '.bee/cells/ must survive a plan-then-swap root escape');
});

// ─── finding 4 (--before does not age-gate closed-record scratch): pinned
// choice — a CLOSED (terminal-phase) record sweeps immediately, regardless
// of --before, because its closure is already the definitive signal. An
// --before cutoff only ever gates ABSENT (no-record) scratch. This is
// deliberate (see the file-header comment), not a bug — pinned here so a
// future change to this behavior is a conscious, tested decision. ─────────

await check('finding 4 (pinned): a CLOSED record sweeps even when --before predates its mtime — --before never age-gates closed scratch', async () => {
  const root = makeFixtureRepo();
  writeDefaultState(root, { feature: null, phase: 'idle' });
  writeLaneFixture(root, 'closed-feat', 'compounding-complete');
  makeScratchDir(root, '.bee/tmp', 'closed-feat', { 'a.txt': 'closed scratch' });

  // A --before cutoff safely in the past — well before closed-feat's mtime —
  // would fail to age-gate an ABSENT dir (see the existing "not old enough"
  // test above), but a CLOSED record is swept regardless.
  const pastIso = new Date(Date.now() - 24 * 60 * 60 * 1000).toISOString();
  const plan = computeSweepPlan(root, { before: pastIso });
  const names = plan.included.map((c) => c.name);
  assert(names.includes('closed-feat'), `a closed record must sweep even when --before predates it, got included=${JSON.stringify(names)}`);
});

// ─── issue #53's adjacent finding (issues-46-53 D7): LOOSE FILES at the ────
// scratch ROOT are entries too. The broom used to list only directories and
// symlinks, so the helper scripts and evidence dumps agents write straight
// into `.bee/tmp/` — the exact directory guards.mjs's refusal message tells
// them to use — were unsweepable by every flag, `--all` included. Measured on
// the bee checkout at the time of the fix: 58 of 76 entries in `.bee/tmp/`
// were loose files, and `tmp sweep --all --dry-run` reported 18.

await check('#53 finding: a LOOSE FILE at the scratch root is a sweep entry — --all clears it ("clears the lot" means the lot)', async () => {
  const root = makeFixtureRepo();
  writeDefaultState(root, { feature: null, phase: 'idle' });
  fs.mkdirSync(path.join(root, '.bee', 'tmp'), { recursive: true });
  makeScratchDir(root, '.bee/tmp', 'a-dir', { 'a.txt': 'dir scratch' });
  fs.writeFileSync(path.join(root, '.bee', 'tmp', 'build_helper.mjs'), 'console.log(1)\n');
  fs.writeFileSync(path.join(root, '.bee', 'tmp', 'f2-2-evidence.json'), '{"x":1}\n');

  const plan = computeSweepPlan(root, { all: true });
  const names = plan.included.map((c) => c.name).sort();
  assert(
    names.join(',') === 'a-dir,build_helper.mjs,f2-2-evidence.json',
    `--all must include loose root files, got included=${JSON.stringify(names)}`,
  );

  const result = runSweep(root, { all: true, dryRun: false });
  assert(!fs.existsSync(path.join(root, '.bee', 'tmp', 'build_helper.mjs')), '--all must actually remove a loose root script');
  assert(!fs.existsSync(path.join(root, '.bee', 'tmp', 'f2-2-evidence.json')), '--all must actually remove a loose root evidence file');
  assert(result.files_freed === 3, `files_freed must count the loose files, got ${result.files_freed}`);
});

await check('#53 finding: --feature <f> sweeps the feature\'s per-cell dirs and its loose <f>-* root files, not just a dir named exactly <f>', async () => {
  const root = makeFixtureRepo();
  writeLaneFixture(root, 'featx', 'compounding-complete');
  makeScratchDir(root, '.bee/tmp', 'featx', { 'a.txt': 'feature home' });
  makeScratchDir(root, '.bee/tmp', 'featx-3', { 'b.txt': 'per-cell scratch' });
  fs.writeFileSync(path.join(root, '.bee', 'tmp', 'featx-3-evidence.json'), '{"x":1}\n');
  fs.writeFileSync(path.join(root, '.bee', 'tmp', 'featx.log'), 'log\n');
  // unrelated neighbours that must survive: no separator boundary, and a
  // different feature entirely.
  fs.writeFileSync(path.join(root, '.bee', 'tmp', 'featxtra.json'), '{"keep":1}\n');
  makeScratchDir(root, '.bee/tmp', 'other', { 'c.txt': 'someone else' });

  const result = runSweep(root, { feature: 'featx', dryRun: false });
  const names = result.removed.map((r) => r.name).sort();
  assert(
    names.join(',') === 'featx,featx-3,featx-3-evidence.json,featx.log',
    `--feature must sweep the feature's dir, per-cell dirs and loose files, got removed=${JSON.stringify(names)}`,
  );
  assert(fs.existsSync(path.join(root, '.bee', 'tmp', 'featxtra.json')), 'a name without a separator boundary must survive (featx vs featxtra)');
  assert(fs.existsSync(path.join(root, '.bee', 'tmp', 'other')), 'an unrelated feature must survive --feature');
});

await check('#53 finding, safety: a prefix match never eats a LIVE sibling — --feature auth leaves auth-v2 alone and REPORTS the refusal', async () => {
  const root = makeFixtureRepo();
  writeLaneFixture(root, 'auth', 'compounding-complete');
  writeLaneFixture(root, 'auth-v2', 'swarming');
  makeScratchDir(root, '.bee/tmp', 'auth', { 'a.txt': 'closed' });
  makeScratchDir(root, '.bee/tmp', 'auth-v2', { 'b.txt': 'LIVE sibling — must survive' });

  assert(isLiveFeature(root, 'auth-v2') === true, 'fixture sanity: auth-v2 must be live');
  const plan = computeSweepPlan(root, { feature: 'auth' });
  const names = plan.included.map((c) => c.name);
  assert(names.join(',') === 'auth', `only the exact name may sweep here, got included=${JSON.stringify(names)}`);
  assert(
    plan.skipped.some((s) => s.name === 'auth-v2' && s.reason === 'live_sibling'),
    `a live sibling refused by the prefix rule must be REPORTED, got skipped=${JSON.stringify(plan.skipped)}`,
  );

  runSweep(root, { feature: 'auth', dryRun: false });
  assert(fs.existsSync(path.join(root, '.bee', 'tmp', 'auth-v2')), 'a LIVE sibling must survive a prefix sweep');
  assert(!fs.existsSync(path.join(root, '.bee', 'tmp', 'auth')), 'the exactly-named feature must still be swept');
});

await check('#53 finding: an EXACT --feature name still sweeps live scratch (the documented override is untouched by the prefix rule)', async () => {
  const root = makeFixtureRepo();
  writeDefaultState(root, { feature: 'live-feat', phase: 'swarming' });
  makeScratchDir(root, '.bee/tmp', 'live-feat', { 'a.txt': 'live' });
  fs.writeFileSync(path.join(root, '.bee', 'tmp', 'live-feat-1.txt'), 'live cell scratch\n');

  const result = runSweep(root, { feature: 'live-feat', dryRun: false });
  const names = result.removed.map((r) => r.name).sort();
  assert(names.join(',') === 'live-feat,live-feat-1.txt', `exact override plus its own loose files, got ${JSON.stringify(names)}`);
});

await check('#53 finding: loose root files stay behind the same no-default-purge discipline as dirs (absent, no --before → never swept)', async () => {
  const root = makeFixtureRepo();
  writeDefaultState(root, { feature: null, phase: 'idle' });
  fs.mkdirSync(path.join(root, '.bee', 'tmp'), { recursive: true });
  fs.writeFileSync(path.join(root, '.bee', 'tmp', 'stray.json'), '{"x":1}\n');

  const plan = computeSweepPlan(root, {});
  assert(plan.included.length === 0, `no flags must sweep nothing, got ${JSON.stringify(plan.included)}`);
  assert(
    plan.skipped.some((s) => s.name === 'stray.json' && s.reason === 'absent_no_before'),
    `a loose file with no record must be reported absent_no_before, got ${JSON.stringify(plan.skipped)}`,
  );
  assert(fs.existsSync(path.join(root, '.bee', 'tmp', 'stray.json')), 'nothing swept without a flag');
});

// ─── summary ────────────────────────────────────────────────────────────────

console.log(`\n${passed} passed, ${failed} failed`);
process.exitCode = failed > 0 ? 1 : 0;
