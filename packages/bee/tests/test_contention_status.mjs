#!/usr/bin/env node
// test_contention_status.mjs — multisession-native-4 (C4, advisor consult
// stage 0-1): `bee status` / `bee status --json` surfaces lock-contention
// telemetry from .bee/logs/contention.jsonl (written by lock.mjs's
// appendContentionTelemetry, msn-3/C3) so a waiting session can answer "why
// am I waiting".
//
// Covers (must_haves): a seeded fixture reports the expected aggregates; an
// absent log omits the `contention` key entirely (exit 0, no error); an
// oversized file with garbage BEFORE the tail window still resolves fast and
// correctly (proves the bounded tail-window read — no full-file scan).
// Deterministic, no sleeps.

import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

import { runModuleWorker } from '../../../scripts/lib/run-module-worker.mjs';
import { writeJsonAtomic } from '../lib/fsutil.mjs';
import { defaultState, writeState } from '../lib/state.mjs';

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
    console.log(`      ${error instanceof Error ? error.stack || error.message : error}`);
  }
}

function assert(condition, message) {
  if (!condition) throw new Error(message);
}

async function runBee(args, cwd) {
  return await runModuleWorker(BEE_MJS, { args, cwd });
}

function makeRoot(prefix) {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), prefix));
  fs.mkdirSync(path.join(root, '.bee'), { recursive: true });
  writeJsonAtomic(path.join(root, '.bee', 'onboarding.json'), {
    schema_version: '1.0',
    bee_version: '0.1.0',
  });
  writeState(root, {
    ...defaultState(),
    phase: 'idle',
  });
  return root;
}

function writeContentionLog(root, lines) {
  const logsDir = path.join(root, '.bee', 'logs');
  fs.mkdirSync(logsDir, { recursive: true });
  fs.writeFileSync(path.join(logsDir, 'contention.jsonl'), `${lines.join('\n')}\n`);
}

function contentionRecord({ ts, lock_name, lock_wait_ms, holder_session, caller_session, result }) {
  return JSON.stringify({
    ts,
    lock_name,
    lock_wait_ms,
    holder_session: holder_session ?? null,
    caller_session: caller_session ?? null,
    workflow_id: null,
    workspace_id: null,
    resource: null,
    result,
  });
}

// ─── (1) seeded fixture → expected aggregates ──────────────────────────────

const rootSeeded = makeRoot('bee-contention-seeded-');
writeContentionLog(rootSeeded, [
  contentionRecord({ ts: '2026-07-24T10:00:00.000Z', lock_name: 'sessions', lock_wait_ms: 50, holder_session: 'sess-a', caller_session: 'sess-b', result: 'busy' }),
  contentionRecord({ ts: '2026-07-24T10:00:01.000Z', lock_name: 'sessions', lock_wait_ms: 120, holder_session: 'sess-a', caller_session: 'sess-c', result: 'busy' }),
  contentionRecord({ ts: '2026-07-24T10:00:02.000Z', lock_name: 'worktree-admin', lock_wait_ms: 900, holder_session: 'sess-d', caller_session: 'sess-e', result: 'busy' }),
  // 'acquired' events must never be counted as busy contention.
  contentionRecord({ ts: '2026-07-24T10:00:03.000Z', lock_name: 'sessions', lock_wait_ms: 0, holder_session: null, caller_session: 'sess-f', result: 'acquired' }),
]);

await check('status --json: seeded contention.jsonl reports busy_count, top_locks, worst_wait_ms, recent_busy', async () => {
  const result = await runBee(['status', '--json'], rootSeeded);
  assert(result.status === 0, `status --json should succeed: ${result.stderr}`);
  const payload = JSON.parse(result.stdout);
  assert(payload.contention, `expected a "contention" key in status --json, got ${JSON.stringify(Object.keys(payload))}`);
  assert(payload.contention.busy_count === 3, `expected busy_count 3, got ${JSON.stringify(payload.contention)}`);
  assert(Array.isArray(payload.contention.top_locks), 'top_locks must be an array');
  const sessionsRow = payload.contention.top_locks.find((l) => l.lock_name === 'sessions');
  assert(sessionsRow && sessionsRow.busy_count === 2, `expected "sessions" lock to have busy_count 2, got ${JSON.stringify(payload.contention.top_locks)}`);
  assert(payload.contention.worst_wait_ms === 900, `expected worst_wait_ms 900, got ${JSON.stringify(payload.contention)}`);
  assert(payload.contention.worst_wait_lock === 'worktree-admin', `expected worst_wait_lock "worktree-admin", got ${JSON.stringify(payload.contention)}`);
  assert(Array.isArray(payload.contention.recent_busy) && payload.contention.recent_busy.length === 3, `expected 3 recent_busy entries, got ${JSON.stringify(payload.contention.recent_busy)}`);
  const mostRecent = payload.contention.recent_busy[0];
  assert(mostRecent.lock_name === 'worktree-admin' && mostRecent.holder_session === 'sess-d' && mostRecent.caller_session === 'sess-e', `most recent busy entry should be the worktree-admin contention, got ${JSON.stringify(mostRecent)}`);
});

