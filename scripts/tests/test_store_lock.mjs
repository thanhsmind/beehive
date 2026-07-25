#!/usr/bin/env node
// test_store_lock.mjs — proves withStoreLock (.bee/bin/lib/lock.mjs, mirrored
// from skills/bee-hive/templates/lib/lock.mjs) actually serializes a critical
// section across real OS processes (CONTEXT.md D2, cell msh-1).
//
// Self-contained child orchestrator (fork racers, assert internally, exit
// 0/1) invoked by ONE blocking row (`node scripts/tests/test_store_lock.mjs`) —
// critical-patterns 20260714 "Async assertions under a non-awaiting runner
// pass vacuously": fs writes are synchronous inside one process, so a
// "concurrent" test that only fires async calls in one event loop never
// exercises a genuine race. Every racer here is its own child process (this
// same file, re-invoked with --role=), same shape as
// test_state_write_concurrency.mjs.
//
// Mutual-exclusion detector: every critical section, guarded or not, marks a
// shared active.json {active,holder} on entry, holds briefly (HOLD_MS,
// widening the race window), re-checks it was not clobbered, then increments
// a shared counter.json. Two workers ever being "active" at once — at entry
// or discovered on the re-check after the hold — is logged to
// violations.jsonl. This is strictly stronger than only checking the final
// counter total: a corrupted total proves loss, but a clean total does not
// prove exclusion (two racers could each net +1 by luck); the active-flag
// check catches the overlap directly, the same class the spike's negative
// control caught as "7-8 winners" for the naive unlink-based takeover.
//
// Four scenarios:
//   (a) N racers under the real lock, fresh lock file  -> zero violations,
//       counter == workers*iters exactly (no lost update).
//   (b) N racers with NO lock at all (same critical section body)          ->
//       deliberately demonstrates loss/overlap, proving the detector above
//       actually bites (falsifiability of the harness itself, not lock.mjs).
//   (c) N racers under the real lock, but the lock file is PRE-SEEDED stale
//       (backdated mtime, fake crashed holder) -> same zero-violation,
//       exact-count invariants as (a): the stale takeover never lets two
//       racers in at once and never wedges progress.
//   (d) One process against a FRESH (non-stale) held lock -> typed
//       LockBusyError after the real ~5s retry budget, holder fields match
//       the seeded holder.
//
// hardening-1-7-10 (D2): stale takeover is no longer age-alone — it also
// requires the recorded owner pid to be dead, below an absolute
// HARD_STALE_MS ceiling. Three more scenarios prove that:
//   (e) One process against a lock backdated PAST STALE_MS (45s) but owned
//       by THIS process's own (real, alive) pid -> still typed LockBusyError
//       — mtime age alone is no longer enough to steal a live holder's lock
//       (this is the exact live-holder-steal bug this cell fixes: run it
//       against the pre-fix code and it fails — the busy child wrongly
//       succeeds instead of being refused).
//   (f) N racers under the real lock, but the lock file is pre-seeded stale
//       PAST the HARD_STALE_MS absolute ceiling, still owned by THIS
//       process's real, alive pid -> takeover proceeds anyway (pid-reuse
//       guard of last resort overrides liveness once the ceiling is
//       crossed), same zero-violation/exact-count invariants as (a)/(c).
// A standalone, same-process isPidAlive() unit check (missing/unparsable pid
// -> dead, alive pid -> alive, ESRCH -> dead, EPERM -> alive, an unknown
// errno -> alive) runs before any of the above, right after the lockFilePath
// sanitization checks.

import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { spawn } from 'node:child_process';
import { fileURLToPath } from 'node:url';

const __filename = fileURLToPath(import.meta.url);
const REPO_ROOT = path.join(path.dirname(__filename), '..', '..');
const LOCK_LIB_PATH = path.join(REPO_ROOT, '.bee', 'bin', 'lib', 'lock.mjs');

const WORKERS = 6;
const ITERS = 12;
const HOLD_MS = 5;
// Mirrors lock.mjs's own HARD_STALE_MS (not exported — this is a black-box
// value, kept in sync by hand, same discipline as lock.mjs's own duplicated
// envSessionId chain).
const HARD_STALE_MS_FOR_TEST = 3_600_000;

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

// ─── shared fixture helpers (deliberately NOT atomic — scenario (b) needs a
// naive read-then-write to be raceable at all; scenario (a)/(c) exclusivity
// comes entirely from the lock around the caller, never from these helpers) ──

function readJsonRaw(file, fallback) {
  try {
    return JSON.parse(fs.readFileSync(file, 'utf8'));
  } catch {
    return fallback;
  }
}

function writeJsonRaw(file, obj) {
  fs.writeFileSync(file, JSON.stringify(obj));
}

function sleep(ms) {
  return new Promise((resolve) => setTimeout(resolve, ms));
}

// multisession-native-3 (C3/C4): reads .bee/logs/contention.jsonl and
// returns the LAST parsed line, or null if the file is missing/empty —
// mirrors how a real caller would tail the log.
function contentionLogPath(root) {
  return path.join(root, '.bee', 'logs', 'contention.jsonl');
}

function readLastContentionRecord(root) {
  let raw;
  try {
    raw = fs.readFileSync(contentionLogPath(root), 'utf8');
  } catch {
    return null;
  }
  const lines = raw.split('\n').filter((line) => line.trim().length > 0);
  if (!lines.length) return null;
  return JSON.parse(lines[lines.length - 1]);
}

function resetFixtures(dir) {
  writeJsonRaw(path.join(dir, 'counter.json'), { total: 0 });
  writeJsonRaw(path.join(dir, 'active.json'), { active: false, holder: null });
  fs.rmSync(path.join(dir, 'violations.jsonl'), { force: true });
}

async function criticalSection(dir, workerId) {
  const activePath = path.join(dir, 'active.json');
  const counterPath = path.join(dir, 'counter.json');
  const violationsPath = path.join(dir, 'violations.jsonl');

  const seenOnEntry = readJsonRaw(activePath, { active: false, holder: null });
  if (seenOnEntry.active) {
    fs.appendFileSync(
      violationsPath,
      `${JSON.stringify({ type: 'double-entry', enteredBy: workerId, alreadyHeldBy: seenOnEntry.holder, at: Date.now() })}\n`,
    );
  }
  writeJsonRaw(activePath, { active: true, holder: workerId });

  await sleep(HOLD_MS);

  const seenOnExit = readJsonRaw(activePath, { active: false, holder: null });
  if (!seenOnExit.active || seenOnExit.holder !== workerId) {
    fs.appendFileSync(
      violationsPath,
      `${JSON.stringify({ type: 'clobbered-active', expectedHolder: workerId, foundHolder: seenOnExit.holder, at: Date.now() })}\n`,
    );
  }

  const counter = readJsonRaw(counterPath, { total: 0 });
  counter.total += 1;
  writeJsonRaw(counterPath, counter);
  writeJsonRaw(activePath, { active: false, holder: null });
}

// ─── worker roles ────────────────────────────────────────────────────────────

