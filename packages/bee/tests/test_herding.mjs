#!/usr/bin/env node
// test_herding.mjs — the permanent regression suite for the bee-herding skill
// adoption (herding-adopt, cell h-2).
//
// It nails down the four things the adversarial review + advisor consult said
// the CONTRIBUTED implementation got wrong, each proven RED before the fix and
// asserted GREEN here:
//
//   1. control-loop.sh: a trailing value-flag (`--interval` with no value)
//      spun at 100% CPU forever (`shift 2` failed silently under `set -u`).
//      Now it refuses fast. This test would HANG (killed by timeout) against
//      the contributed script.
//   2. control-loop.sh: a non-numeric interval (`--interval abc`) turned the
//      60s loop into a hot loop — 872 iterations in 4 seconds, each an agent
//      invocation. Now it refuses. Zero is refused too (a 0s interval is the
//      same hot-loop defect).
//   3. dispatch-interlock.mjs: dispatch must build NOTHING without an
//      owner-created enable marker (D10). No marker → refusal (exit 3); marker
//      present → enabled (exit 0).
//   4. bootstrap-cockpit.sh: starts ONLY the dispatch loop, never a merge loop
//      (D11 — merge is a single-shot owner gesture).
//
// Everything here is a LOCAL subprocess: the control loop runs under its own
// `--command` test hook (never a real `claude` or a real pane), the interlock
// is a pure filesystem check in a temp dir, and bootstrap runs `--dry-run`
// (prints the herdr commands, touches no workspace). Nothing starts a pane or
// an agent.

import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { spawnSync } from 'node:child_process';
import { fileURLToPath } from 'node:url';

const TESTS_DIR = path.dirname(fileURLToPath(import.meta.url));
const REPO_ROOT = path.join(TESTS_DIR, '..', '..', '..');
const SKILL_SCRIPTS = path.join(REPO_ROOT, 'skills', 'bee-herding', 'scripts');
const CONTROL_LOOP = path.join(SKILL_SCRIPTS, 'control-loop.sh');
const BOOTSTRAP = path.join(SKILL_SCRIPTS, 'bootstrap-cockpit.sh');
const INTERLOCK = path.join(SKILL_SCRIPTS, 'dispatch-interlock.mjs');

let passed = 0;
let failed = 0;

function check(name, fn) {
  try {
    fn();
    passed += 1;
    console.log(`PASS  ${name}`);
  } catch (error) {
    failed += 1;
    console.log(`FAIL  ${name}`);
    console.log(`      ${error instanceof Error ? error.stack || error.message : error}`);
  }
}

function assert(condition, message) {
  if (!condition) throw new Error(message);
}

// A wall-clock ceiling on every subprocess. If a script SPINS, spawnSync kills
// it after `timeout` ms and reports result.signal — which is exactly how a
// spin/hot-loop is caught: a healthy refusal exits on its own (signal null)
// well under the ceiling, a spin is killed (signal set).
function run(cmd, args, opts = {}) {
  return spawnSync(cmd, args, {
    encoding: 'utf8',
    timeout: opts.timeout ?? 6000,
    ...opts,
  });
}

function assertNotSpun(result, label) {
  assert(
    result.signal === null,
    `${label}: process did not exit on its own — killed by the ${result.signal} watchdog, i.e. it SPUN (the pre-fix defect). status=${result.status}`,
  );
}

function tmpMainRoot(label) {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), `bee-herding-${label}-`));
  fs.mkdirSync(path.join(root, '.bee', 'tmp'), { recursive: true });
  return root;
}

// ─── Defect 1 — trailing value-flag must refuse, not spin ───────────────────

check('control-loop: a trailing --interval (no value) refuses fast, never spins (defect 1)', () => {
  const r = run('bash', [CONTROL_LOOP, '--role', 'dispatch', '--command', 'true', '--interval']);
  assertNotSpun(r, 'trailing --interval');
  assert(r.status === 1, `expected exit 1 (refused), got ${r.status}`);
  assert(
    /requires a value/.test(r.stderr || ''),
    `expected a "requires a value" refusal, got stderr: ${JSON.stringify(r.stderr)}`,
  );
});

check('control-loop: a trailing --role (no value) refuses fast, never spins (defect 1, generalised)', () => {
  const r = run('bash', [CONTROL_LOOP, '--command', 'true', '--role']);
  assertNotSpun(r, 'trailing --role');
  assert(r.status === 1, `expected exit 1 (refused), got ${r.status}`);
  assert(/requires a value/.test(r.stderr || ''), `expected refusal, got: ${JSON.stringify(r.stderr)}`);
});

// ─── Defect 2 — non-numeric / zero interval must refuse, not hot-loop ────────

check('control-loop: a non-numeric --interval abc refuses, never hot-loops (defect 2)', () => {
  const r = run('bash', [CONTROL_LOOP, '--role', 'dispatch', '--interval', 'abc', '--command', 'true']);
  assertNotSpun(r, '--interval abc');
  assert(r.status === 1, `expected exit 1 (refused), got ${r.status}`);
  assert(
    /positive integer/.test(r.stderr || ''),
    `expected a "positive integer" refusal, got stderr: ${JSON.stringify(r.stderr)}`,
  );
});

