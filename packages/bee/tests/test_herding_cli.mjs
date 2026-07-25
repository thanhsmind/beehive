#!/usr/bin/env node
// test_herding_cli.mjs — "bee herding enable/disable/status" CLI verb group
// (herding-dispatch-lock-toggle, decisions D1-D5). Proves the new verbs
// perform byte-for-byte the same marker-file operation as today's manual
// `touch`/`rm` gesture, and — the load-bearing truth — that they and the
// existing read-only interlock (.claude/skills/bee-herding/scripts/
// dispatch-interlock.mjs) agree on the exact same file when both resolve the
// MAIN checkout root independently via git, from a REAL git repo (not the
// synthetic `.git` directory makeTempRepo/makeStateRepo use for git-agnostic
// suites, which `git rev-parse --git-common-dir` refuses as "not a git
// repository").

import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { spawnSync } from 'node:child_process';
import { fileURLToPath } from 'node:url';
import { runModuleWorker } from '../../../scripts/lib/run-module-worker.mjs';
import { check, assert, printSummaryAndExit } from '../../../scripts/lib/test-fixture.mjs';
import { writeJsonAtomic } from '../lib/fsutil.mjs';

const TESTS_DIR = path.dirname(fileURLToPath(import.meta.url));
const REPO_ROOT = path.resolve(TESTS_DIR, '..', '..', '..');
const INTERLOCK = path.join(REPO_ROOT, '.claude', 'skills', 'bee-herding', 'scripts', 'dispatch-interlock.mjs');

function beeModulePath() {
  return fileURLToPath(new URL('../bee.mjs', import.meta.url));
}

function runBeeHerding(cwd, args) {
  return runModuleWorker(beeModulePath(), { args: ['herding', ...args], cwd });
}

function git(cwd, args) {
  const r = spawnSync('git', args, { cwd, encoding: 'utf8' });
  assert(r.status === 0, `git ${args.join(' ')} (cwd=${cwd}) failed: ${r.stderr}`);
  return r.stdout;
}

// A REAL git repo (git init, not a bare mkdir .git) — resolveHerdingMainRoot
// and dispatch-interlock.mjs both run `git rev-parse --git-common-dir`, which
// fails against a synthetic `.git` directory with no real repository state.
function makeHerdingRepo(prefix) {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), prefix));
  git(root, ['init', '-q', '-b', 'main']);
  git(root, ['config', 'user.email', 's@e']);
  git(root, ['config', 'user.name', 's']);
  fs.writeFileSync(path.join(root, 'f'), 'x');
  git(root, ['add', '.']);
  git(root, ['commit', '-q', '-m', 'init']);
  fs.mkdirSync(path.join(root, '.bee'), { recursive: true });
  writeJsonAtomic(path.join(root, '.bee', 'onboarding.json'), {
    schema_version: '1.0',
    bee_version: '0.1.0',
  });
  return fs.realpathSync(root);
}

function markerFile(root) {
  return path.join(root, '.bee', 'tmp', 'bee-herding.enable');
}

await check('bee herding status --json on a repo with no marker reports {enabled:false, marker, main_root}', async () => {
  const root = makeHerdingRepo('bee-herding-status-off-');
  try {
    const result = await runBeeHerding(root, ['status', '--json']);
    assert(result.status === 0, `status should succeed, got ${result.status}: ${result.stderr}`);
    const out = JSON.parse(result.stdout);
    assert(out.enabled === false, `expected enabled:false, got ${JSON.stringify(out)}`);
    assert(out.marker === markerFile(root), `expected marker ${markerFile(root)}, got ${out.marker}`);
    assert(out.main_root === root, `expected main_root ${root}, got ${out.main_root}`);
  } finally {
    fs.rmSync(root, { recursive: true, force: true });
  }
});

await check('bee herding enable creates the marker, and running it twice in a row is not an error', async () => {
  const root = makeHerdingRepo('bee-herding-enable-idem-');
  try {
    const first = await runBeeHerding(root, ['enable', '--json']);
    assert(first.status === 0, `enable should succeed, got ${first.status}: ${first.stderr}`);
    assert(fs.existsSync(markerFile(root)), 'marker file created');
    const firstOut = JSON.parse(first.stdout);
    assert(firstOut.enabled === true, `expected enabled:true, got ${JSON.stringify(firstOut)}`);

    const second = await runBeeHerding(root, ['enable', '--json']);
    assert(second.status === 0, `re-running enable on an already-enabled marker must not error, got ${second.status}: ${second.stderr}`);
    assert(fs.existsSync(markerFile(root)), 'marker file still present after the second enable');
  } finally {
    fs.rmSync(root, { recursive: true, force: true });
  }
});

