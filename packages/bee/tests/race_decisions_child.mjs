#!/usr/bin/env node
// race_decisions_child.mjs — self-contained multi-worker race orchestrator
// for decisions.mjs's archive verb (decision-propagation dp-3).
//
// Same harness constraint and shape as race_claims_child.mjs (test_lib.mjs's
// check() runner is synchronous and never awaits an async fn passed to it,
// so the ENTIRE race lives HERE as a self-contained module): invoked with a
// scenario argument (process.argv[2]), starts its own barrier-synchronized
// Worker racers, asserts internally, prints ONE summary line, exits 0/1.
// test_decisions_propagation.mjs runs this through the shared runModuleWorker
// (same primitive `runBee` uses) and asserts exit code + summary line.

import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { Worker, workerData } from 'node:worker_threads';
import { logDecision, archiveDecisions, activeDecisions } from '../lib/decisions.mjs';
import { readJsonl, writeJsonAtomic } from '../lib/fsutil.mjs';

const self = fileURLToPath(import.meta.url);

if (workerData?.raceRole) {
  runRacer(workerData.raceRole);
} else {
  main();
}

function runRacer(role) {
  switch (role.kind) {
    case 'log':
      return raceLog(role);
    case 'archive':
      return raceArchive(role);
    default:
      process.exit(2);
  }
}

function spinUntil(goFile) {
  while (!fs.existsSync(goFile)) { /* spin */ }
}

function sleepSyncMs(ms) {
  Atomics.wait(new Int32Array(new SharedArrayBuffer(4)), 0, 0, ms);
}

// scenario racer: repeatedly append decide events until the stop file
// appears, reporting every id it successfully logged (one per line on
// stdout, prefixed "ID ") so the orchestrator can verify none were lost.
function raceLog({ root, goFile, stopFile, workerIndex }) {
  spinUntil(goFile);
  let i = 0;
  let failed = null;
  while (!fs.existsSync(stopFile)) {
    try {
      const event = logDecision(root, {
        decision: `race decide w${workerIndex}-${i}`,
        rationale: 'race fixture — log-vs-archive concurrency (dp-3)',
        scope: 'race',
      });
      process.stdout.write(`ID ${event.id}\n`);
    } catch (error) {
      failed = error instanceof Error ? error.message : String(error);
      break;
    }
    i += 1;
  }
  if (failed) {
    process.stderr.write(`raceLog w${workerIndex} failed: ${failed}\n`);
    process.exit(1);
  }
  process.exit(0);
}

// scenario racer: repeatedly archives events strictly older than `before`
// while loggers race. `before` is pinned well before the race window opens,
// so only the pre-seeded corpus is ever eligible — live-logged events (dated
// "now") must never be swept up mid-write, keeping the correctness check
// (no id ever lost, no id ever duplicated across both files) unambiguous.
function raceArchive({ root, goFile, stopFile, before }) {
  spinUntil(goFile);
  let sawBusy = 0;
  while (!fs.existsSync(stopFile)) {
    try {
      archiveDecisions(root, { before });
    } catch (error) {
      // NOTHING_QUALIFIES is expected once the pre-seeded corpus is fully
      // moved — a clean, typed no-op, not a race failure. LOCK_BUSY under
      // heavy logger contention is also tolerated here (the bounded retry
      // already gave it its budget) — a real failure is anything else.
      const code = error && error.code;
      if (code === 'DECISIONS_ARCHIVE_NOTHING_QUALIFIES') {
        // nothing left to archive right now — brief pause, try again later
      } else if (code === 'DECISIONS_LOCK_BUSY') {
        sawBusy += 1;
      } else {
        process.stderr.write(`raceArchive failed: ${error instanceof Error ? error.stack : error}\n`);
        process.exit(1);
      }
    }
    // A real archive verb is a human/scheduled action, never a tight loop —
    // pausing between attempts (outside the lock) gives loggers real windows
    // to acquire it, matching the doctrine that contention should be brief
    // and occasional, not constant. Without this the archiver alone can
    // monopolize the lock for longer than a logger's whole bounded-retry
    // budget, which is a race-fixture artifact, not a real defect.
    sleepSyncMs(15);
  }
  process.stdout.write(`BUSY ${sawBusy}\n`);
  process.exit(0);
}

// ─── orchestrator (parent) side ─────────────────────────────────────────────

function startRacer(role) {
  return new Worker(self, { workerData: { raceRole: role }, stdout: true, stderr: true });
}

function collect(stream) {
  return new Promise((resolve) => {
    let out = '';
    stream.setEncoding('utf8');
    stream.on('data', (chunk) => { out += chunk; });
    stream.on('end', () => resolve(out));
  });
}

function waitWorker(worker) {
  const exitP = new Promise((resolve) => worker.once('exit', resolve));
  const stdoutP = collect(worker.stdout);
  const stderrP = collect(worker.stderr);
  return Promise.all([exitP, stdoutP, stderrP]).then(([code, stdout, stderr]) => ({ code, stdout, stderr }));
}

function sleep(ms) {
  return new Promise((resolve) => setTimeout(resolve, ms));
}

function freshRoot(prefix) {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), prefix));
  fs.mkdirSync(path.join(root, '.git'), { recursive: true });
  fs.mkdirSync(path.join(root, '.bee'), { recursive: true });
  writeJsonAtomic(path.join(root, '.bee', 'onboarding.json'), { schema_version: '1.0', bee_version: '0.1.0' });
  return root;
}

