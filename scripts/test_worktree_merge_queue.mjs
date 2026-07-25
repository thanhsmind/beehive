#!/usr/bin/env node
// test_worktree_merge_queue.mjs — multisession-native-22 (CONTEXT.md D8
// stage 5, D9 invariant 12): real CLI-level, two-OS-process proof that
// `bee worktree merge` now serializes competing merges against the SAME
// mainRoot through the integration queue instead of racing the
// 'worktree-admin' lock (whose loser used to hit an opaque
// WORKTREE_MERGE_MAIN_DIRTY refusal mid-verify — see
// skills/bee-hive/templates/lib/integration-queue.mjs's own module header
// for the full problem statement). Runs the REAL vendored dispatcher
// (.bee/bin/bee.mjs) as two genuine child OS processes for the contested
// case (never spawnSync for that call — a blocking spawnSync would itself
// serialize the two attempts trivially and prove nothing about the queue),
// same child-process-is-the-only-real-concurrency-proof discipline
// scripts/test_worktree_holds_race.mjs already documents
// (critical-patterns 20260714).
//
// Covers:
//   (a) two real merge requests against the SAME main checkout serialize:
//       the second never starts its own verify child until the first has
//       fully released the processor lease — proven with an fs-marker
//       barrier inside each injected verify script (same technique
//       scripts/test_worktree_cli.mjs's hardening-4c tests and
//       scripts/test_worktree_holds_race.mjs already use), not by timing
//       luck.
//   (b) no filesystem lock ('worktree-admin' OR 'integration-queue') is
//       held while a merge's verify child is running — regression (the
//       existing hardening-4c coverage, re-proven here under the queue
//       wrapper) PLUS the new queue-lock proof.
//   (c) a merge that never gets its turn within --queue-wait-ms returns a
//       typed, unambiguous non-success result — never readable as success —
//       naming its queue position (advisor condition B).
//   (d) solo merge (queue empty): CLI text/JSON carry no queue-specific
//       vocabulary at all — the byte-identical contract at the CLI surface.

import { spawn, spawnSync } from 'node:child_process';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { fileURLToPath, pathToFileURL } from 'node:url';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const REPO_ROOT = path.join(__dirname, '..');
const BEE_MJS = path.join(REPO_ROOT, '.bee', 'bin', 'bee.mjs');
const LOCK_LIB = path.join(REPO_ROOT, '.bee', 'bin', 'lib', 'lock.mjs');

const results = [];
function record(desc, passed, detail) {
  results.push({ desc, passed });
  console.log((passed ? 'PASS ' : 'FAIL ') + desc + (passed ? '' : ` -- ${detail}`));
}

function git(cwd, args, { allowFailure = false } = {}) {
  const r = spawnSync('git', args, { cwd, encoding: 'utf8' });
  if (!allowFailure && r.status !== 0) throw new Error(`git ${args.join(' ')} (cwd=${cwd}) failed: ${r.stderr}`);
  return r.stdout;
}

function bee(cwd, args, { env } = {}) {
  return spawnSync('node', [BEE_MJS, ...args], { cwd, encoding: 'utf8', env: env ? { ...process.env, ...env } : process.env });
}

// beeAsync — the async-`spawn` sibling of `bee` above, for the contended
// call that must run concurrently with another real OS process. `env`
// (merged over the current process's own env, never replacing it) is how
// each concurrent invocation tells its shared, single `commands.verify`
// script WHICH marker-file name to use — see writeBarrierVerifyScript below
// for why this can't just be baked into a per-call script file (there is
// only ONE commands.verify per project; both concurrent merges run the
// exact same command).
function beeAsync(cwd, args, { env } = {}) {
  return new Promise((resolve) => {
    const child = spawn('node', [BEE_MJS, ...args], { cwd, env: env ? { ...process.env, ...env } : process.env });
    let stdout = '';
    let stderr = '';
    child.stdout.on('data', (d) => {
      stdout += d.toString();
    });
    child.stderr.on('data', (d) => {
      stderr += d.toString();
    });
    child.on('exit', (code) => resolve({ status: code, stdout, stderr }));
    child.on('error', (err) => resolve({ status: null, stdout, stderr: String(err) }));
  });
}