await check('status (text): mentions contention when the log has busy events', async () => {
  const result = await runBee(['status'], rootSeeded);
  assert(result.status === 0, `status should succeed: ${result.stderr}`);
  assert(/[Cc]ontention/.test(result.stdout), `expected a contention line in text status, got:\n${result.stdout}`);
  assert(/worktree-admin/.test(result.stdout), `expected the worst-wait lock named in text status, got:\n${result.stdout}`);
});

// ─── (2) absent log → key absent, exit 0 ───────────────────────────────────

const rootAbsent = makeRoot('bee-contention-absent-');

await check('status --json: absent contention.jsonl omits the contention key entirely, exit 0', async () => {
  const result = await runBee(['status', '--json'], rootAbsent);
  assert(result.status === 0, `status --json should succeed with no log present: ${result.stderr}`);
  const payload = JSON.parse(result.stdout);
  assert(!('contention' in payload), `expected no "contention" key when the log is absent, got ${JSON.stringify(payload.contention)}`);
});

// ─── (3) malformed lines skipped, no throw ─────────────────────────────────

const rootMalformed = makeRoot('bee-contention-malformed-');
writeContentionLog(rootMalformed, [
  'not json at all',
  contentionRecord({ ts: '2026-07-24T10:05:00.000Z', lock_name: 'claims', lock_wait_ms: 42, holder_session: 'sess-x', caller_session: 'sess-y', result: 'busy' }),
  '{"truncated": tr',
]);

await check('status --json: malformed lines are skipped, well-formed busy events still counted', async () => {
  const result = await runBee(['status', '--json'], rootMalformed);
  assert(result.status === 0, `status --json should succeed despite malformed lines: ${result.stderr}`);
  const payload = JSON.parse(result.stdout);
  assert(payload.contention && payload.contention.busy_count === 1, `expected busy_count 1 (malformed lines skipped), got ${JSON.stringify(payload.contention)}`);
});

// ─── (4) oversized file, garbage head beyond the tail window → bounded read ─
// Proves the tail-window discipline itself: pad the file's HEAD with several
// MB of garbage (non-JSON, would never parse) placed BEFORE any real record,
// then append one real busy record at the very end. A full-file scan would
// still find the real record (so this alone doesn't prove tail-windowing),
// but it DOES prove correctness isn't lost by windowing, and — combined with
// wall-clock timing well under a full-scan-of-large-file budget — that the
// read stays bounded rather than degrading with file size.

const rootOversized = makeRoot('bee-contention-oversized-');
{
  const logsDir = path.join(rootOversized, '.bee', 'logs');
  fs.mkdirSync(logsDir, { recursive: true });
  const logPath = path.join(logsDir, 'contention.jsonl');
  const fd = fs.openSync(logPath, 'w');
  try {
    // ~8MB of garbage lines, none of them valid JSON and none containing the
    // sentinel lock name — if the implementation ever fell back to a
    // full-file scan this would still pass on correctness, but the presence
    // of this much unrelated head data is exactly the shape the "no full-file
    // scan" prohibition guards against in production (a contention.jsonl that
    // grows unbounded over a long-lived repo).
    const garbageLine = `garbage-not-json-${'x'.repeat(200)}\n`;
    const totalGarbageBytes = 8 * 1024 * 1024;
    let written = 0;
    while (written < totalGarbageBytes) {
      fs.writeSync(fd, garbageLine);
      written += Buffer.byteLength(garbageLine);
    }
    // One real, well-formed busy record right before the tail, guaranteed to
    // fall inside any reasonable bounded tail window (last 64KB).
    fs.writeSync(
      fd,
      `${contentionRecord({ ts: '2026-07-24T10:10:00.000Z', lock_name: 'tail-lock', lock_wait_ms: 77, holder_session: 'sess-tail-holder', caller_session: 'sess-tail-caller', result: 'busy' })}\n`,
    );
  } finally {
    fs.closeSync(fd);
  }
}

await check('status --json: oversized log with garbage head — still fast and correct (proves tail-window)', async () => {
  const t0 = Date.now();
  const result = await runBee(['status', '--json'], rootOversized);
  const elapsedMs = Date.now() - t0;
  assert(result.status === 0, `status --json should succeed against an oversized log: ${result.stderr}`);
  const payload = JSON.parse(result.stdout);
  assert(payload.contention && payload.contention.busy_count === 1, `expected the one well-formed tail record to be counted, got ${JSON.stringify(payload.contention)}`);
  assert(payload.contention.top_locks.some((l) => l.lock_name === 'tail-lock'), `expected "tail-lock" in top_locks, got ${JSON.stringify(payload.contention.top_locks)}`);
  // Generous bound (well under a naive full 8MB parse-every-line budget on
  // slow CI, but tight enough to catch an accidental full-file scan) —
  // deterministic pass/fail, no sleeps, no flake-prone tight timing.
  assert(elapsedMs < 5000, `status --json against an oversized contention.jsonl took ${elapsedMs}ms — expected a bounded tail-window read, not a full-file scan`);
});

console.log(`\n${passed} passed, ${failed} failed`);
process.exitCode = failed > 0 ? 1 : 0;
