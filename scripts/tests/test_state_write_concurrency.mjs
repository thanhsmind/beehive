#!/usr/bin/env node
// test_state_write_concurrency.mjs — proves writeJsonAtomic (.bee/bin/lib/fsutil.mjs,
// mirrored from skills/bee-hive/templates/lib/fsutil.mjs) survives real concurrent
// writers hammering ONE shared target file.
//
// Why real OS processes, not async calls in one event loop: fs.writeFileSync and
// fs.renameSync are synchronous — inside a single Node process they can never
// actually interleave, so a "concurrent" test that just fires off several async
// calls in one process never exercises a genuine race. This spawns each writer
// as its own child process (this same file, re-invoked with --role=) so the
// races are real: separate processes racing to create/rename the same tmp path
// before the fix (advisor pre-Gate-3 finding 1), or racing on unique per-
// invocation tmp paths after it.
//
// Two writer shapes, per the cell:
//   hammer — repeatedly writes a fresh object with no read (raw contention on
//            the tmp path / rename target).
//   rmw    — a bee-state-sync-shaped loop: read current JSON, mutate a field,
//            write it back (hooks/bee-state-sync.mjs's read-modify-write shape).
// A concurrent monitor polls the target file throughout both bursts and
// verifies every read is well-formed JSON (never empty, never truncated,
// never a half-written fragment).
//
// HONEST LIMIT (advisor explicit — do not remove this note): this proves the
// tmp-name collision/corruption/crash class is gone — every writer gets its
// own uniquely-named tmp file, so one writer's write/rename can no longer
// clobber or race another writer's tmp file. It does NOT serialize the
// LOGICAL read-modify-write: two rmw workers can still both read the same
// "before" state, each compute their own "after", and whichever writes last
// wins — the earlier writer's update is silently overwritten (last-writer-
// wins). That race pre-exists today (Claude's own state-sync hook has it) and
// is explicitly OUT OF SCOPE for this cell; serializing state.mjs writes is a
// separate, future concern (named in CONTEXT.md D-cnr2-5 / cell cnr2-5).

import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { spawn } from 'node:child_process';
import { fileURLToPath } from 'node:url';

const __filename = fileURLToPath(import.meta.url);
const REPO_ROOT = path.join(path.dirname(__filename), '..', '..');
const FSUTIL_PATH = path.join(REPO_ROOT, '.bee', 'bin', 'lib', 'fsutil.mjs');

function argVal(flag) {
  const found = process.argv.find((a) => a.startsWith(`${flag}=`));
  return found ? found.slice(flag.length + 1) : undefined;
}

const role = argVal('--role');

if (role) {
  await runWorker(role);
} else {
  await runOrchestrator();
}

async function runWorker(workerRole) {
  const target = argVal('--target');
  const id = argVal('--id');
  const iters = Number(argVal('--iters'));
  const { writeJsonAtomic, readJson } = await import(FSUTIL_PATH);
  try {
    for (let i = 0; i < iters; i++) {
      if (workerRole === 'hammer') {
        writeJsonAtomic(target, {
          shape: 'hammer',
          writer: id,
          iter: i,
          pid: process.pid,
          ts: Date.now(),
          pad: 'x'.repeat(120),
        });
      } else if (workerRole === 'rmw') {
        // state-sync-shaped: read the current file, mutate one field, write
        // the whole object back — same shape as hooks/bee-state-sync.mjs.
        const current = readJson(target, { counts: {} });
        current.counts = current.counts && typeof current.counts === 'object' ? current.counts : {};
        current.counts[id] = (current.counts[id] || 0) + 1;
        current.last_activity = new Date().toISOString();
        writeJsonAtomic(target, current);
      } else {
        throw new Error(`unknown role: ${workerRole}`);
      }
    }
    process.exit(0);
  } catch (err) {
    console.error(`WORKER-CRASH id=${id} role=${workerRole}: ${err && err.stack ? err.stack : err}`);
    process.exit(1);
  }
}