// Scenario: log-vs-archive — several loggers append decide events while an
// archiver concurrently sweeps a pre-seeded old corpus, under the shared
// decisions store lock. Truth: zero data loss (every id — seeded or logged
// — ends up in exactly the active file, the archive file, or both is never
// true for the same id under a normal, non-crash run) and both files stay
// valid JSONL throughout.
async function logVsArchive() {
  const root = freshRoot('bee-race-decisions-');
  const failures = [];

  // Pre-seed an old corpus the archiver can always find something to do
  // with, dated well before the race window's `before` cutoff.
  const OLD_DATE = '2020-01-01T00:00:00.000Z';
  const CUTOFF = '2021-01-01T00:00:00.000Z';
  const SEED_COUNT = 10;
  const seededIds = [];
  for (let i = 0; i < SEED_COUNT; i += 1) {
    const event = logDecision(root, { decision: `seed ${i}`, rationale: 'race fixture seed', scope: 'race-seed' });
    seededIds.push(event.id);
  }
  // Backdate the seeds directly in the store (logDecision always stamps
  // "now" — race fixture needs deterministic old dates instead).
  const activePath = path.join(root, '.bee', 'decisions.jsonl');
  const seeded = readJsonl(activePath).map((event) =>
    seededIds.includes(event.id) ? { ...event, date: OLD_DATE } : event,
  );
  fs.writeFileSync(activePath, `${seeded.map((e) => JSON.stringify(e)).join('\n')}\n`, 'utf8');

  const LOGGERS = 3;
  const goFile = path.join(root, 'go');
  const stopFile = path.join(root, 'stop');
  const workers = [];
  for (let w = 0; w < LOGGERS; w += 1) {
    workers.push(startRacer({ kind: 'log', root, goFile, stopFile, workerIndex: w }));
  }
  const archiver = startRacer({ kind: 'archive', root, goFile, stopFile, before: CUTOFF });

  const waits = Promise.all([...workers.map(waitWorker), waitWorker(archiver)]);
  await sleep(120); // scheduling nudge only — the goFile barrier is the correctness mechanism
  fs.writeFileSync(goFile, '1');
  await sleep(400); // race window
  fs.writeFileSync(stopFile, '1');
  const [loggerResults, archiverResult] = await waits.then((all) => [all.slice(0, LOGGERS), all[LOGGERS]]);

  const loggedIds = [];
  for (const result of loggerResults) {
    if (result.code !== 0) {
      failures.push(`logger failed: ${result.stderr || result.stdout}`);
      continue;
    }
    for (const line of result.stdout.split('\n')) {
      if (line.startsWith('ID ')) loggedIds.push(line.slice(3).trim());
    }
  }
  if (archiverResult.code !== 0) {
    failures.push(`archiver failed: ${archiverResult.stderr || archiverResult.stdout}`);
  }

  // Drain any events still trapped between the loggers stopping and the
  // archiver's last pass — run one final archive pass so the assertions
  // below see a settled store (a NOTHING_QUALIFIES throw here is fine).
  try {
    archiveDecisions(root, { before: CUTOFF });
  } catch {
    /* nothing left, or nothing new since the last sweep — both fine */
  }

  const archivePath = path.join(root, '.bee', 'decisions-archive.jsonl');
  const activeEvents = readJsonl(activePath);
  const archiveEvents = fs.existsSync(archivePath) ? readJsonl(archivePath) : [];
  const activeIds = new Set(activeEvents.map((e) => e.id));
  const archiveIds = new Set(archiveEvents.map((e) => e.id));

  const expectedIds = [...seededIds, ...loggedIds];
  const missing = expectedIds.filter((id) => !activeIds.has(id) && !archiveIds.has(id));
  if (missing.length) {
    failures.push(`${missing.length} id(s) lost from both files entirely: ${missing.slice(0, 5).join(', ')}`);
  }
  const duplicated = expectedIds.filter((id) => activeIds.has(id) && archiveIds.has(id));
  if (duplicated.length) {
    failures.push(`${duplicated.length} id(s) present in BOTH files under a non-crash run: ${duplicated.slice(0, 5).join(', ')}`);
  }
  // Every logged event must have landed in the active file — none should
  // ever be old enough to be swept by this race's CUTOFF.
  const loggedButNotActive = loggedIds.filter((id) => !activeIds.has(id));
  if (loggedButNotActive.length) {
    failures.push(`${loggedButNotActive.length} logged id(s) not in the active file: ${loggedButNotActive.slice(0, 5).join(', ')}`);
  }

  fs.rmSync(root, { recursive: true, force: true });
  if (failures.length) {
    console.log(`FAIL  log-vs-archive: ${failures.join(' | ')}`);
    return false;
  }
  console.log(
    `PASS  log-vs-archive: ${LOGGERS} loggers x ${loggedIds.length} events logged, ${SEED_COUNT} seeded, zero lost, zero cross-file duplicates`,
  );
  return true;
}

async function main() {
  const scenario = process.argv[2];
  const scenarios = { 'log-vs-archive': logVsArchive };
  const fn = scenarios[scenario];
  if (!fn) {
    console.log(`FAIL  unknown scenario "${scenario}" (expected one of ${Object.keys(scenarios).join(', ')})`);
    process.exit(1);
    return;
  }
  try {
    const ok = await fn();
    process.exit(ok ? 0 : 1);
  } catch (error) {
    console.log(`FAIL  ${scenario} threw: ${error && error.stack ? error.stack : error}`);
    process.exit(1);
  }
}