await check('bee herding disable removes the marker, and running it twice in a row (or on an already-absent marker) is not an error', async () => {
  const root = makeHerdingRepo('bee-herding-disable-idem-');
  try {
    fs.mkdirSync(path.dirname(markerFile(root)), { recursive: true });
    fs.writeFileSync(markerFile(root), '');

    const first = await runBeeHerding(root, ['disable', '--json']);
    assert(first.status === 0, `disable should succeed, got ${first.status}: ${first.stderr}`);
    assert(!fs.existsSync(markerFile(root)), 'marker file removed');
    const firstOut = JSON.parse(first.stdout);
    assert(firstOut.enabled === false, `expected enabled:false, got ${JSON.stringify(firstOut)}`);

    const second = await runBeeHerding(root, ['disable', '--json']);
    assert(second.status === 0, `re-running disable on an already-absent marker must not error, got ${second.status}: ${second.stderr}`);
    assert(!fs.existsSync(markerFile(root)), 'marker file still absent after the second disable');
  } finally {
    fs.rmSync(root, { recursive: true, force: true });
  }
});

await check('after "bee herding enable", the existing read-only interlock (dispatch-interlock.mjs) independently resolves the SAME marker and reports enabled:true', async () => {
  const root = makeHerdingRepo('bee-herding-agree-');
  try {
    const enableResult = await runBeeHerding(root, ['enable', '--json']);
    assert(enableResult.status === 0, `enable should succeed, got ${enableResult.status}: ${enableResult.stderr}`);

    // dispatch-interlock.mjs resolves the main root itself via
    // `git rev-parse --git-common-dir` run from `root` as cwd — no
    // --main-root flag given, so this is a genuinely independent resolution,
    // not a shared explicit input.
    const interlockResult = spawnSync('node', [INTERLOCK], { cwd: root, encoding: 'utf8' });
    assert(interlockResult.status === 0, `interlock should report enabled (exit 0), got ${interlockResult.status}: ${interlockResult.stderr}`);
    const interlockOut = JSON.parse(interlockResult.stdout);
    assert(interlockOut.enabled === true, `interlock should agree the marker is enabled, got ${JSON.stringify(interlockOut)}`);
    assert(interlockOut.marker === markerFile(root), `interlock should resolve the SAME marker path, got ${interlockOut.marker}`);
  } finally {
    fs.rmSync(root, { recursive: true, force: true });
  }
});

await check('bee herding status --json reflects enabled:true once the marker exists', async () => {
  const root = makeHerdingRepo('bee-herding-status-on-');
  try {
    fs.mkdirSync(path.dirname(markerFile(root)), { recursive: true });
    fs.writeFileSync(markerFile(root), '');
    const result = await runBeeHerding(root, ['status', '--json']);
    assert(result.status === 0, `status should succeed, got ${result.status}: ${result.stderr}`);
    const out = JSON.parse(result.stdout);
    assert(out.enabled === true, `expected enabled:true, got ${JSON.stringify(out)}`);
  } finally {
    fs.rmSync(root, { recursive: true, force: true });
  }
});

await check('bee herding with no verb prints a Use: line listing enable/disable/status and exits non-zero', async () => {
  const root = makeHerdingRepo('bee-herding-noverb-');
  try {
    const result = await runBeeHerding(root, []);
    assert(result.status !== 0, 'no-verb invocation exits non-zero');
    assert(/Use:/.test(result.stderr), `expected a "Use:" line, got stderr="${result.stderr}"`);
    assert(
      /enable/.test(result.stderr) && /disable/.test(result.stderr) && /status/.test(result.stderr),
      `Use: line should list all three verbs, got ${result.stderr}`,
    );
  } finally {
    fs.rmSync(root, { recursive: true, force: true });
  }
});

printSummaryAndExit();