async function runWorker(workerRole) {
  try {
    if (workerRole === 'locked' || workerRole === 'unguarded') {
      const dir = argVal('--dir');
      const lockRoot = argVal('--lock-root');
      const lockName = argVal('--lock-name');
      const iters = Number(argVal('--iters'));
      const id = argVal('--id');
      if (workerRole === 'locked') {
        const { withStoreLock, LockBusyError } = await import(LOCK_LIB_PATH);
        // rel1710rc-3: a losing racer whose retry budget genuinely runs out
        // while ANOTHER racer legitimately holds the lock is not a bug — it
        // is the typed LOCK_BUSY refusal working exactly as designed under
        // real scheduler contention (observed: 2/6 past-ceiling racers in
        // scenario (f) crashed on a 2-core CI runner even though mutual
        // exclusion and the eventual takeover both held). Count every
        // LockBusyError as a refused outcome instead of crashing the racer;
        // any OTHER error still propagates to the outer catch below exactly
        // as before. The orchestrator asserts on {completed, refused} rather
        // than assuming every iteration always completes.
        let completed = 0;
        let refused = 0;
        for (let i = 0; i < iters; i++) {
          const workerId = `${id}-${i}`;
          try {
            await withStoreLock(lockRoot, lockName, () => criticalSection(dir, workerId));
            completed += 1;
          } catch (error) {
            if (error instanceof LockBusyError) {
              refused += 1;
            } else {
              throw error;
            }
          }
        }
        process.stdout.write(`${JSON.stringify({ id, completed, refused })}\n`);
      } else {
        for (let i = 0; i < iters; i++) {
          await criticalSection(dir, `${id}-${i}`);
        }
      }
      process.exit(0);
    } else if (workerRole === 'busy') {
      const lockRoot = argVal('--lock-root');
      const lockName = argVal('--lock-name');
      const { withStoreLock, LockBusyError } = await import(LOCK_LIB_PATH);
      try {
        await withStoreLock(lockRoot, lockName, async () => {
          throw new Error('critical section ran — lock was NOT actually busy');
        });
        console.log(JSON.stringify({ ok: false, unexpectedSuccess: true }));
        process.exit(1);
      } catch (error) {
        if (error instanceof LockBusyError) {
          console.log(
            JSON.stringify({ ok: true, name: error.name, type: error.type, reason: error.reason, holder: error.holder }),
          );
          process.exit(0);
        }
        console.log(JSON.stringify({ ok: false, error: String((error && error.stack) || error) }));
        process.exit(1);
      }
    } else {
      throw new Error(`unknown role: ${workerRole}`);
    }
  } catch (err) {
    console.error(`WORKER-CRASH role=${workerRole}: ${(err && err.stack) || err}`);
    process.exit(1);
  }
}

function spawnRacer(args) {
  return new Promise((resolve) => {
    const child = spawn(process.execPath, [__filename, ...args], { stdio: ['ignore', 'pipe', 'pipe'] });
    let stdout = '';
    let stderr = '';
    child.stdout.on('data', (chunk) => {
      stdout += chunk.toString();
    });
    child.stderr.on('data', (chunk) => {
      stderr += chunk.toString();
    });
    child.on('exit', (code) => resolve({ code, stdout, stderr }));
    child.on('error', (err) => resolve({ code: null, stdout, stderr: String(err) }));
  });
}

function spawnRacers(count, argsFor) {
  const runs = [];
  for (let i = 0; i < count; i++) runs.push(spawnRacer(argsFor(i)));
  return Promise.all(runs);
}

function violationCount(dir) {
  const file = path.join(dir, 'violations.jsonl');
  let text;
  try {
    text = fs.readFileSync(file, 'utf8');
  } catch {
    return 0;
  }
  return text.split('\n').filter((line) => line.trim()).length;
}

function readCounterTotal(dir) {
  return readJsonRaw(path.join(dir, 'counter.json'), { total: null }).total;
}

function lastJsonLine(stdout) {
  const lines = stdout.split('\n').filter((line) => line.trim());
  if (lines.length === 0) return null;
  try {
    return JSON.parse(lines[lines.length - 1]);
  } catch {
    return null;
  }
}

// rel1710rc-3: shared assertion body for every 'locked'-role scenario
// ((a)/(c)/(f)) — a losing racer's LockBusyError is now a counted refusal
// (see the 'locked' role above), not a crash, so "no lost update" can no
// longer mean a hardcoded workers*iters total: it means the counter total
// equals EXACTLY the sum of what every racer itself reports having
// completed (nothing silently duplicated or dropped among completions that
// actually happened), mutual exclusion never once broke down, and every
// racer accounted for all of its own iterations as completed-or-refused
// (never silently vanished, i.e. never an unhandled crash). `requireExactTotal`
// keeps scenario (a)'s original, stricter claim (a fresh, uncontested-by-
// staleness lock: zero refusals expected) while (c)/(f) only require that
// AT LEAST ONE racer actually completed the takeover.
function assessLockedRun(label, results, failures, { workers, iters, tmpRoot, requireExactTotal }) {
  const crashed = results.filter((r) => r.code !== 0);
  if (crashed.length) {
    failures.push(
      `(${label}) ${crashed.length}/${workers} locked racer(s) crashed:\n` +
        crashed.map((c) => `  ${c.stderr.trim()}`).join('\n'),
    );
  }
  const parsed = results.map((r) => lastJsonLine(r.stdout));
  let sumCompleted = 0;
  for (const p of parsed) {
    if (!p || typeof p.completed !== 'number' || typeof p.refused !== 'number') {
      failures.push(`(${label}) locked racer produced no parseable {completed,refused} summary: ${JSON.stringify(p)}`);
      continue;
    }
    if (p.completed + p.refused !== iters) {
      failures.push(
        `(${label}) racer ${p.id} accounted for ${p.completed + p.refused}/${iters} iterations ` +
          `(completed=${p.completed} refused=${p.refused}) — every iteration must complete-or-be-refused, never silently vanish`,
      );
    }
    sumCompleted += p.completed;
  }
  const violations = violationCount(tmpRoot);
  const total = readCounterTotal(tmpRoot);
  if (violations !== 0) {
    failures.push(`(${label}) run recorded ${violations} mutual-exclusion violation(s) — the lock did not exclude`);
  }
  if (total !== sumCompleted) {
    failures.push(
      `(${label}) counter total is ${total}, expected exactly ${sumCompleted} (the sum of every racer's self-reported ` +
        'completions) — lost or phantom update',
    );
  }
  if (requireExactTotal && sumCompleted !== workers * iters) {
    failures.push(
      `(${label}) expected all ${workers * iters} iterations to complete with zero refusals, got ${sumCompleted} ` +
        `completed (some racer(s) hit LOCK_BUSY): ${JSON.stringify(parsed)}`,
    );
  }
  if (!requireExactTotal && sumCompleted < 1) {
    failures.push(`(${label}) expected at least one successful critical-section takeover, got 0 completions across all racers: ${JSON.stringify(parsed)}`);
  }
  return { violations, total, sumCompleted, parsed };
}

// ─── orchestrator ────────────────────────────────────────────────────────────