async function waitForFile(filePath, { timeoutMs = 20_000, pollMs = 20 } = {}) {
  const deadline = Date.now() + timeoutMs;
  for (;;) {
    if (fs.existsSync(filePath)) return;
    if (Date.now() > deadline) {
      throw new Error(`waitForFile: timed out after ${timeoutMs}ms waiting for ${filePath}`);
    }
    // eslint-disable-next-line no-await-in-loop
    await new Promise((resolve) => setTimeout(resolve, pollMs));
  }
}

// Same D8a gitignore fixture scripts/test_worktree_cli.mjs uses, so a
// bootstrapped .bee store never registers as dirty to the merge pre-check.
// `*.marker` is THIS file's own addition: writeBarrierVerifyScript below
// leaves started-*/release-*.marker files sitting untracked in `main`'s
// working tree (isTreeDirty's `git status --porcelain` counts untracked
// litter as dirty, unlike the tracked-only `--untracked-files=no` check
// mainUntouchedProof uses) — invisible in every OTHER worktree_cli test
// because none of them run a SECOND merge against the same main afterward;
// this file's serialization scenario (a) does exactly that, so the leftover
// marker from merge A would otherwise make merge B's isTreeDirty pre-check
// fail for a reason that has nothing to do with the queue this cell adds.
const BEE_GITIGNORE = [
  '.bee/state.json',
  '.bee/reservations.json',
  '.bee/workers/',
  '.bee/logs/',
  '.bee/capture-queue.jsonl',
  '.bee/feedback-digest.json',
  '.bee/.inject-cache.json',
  '.bee/HANDOFF.json',
  '.bee/spikes/',
  '.bee/manifest-hash.json',
  '.bee/sessions/',
  '.bee/claims/',
  '.bee/runtime/',
  '.bee/cache/',
  '.bee/locks/',
  '*.marker',
  '',
].join('\n');

function initMain(main) {
  fs.mkdirSync(main, { recursive: true });
  git(main, ['init', '-q', '-b', 'main']);
  git(main, ['config', 'user.email', 's@e']);
  git(main, ['config', 'user.name', 's']);
  fs.writeFileSync(path.join(main, '.gitignore'), BEE_GITIGNORE);
  fs.mkdirSync(path.join(main, '.bee'), { recursive: true });
  fs.writeFileSync(path.join(main, '.bee', 'onboarding.json'), JSON.stringify({ schema_version: '1.0', bee_version: '0.0.0' }));
  fs.writeFileSync(path.join(main, '.bee', 'config.json'), JSON.stringify({ commands: {} }));
  fs.writeFileSync(path.join(main, 'f'), 'x');
  git(main, ['add', '.']);
  git(main, ['commit', '-q', '-m', 'init']);
}

function configureVerify(main, command) {
  fs.writeFileSync(path.join(main, '.bee', 'config.json'), JSON.stringify({ commands: { verify: command } }));
  git(main, ['add', '.bee/config.json']);
  git(main, ['commit', '-q', '-m', 'configure verify'], { allowFailure: true });
}

// writeBarrierVerifyScript — a SINGLE shared verify script (there is only
// ever one commands.verify per project, and BOTH concurrent merges in
// scenario (a) below run it) that reads its marker-file name from
// BEE_QUEUE_TEST_MARKER (set per-invocation via beeAsync's `env` option,
// never baked into the script file itself): touches "started-<name>.marker"
// the instant it runs, then busy-polls for "release-<name>.marker" before
// exiting 0. Same fs-barrier technique scripts/test_worktree_cli.mjs's
// hardening-4c tests and scripts/test_worktree_holds_race.mjs both use:
// deterministic, no fixed sleep/timing window to get unlucky on.
function writeBarrierVerifyScript(main) {
  const scriptName = 'verify-barrier.mjs';
  fs.writeFileSync(
    path.join(main, scriptName),
    [
      "import fs from 'node:fs';",
      "const name = process.env.BEE_QUEUE_TEST_MARKER || 'default';",
      "fs.writeFileSync(`started-${name}.marker`, '');",
      'while (!fs.existsSync(`release-${name}.marker`)) { /* spin */ }',
      'process.exit(0);',
      '',
    ].join('\n'),
  );
  git(main, ['add', scriptName]);
  git(main, ['commit', '-q', '-m', 'add shared verify barrier script']);
  return `node ${scriptName}`;
}