function spawnWorkers(target, workerRole, count, iters) {
  const runs = [];
  for (let i = 0; i < count; i++) {
    runs.push(
      new Promise((resolve) => {
        const child = spawn(
          process.execPath,
          [__filename, `--role=${workerRole}`, `--target=${target}`, `--id=${workerRole}-${i}`, `--iters=${iters}`],
          { stdio: ['ignore', 'ignore', 'pipe'] },
        );
        let stderr = '';
        child.stderr.on('data', (chunk) => {
          stderr += chunk.toString();
        });
        child.on('exit', (code) => resolve({ id: i, code, stderr }));
        child.on('error', (err) => resolve({ id: i, code: null, stderr: String(err) }));
      }),
    );
  }
  return Promise.all(runs);
}

async function runOrchestrator() {
  const { writeJsonAtomic } = await import(FSUTIL_PATH);

  const tmpRoot = fs.mkdtempSync(path.join(os.tmpdir(), 'test-state-write-concurrency-'));
  const target = path.join(tmpRoot, 'shared-state.json');

  let corruptionSeen = null;
  let emptyReadSeen = false;
  let monitorReads = 0;
  const monitor = setInterval(() => {
    fs.readFile(target, 'utf8', (err, text) => {
      if (err) return; // ENOENT before the seed write lands is not a failure signal here
      monitorReads++;
      if (text.length === 0) {
        emptyReadSeen = true;
        return;
      }
      try {
        JSON.parse(text);
      } catch (parseErr) {
        corruptionSeen = corruptionSeen || `${parseErr.message} :: raw=${JSON.stringify(text.slice(0, 200))}`;
      }
    });
  }, 2);

  const failures = [];

  try {
    writeJsonAtomic(target, { counts: {}, seeded: true });

    const HAMMER_WORKERS = 10;
    const HAMMER_ITERS = 80;
    const hammerResults = await spawnWorkers(target, 'hammer', HAMMER_WORKERS, HAMMER_ITERS);
    const crashedHammer = hammerResults.filter((r) => r.code !== 0);
    if (crashedHammer.length) {
      failures.push(
        `${crashedHammer.length}/${HAMMER_WORKERS} hammer worker(s) crashed:\n` +
          crashedHammer.map((c) => `  worker ${c.id} exit=${c.code}\n  ${c.stderr.trim()}`).join('\n'),
      );
    }

    // Reseed via the real write path (not a bare fs write) between bursts so
    // the rmw counts below are interpretable.
    writeJsonAtomic(target, { counts: {}, seeded: true });

    const RMW_WORKERS = 8;
    const RMW_ITERS = 50;
    const rmwResults = await spawnWorkers(target, 'rmw', RMW_WORKERS, RMW_ITERS);
    const crashedRmw = rmwResults.filter((r) => r.code !== 0);
    if (crashedRmw.length) {
      failures.push(
        `${crashedRmw.length}/${RMW_WORKERS} read-modify-write worker(s) crashed:\n` +
          crashedRmw.map((c) => `  worker ${c.id} exit=${c.code}\n  ${c.stderr.trim()}`).join('\n'),
      );
    }
  } finally {
    clearInterval(monitor);
    // let any in-flight async monitor reads settle before the final check
    await new Promise((resolve) => setTimeout(resolve, 20));
  }

  let finalParsed = null;
  try {
    finalParsed = JSON.parse(fs.readFileSync(target, 'utf8'));
  } catch (err) {
    failures.push(`final target file is not valid JSON: ${err.message}`);
  }

  fs.rmSync(tmpRoot, { recursive: true, force: true });

  if (emptyReadSeen) failures.push('monitor observed an empty read of the target file during concurrency');
  if (corruptionSeen) failures.push(`monitor observed a non-parseable read during concurrency: ${corruptionSeen}`);
  if (monitorReads < 20) {
    failures.push(
      `monitor only completed ${monitorReads} reads — too few to have exercised real concurrency; the test is not proving anything`,
    );
  }
  if (finalParsed && (typeof finalParsed !== 'object' || finalParsed === null)) {
    failures.push('final target file parsed but is not a JSON object');
  }

  if (failures.length) {
    console.error('FAIL test_state_write_concurrency:');
    for (const f of failures) console.error(`  - ${f}`);
    process.exit(1);
  }

  console.log(
    `PASS test_state_write_concurrency: 10 hammer + 8 read-modify-write workers ` +
      `(80/50 iters each, 18 real OS processes total) hit one shared target with ` +
      `zero crashes and zero corrupt/truncated/empty reads (${monitorReads} monitor reads observed)`,
  );
}