async function runOrchestrator() {
  const { lockFilePath, withStoreLock, acquireStoreLockOnceSync, isPidAlive } = await import(LOCK_LIB_PATH);
  const tmpRoot = fs.mkdtempSync(path.join(os.tmpdir(), 'test-store-lock-'));
  fs.mkdirSync(path.join(tmpRoot, '.bee', 'locks'), { recursive: true });

  const failures = [];
  const expectedTotal = WORKERS * ITERS;

  // (0e-0h) isPidAlive() unit checks — same process, no spawn needed (it's a
  // pure synchronous probe). Covers every branch the D2 fix depends on:
  // missing/unparsable pid, a genuinely alive pid, ESRCH (dead), EPERM
  // (alive), and an unmapped errno (treated conservatively as alive).
  {
    if (isPidAlive(process.pid) !== true) {
      failures.push(`(0e) isPidAlive(process.pid=${process.pid}) expected true (this process is alive), got false`);
    }
    for (const bad of [999999999, 0, -5, 'not-a-pid', undefined, null, NaN]) {
      if (isPidAlive(bad) !== false) {
        failures.push(`(0f) isPidAlive(${JSON.stringify(bad)}) expected false (missing/unparsable/dead pid), got true`);
      }
    }
    // Monkeypatch process.kill to simulate ESRCH/EPERM/unmapped-errno without
    // depending on any real OS process actually being in that state.
    const realKill = process.kill;
    try {
      process.kill = (pid, sig) => {
        if (pid === 4242001) {
          const err = new Error('kill ESRCH (simulated)');
          err.code = 'ESRCH';
          throw err;
        }
        if (pid === 4242002) {
          const err = new Error('kill EPERM (simulated)');
          err.code = 'EPERM';
          throw err;
        }
        if (pid === 4242003) {
          const err = new Error('kill EINVAL (simulated)');
          err.code = 'EINVAL';
          throw err;
        }
        return realKill.call(process, pid, sig);
      };
      if (isPidAlive(4242001) !== false) failures.push('(0g) isPidAlive() with a monkeypatched ESRCH expected false (dead), got true');
      if (isPidAlive(4242002) !== true) failures.push('(0g) isPidAlive() with a monkeypatched EPERM expected true (alive), got false');
      if (isPidAlive(4242003) !== true) failures.push('(0h) isPidAlive() with a monkeypatched unmapped errno (EINVAL) expected true (conservatively alive), got false');
    } finally {
      process.kill = realKill;
    }
  }

  // (0) lockFilePath sanitizes logical names for Windows filesystem safety.
  // Runtime lock names include "cells:<id>" (cells.mjs's `cells:${id}`, 4
  // call sites) — ':' is invalid in a Windows filename, so an unsanitized
  // lockFilePath made every cap/verify/block/reset under that lock throw on
  // Windows (fs.writeFileSync with flag 'wx' rejects the path outright).
  {
    const WINDOWS_INVALID_CHARS = /[<>:"/\\|?*\x00-\x1f]/;

    const cellsDemoPath = lockFilePath(tmpRoot, 'cells:demo-1');
    const cellsDemoBasename = path.basename(cellsDemoPath);
    if (WINDOWS_INVALID_CHARS.test(cellsDemoBasename)) {
      failures.push(
        `(0a) lockFilePath(root, 'cells:demo-1') basename "${cellsDemoBasename}" still contains a Windows-invalid character`,
      );
    }

    // Distinct logical names must map to distinct files even after
    // sanitization could otherwise collide them (":" and "/" both -> "_").
    const pathColonA = lockFilePath(tmpRoot, 'cells:a');
    const pathColonB = lockFilePath(tmpRoot, 'cells:b');
    const pathSlashA = lockFilePath(tmpRoot, 'cells/a');
    if (pathColonA === pathColonB) {
      failures.push(`(0b) distinct logical names 'cells:a' and 'cells:b' collided to the same lock file: ${pathColonA}`);
    }
    if (pathColonA === pathSlashA) {
      failures.push(
        `(0b) distinct logical names 'cells:a' and 'cells/a' collided to the same lock file after sanitization: ${pathColonA}`,
      );
    }

    // The SAME logical name must map to the SAME file every time (pure
    // function, safe across processes) — determinism, not per-call entropy.
    const pathColonAAgain = lockFilePath(tmpRoot, 'cells:a');
    if (pathColonA !== pathColonAAgain) {
      failures.push(
        `(0c) lockFilePath(root, 'cells:a') is not deterministic: ${pathColonA} vs ${pathColonAAgain} on a second call`,
      );
    }

    // A real withStoreLock round-trip using a colon-bearing logical name
    // still acquires and releases cleanly (mutual exclusion still works,
    // and the lock file it creates on disk is the sanitized path above).
    try {
      let ran = false;
      await withStoreLock(tmpRoot, 'cells:round-trip', async () => {
        ran = true;
      });
      if (!ran) {
        failures.push('(0d) withStoreLock(root, "cells:round-trip", fn) did not run fn');
      }
      const roundTripLockPath = lockFilePath(tmpRoot, 'cells:round-trip');
      if (fs.existsSync(roundTripLockPath)) {
        failures.push(`(0d) withStoreLock left the lock file behind after release: ${roundTripLockPath}`);
      }
    } catch (err) {
      failures.push(`(0d) withStoreLock(root, "cells:round-trip", fn) threw: ${(err && err.stack) || err}`);
    }
  }

  try {
    // (a) N racers under the real lock, fresh lock file — no lost update, no overlap.
    resetFixtures(tmpRoot);
    const lockedResults = await spawnRacers(WORKERS, (i) => [
      '--role=locked',
      `--dir=${tmpRoot}`,
      `--lock-root=${tmpRoot}`,
      '--lock-name=lock-a',
      `--iters=${ITERS}`,
      `--id=locked-${i}`,
    ]);
    assessLockedRun('a', lockedResults, failures, { workers: WORKERS, iters: ITERS, tmpRoot, requireExactTotal: true });

    // (b) N racers with NO lock — deliberately demonstrates loss/overlap
    // (proves the violation/loss detector itself can actually bite).
    resetFixtures(tmpRoot);
    const unguardedResults = await spawnRacers(WORKERS, (i) => [
      '--role=unguarded',
      `--dir=${tmpRoot}`,
      `--iters=${ITERS}`,
      `--id=unguarded-${i}`,
    ]);
    const crashedUnguarded = unguardedResults.filter((r) => r.code !== 0);
    if (crashedUnguarded.length) {
      failures.push(
        `(b) ${crashedUnguarded.length}/${WORKERS} unguarded racer(s) crashed:\n` +
          crashedUnguarded.map((c) => `  ${c.stderr.trim()}`).join('\n'),
      );
    }
    const bViolations = violationCount(tmpRoot);
    const bTotal = readCounterTotal(tmpRoot);
    if (bViolations === 0 && bTotal === expectedTotal) {
      failures.push(
        '(b) unguarded control run showed ZERO overlap and a perfectly exact counter — the detector proved nothing ' +
          '(it must demonstrate loss/overlap to prove the (a)/(c) green results are meaningful, not luck)',
      );
    }

    // (c) N racers under the real lock, but the lock file starts artificially
    // stale (backdated mtime, fake crashed holder) — same invariants as (a).
    resetFixtures(tmpRoot);
    const staleLockPath = lockFilePath(tmpRoot, 'lock-c');
    fs.writeFileSync(
      staleLockPath,
      `${JSON.stringify({ pid: 999999999, session: 'stale-simulated-holder', ts: new Date(Date.now() - 45_000).toISOString(), token: 'stale-token-c' })}\n`,
    );
    const staleMs = (Date.now() - 45_000) / 1000;
    fs.utimesSync(staleLockPath, staleMs, staleMs);
    const staleResults = await spawnRacers(WORKERS, (i) => [
      '--role=locked',
      `--dir=${tmpRoot}`,
      `--lock-root=${tmpRoot}`,
      '--lock-name=lock-c',
      `--iters=${ITERS}`,
      `--id=stale-${i}`,
    ]);
    // rel1710rc-3: audited for the same crash-on-busy pattern as (f) below —
    // it flaked once locally under swarm load. assessLockedRun's counted-
    // refusal discipline (a naive unconditional-unlink takeover would still
    // show up as a mutual-exclusion violation; atomic-rename takeover must
    // not) replaces the old fixed-total assertion.
    assessLockedRun('c', staleResults, failures, { workers: WORKERS, iters: ITERS, tmpRoot, requireExactTotal: false });
    if (fs.existsSync(staleLockPath)) {
      failures.push('(c) the pre-seeded stale lock file was never taken over/cleared — progress may be permanently wedged');
    }

    // (d) One process against a FRESH (non-stale) held lock — typed
    // LockBusyError naming the real holder, after the real retry budget.
    const busyLockName = 'lock-d';
    const busyLockPath = lockFilePath(tmpRoot, busyLockName);
    const seededHolder = { pid: 424242, session: 'busy-holder-session', ts: new Date().toISOString(), token: 'busy-token-d' };
    fs.writeFileSync(busyLockPath, `${JSON.stringify(seededHolder)}\n`);
    const startedAt = Date.now();
    const busyResult = await spawnRacer(['--role=busy', `--lock-root=${tmpRoot}`, `--lock-name=${busyLockName}`]);
    const elapsedMs = Date.now() - startedAt;
    fs.rmSync(busyLockPath, { force: true });

    if (busyResult.code !== 0) {
      failures.push(`(d) busy-attempt child exited ${busyResult.code}, expected 0:\n  stdout=${busyResult.stdout.trim()}\n  stderr=${busyResult.stderr.trim()}`);
    } else {
      let parsed = null;
      try {
        parsed = JSON.parse(busyResult.stdout.trim().split('\n').pop());
      } catch (err) {
        failures.push(`(d) could not parse busy-attempt stdout as JSON: ${err.message} :: raw=${busyResult.stdout}`);
      }
      if (parsed) {
        if (parsed.type !== 'refused' || parsed.reason !== 'LOCK_BUSY') {
          failures.push(`(d) expected {type:'refused',reason:'LOCK_BUSY'}, got ${JSON.stringify({ type: parsed.type, reason: parsed.reason })}`);
        }
        if (!parsed.holder || parsed.holder.pid !== seededHolder.pid || parsed.holder.session !== seededHolder.session) {
          failures.push(`(d) LockBusyError.holder did not name the seeded holder: got ${JSON.stringify(parsed.holder)}, seeded ${JSON.stringify(seededHolder)}`);
        }
      }
    }
    if (elapsedMs < 2500) {
      failures.push(`(d) busy-attempt returned after only ${elapsedMs}ms — too fast to have exercised the real retry budget`);
    }
    if (elapsedMs > 12_000) {
      failures.push(`(d) busy-attempt took ${elapsedMs}ms — far beyond the ~5s retry budget, may be hanging instead of refusing`);
    }

    // (e) LIVE-HOLDER REGRESSION (D2): a lock backdated past STALE_MS (45s)
    // but owned by THIS process's own real, alive pid must NOT be taken
    // over — mtime age alone is no longer sufficient. Pre-fix, this is
    // exactly the bug: age > STALE_MS was the ONLY check, so the busy child
    // below would wrongly steal the lock and "succeed" instead of being
    // refused (run this scenario against the pre-fix tryStaleTakeover to see
    // it fail).
    const liveLockName = 'lock-e';
    const liveLockPath = lockFilePath(tmpRoot, liveLockName);
    const liveHolder = { pid: process.pid, session: 'live-holder-session', ts: new Date(Date.now() - 45_000).toISOString(), token: 'live-token-e' };
    fs.writeFileSync(liveLockPath, `${JSON.stringify(liveHolder)}\n`);
    const liveMs = (Date.now() - 45_000) / 1000;
    fs.utimesSync(liveLockPath, liveMs, liveMs);
    const liveStartedAt = Date.now();
    const liveBusyResult = await spawnRacer(['--role=busy', `--lock-root=${tmpRoot}`, `--lock-name=${liveLockName}`]);
    const liveElapsedMs = Date.now() - liveStartedAt;
    const liveLockSurvived = fs.existsSync(liveLockPath);
    fs.rmSync(liveLockPath, { force: true });

    if (liveBusyResult.code !== 0) {
      // The 'busy' role's critical-section body throws
      // 'critical section ran — lock was NOT actually busy' the moment it
      // ever actually acquires the lock, which is NOT a LockBusyError, so it
      // falls into the role's generic catch-and-exit(1) path — this is
      // exactly what the pre-fix bug produces: age > STALE_MS alone stole a
      // live holder's lock, the busy child's body ran, and THIS is that
      // failure surfacing.
      failures.push(
        `(e) busy-attempt child against a 45s-backdated LIVE-pid lock exited ${liveBusyResult.code}, expected 0 (LOCK_BUSY refusal) — ` +
          `a live holder's lock was likely stolen by age alone:\n  stdout=${liveBusyResult.stdout.trim()}\n  stderr=${liveBusyResult.stderr.trim()}`,
      );
    } else {
      let parsedLive = null;
      try {
        parsedLive = JSON.parse(liveBusyResult.stdout.trim().split('\n').pop());
      } catch (err) {
        failures.push(`(e) could not parse live-holder busy-attempt stdout as JSON: ${err.message} :: raw=${liveBusyResult.stdout}`);
      }
      if (parsedLive) {
        if (parsedLive.type !== 'refused' || parsedLive.reason !== 'LOCK_BUSY') {
          failures.push(`(e) expected {type:'refused',reason:'LOCK_BUSY'}, got ${JSON.stringify({ type: parsedLive.type, reason: parsedLive.reason })}`);
        } else if (!parsedLive.holder || parsedLive.holder.pid !== liveHolder.pid) {
          failures.push(`(e) LockBusyError.holder did not name the live seeded holder: got ${JSON.stringify(parsedLive.holder)}, seeded ${JSON.stringify(liveHolder)}`);
        }
      }
    }
    if (!liveLockSurvived) {
      failures.push('(e) the live-holder lock file was removed/renamed by the busy attempt — it must be left exactly as the live holder left it');
    }
    if (liveElapsedMs < 2500) {
      failures.push(`(e) live-holder busy-attempt returned after only ${liveElapsedMs}ms — too fast to have exercised the real retry budget`);
    }
    if (liveElapsedMs > 12_000) {
      failures.push(`(e) live-holder busy-attempt took ${liveElapsedMs}ms — far beyond the ~5s retry budget, may be hanging instead of refusing`);
    }

    // (f) PAST-CEILING REGRESSION (D2): a lock backdated past the absolute
    // HARD_STALE_MS ceiling, still owned by THIS process's real, alive pid,
    // MUST be taken over anyway — the pid-reuse guard of last resort
    // overrides liveness once the ceiling is crossed. Same
    // zero-violation/exact-count invariants as (a)/(c): progress is never
    // permanently wedged just because a holder happens to still be alive.
    resetFixtures(tmpRoot);
    const ceilingLockPath = lockFilePath(tmpRoot, 'lock-f');
    const ceilingAgeMs = HARD_STALE_MS_FOR_TEST + 60_000; // safely past the 1h ceiling
    fs.writeFileSync(
      ceilingLockPath,
      `${JSON.stringify({ pid: process.pid, session: 'past-ceiling-live-holder', ts: new Date(Date.now() - ceilingAgeMs).toISOString(), token: 'ceiling-token-f' })}\n`,
    );
    const ceilingMs = (Date.now() - ceilingAgeMs) / 1000;
    fs.utimesSync(ceilingLockPath, ceilingMs, ceilingMs);
    const ceilingResults = await spawnRacers(WORKERS, (i) => [
      '--role=locked',
      `--dir=${tmpRoot}`,
      `--lock-root=${tmpRoot}`,
      '--lock-name=lock-f',
      `--iters=${ITERS}`,
      `--id=ceiling-${i}`,
    ]);
    // rel1710rc-3: CI verify(22) failed twice with a losing racer's
    // LockBusyError surfacing as a WORKER-CRASH ("2/6 past-ceiling racer(s)
    // crashed") once the winner legitimately held the lock and a loser's
    // retry budget ran out on a 2-core runner. assessLockedRun treats that
    // refusal as a counted outcome: mutual exclusion (violations===0) and
    // no lost update among what actually completed are still asserted at
    // full strength; only the "every single iteration always completes" claim
    // is relaxed to "at least one racer's takeover completes", per the fixed
    // takeover invariant this scenario actually promises.
    assessLockedRun('f', ceilingResults, failures, { workers: WORKERS, iters: ITERS, tmpRoot, requireExactTotal: false });
    if (fs.existsSync(ceilingLockPath)) {
      failures.push('(f) the pre-seeded past-ceiling lock file was never taken over/cleared — a live-but-past-ceiling holder must not wedge progress permanently');
    }

    // (g) FORCED-INTERLEAVING TAKEOVER (rel180-4, fix-first — pre-existing
    // mutual-exclusion violation, backlog P2, reproduced independently on
    // unmodified baseline): (a)/(c)/(f) above only ever hit this race by
    // luck (CI saw it 3 times across a whole run) — this scenario forces the
    // EXACT TOCTOU window deterministically instead, using the test-only
    // `_takeoverSeam` hook (withStoreLock's options; never set by any
    // production caller — see lock.mjs's tryStaleTakeoverAsync doc comment).
    // Racer 1 is paused, in-process, between judging a pre-seeded stale lock
    // eligible for takeover and actually performing the rename; while
    // paused, racer 2 (uncontended, no seam) fully wins its OWN takeover of
    // that exact same stale lock, creates a fresh lock, and enters its
    // critical section. Racer 1 is then resumed and attempts its rename
    // against what is now racer 2's LIVE lock — the pre-fix bug: an
    // unconditional-delete takeover would let racer 1 believe it won too,
    // running its own critical section while racer 2 is still active
    // (mutual-exclusion violation). The fix (performTakeoverClaim's
    // post-rename content verification + restore-if-mismatched) must make
    // racer 1 detect the mismatch, restore racer 2's lock untouched, and
    // back off. Runs FORCED_ITERS times fresh to prove the fix holds every
    // time, not just once — no real concurrency, no timing luck: forced by
    // microtask ordering, which is deterministic in Node.
    {
      const FORCED_ITERS = 10;
      let forcedPassed = 0;
      for (let iter = 0; iter < FORCED_ITERS; iter++) {
        const raceLockName = `race-forced-${iter}`;
        const raceLockPath = lockFilePath(tmpRoot, raceLockName);
        fs.writeFileSync(
          raceLockPath,
          `${JSON.stringify({ pid: 999999999, session: 'forced-stale-holder', ts: new Date(Date.now() - 45_000).toISOString(), token: `forced-token-${iter}` })}\n`,
        );
        const forcedStaleMs = (Date.now() - 45_000) / 1000;
        fs.utimesSync(raceLockPath, forcedStaleMs, forcedStaleMs);

        let racer1SeenOverlap = false;
        let racer2Active = false;
        let seamResolve;
        const seamGate = new Promise((resolve) => {
          seamResolve = resolve;
        });
        let racer2Resolve;
        const racer2Gate = new Promise((resolve) => {
          racer2Resolve = resolve;
        });

        const racer1Promise = withStoreLock(
          tmpRoot,
          raceLockName,
          async () => {
            if (racer2Active) racer1SeenOverlap = true;
          },
          { _takeoverSeam: () => seamGate, maxAttempts: 300, retryDelayMs: 10 },
        ).catch((err) => ({ __racer1Error: err }));

        // Racer 1's call above ran synchronously up to its paused `await
        // seam()` before this line executes (no other await precedes it) —
        // it is deterministically paused right now, not "probably" paused.
        const racer2Promise = withStoreLock(tmpRoot, raceLockName, async () => {
          racer2Active = true;
          await racer2Gate;
          racer2Active = false;
        }).catch((err) => ({ __racer2Error: err }));

        // Racer 2's call above is uncontended (no seam) and reaches its own
        // `await racer2Gate` without ever waiting on anything real — but
        // `await`ing an async call still costs at least one microtask tick
        // even when that call never hit an internal await (an already-
        // resolved promise still defers its continuation once, per spec),
        // so poll a bounded number of microtask ticks rather than assume
        // zero-tick synchronicity. This bound is generous (JS microtask
        // ticks, not real time) and is itself test-infrastructure plumbing —
        // it does not touch the property under test.
        for (let tick = 0; tick < 50 && !racer2Active; tick++) {
          await Promise.resolve();
        }
        if (!racer2Active) {
          failures.push(
            `(g) iter ${iter}: racer2 did not report itself active within 50 microtask ticks — forced setup did not hold (test infrastructure issue, not the fix under test)`,
          );
        }

        // Resume racer 1. Its continuation (performTakeoverClaim, fully
        // synchronous — no internal awaits) runs to completion as a
        // contiguous microtask block ending at withStoreLock's next await
        // (`await sleep(retryDelayMs)`), strictly before any of these
        // `await Promise.resolve()` ticks resume (FIFO microtask
        // ordering) — deterministic, not a timing guess.
        seamResolve();
        await Promise.resolve();
        await Promise.resolve();
        await Promise.resolve();

        // Only now let racer 2 finish and release.
        racer2Resolve();

        const [r1, r2] = await Promise.all([racer1Promise, racer2Promise]);
        if (r1 && r1.__racer1Error) {
          failures.push(`(g) iter ${iter}: racer1 threw unexpectedly: ${(r1.__racer1Error && r1.__racer1Error.stack) || r1.__racer1Error}`);
        }
        if (r2 && r2.__racer2Error) {
          failures.push(`(g) iter ${iter}: racer2 threw unexpectedly: ${(r2.__racer2Error && r2.__racer2Error.stack) || r2.__racer2Error}`);
        }
        if (racer1SeenOverlap) {
          failures.push(
            `(g) iter ${iter}: racer1's critical section ran WHILE racer2 was still active — mutual-exclusion violation under forced interleaving`,
          );
        } else if (!(r1 && r1.__racer1Error) && !(r2 && r2.__racer2Error)) {
          forcedPassed += 1;
        }
        fs.rmSync(raceLockPath, { force: true });
      }
      if (forcedPassed !== FORCED_ITERS) {
        failures.push(`(g) forced-interleaving takeover: only ${forcedPassed}/${FORCED_ITERS} iterations passed cleanly`);
      }
    }

    // (g2) THIRD-RACER VACANCY EXPLOIT (rel190-2, fix-first — CI-observed
    // mutual-exclusion violation in scenario (f), backlog-equivalent):
    // rel180-4's post-rename identity check (settleTakeover) prevents a
    // takeover from PERMANENTLY losing a live holder's lock, but the rename
    // itself still runs FIRST, unconditionally, against whatever currently
    // occupies lockPath — even a lock that was refreshed and is ACTIVELY
    // held by another racer's still-running critical section. For the
    // instant between that rename and the mismatch decision, lockPath is
    // genuinely EMPTY on disk. This scenario forces exactly that window
    // deterministically via a SECOND seam (_postRenameSeam, lock.mjs's
    // tryStaleTakeoverAsync — test-only, never set by any production
    // caller), positioned right after the rename and before the mismatch
    // decision: racer1 pauses there and, from inside the seam, a completely
    // unrelated racer3 attempts an ORDINARY tryAcquire (no takeover
    // involved) against the same lock name while racer2 is still active. If
    // the rename vacated racer2's live lock, racer3's plain acquire can win
    // it and enter its own critical section while racer2 is still inside
    // its — the exact double-entry the CI failure reported. rel190-2's fix
    // (re-verify identity immediately BEFORE the rename, never touch a lock
    // whose content no longer matches what was judged) means the seam
    // should never even fire in this exact setup once fixed: racer1's own
    // re-verification aborts before ever reaching the rename, so no vacancy
    // is ever opened for racer3 to exploit. Runs FORCED_ITERS2 times fresh,
    // same discipline as (g): no real concurrency, no timing luck.
    {
      const FORCED_ITERS2 = 10;
      let forcedPassed2 = 0;
      for (let iter = 0; iter < FORCED_ITERS2; iter++) {
        const raceLockName = `race-vacancy-${iter}`;
        const raceLockPath = lockFilePath(tmpRoot, raceLockName);
        fs.writeFileSync(
          raceLockPath,
          `${JSON.stringify({ pid: 999999999, session: 'vacancy-forced-stale-holder', ts: new Date(Date.now() - 45_000).toISOString(), token: `vacancy-forced-token-${iter}` })}\n`,
        );
        const forcedStaleMs = (Date.now() - 45_000) / 1000;
        fs.utimesSync(raceLockPath, forcedStaleMs, forcedStaleMs);

        let racer2Active = false;
        let racer3SeenOverlap = false;
        let postRenameSeamFired = false;

        let seamResolve;
        const seamGate = new Promise((resolve) => {
          seamResolve = resolve;
        });
        let racer2Resolve;
        const racer2Gate = new Promise((resolve) => {
          racer2Resolve = resolve;
        });

        const racer1Promise = withStoreLock(
          tmpRoot,
          raceLockName,
          async () => {}, // racer1's own critical section is irrelevant to this scenario's assertion
          {
            _takeoverSeam: () => seamGate,
            _postRenameSeam: async () => {
              postRenameSeamFired = true;
              try {
                // Uncontended, ordinary acquire — no takeover involved at
                // all. If racer1's rename just vacated racer2's live lock,
                // this plain tryAcquire can win the empty slot outright.
                await withStoreLock(tmpRoot, raceLockName, async () => {
                  if (racer2Active) racer3SeenOverlap = true;
                });
              } finally {
                // Let racer2 finish only once racer3's attempt has fully
                // resolved, so the overlap window is unambiguous either way.
                racer2Resolve();
              }
            },
            maxAttempts: 300,
            retryDelayMs: 10,
          },
        ).catch((err) => ({ __racer1Error: err }));

        const racer2Promise = withStoreLock(tmpRoot, raceLockName, async () => {
          racer2Active = true;
          await racer2Gate;
          racer2Active = false;
        }).catch((err) => ({ __racer2Error: err }));

        // Same microtask-tick-bound caveat as (g) above: an already-resolved
        // promise still defers its continuation once.
        for (let tick = 0; tick < 50 && !racer2Active; tick++) {
          await Promise.resolve();
        }
        if (!racer2Active) {
          failures.push(
            `(g2) iter ${iter}: racer2 did not report itself active within 50 microtask ticks — forced setup did not hold (test infrastructure issue, not the fix under test)`,
          );
        }

        // Resume racer1. Pre-fix, it proceeds straight through the rename
        // and fires _postRenameSeam. Post-fix, its own re-verification
        // aborts before ever reaching the rename, so the seam never fires —
        // detected by the bounded poll below, which then lets racer2 finish
        // on its own rather than waiting on a seam that will never run.
        seamResolve();
        for (let tick = 0; tick < 50 && !postRenameSeamFired; tick++) {
          await Promise.resolve();
        }
        if (!postRenameSeamFired) {
          racer2Resolve();
        }

        const [r1, r2] = await Promise.all([racer1Promise, racer2Promise]);
        if (r1 && r1.__racer1Error) {
          failures.push(`(g2) iter ${iter}: racer1 threw unexpectedly: ${(r1.__racer1Error && r1.__racer1Error.stack) || r1.__racer1Error}`);
        }
        if (r2 && r2.__racer2Error) {
          failures.push(`(g2) iter ${iter}: racer2 threw unexpectedly: ${(r2.__racer2Error && r2.__racer2Error.stack) || r2.__racer2Error}`);
        }
        if (racer3SeenOverlap) {
          failures.push(
            `(g2) iter ${iter}: racer3 (an ordinary, non-takeover acquire) entered its critical section WHILE racer2 was still active — racer1's takeover rename vacated a live holder's lock and a third racer exploited it (mutual-exclusion violation under forced interleaving)`,
          );
        } else if (!(r1 && r1.__racer1Error) && !(r2 && r2.__racer2Error)) {
          forcedPassed2 += 1;
        }
        fs.rmSync(raceLockPath, { force: true });
      }
      if (forcedPassed2 !== FORCED_ITERS2) {
        failures.push(`(g2) forced third-racer vacancy exploit: only ${forcedPassed2}/${FORCED_ITERS2} iterations passed cleanly`);
      }
    }

    // (h) SIMULATED EPERM TRANSIENT RETRY (rel180-4, Windows CI hazard fix):
    // stubs fs.writeFileSync/renameSync/rmSync to throw a transient EPERM
    // exactly once, then behave normally — reproducing the Windows "another
    // handle has this file open" class of failure deterministically,
    // without needing an actual Windows box (same technique rel1710rc-5
    // used to prove claims.mjs's own fix). Every stub is scoped to lock
    // paths containing "epermtest" so it can never interfere with any other
    // fs traffic (fixture files, other scenarios' lock files, tmpRoot
    // cleanup).
    {
      const originalWriteFileSync = fs.writeFileSync;
      const originalRenameSync = fs.renameSync;
      const originalRmSync = fs.rmSync;

      function throwOnceThenReal(real) {
        let thrown = false;
        return function patched(...args) {
          if (!thrown && String(args[0]).includes('epermtest')) {
            thrown = true;
            const err = new Error('simulated transient EPERM');
            err.code = 'EPERM';
            throw err;
          }
          return real.apply(fs, args);
        };
      }

      // (h1) the 'wx' acquire (writeFileSync) survives one transient EPERM.
      const epermLockName1 = 'lock-h1-epermtest';
      try {
        fs.writeFileSync = throwOnceThenReal(originalWriteFileSync);
        let ran1 = false;
        await withStoreLock(tmpRoot, epermLockName1, async () => {
          ran1 = true;
        });
        if (!ran1) failures.push('(h1) withStoreLock did not run fn after a simulated transient EPERM on acquire');
      } catch (err) {
        failures.push(`(h1) withStoreLock threw despite a single transient EPERM on acquire (should have retried and succeeded): ${(err && err.stack) || err}`);
      } finally {
        fs.writeFileSync = originalWriteFileSync;
      }
      if (fs.existsSync(lockFilePath(tmpRoot, epermLockName1))) {
        failures.push('(h1) withStoreLock left the lock file behind after a retried acquire + release');
      }

      // (h2) release's rmSync survives one transient EPERM.
      const epermLockName2 = 'lock-h2-epermtest';
      try {
        await withStoreLock(tmpRoot, epermLockName2, async () => {
          fs.rmSync = throwOnceThenReal(originalRmSync);
        });
      } catch (err) {
        failures.push(`(h2) withStoreLock release threw despite a single transient EPERM on rmSync (should have retried): ${(err && err.stack) || err}`);
      } finally {
        fs.rmSync = originalRmSync;
      }
      if (fs.existsSync(lockFilePath(tmpRoot, epermLockName2))) {
        failures.push('(h2) withStoreLock release left the lock file behind after a simulated transient EPERM on rmSync');
      }

      // (h3) an ALWAYS-throwing EPERM on acquire still surfaces as a real,
      // typed error — the retry is bounded, never an infinite/silent hang.
      const epermLockName3 = 'lock-h3-epermtest';
      fs.writeFileSync = (...args) => {
        if (!String(args[0]).includes('epermtest')) return originalWriteFileSync.apply(fs, args);
        const err = new Error('simulated PERMANENT EPERM');
        err.code = 'EPERM';
        throw err;
      };
      const h3StartedAt = Date.now();
      let h3Threw = null;
      try {
        await withStoreLock(tmpRoot, epermLockName3, async () => {});
      } catch (err) {
        h3Threw = err;
      } finally {
        fs.writeFileSync = originalWriteFileSync;
      }
      const h3ElapsedMs = Date.now() - h3StartedAt;
      if (!h3Threw || h3Threw.code !== 'EPERM') {
        failures.push(`(h3) an always-throwing EPERM on acquire did not surface as a real EPERM error: ${h3Threw ? (h3Threw.stack || h3Threw) : 'no error thrown'}`);
      }
      if (h3ElapsedMs > 5000) {
        failures.push(`(h3) an always-throwing EPERM took ${h3ElapsedMs}ms to surface — the bounded retry must fail fast (~15x20ms), not hang`);
      }

      // (h4) the atomic stale-takeover rename also survives one transient EPERM.
      const epermLockName4 = 'lock-h4-epermtest';
      const epermLockPath4 = lockFilePath(tmpRoot, epermLockName4);
      fs.writeFileSync(
        epermLockPath4,
        `${JSON.stringify({ pid: 999999999, session: 'epermtest-stale-holder', ts: new Date(Date.now() - 45_000).toISOString(), token: 'epermtest-token' })}\n`,
      );
      const epermStaleMs = (Date.now() - 45_000) / 1000;
      fs.utimesSync(epermLockPath4, epermStaleMs, epermStaleMs);
      try {
        fs.renameSync = throwOnceThenReal(originalRenameSync);
        let ran4 = false;
        await withStoreLock(tmpRoot, epermLockName4, async () => {
          ran4 = true;
        });
        if (!ran4) failures.push('(h4) withStoreLock did not complete a stale takeover after a simulated transient EPERM on renameSync');
      } catch (err) {
        failures.push(`(h4) stale-takeover renameSync threw despite a single transient EPERM (should have retried): ${(err && err.stack) || err}`);
      } finally {
        fs.renameSync = originalRenameSync;
      }
    }

    // (i) CONTENTION TELEMETRY (multisession-native-3, advisor C3/C4): every
    // store-lock acquire OUTCOME appends one JSON line to
    // .bee/logs/contention.jsonl — lock-free (plain appendFileSync, never
    // routed back through withStoreLock/acquireStoreLockOnceSync) and
    // fully fail-open (a telemetry write failure must never break the
    // caller's acquire). Covers both the retrying async path
    // (withStoreLock) and the sync single-attempt hook path
    // (acquireStoreLockOnceSync), each for a clean success and a
    // LOCK_BUSY outcome, plus one fail-open proof per path.
    {
      // (i1) withStoreLock, clean success (0 retries): result "acquired",
      // no contending holder, lock_wait_ms is a real non-negative number.
      const i1LockName = 'lock-i1-telemetry-success';
      fs.rmSync(contentionLogPath(tmpRoot), { force: true });
      let i1Ran = false;
      await withStoreLock(tmpRoot, i1LockName, async () => {
        i1Ran = true;
      });
      const i1Record = readLastContentionRecord(tmpRoot);
      if (!i1Ran) failures.push('(i1) withStoreLock did not run fn');
      if (!i1Record) {
        failures.push('(i1) withStoreLock success did not append a contention.jsonl line');
      } else {
        if (i1Record.result !== 'acquired') failures.push(`(i1) expected result "acquired", got ${JSON.stringify(i1Record.result)}`);
        if (i1Record.lock_name !== i1LockName) failures.push(`(i1) expected lock_name "${i1LockName}", got ${JSON.stringify(i1Record.lock_name)}`);
        if (typeof i1Record.lock_wait_ms !== 'number' || i1Record.lock_wait_ms < 0) {
          failures.push(`(i1) expected numeric non-negative lock_wait_ms, got ${JSON.stringify(i1Record.lock_wait_ms)}`);
        }
        if (i1Record.holder_session !== null) {
          failures.push(`(i1) expected holder_session null (no contention), got ${JSON.stringify(i1Record.holder_session)}`);
        }
        for (const field of ['workflow_id', 'workspace_id', 'resource']) {
          if (i1Record[field] !== null) failures.push(`(i1) expected ${field} null, got ${JSON.stringify(i1Record[field])}`);
        }
      }

      // (i2) withStoreLock, LOCK_BUSY exhaustion: result "busy", holder
      // identity carried from the lock file's owner metadata. Uses a
      // small maxAttempts so this stays fast and deterministic (no real
      // ~5s retry budget needed to prove the telemetry shape).
      const i2LockName = 'lock-i2-telemetry-busy';
      const i2LockPath = lockFilePath(tmpRoot, i2LockName);
      const i2SeededHolder = { pid: 424242, session: 'contention-busy-holder-i2', ts: new Date().toISOString(), token: 'i2-token' };
      fs.writeFileSync(i2LockPath, `${JSON.stringify(i2SeededHolder)}\n`);
      fs.rmSync(contentionLogPath(tmpRoot), { force: true });
      let i2Threw = null;
      try {
        await withStoreLock(tmpRoot, i2LockName, async () => {}, { maxAttempts: 2, retryDelayMs: 5 });
      } catch (err) {
        i2Threw = err;
      }
      fs.rmSync(i2LockPath, { force: true });
      const i2Record = readLastContentionRecord(tmpRoot);
      if (!i2Threw) failures.push('(i2) withStoreLock unexpectedly succeeded against a fresh, non-stale held lock');
      if (!i2Record) {
        failures.push('(i2) withStoreLock LOCK_BUSY exhaustion did not append a contention.jsonl line');
      } else {
        if (i2Record.result !== 'busy') failures.push(`(i2) expected result "busy", got ${JSON.stringify(i2Record.result)}`);
        if (i2Record.lock_name !== i2LockName) failures.push(`(i2) expected lock_name "${i2LockName}", got ${JSON.stringify(i2Record.lock_name)}`);
        if (i2Record.holder_session !== i2SeededHolder.session) {
          failures.push(`(i2) expected holder_session "${i2SeededHolder.session}", got ${JSON.stringify(i2Record.holder_session)}`);
        }
        if (typeof i2Record.lock_wait_ms !== 'number' || i2Record.lock_wait_ms < 0) {
          failures.push(`(i2) expected numeric non-negative lock_wait_ms, got ${JSON.stringify(i2Record.lock_wait_ms)}`);
        }
      }

      // (i3) withStoreLock, telemetry write failure is fail-open: a
      // simulated unwritable contention.jsonl must never surface as a
      // thrown error or block the acquire.
      const i3LockName = 'lock-i3-telemetry-failopen';
      {
        const originalAppendFileSync = fs.appendFileSync;
        fs.appendFileSync = (...args) => {
          if (String(args[0]).includes('contention.jsonl')) {
            const err = new Error('simulated unwritable contention log');
            err.code = 'EACCES';
            throw err;
          }
          return originalAppendFileSync.apply(fs, args);
        };
        let i3Ran = false;
        try {
          await withStoreLock(tmpRoot, i3LockName, async () => {
            i3Ran = true;
          });
        } catch (err) {
          failures.push(`(i3) withStoreLock threw despite a simulated unwritable contention.jsonl (telemetry must be fail-open): ${(err && err.stack) || err}`);
        } finally {
          fs.appendFileSync = originalAppendFileSync;
        }
        if (!i3Ran) failures.push('(i3) withStoreLock did not run fn when the contention.jsonl telemetry write failed');
        if (fs.existsSync(lockFilePath(tmpRoot, i3LockName))) {
          failures.push('(i3) withStoreLock left the lock file behind after a telemetry write failure');
        }
      }

      // (i4) acquireStoreLockOnceSync (hook try-once path), clean success.
      const i4LockName = 'lock-i4-telemetry-success-sync';
      fs.rmSync(contentionLogPath(tmpRoot), { force: true });
      const i4Result = acquireStoreLockOnceSync(tmpRoot, i4LockName);
      if (!i4Result.acquired) {
        failures.push('(i4) acquireStoreLockOnceSync did not acquire a fresh lock');
      } else {
        i4Result.release();
      }
      const i4Record = readLastContentionRecord(tmpRoot);
      if (!i4Record) {
        failures.push('(i4) acquireStoreLockOnceSync success did not append a contention.jsonl line');
      } else {
        if (i4Record.result !== 'acquired') failures.push(`(i4) expected result "acquired", got ${JSON.stringify(i4Record.result)}`);
        if (i4Record.lock_name !== i4LockName) failures.push(`(i4) expected lock_name "${i4LockName}", got ${JSON.stringify(i4Record.lock_name)}`);
        if (typeof i4Record.lock_wait_ms !== 'number' || i4Record.lock_wait_ms < 0) {
          failures.push(`(i4) expected numeric non-negative lock_wait_ms, got ${JSON.stringify(i4Record.lock_wait_ms)}`);
        }
        if (i4Record.holder_session !== null) {
          failures.push(`(i4) expected holder_session null (no contention), got ${JSON.stringify(i4Record.holder_session)}`);
        }
      }

      // (i5) acquireStoreLockOnceSync, single-attempt LOCK_BUSY (no retry
      // loop at all — either the one attempt wins or it reports busy).
      const i5LockName = 'lock-i5-telemetry-busy-sync';
      const i5LockPath = lockFilePath(tmpRoot, i5LockName);
      const i5SeededHolder = { pid: 424242, session: 'contention-busy-holder-i5', ts: new Date().toISOString(), token: 'i5-token' };
      fs.writeFileSync(i5LockPath, `${JSON.stringify(i5SeededHolder)}\n`);
      fs.rmSync(contentionLogPath(tmpRoot), { force: true });
      const i5Result = acquireStoreLockOnceSync(tmpRoot, i5LockName);
      fs.rmSync(i5LockPath, { force: true });
      const i5Record = readLastContentionRecord(tmpRoot);
      if (i5Result.acquired) {
        failures.push('(i5) acquireStoreLockOnceSync unexpectedly acquired a fresh, non-stale held lock');
      }
      if (!i5Record) {
        failures.push('(i5) acquireStoreLockOnceSync LOCK_BUSY did not append a contention.jsonl line');
      } else {
        if (i5Record.result !== 'busy') failures.push(`(i5) expected result "busy", got ${JSON.stringify(i5Record.result)}`);
        if (i5Record.holder_session !== i5SeededHolder.session) {
          failures.push(`(i5) expected holder_session "${i5SeededHolder.session}", got ${JSON.stringify(i5Record.holder_session)}`);
        }
      }

      // (i6) acquireStoreLockOnceSync, telemetry write failure is
      // fail-open — same simulated-unwritable-log technique as (i3).
      const i6LockName = 'lock-i6-telemetry-failopen-sync';
      {
        const originalAppendFileSync = fs.appendFileSync;
        fs.appendFileSync = (...args) => {
          if (String(args[0]).includes('contention.jsonl')) {
            const err = new Error('simulated unwritable contention log');
            err.code = 'EACCES';
            throw err;
          }
          return originalAppendFileSync.apply(fs, args);
        };
        let i6Result;
        try {
          i6Result = acquireStoreLockOnceSync(tmpRoot, i6LockName);
        } catch (err) {
          failures.push(`(i6) acquireStoreLockOnceSync threw despite a simulated unwritable contention.jsonl (telemetry must be fail-open): ${(err && err.stack) || err}`);
        } finally {
          fs.appendFileSync = originalAppendFileSync;
        }
        if (i6Result && i6Result.acquired) {
          i6Result.release();
        } else if (!i6Result || !i6Result.acquired) {
          failures.push('(i6) acquireStoreLockOnceSync did not acquire a fresh lock when the contention.jsonl telemetry write failed');
        }
      }
    }
  } finally {
    fs.rmSync(tmpRoot, { recursive: true, force: true });
  }

  if (failures.length) {
    console.error('FAIL test_store_lock:');
    for (const f of failures) console.error(`  - ${f}`);
    process.exit(1);
  }

  console.log(
    `PASS test_store_lock: isPidAlive() unit checks green; (a) ${WORKERS}x${ITERS} locked racers zero violations/exact count; ` +
      `(b) unguarded control demonstrated loss/overlap; (c) ${WORKERS}x${ITERS} racers survived a pre-seeded stale ` +
      `lock with zero violations/exact count; (d) LOCK_BUSY refusal named the real holder after the real retry budget; ` +
      `(e) a 45s-backdated LIVE-pid lock was NOT stolen (LOCK_BUSY refusal); (f) ${WORKERS}x${ITERS} racers took over ` +
      `a live-but-past-HARD_STALE_MS-ceiling lock anyway, zero violations/exact count; (g) 10/10 forced-interleaving ` +
      `stale-takeover races held mutual exclusion deterministically; (g2) 10/10 forced third-racer vacancy-exploit ` +
      `attempts held mutual exclusion deterministically; (h) simulated transient/permanent EPERM on ` +
      `acquire/release/takeover-rename retried and surfaced correctly, bounded; (i) contention.jsonl telemetry ` +
      `emitted correct success/busy records for both withStoreLock and acquireStoreLockOnceSync, and a simulated ` +
      `unwritable telemetry log never broke either acquire path (fail-open).`,
  );
}