function mergeNewWorktree(main, feature) {
  const r = bee(main, ['worktree', 'new', '--feature', feature, '--json']);
  if (r.status !== 0) throw new Error(`worktree new --feature ${feature} failed: status=${r.status} stdout=${r.stdout} stderr=${r.stderr}`);
  return JSON.parse(r.stdout);
}

function addWork(worktreeRoot, fileName, message) {
  fs.writeFileSync(path.join(worktreeRoot, fileName), 'x\n');
  git(worktreeRoot, ['add', fileName]);
  git(worktreeRoot, ['commit', '-q', '-m', message]);
}

// Deliberately does NOT contain the substring "queue" anywhere in the tmp
// path — scenario (d) below asserts solo-merge output text carries no
// queue-specific vocabulary at all, and a tmp-dir-derived worktreeRoot path
// is part of that output; a prefix like "...-merge-queue-..." would leak
// into that assertion as a false positive that has nothing to do with the
// production text this cell actually changed.
const tmp = fs.mkdtempSync(path.join(os.tmpdir(), 'bee-wt-merge-mq-'));

try {
  // ── (a) + (b): two real merges against the SAME main checkout serialize,
  // and neither store lock is held while either verify child runs. ─────────
  {
    const main = path.join(tmp, 'serialize');
    initMain(main);
    configureVerify(main, writeBarrierVerifyScript(main));

    const createdA = mergeNewWorktree(main, 'demo-a');
    addWork(createdA.worktreeRoot, 'work-a.txt', 'work a');
    const createdB = mergeNewWorktree(main, 'demo-b');
    addWork(createdB.worktreeRoot, 'work-b.txt', 'work b');

    const startedA = path.join(main, 'started-a.marker');
    const startedB = path.join(main, 'started-b.marker');

    // A becomes the processor: launch it as a real background OS process.
    const pA = beeAsync(main, ['worktree', 'merge', '--id', createdA.id, '--json'], { env: { BEE_QUEUE_TEST_MARKER: 'a' } });
    await waitForFile(startedA);
    record('(a) merge A reached its verify child (the direct-processor path, queue empty at enqueue time)', true, '');

    const { lockFilePath } = await import(pathToFileURL(LOCK_LIB).href);
    const worktreeAdminHeld = fs.existsSync(lockFilePath(main, 'worktree-admin'));
    const queueLockHeld = fs.existsSync(lockFilePath(main, 'integration-queue'));
    record("(b) 'worktree-admin' lock is ABSENT while merge A's verify child runs (hardening-4c regression, under the new queue wrapper)", worktreeAdminHeld === false, `held=${worktreeAdminHeld}`);
    record("(b) 'integration-queue' lock is ABSENT while merge A's verify child runs (new queue-lock proof, invariant 12)", queueLockHeld === false, `held=${queueLockHeld}`);

    // Request B against the SAME main checkout — main is genuinely dirty
    // right now (A's staged, uncommitted merge). The OLD behavior would
    // refuse WORKTREE_MERGE_MAIN_DIRTY outright; B must instead enqueue and
    // wait its turn.
    const pB = beeAsync(main, ['worktree', 'merge', '--id', createdB.id, '--json'], { env: { BEE_QUEUE_TEST_MARKER: 'b' } });
    await new Promise((resolve) => setTimeout(resolve, 300)); // let B enqueue and take at least one poll tick
    record('(a) merge B has NOT started its own verify child while A still holds the processor lease', !fs.existsSync(startedB), `exists=${fs.existsSync(startedB)}`);

    const releasedAAtMs = Date.now();
    fs.writeFileSync(path.join(main, 'release-a.marker'), '');
    const resultA = await pA;
    {
      let parsedA = null;
      try {
        parsedA = JSON.parse(resultA.stdout);
      } catch {
        /* checked below */
      }
      record('(a) merge A itself exits 0 and reports ok:true', resultA.status === 0 && parsedA && parsedA.ok === true, `status=${resultA.status} stdout=${resultA.stdout}`);
    }

    await waitForFile(startedB);
    const startedBAtMs = fs.statSync(startedB).mtimeMs;
    record(
      '(a) merge B only started its own verify child AFTER A released the processor lease (strict serialization, not a timing coincidence)',
      startedBAtMs >= releasedAAtMs,
      `startedBAtMs=${startedBAtMs} releasedAAtMs=${releasedAAtMs}`,
    );

    fs.writeFileSync(path.join(main, 'release-b.marker'), '');
    const resultB = await pB;
    {
      let parsedB = null;
      try {
        parsedB = JSON.parse(resultB.stdout);
      } catch {
        /* checked below */
      }
      record('(a) merge B (once its turn came) exits 0 and reports ok:true — a request that WAITED still eventually succeeds', resultB.status === 0 && parsedB && parsedB.ok === true, `status=${resultB.status} stdout=${resultB.stdout}`);
    }

    const log = git(main, ['log', '--oneline', 'main']);
    record('(a) both merge commits landed on main', /demo-a/.test(log) && /demo-b/.test(log), log);
  }

  // ── (c): a merge that never gets its turn within --queue-wait-ms returns
  // a typed, unambiguous non-success result naming its queue position. ─────
  {
    const main = path.join(tmp, 'timeout');
    initMain(main);
    configureVerify(main, writeBarrierVerifyScript(main));

    const createdStuck = mergeNewWorktree(main, 'demo-stuck');
    addWork(createdStuck.worktreeRoot, 'work-stuck.txt', 'work stuck');
    const createdImpatient = mergeNewWorktree(main, 'demo-impatient');
    addWork(createdImpatient.worktreeRoot, 'work-impatient.txt', 'work impatient');

    const pStuck = beeAsync(main, ['worktree', 'merge', '--id', createdStuck.id, '--json'], { env: { BEE_QUEUE_TEST_MARKER: 'stuck' } });
    await waitForFile(path.join(main, 'started-stuck.marker'));

    const rImpatient = bee(main, ['worktree', 'merge', '--id', createdImpatient.id, '--queue-wait-ms', '800', '--json']);
    record('(c) an impatient merge whose turn never comes within --queue-wait-ms exits non-zero — never a success exit code', rImpatient.status !== 0, `status=${rImpatient.status}`);
    let parsedImpatient = null;
    try {
      parsedImpatient = JSON.parse(rImpatient.stdout);
    } catch {
      /* checked below */
    }
    record(
      '(c) --json reports ok:false, code:"INTEGRATION_QUEUE_TIMEOUT", merged:false',
      Boolean(parsedImpatient) && parsedImpatient.ok === false && parsedImpatient.code === 'INTEGRATION_QUEUE_TIMEOUT' && parsedImpatient.merged === false,
      rImpatient.stdout,
    );
    record('(c) the TEXT output unambiguously says the merge did NOT run', /did NOT run/.test(rImpatient.stdout), rImpatient.stdout);
    record(
      '(c) the result carries a numeric queue position',
      Boolean(parsedImpatient) && parsedImpatient.queue && typeof parsedImpatient.queue.position === 'number',
      JSON.stringify(parsedImpatient && parsedImpatient.queue),
    );

    // Clean up the still-running stuck processor before this scenario's
    // fixture is torn down.
    fs.writeFileSync(path.join(main, 'release-stuck.marker'), '');
    await pStuck;
  }

  // ── (d): solo merge (queue empty) — CLI text/JSON carry no queue-specific
  // vocabulary at all (the byte-identical-at-the-surface contract). ────────
  {
    const main = path.join(tmp, 'solo');
    initMain(main);
    const created = mergeNewWorktree(main, 'demo-solo');
    addWork(created.worktreeRoot, 'work-solo.txt', 'work solo');

    const r = bee(main, ['worktree', 'merge', '--id', created.id, '--json']);
    record('(d) solo merge (queue empty) exits 0', r.status === 0, `status=${r.status} stdout=${r.stdout} stderr=${r.stderr}`);
    record('(d) solo merge output text carries no queue-specific vocabulary at all', !/queue/i.test(r.stdout), r.stdout);
    const parsed = JSON.parse(r.stdout);
    record(
      '(d) solo merge JSON has no leftover queue field and never carries code:"INTEGRATION_QUEUE_TIMEOUT"',
      parsed.ok === true && !('queue' in parsed) && parsed.code !== 'INTEGRATION_QUEUE_TIMEOUT',
      JSON.stringify(parsed),
    );
  }
} finally {
  fs.rmSync(tmp, { recursive: true, force: true });
}

const failed = results.filter((r) => !r.passed).length;
console.log(`SUMMARY: ${results.length - failed}/${results.length} passed`);
process.exit(failed > 0 ? 1 : 0);