check('control-loop: a zero --interval refuses (0s is the same hot-loop defect)', () => {
  const r = run('bash', [CONTROL_LOOP, '--role', 'dispatch', '--interval', '0', '--command', 'true']);
  assertNotSpun(r, '--interval 0');
  assert(r.status === 1, `expected exit 1 (refused), got ${r.status}`);
  assert(/positive integer/.test(r.stderr || ''), `expected refusal, got: ${JSON.stringify(r.stderr)}`);
});

// ─── Bound — a failing iteration cannot 127-retry forever (D4) ───────────────

check('control-loop: a persistently failing iteration hits the consecutive-failure ceiling and exits (D4)', () => {
  // Mimics a missing `claude` binary (exit 127) that used to retry forever.
  const r = run(
    'bash',
    [
      CONTROL_LOOP,
      '--role', 'dispatch',
      '--command', 'exit 127',
      '--interval', '1',
      '--max-consecutive-failures', '2',
    ],
    { timeout: 8000 },
  );
  assertNotSpun(r, 'failing-iteration loop');
  assert(r.status === 1, `expected exit 1 (gave up at the ceiling), got ${r.status}`);
  assert(
    /consecutive failed iterations/.test(r.stderr || ''),
    `expected the ceiling message, got stderr: ${JSON.stringify(r.stderr)}`,
  );
});

check('control-loop: a passing --once iteration still exits 0 (happy path intact)', () => {
  const r = run('bash', [CONTROL_LOOP, '--role', 'dispatch', '--command', 'true', '--once']);
  assertNotSpun(r, '--once happy path');
  assert(r.status === 0, `expected exit 0, got ${r.status} (stderr: ${JSON.stringify(r.stderr)})`);
});

// ─── Interlock — dispatch builds nothing without the owner enable marker (D10) ─

check('interlock: no enable marker → dispatch refused (enabled:false, exit 3)', () => {
  const root = tmpMainRoot('interlock-off');
  try {
    const r = run('node', [INTERLOCK, '--main-root', root]);
    assertNotSpun(r, 'interlock (no marker)');
    assert(r.status === 3, `expected exit 3 (disabled), got ${r.status} (stderr: ${JSON.stringify(r.stderr)})`);
    const out = JSON.parse(r.stdout);
    assert(out.enabled === false, `expected enabled:false, got ${JSON.stringify(out)}`);
    assert(/enable marker/.test(out.reason), `reason should name the missing marker, got ${JSON.stringify(out.reason)}`);
  } finally {
    fs.rmSync(root, { recursive: true, force: true });
  }
});

check('interlock: owner enable marker present → dispatch permitted (enabled:true, exit 0)', () => {
  const root = tmpMainRoot('interlock-on');
  try {
    fs.writeFileSync(path.join(root, '.bee', 'tmp', 'bee-herding.enable'), '');
    const r = run('node', [INTERLOCK, '--main-root', root]);
    assertNotSpun(r, 'interlock (marker present)');
    assert(r.status === 0, `expected exit 0 (enabled), got ${r.status} (stderr: ${JSON.stringify(r.stderr)})`);
    const out = JSON.parse(r.stdout);
    assert(out.enabled === true, `expected enabled:true, got ${JSON.stringify(out)}`);
  } finally {
    fs.rmSync(root, { recursive: true, force: true });
  }
});

// ─── Bootstrap — starts ONE loop (dispatch), never a merge loop (D11) ────────

check('bootstrap: dry-run starts exactly ONE dispatch loop and ZERO merge loops (D11)', () => {
  const root = tmpMainRoot('bootstrap');
  try {
    const r = run('bash', [BOOTSTRAP, '--workspace', 'w1', '--main-root', root, '--dry-run']);
    assertNotSpun(r, 'bootstrap dry-run');
    assert(r.status === 0, `expected exit 0, got ${r.status} (stderr: ${JSON.stringify(r.stderr)})`);
    const lines = (r.stdout || '').split('\n');
    const paneRuns = lines.filter((l) => /herdr pane run/.test(l));
    const dispatchStarts = paneRuns.filter((l) => /--role dispatch/.test(l));
    const mergeStarts = paneRuns.filter((l) => /--role merge/.test(l));
    assert(
      dispatchStarts.length === 1,
      `expected exactly ONE dispatch loop start, got ${dispatchStarts.length}: ${JSON.stringify(paneRuns)}`,
    );
    assert(
      mergeStarts.length === 0,
      `expected ZERO merge loop starts (merge is a gesture, D11), got ${mergeStarts.length}: ${JSON.stringify(paneRuns)}`,
    );
  } finally {
    fs.rmSync(root, { recursive: true, force: true });
  }
});

check('bootstrap: a trailing --workspace (no value) refuses, never spins', () => {
  const r = run('bash', [BOOTSTRAP, '--workspace']);
  assertNotSpun(r, 'bootstrap trailing --workspace');
  assert(r.status === 1, `expected exit 1 (refused), got ${r.status}`);
  assert(/requires a value/.test(r.stderr || ''), `expected refusal, got: ${JSON.stringify(r.stderr)}`);
});

console.log(`\n${failed === 0 ? 'PASS' : 'FAIL'} test_herding: ${passed} passed, ${failed} failed`);
process.exit(failed === 0 ? 0 : 1);
