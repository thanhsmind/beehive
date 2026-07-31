#!/usr/bin/env node
// test_claim_race.mjs — proves `cells claim --id` (bee.mjs handleCellsClaim ->
// cells.mjs claimCellCrossSession) is race-safe end to end (CONTEXT.md D1,
// cell msh-2): the O_EXCL claim file (claims.mjs claimCellFile) is acquired
// BEFORE the cell JSON flips, so concurrent claimants on the SAME cell
// produce exactly one winner and N-1 typed CLAIMED refusals naming the
// owner's session + expiry — never a silent double-claim.
//
// Self-contained child-orchestrator (fork racers, assert internally, exit
// 0/1) invoked by ONE blocking row (critical-patterns 20260714 "Async
// assertions under a non-awaiting runner pass vacuously" — fs writes are
// synchronous inside one process, so "concurrent" async calls fired inside a
// single event loop never exercise a genuine race). Every racer is its own
// OS process (this same file, re-invoked with --role=), same shape as
// scripts/tests/test_store_lock.mjs / scripts/tests/test_state_write_concurrency.mjs.
//
// Three scenarios:
//   (a) SAFE — N racers through the REAL claimCellCrossSession on one open
//       cell: exactly one ok:true winner, N-1 typed CLAIMED refusals naming
//       the winner's session + expiry. The O_EXCL claim file makes this a
//       STRUCTURAL exclusion (only the file's one winner ever calls the
//       cell-JSON mutator at all — the other N-1 never get past
//       claimCellFile), so it is deterministic with no artificial delay.
//   (b) DELIBERATE RED (falsifiability, critical-patterns 20260714) — N
//       racers through a test-owned read-check-write proxy that mimics the
//       PRE-FIX shape (read the cell, check status open, WIDEN the window
//       with a short sleep, then write claimed) with NO claim-file gate in
//       front of it. Demonstrates more than one racer can see "open" before
//       any writes land — the exact double-claim class D1 exists to kill.
//       (Widened window, same discipline as test_store_lock.mjs's own
//       unguarded control: real synchronous fs calls are too fast apart to
//       race reliably without deliberately holding the window open — this is
//       a test-owned proxy of the hazard, not a claim that production code
//       has an artificial delay.)
//   (c) SAME-SESSION ROUND TRIP — claim -> block -> reopen -> claim (same
//       session) succeeds cleanly with no self-refusing CLAIMED, proving
//       block and reopen both release the claim file (D1 Δ2-amendment: else
//       the claim file created by the first claim would still exist and the
//       second claim would lose to itself for the full TTL).
//   (d) BUDGET RACER (self-correcting-loop D2+Δ2) — N racers on a cell whose
//       trace.attempts ledger is PRE-SEEDED at exactly its max_claims budget
//       (3 distinct claim epochs already recorded), so the very next claim
//       must refuse. Because the budget check runs INSIDE claimCellCrossSession's
//       O_EXCL critical section, real OS-process scheduling interleaves two
//       distinct losing shapes: a racer that wins the transient O_EXCL file
//       gets refused CELL_BUDGET_EXHAUSTED (then unwinds it); a racer that
//       attempts DURING another racer's brief file-holding window gets the
//       ordinary typed CLAIMED (same as scenario (a) — it never even reaches
//       the budget check). Both are legitimate "never wins" outcomes; the
//       invariants this proves are (1) zero winners ever (the ledger never
//       changes mid-race, so nobody's claim can succeed), (2) at least one
//       racer actually hit CELL_BUDGET_EXHAUSTED (the budget path is genuinely
//       exercised, not just accidentally missed by timing), and (3) zero
//       orphaned claim-store files survive (every refused racer's unwind —
//       Δ2 on the budget path, the ordinary loss path on the CLAIMED one —
//       actually runs, so the O_EXCL file never leaks held-forever).

import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { spawn } from 'node:child_process';
import { fileURLToPath, pathToFileURL } from 'node:url';
import { envSkipLine } from '../lib/env-capabilities.mjs';

// The DELIBERATE-RED negative controls below race N unguarded processes
// through writeJsonAtomic's rename onto ONE target file. On POSIX, rename(2)
// atomically replaces and the hazard shows up as a silent lost update — the
// exact thing the controls exist to demonstrate. On win32, MoveFileEx refuses
// a concurrent replace with EPERM instead: racers CRASH rather than silently
// losing updates, so the lost-update fixture cannot fire here at all. The
// SAFE (guarded, product-path) scenarios are unaffected and still run.
const WIN32_UNGUARDED_RENAME =
  'win32 rename refuses concurrent unguarded replace (EPERM) — the silent-lost-update negative-control fixture cannot fire';

const __filename = fileURLToPath(import.meta.url);
const REPO_ROOT = path.join(path.dirname(__filename), '..', '..');
// pathToFileURL: dynamic import() of a bare absolute path breaks on win32
// (a `d:\...` specifier parses as protocol "d:"), so every *_LIB_PATH used
// with import() is a file:// URL string, valid on every platform.
const CELLS_LIB_PATH = pathToFileURL(path.join(REPO_ROOT, '.bee', 'bin', 'lib', 'cells.mjs')).href;
const CLAIMS_LIB_PATH = pathToFileURL(path.join(REPO_ROOT, '.bee', 'bin', 'lib', 'claims.mjs')).href;
const FSUTIL_LIB_PATH = pathToFileURL(path.join(REPO_ROOT, '.bee', 'bin', 'lib', 'fsutil.mjs')).href;

const RACERS = 8;
const UNSAFE_WIDEN_MS = 50; // hardening-4b: bumped from 30ms for extra margin under concurrent-verify (parallel suite) CPU contention — observed one transient miss on the new (g-unsafe) control at 30ms, clean on rerun; matches this file's own documented "widen enough to reliably demonstrate the hazard" discipline, not a semantics change.

function argVal(flag) {
  const found = process.argv.find((a) => a.startsWith(`${flag}=`));
  return found ? found.slice(flag.length + 1) : undefined;
}

// ─── deterministic barrier (hardening-1-7-10 D1) ────────────────────────────
// The (g-unsafe) negative control used to order its racers with a fixed
// UNSAFE_WIDEN_MS sleep — real OS scheduling could still let the sole
// raw-claim racer commit AFTER every unsafe racer's write instead of before
// it, so the "detector bites" assertion was measured flaky (~3/10 runs) under
// concurrent-verify CPU contention. An fs-based ready-file handshake between
// the racer processes replaces the sleep with a real ordering guarantee: every
// unsafe racer's stale read is proven to happen-before the real claim (each
// signals a `read-<id>.ready` file; raw-claim waits for all of them before
// claiming), and every unsafe racer's write is proven to happen-after it (raw-
// claim signals `claim-committed.ready` only once its own write lands; each
// unsafe racer waits for that file before writing its stale-snapshot merge).
// That makes the revert 10/10, not "usually".
function touchFile(filePath) {
  fs.mkdirSync(path.dirname(filePath), { recursive: true });
  fs.writeFileSync(filePath, '');
}

async function waitForFile(filePath, { timeoutMs = 10_000, pollMs = 5 } = {}) {
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

const role = argVal('--role');

if (role) {
  await runWorker(role);
} else {
  await runOrchestrator();
}

// ─── worker roles (each its own OS process) ─────────────────────────────────

async function runWorker(workerRole) {
  try {
    if (workerRole === 'safe-racer') {
      const root = argVal('--root');
      const cellId = argVal('--cell');
      const id = argVal('--id');
      const { claimCellCrossSession } = await import(CELLS_LIB_PATH);
      const result = await claimCellCrossSession(root, {
        sessionId: `sess-safe-${id}`,
        worker: `worker-safe-${id}`,
        cellId,
      });
      process.stdout.write(`${JSON.stringify(result)}\n`);
      process.exit(0);
    } else if (workerRole === 'unsafe-racer') {
      const root = argVal('--root');
      const cellId = argVal('--cell');
      const id = argVal('--id');
      const barrier = argVal('--barrier');
      const barrierRacers = Number(argVal('--racers') || 0);
      const { readJson, writeJsonAtomic } = await import(FSUTIL_LIB_PATH);
      const cellPath = path.join(root, '.bee', 'cells', `${cellId}.json`);
      // Pre-fix shape: read, check open, WIDEN the window, then write — no
      // claim-file gate in front of it (the exact hazard D1 removes from the
      // real `cells claim --id` path).
      //
      // rel1710rc-3: a fixed UNSAFE_WIDEN_MS sleep is only a probabilistic
      // ordering — same class of scheduler-timing-dependent assertion as
      // test_worktree_holds_race.mjs's old scenario (c) and this file's own
      // (g-unsafe) control (rel1710rc-2 fixed both with a deterministic
      // fs-based barrier). Under real OS scheduling on a slow/contended
      // runner, one racer's full read-sleep-write sequence can complete
      // before a second racer even takes its own read, collapsing "MORE than
      // one racer saw open" down to 1/RACERS ("DETECTOR DID NOT BITE",
      // reproduced locally under `taskset -c 0,1` at high concurrency). The
      // same ready-file handshake proves every racer's read happens-before
      // every racer's write, making the multi-racer overlap structural
      // rather than timing-dependent.
      const seen = readJson(cellPath, null);
      const sawOpen = !!seen && seen.status === 'open';
      if (barrier && barrierRacers > 0) {
        touchFile(path.join(barrier, `read-${id}.ready`));
        for (let i = 0; i < barrierRacers; i += 1) {
          // eslint-disable-next-line no-await-in-loop
          await waitForFile(path.join(barrier, `read-${i}.ready`));
        }
      } else {
        await new Promise((resolve) => setTimeout(resolve, UNSAFE_WIDEN_MS));
      }
      if (sawOpen) {
        const fresh = readJson(cellPath, seen);
        fresh.status = 'claimed';
        fresh.trace = { ...(fresh.trace || {}), worker: `worker-unsafe-${id}` };
        writeJsonAtomic(cellPath, fresh);
      }
      process.stdout.write(`${JSON.stringify({ sawOpen, id })}\n`);
      process.exit(0);
    } else if (workerRole === 'budget-racer') {
      const root = argVal('--root');
      const cellId = argVal('--cell');
      const id = argVal('--id');
      const { claimCellCrossSession } = await import(CELLS_LIB_PATH);
      const result = await claimCellCrossSession(root, {
        sessionId: `sess-budget-${id}`,
        worker: `worker-budget-${id}`,
        cellId,
      });
      process.stdout.write(`${JSON.stringify(result)}\n`);
      process.exit(0);
    } else if (workerRole === 'verify-racer') {
      // Real path: N racers calling the ACTUAL attempt-ledger appender
      // (recordTestsRedAttempt, cells.mjs — test-simple's writer; `cells
      // verify` is deleted) on ONE cell. Pre-fix (no lock around the
      // read-mutate-write body), real OS-process scheduling around the
      // sub-millisecond critical section can silently drop a trace.attempts
      // entry (GH #27). Post-fix, withStoreLock makes this deterministic with
      // zero loss — same "no artificial widening needed once exclusion is
      // structural" shape as scenario (a).
      const root = argVal('--root');
      const cellId = argVal('--cell');
      const id = argVal('--id');
      const { recordTestsRedAttempt } = await import(CELLS_LIB_PATH);
      try {
        const cell = await recordTestsRedAttempt(root, cellId, { excerptLine: `FAIL racer-${id}` });
        process.stdout.write(`${JSON.stringify({ ok: true, id, attempts: cell.trace.attempts.length })}\n`);
      } catch (err) {
        process.stdout.write(`${JSON.stringify({ ok: false, id, error: String((err && err.message) || err) })}\n`);
      }
      process.exit(0);
    } else if (workerRole === 'verify-racer-unsafe') {
      // Test-owned proxy mimicking recordVerify's PRE-FIX read-mutate-write
      // shape (read cell, WIDEN the window, append based on the STALE
      // snapshot, write) with no lock in front of it — same discipline as
      // the existing 'unsafe-racer' role above: proves the ledger-loss
      // hazard class is real independent of whether the real function
      // happens to race under ordinary OS timing.
      //
      // rel1710rc-3: same class of scheduler-timing-dependent assertion as
      // 'unsafe-racer' above — a fixed UNSAFE_WIDEN_MS sleep is only a
      // probabilistic ordering, and on a slow/contended 2-core runner all
      // RACERS reads-sleep-writes can fully serialize instead of overlapping,
      // collapsing "fewer than RACERS survive" down to "exactly RACERS
      // survive" ("DETECTOR DID NOT BITE"). The same ready-file handshake
      // proves every racer's stale read happens-before every racer's write.
      const root = argVal('--root');
      const cellId = argVal('--cell');
      const id = argVal('--id');
      const barrier = argVal('--barrier');
      const barrierRacers = Number(argVal('--racers') || 0);
      const { readJson, writeJsonAtomic } = await import(FSUTIL_LIB_PATH);
      const cellPath = path.join(root, '.bee', 'cells', `${cellId}.json`);
      const seen = readJson(cellPath, null);
      const staleAttempts = Array.isArray(seen && seen.trace && seen.trace.attempts) ? seen.trace.attempts : [];
      if (barrier && barrierRacers > 0) {
        touchFile(path.join(barrier, `read-${id}.ready`));
        for (let i = 0; i < barrierRacers; i += 1) {
          // eslint-disable-next-line no-await-in-loop
          await waitForFile(path.join(barrier, `read-${i}.ready`));
        }
      } else {
        await new Promise((resolve) => setTimeout(resolve, UNSAFE_WIDEN_MS));
      }
      const fresh = readJson(cellPath, seen);
      fresh.trace = {
        ...(fresh.trace || {}),
        attempts: [...staleAttempts, { n: staleAttempts.length + 1, note: `unsafe-${id}` }],
      };
      writeJsonAtomic(cellPath, fresh);
      process.stdout.write(`${JSON.stringify({ id })}\n`);
      process.exit(0);
    } else if (workerRole === 'raw-claim') {
      // hardening-4b (g): the REAL cells.mjs claimCell, called DIRECTLY (not
      // via claimCellCrossSession/claims.mjs) — races against concurrent
      // raw-update racers below to prove claimCell and updateCell now
      // serialize against EACH OTHER (same `cells:<id>` lock name), not just
      // against themselves.
      const root = argVal('--root');
      const cellId = argVal('--cell');
      const id = argVal('--id');
      const barrier = argVal('--barrier');
      const barrierRacers = Number(argVal('--racers') || 0);
      const { claimCell } = await import(CELLS_LIB_PATH);
      try {
        // Deterministic barrier (hardening-1-7-10 D1, g-unsafe only): wait
        // until every unsafe racer has taken its stale read BEFORE this real
        // claim commits, so the negative control's revert is guaranteed
        // rather than timing-dependent. Scenario (g)'s plain raw-update
        // racers never pass --barrier, so this is a no-op there.
        if (barrier && barrierRacers > 0) {
          for (let i = 0; i < barrierRacers; i += 1) {
            // eslint-disable-next-line no-await-in-loop
            await waitForFile(path.join(barrier, `read-${i}.ready`));
          }
        }
        const cell = await claimCell(root, cellId, `worker-mc-${id}`);
        if (barrier) touchFile(path.join(barrier, 'claim-committed.ready'));
        process.stdout.write(`${JSON.stringify({ ok: true, id, status: cell.status })}\n`);
      } catch (err) {
        if (barrier) touchFile(path.join(barrier, 'claim-committed.ready'));
        process.stdout.write(`${JSON.stringify({ ok: false, id, error: String((err && err.message) || err) })}\n`);
      }
      process.exit(0);
    } else if (workerRole === 'raw-update') {
      // hardening-4b (g): the REAL cells.mjs updateCell, patching `title`
      // only (never `status`) — under the fix, a raw-update that runs AFTER
      // the raw-claim commits reads the FRESH "claimed" cell under the same
      // lock and correctly refuses (updateCell only allows open/blocked); one
      // that runs BEFORE it succeeds normally. Either way, the claim itself
      // must never be silently reverted by a stale merge (that is exactly
      // the (g)-unsafe negative control below).
      const root = argVal('--root');
      const cellId = argVal('--cell');
      const id = argVal('--id');
      const { updateCell } = await import(CELLS_LIB_PATH);
      try {
        const cell = await updateCell(root, cellId, { title: `mutator-race-title-${id}` });
        process.stdout.write(`${JSON.stringify({ ok: true, id, status: cell.status, title: cell.title })}\n`);
      } catch (err) {
        process.stdout.write(`${JSON.stringify({ ok: false, id, error: String((err && err.message) || err) })}\n`);
      }
      process.exit(0);
    } else if (workerRole === 'unsafe-update-racer') {
      // hardening-4b (g), DELIBERATE RED (falsifiability): a test-owned proxy
      // mimicking updateCell's PRE-FIX read-check-write shape (read the cell,
      // WIDEN the window, merge the title patch onto the STALE snapshot,
      // write) with NO lock in front of it. Raced against the REAL (locked)
      // raw-claim above: if this proxy's stale merge ever lands AFTER the
      // real claim committed, it silently carries the stale `status: 'open'`
      // back onto disk, REVERTING the claim — the exact class of bug D1/
      // hardening-4b's cross-mutator locking removes. Proves the hazard is
      // real, independent of whether claimCell itself happens to race.
      const root = argVal('--root');
      const cellId = argVal('--cell');
      const id = argVal('--id');
      const barrier = argVal('--barrier');
      const { readJson, writeJsonAtomic } = await import(FSUTIL_LIB_PATH);
      const cellPath = path.join(root, '.bee', 'cells', `${cellId}.json`);
      const seen = readJson(cellPath, null);
      // Deterministic barrier (hardening-1-7-10 D1): signal this stale read is
      // done, then wait for the real raw-claim racer's commit signal before
      // writing — guarantees this write lands strictly after the real claim,
      // instead of merely hoping a fixed sleep widened the window enough.
      if (barrier) {
        touchFile(path.join(barrier, `read-${id}.ready`));
        await waitForFile(path.join(barrier, 'claim-committed.ready'));
      } else {
        await new Promise((resolve) => setTimeout(resolve, UNSAFE_WIDEN_MS));
      }
      const fresh = readJson(cellPath, seen); // "fresh" in name only — merge base below is the STALE `seen`, the pre-fix bug
      const merged = { ...seen, title: `unsafe-update-title-${id}` };
      writeJsonAtomic(cellPath, merged);
      process.stdout.write(`${JSON.stringify({ id, sawStatus: seen && seen.status, staleStatusWasOpen: !!(seen && seen.status === 'open'), freshExisted: !!fresh })}\n`);
      process.exit(0);
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

function lastJsonLine(stdout) {
  const lines = stdout.split('\n').filter((line) => line.trim());
  if (lines.length === 0) return null;
  try {
    return JSON.parse(lines[lines.length - 1]);
  } catch {
    return null;
  }
}

// ─── fixture helpers (mirrors test_lib.mjs's makeStateRepo/makeCellFile) ───

function makeRoot() {
  const dir = fs.mkdtempSync(path.join(os.tmpdir(), 'test-claim-race-'));
  fs.mkdirSync(path.join(dir, '.bee'), { recursive: true });
  return dir;
}

async function writeApprovedState(dir) {
  const { writeJsonAtomic } = await import(FSUTIL_LIB_PATH);
  writeJsonAtomic(path.join(dir, '.bee', 'state.json'), {
    schema_version: '1.0',
    phase: 'swarming',
    feature: 'race-feat',
    mode: 'high-risk',
    approved_gates: { context: true, shape: true, execution: true, review: false },
    workers: [],
  });
}

async function makeCell(dir, id) {
  const { writeJsonAtomic } = await import(FSUTIL_LIB_PATH);
  fs.mkdirSync(path.join(dir, '.bee', 'cells'), { recursive: true });
  const cell = {
    id,
    feature: 'race-feat',
    title: `Cell ${id}`,
    lane: 'tiny',
    status: 'open',
    deps: [],
    action: 'race target',
    verify: 'node -e "process.exit(0)"',
    trace: {},
  };
  writeJsonAtomic(path.join(dir, '.bee', 'cells', `${id}.json`), cell);
  return cell;
}

// D2 (self-correcting-loop): a cell whose ledger already sits at exactly the
// default max_claims budget (3 distinct claim epochs, all "pass" so
// max_failed_attempts/max_same_signature never also trigger — isolates the
// scenario to max_claims alone). The next claim, from anyone, must refuse.
async function makeBudgetExhaustedCell(dir, id) {
  const { writeJsonAtomic } = await import(FSUTIL_LIB_PATH);
  fs.mkdirSync(path.join(dir, '.bee', 'cells'), { recursive: true });
  const attempts = [0, 1, 2].map((i) => ({
    n: i + 1,
    at: new Date(Date.now() - (10 - i) * 1000).toISOString(),
    claim_session: `sess-prior-${i}`,
    claimed_at: new Date(Date.now() - (10 - i) * 1000).toISOString(),
    worker: 'worker-prior',
    verdict: 'pass',
    failure_signature: null,
    note: null,
  }));
  const cell = {
    id,
    feature: 'race-feat',
    title: `Cell ${id}`,
    lane: 'tiny',
    status: 'open',
    deps: [],
    action: 'race target',
    verify: 'node -e "process.exit(0)"',
    trace: { attempts },
  };
  writeJsonAtomic(path.join(dir, '.bee', 'cells', `${id}.json`), cell);
  return cell;
}

// ─── orchestrator ────────────────────────────────────────────────────────────

async function runOrchestrator() {
  const { readJson, writeJsonAtomic } = await import(FSUTIL_LIB_PATH);
  const { readClaim, claimGatePath } = await import(CLAIMS_LIB_PATH);
  const failures = [];

  // (a) SAFE — real claimCellCrossSession, N racers on one open cell.
  {
    const dir = makeRoot();
    try {
      await writeApprovedState(dir);
      await makeCell(dir, 'race-a');

      const results = await spawnRacers(RACERS, (i) => [
        '--role=safe-racer',
        `--root=${dir}`,
        '--cell=race-a',
        `--id=${i}`,
      ]);
      const crashed = results.filter((r) => r.code !== 0);
      if (crashed.length) {
        failures.push(
          `(a) ${crashed.length}/${RACERS} safe racer(s) crashed:\n` +
            crashed.map((c) => `  exit=${c.code} stderr=${c.stderr.trim()}`).join('\n'),
        );
      }
      const parsed = results.map((r) => lastJsonLine(r.stdout));
      const winners = parsed.filter((p) => p && p.ok === true);
      const losers = parsed.filter((p) => p && p.ok === false);

      if (winners.length !== 1) {
        failures.push(`(a) expected exactly 1 winner, got ${winners.length}: ${JSON.stringify(winners)}`);
      }
      if (losers.length !== RACERS - 1) {
        failures.push(`(a) expected ${RACERS - 1} losers, got ${losers.length}: ${JSON.stringify(parsed)}`);
      }
      for (const loser of losers) {
        if (loser.code !== 'CLAIMED') {
          failures.push(`(a) loser code should be typed CLAIMED, got ${JSON.stringify(loser)}`);
        }
        if (typeof loser.reason !== 'string' || !/expir/i.test(loser.reason)) {
          failures.push(`(a) loser reason must name the expiry, got ${JSON.stringify(loser)}`);
        }
      }
      const winnerSession = winners[0] && winners[0].claim && winners[0].claim.session;
      if (!winnerSession || !losers.every((l) => typeof l.reason === 'string' && l.reason.includes(winnerSession))) {
        failures.push(`(a) every loser's reason must name the actual winner's session "${winnerSession}", got ${JSON.stringify(losers)}`);
      }

      const finalCell = readJson(path.join(dir, '.bee', 'cells', 'race-a.json'), null);
      if (!finalCell || finalCell.status !== 'claimed') {
        failures.push(`(a) final cell status should be "claimed", got ${JSON.stringify(finalCell)}`);
      }
      const finalClaim = readClaim(dir, 'race-a');
      if (!finalClaim || finalClaim.session !== winnerSession) {
        failures.push(`(a) claims-store claim should belong to the winner "${winnerSession}", got ${JSON.stringify(finalClaim)}`);
      }
      if (fs.existsSync(claimGatePath(dir, 'race-a'))) {
        failures.push('(a) no gate file should leak after the race settles');
      }
    } finally {
      fs.rmSync(dir, { recursive: true, force: true });
    }
  }

  // (b) DELIBERATE RED — widened-window read-check-write with NO claim-file
  // gate, proving the pre-fix hazard is real (falsifiability: the safe result
  // above is not simply "nothing ever races here"). rel1710rc-3: a
  // deterministic fs-based barrier (same as (g-unsafe) below) proves every
  // racer's stale read happens-before every racer's write, making the
  // multi-racer overlap structural (exactly RACERS saw "open") rather than
  // timing-dependent — the old fixed-sleep version could collapse to 1/RACERS
  // under real scheduler contention ("DETECTOR DID NOT BITE").
  if (process.platform === 'win32') {
    console.log(envSkipLine(WIN32_UNGUARDED_RENAME, '(b) DELIBERATE RED — unguarded claim racers'));
  } else {
    const dir = makeRoot();
    const barrier = path.join(dir, '.race-barrier-b-unsafe');
    fs.mkdirSync(barrier, { recursive: true });
    try {
      await writeApprovedState(dir);
      await makeCell(dir, 'race-b');

      const results = await spawnRacers(RACERS, (i) => [
        '--role=unsafe-racer',
        `--root=${dir}`,
        '--cell=race-b',
        `--id=${i}`,
        `--barrier=${barrier}`,
        `--racers=${RACERS}`,
      ]);
      const crashed = results.filter((r) => r.code !== 0);
      if (crashed.length) {
        failures.push(
          `(b) ${crashed.length}/${RACERS} unsafe racer(s) crashed:\n` +
            crashed.map((c) => `  exit=${c.code} stderr=${c.stderr.trim()}`).join('\n'),
        );
      }
      const parsed = results.map((r) => lastJsonLine(r.stdout));
      const sawOpenCount = parsed.filter((p) => p && p.sawOpen === true).length;
      if (sawOpenCount !== RACERS) {
        failures.push(
          `(b) DETECTOR DID NOT BITE: only ${sawOpenCount}/${RACERS} unguarded racer(s) saw the cell "open" — the ` +
            'barrier guarantees every read happens before any write, so this negative control must show ALL racers ' +
            'believing it won, or the (a) green result proves nothing.',
        );
      }
      const finalCell = readJson(path.join(dir, '.bee', 'cells', 'race-b.json'), null);
      if (!finalCell || finalCell.status !== 'claimed') {
        failures.push(`(b) final unguarded cell status should still land on "claimed" (last writer wins), got ${JSON.stringify(finalCell)}`);
      }
    } finally {
      fs.rmSync(dir, { recursive: true, force: true });
    }
  }

  // (c) SAME-SESSION ROUND TRIP — claim -> block -> reopen -> claim (same
  // session) must succeed, proving block/reopen both release the claim file.
  {
    const dir = makeRoot();
    try {
      await writeApprovedState(dir);
      await makeCell(dir, 'race-c');
      const { claimCellCrossSession, blockCell, reopenCell } = await import(CELLS_LIB_PATH);

      const firstClaim = await claimCellCrossSession(dir, { sessionId: 'sess-roundtrip', worker: 'worker-rt', cellId: 'race-c' });
      if (!firstClaim.ok) failures.push(`(c) first claim should succeed, got ${JSON.stringify(firstClaim)}`);

      await blockCell(dir, 'race-c', 'round-trip test block', { sessionId: 'sess-roundtrip' });
      if (readClaim(dir, 'race-c') !== null) {
        failures.push('(c) block must release the claim file (D1 Δ2)');
      }

      await reopenCell(dir, 'race-c', 'round-trip test reopen', { sessionId: 'sess-roundtrip' });
      if (readClaim(dir, 'race-c') !== null) {
        failures.push('(c) reopen must release the claim file (D1 Δ2)');
      }

      const secondClaim = await claimCellCrossSession(dir, { sessionId: 'sess-roundtrip', worker: 'worker-rt', cellId: 'race-c' });
      if (!secondClaim.ok) {
        failures.push(
          `(c) same-session round-trip re-claim must succeed (no self-refusal), got ${JSON.stringify(secondClaim)} — ` +
            'a stale claim file from the first claim was never released.',
        );
      }
      if (secondClaim.ok && secondClaim.cell.status !== 'claimed') {
        failures.push(`(c) re-claimed cell should be "claimed", got ${JSON.stringify(secondClaim.cell)}`);
      }
    } finally {
      fs.rmSync(dir, { recursive: true, force: true });
    }
  }

  // (d) BUDGET RACER — N racers on a cell whose ledger is pre-seeded at
  // exactly the max_claims budget: zero winners, every racer refused typed
  // CELL_BUDGET_EXHAUSTED, and no orphaned claim-store file survives the race
  // (D2+Δ2's unwind-inside-critical-section must hold under real contention,
  // not just a single sequential call).
  {
    const dir = makeRoot();
    try {
      await writeApprovedState(dir);
      await makeBudgetExhaustedCell(dir, 'race-d');

      const results = await spawnRacers(RACERS, (i) => [
        '--role=budget-racer',
        `--root=${dir}`,
        '--cell=race-d',
        `--id=${i}`,
      ]);
      const crashed = results.filter((r) => r.code !== 0);
      if (crashed.length) {
        failures.push(
          `(d) ${crashed.length}/${RACERS} budget racer(s) crashed:\n` +
            crashed.map((c) => `  exit=${c.code} stderr=${c.stderr.trim()}`).join('\n'),
        );
      }
      const parsed = results.map((r) => lastJsonLine(r.stdout));
      const winners = parsed.filter((p) => p && p.ok === true);
      const losers = parsed.filter((p) => p && p.ok === false);

      if (winners.length !== 0) {
        failures.push(`(d) expected 0 winners against an already-exhausted budget, got ${winners.length}: ${JSON.stringify(winners)}`);
      }
      if (losers.length !== RACERS) {
        failures.push(`(d) expected all ${RACERS} racers refused, got ${losers.length}: ${JSON.stringify(parsed)}`);
      }
      // Real OS scheduling interleaves two legitimate loss shapes here (see
      // the file-header comment): a racer that actually wins the O_EXCL file
      // gets CELL_BUDGET_EXHAUSTED; a racer that attempts during another's
      // brief hold gets the ordinary CLAIMED without ever reaching the budget
      // check. Both are "never wins"; only an unrecognized code is a failure.
      for (const loser of losers) {
        if (loser.code !== 'CELL_BUDGET_EXHAUSTED' && loser.code !== 'CLAIMED') {
          failures.push(`(d) every racer's refusal should be typed CELL_BUDGET_EXHAUSTED or CLAIMED, got ${JSON.stringify(loser)}`);
        }
      }
      const budgetRefusals = losers.filter((l) => l.code === 'CELL_BUDGET_EXHAUSTED');
      if (budgetRefusals.length === 0) {
        failures.push(`(d) the budget path was never actually exercised — every racer lost to CLAIMED instead: ${JSON.stringify(losers)}`);
      }

      const finalCell = readJson(path.join(dir, '.bee', 'cells', 'race-d.json'), null);
      if (!finalCell || finalCell.status !== 'open') {
        failures.push(`(d) an exhausted-budget cell must stay "open" — nobody may claim it, got ${JSON.stringify(finalCell)}`);
      }
      if (finalCell && Array.isArray(finalCell.trace.attempts) && finalCell.trace.attempts.length !== 3) {
        failures.push(`(d) the pre-seeded ledger must be untouched by refused claims, got ${JSON.stringify(finalCell.trace.attempts)}`);
      }
      if (readClaim(dir, 'race-d') !== null) {
        failures.push('(d) no claim-store file may survive the race — every refused racer must unwind its own O_EXCL acquisition (Δ2)');
      }
      if (fs.existsSync(claimGatePath(dir, 'race-d'))) {
        failures.push('(d) no gate file should leak after the race settles');
      }
    } finally {
      fs.rmSync(dir, { recursive: true, force: true });
    }
  }

  // (e) LEDGER-LOCK — real path: N racers calling the ACTUAL attempt-ledger
  // appender (recordTestsRedAttempt) on ONE cell (GH #27.2, cell ghf-4). Every racer must succeed and every one
  // of its trace.attempts entries must survive — zero lost updates. Post-fix
  // this is deterministic (withStoreLock serializes the read-mutate-write
  // body), same "no artificial widening needed" shape as scenario (a)'s real
  // O_EXCL exclusion.
  {
    const dir = makeRoot();
    try {
      await writeApprovedState(dir);
      await makeCell(dir, 'race-e');

      const results = await spawnRacers(RACERS, (i) => [
        '--role=verify-racer',
        `--root=${dir}`,
        '--cell=race-e',
        `--id=${i}`,
      ]);
      const crashed = results.filter((r) => r.code !== 0);
      if (crashed.length) {
        failures.push(
          `(e) ${crashed.length}/${RACERS} verify racer(s) crashed:\n` +
            crashed.map((c) => `  exit=${c.code} stderr=${c.stderr.trim()}`).join('\n'),
        );
      }
      const parsed = results.map((r) => lastJsonLine(r.stdout));
      const errored = parsed.filter((p) => !p || p.ok !== true);
      if (errored.length) {
        failures.push(`(e) every ledger-append racer should succeed, got failure(s): ${JSON.stringify(errored)}`);
      }
      const finalCell = readJson(path.join(dir, '.bee', 'cells', 'race-e.json'), null);
      const finalAttempts = finalCell && Array.isArray(finalCell.trace.attempts) ? finalCell.trace.attempts : [];
      if (finalAttempts.length !== RACERS) {
        failures.push(
          `(e) expected exactly ${RACERS} trace.attempts entries (one per racer, zero lost), got ${finalAttempts.length}: ` +
            `${JSON.stringify(finalAttempts)}`,
        );
      }
      const ns = finalAttempts.map((a) => a.n);
      const uniqueNs = new Set(ns);
      if (uniqueNs.size !== finalAttempts.length) {
        failures.push(`(e) trace.attempts has duplicate/clobbered "n" values (a lost-update fingerprint): ${JSON.stringify(ns)}`);
      }
    } finally {
      fs.rmSync(dir, { recursive: true, force: true });
    }
  }

  // (f) DELIBERATE RED (falsifiability) — a test-owned proxy mimicking
  // recordVerify's PRE-FIX read-mutate-write shape (no lock, widened window)
  // proves the ledger-loss hazard class scenario (e) guards against is real,
  // independent of whether the real function happens to race under ordinary
  // OS timing. Same negative-control discipline as scenario (b). rel1710rc-3:
  // a deterministic fs-based barrier (same as (b)'s) proves every racer's
  // stale read happens-before every racer's write, making the lost-update
  // collapse structural rather than timing-dependent.
  if (process.platform === 'win32') {
    console.log(envSkipLine(WIN32_UNGUARDED_RENAME, '(f) DELIBERATE RED — unguarded verify-ledger racers'));
  } else {
    const dir = makeRoot();
    const barrier = path.join(dir, '.race-barrier-f-unsafe');
    fs.mkdirSync(barrier, { recursive: true });
    try {
      await writeApprovedState(dir);
      await makeCell(dir, 'race-f');

      const results = await spawnRacers(RACERS, (i) => [
        '--role=verify-racer-unsafe',
        `--root=${dir}`,
        '--cell=race-f',
        `--id=${i}`,
        `--barrier=${barrier}`,
        `--racers=${RACERS}`,
      ]);
      const crashed = results.filter((r) => r.code !== 0);
      if (crashed.length) {
        failures.push(
          `(f) ${crashed.length}/${RACERS} unsafe verify racer(s) crashed:\n` +
            crashed.map((c) => `  exit=${c.code} stderr=${c.stderr.trim()}`).join('\n'),
        );
      }
      const finalCell = readJson(path.join(dir, '.bee', 'cells', 'race-f.json'), null);
      const finalAttempts = finalCell && Array.isArray(finalCell.trace.attempts) ? finalCell.trace.attempts : [];
      if (finalAttempts.length >= RACERS) {
        failures.push(
          `(f) DETECTOR DID NOT BITE: unsafe proxy recorded ${finalAttempts.length}/${RACERS} entries — this negative ` +
            'control must demonstrate lost updates (last-writer-wins clobbering the stale-snapshot appends), or the ' +
            '(e) green result proves nothing.',
        );
      }
    } finally {
      fs.rmSync(dir, { recursive: true, force: true });
    }
  }

  // (g) hardening-4b — MUTATOR CROSS-RACE: the REAL, now-locked claimCell
  // racing against the REAL, now-locked updateCell on ONE open cell. Both
  // share the exact same `cells:<id>` lock name, so this proves the fix
  // serializes DIFFERENT mutators against each other, not merely a mutator
  // against itself (scenarios a/e above). Exactly ONE raw-claim racer (so the
  // claim's own success is deterministic regardless of interleaving — only
  // it ever touches `status`) plus RACERS raw-update racers patching `title`
  // only. The invariant: the claim must never be silently reverted — final
  // status stays "claimed", and every update racer's own reported outcome is
  // internally consistent (never a crash, never a torn write).
  {
    const dir = makeRoot();
    try {
      await writeApprovedState(dir);
      await makeCell(dir, 'race-g');

      const [claimResults, updateResults] = await Promise.all([
        spawnRacers(1, () => ['--role=raw-claim', `--root=${dir}`, '--cell=race-g', '--id=claim']),
        spawnRacers(RACERS, (i) => ['--role=raw-update', `--root=${dir}`, '--cell=race-g', `--id=${i}`]),
      ]);
      const crashed = [...claimResults, ...updateResults].filter((r) => r.code !== 0);
      if (crashed.length) {
        failures.push(
          `(g) ${crashed.length} mutator-race racer(s) crashed:\n` +
            crashed.map((c) => `  exit=${c.code} stderr=${c.stderr.trim()}`).join('\n'),
        );
      }
      const claimParsed = lastJsonLine(claimResults[0].stdout);
      if (!claimParsed || claimParsed.ok !== true) {
        failures.push(`(g) the sole raw-claim racer should always win (nothing else touches status), got ${JSON.stringify(claimParsed)}`);
      }
      const updateParsed = updateResults.map((r) => lastJsonLine(r.stdout));
      const badUpdates = updateParsed.filter(
        (p) => !p || (p.ok === true && p.status !== 'claimed' && p.status !== 'open') || (p.ok === false && !/open|blocked/.test(String(p.error))),
      );
      if (badUpdates.length) {
        failures.push(`(g) every raw-update outcome must be a clean success (status open/claimed) or a typed open/blocked refusal, got ${JSON.stringify(badUpdates)}`);
      }
      const finalCell = readJson(path.join(dir, '.bee', 'cells', 'race-g.json'), null);
      if (!finalCell || finalCell.status !== 'claimed') {
        failures.push(`(g) final cell status must be "claimed" — a lost update would silently revert it to "open", got ${JSON.stringify(finalCell)}`);
      }
      if (finalCell && finalCell.trace.worker !== 'worker-mc-claim') {
        failures.push(`(g) the claim's own worker must survive every concurrent update, got ${JSON.stringify(finalCell && finalCell.trace)}`);
      }
    } finally {
      fs.rmSync(dir, { recursive: true, force: true });
    }
  }

  // (g) DELIBERATE RED (falsifiability) — the SAME raw-claim racer, now raced
  // against RACERS copies of the UNSAFE, unlocked updateCell-shaped proxy, via
  // a deterministic fs-based ready-file barrier (not a sleep — see the barrier
  // helpers near the top of this file). Demonstrates the exact hazard the fix
  // removes: an unlocked "update" can read the cell BEFORE the claim commits,
  // then write its stale snapshot's `status: 'open'` back to disk AFTER the
  // claim committed — silently reverting it. Every unsafe racer is guaranteed
  // to have observed "open" pre-claim (the barrier ensures ALL of their reads
  // precede the claim) and every one of their writes is guaranteed to land
  // after the claim commits, so the revert is deterministic, 10/10 — the
  // detector bites when the FINAL status is not "claimed".
  if (process.platform === 'win32') {
    console.log(envSkipLine(WIN32_UNGUARDED_RENAME, '(g-unsafe) DELIBERATE RED — unlocked update racers vs raw claim'));
  } else {
    const dir = makeRoot();
    const barrier = path.join(dir, '.race-barrier-g-unsafe');
    try {
      await writeApprovedState(dir);
      await makeCell(dir, 'race-g-unsafe');
      fs.mkdirSync(barrier, { recursive: true });

      const [claimResults, unsafeResults] = await Promise.all([
        spawnRacers(1, () => [
          '--role=raw-claim',
          `--root=${dir}`,
          '--cell=race-g-unsafe',
          '--id=claim',
          `--barrier=${barrier}`,
          `--racers=${RACERS}`,
        ]),
        spawnRacers(RACERS, (i) => [
          '--role=unsafe-update-racer',
          `--root=${dir}`,
          '--cell=race-g-unsafe',
          `--id=${i}`,
          `--barrier=${barrier}`,
        ]),
      ]);
      const crashed = [...claimResults, ...unsafeResults].filter((r) => r.code !== 0);
      if (crashed.length) {
        failures.push(
          `(g-unsafe) ${crashed.length} racer(s) crashed:\n` + crashed.map((c) => `  exit=${c.code} stderr=${c.stderr.trim()}`).join('\n'),
        );
      }
      const unsafeParsed = unsafeResults.map((r) => lastJsonLine(r.stdout));
      const sawOpenCount = unsafeParsed.filter((p) => p && p.staleStatusWasOpen === true).length;
      if (sawOpenCount === 0) {
        failures.push('(g-unsafe) no unsafe racer observed the cell "open" pre-claim — the control never even got a chance to bite; widen the window or check scheduling');
      }
      const finalCell = readJson(path.join(dir, '.bee', 'cells', 'race-g-unsafe.json'), null);
      if (!finalCell || finalCell.status === 'claimed') {
        failures.push(
          `(g-unsafe) DETECTOR DID NOT BITE: final status is "${finalCell && finalCell.status}" — this negative control must demonstrate ` +
            'the claim getting silently reverted by an unlocked stale-snapshot write, or the (g) green result proves nothing.',
        );
      }
    } finally {
      fs.rmSync(dir, { recursive: true, force: true });
    }
  }

  // (h) hardening-4b — SWEEP-RESET: sweepExpiredClaims (claims.mjs) now ALSO
  // resets a cell it just swept from claimed back to open, guarded by
  // trace.claim_session matching the swept claim exactly (never clobbering a
  // fresher claim). Not itself a race (single-process, time simulated via
  // sweepExpiredClaims' own `now` override — same technique test_claims.mjs
  // already uses) — sequential proof that the reset fires, is audited, and
  // that a subsequent claimant can genuinely reclaim the cell.
  {
    const dir = makeRoot();
    try {
      await writeApprovedState(dir);
      await makeCell(dir, 'race-h');
      const { claimCellCrossSession } = await import(CELLS_LIB_PATH);
      const { DEFAULT_CLAIM_TTL_SECONDS, DEFAULT_HEARTBEAT_STALE_SECONDS } = await import(CLAIMS_LIB_PATH);

      const claimed = await claimCellCrossSession(dir, { sessionId: 'sess-sweep-reset', worker: 'worker-sweep', cellId: 'race-h' });
      if (!claimed.ok) failures.push(`(h) setup claim should succeed, got ${JSON.stringify(claimed)}`);

      const cellPathH = path.join(dir, '.bee', 'cells', 'race-h.json');
      const afterClaim = readJson(cellPathH, null);
      if (!afterClaim || afterClaim.trace.claim_session !== 'sess-sweep-reset') {
        failures.push(`(h) claimCell must stamp trace.claim_session with the acting session, got ${JSON.stringify(afterClaim && afterClaim.trace)}`);
      }

      // Simulate both TTL expiry and heartbeat staleness at once via `now`,
      // rather than a real sleep (same technique claims.mjs's own sweep
      // tests use) — well past both default thresholds.
      const futureNow = Date.now() + (DEFAULT_CLAIM_TTL_SECONDS + DEFAULT_HEARTBEAT_STALE_SECONDS + 60) * 1000;
      const sweepResult = await sweepExpiredClaimsFrom(dir, futureNow);
      if (!sweepResult.ok || !sweepResult.swept.includes('race-h')) {
        failures.push(`(h) the expired, stale claim on race-h should sweep, got ${JSON.stringify(sweepResult)}`);
      }
      if (!Array.isArray(sweepResult.reset) || !sweepResult.reset.includes('race-h')) {
        failures.push(`(h) sweepExpiredClaims must report race-h in .reset — cell reset is the must-have truth under test, got ${JSON.stringify(sweepResult)}`);
      }
      const afterSweep = readJson(cellPathH, null);
      if (!afterSweep || afterSweep.status !== 'open') {
        failures.push(`(h) race-h must be back to "open" after the sweep-reset, got ${JSON.stringify(afterSweep)}`);
      }
      if (afterSweep && (afterSweep.trace.claim_session !== null || afterSweep.trace.worker !== null)) {
        failures.push(`(h) sweep-reset must clear trace.claim_session and trace.worker, got ${JSON.stringify(afterSweep.trace)}`);
      }
      const decisionsPath = path.join(dir, '.bee', 'decisions.jsonl');
      const decisionsText = fs.existsSync(decisionsPath) ? fs.readFileSync(decisionsPath, 'utf8') : '';
      if (!/race-h/.test(decisionsText) || !/sweep/i.test(decisionsText)) {
        failures.push('(h) sweep-reset must log one audit decision line naming the reset cell — none found in decisions.jsonl');
      }

      const reclaimed = await claimCellCrossSession(dir, { sessionId: 'sess-fresh-claimant', worker: 'worker-fresh', cellId: 'race-h' });
      if (!reclaimed.ok) {
        failures.push(`(h) a fresh claimant must be able to reclaim race-h after the sweep-reset, got ${JSON.stringify(reclaimed)}`);
      }
    } finally {
      fs.rmSync(dir, { recursive: true, force: true });
    }
  }

  // (h2) hardening-4b — SWEEP-RESET NEVER CLOBBERS A FRESHER CLAIM: simulates
  // (deterministically, no real race needed) the exact guard
  // sweepExpiredClaims' reset checks — a cell whose trace.claim_session no
  // longer matches the just-swept claim's session (someone else's claim
  // already won the cell by the time the reset's own lock acquisition runs)
  // must be left untouched, never forced back to open out from under the
  // fresher claimant.
  {
    const dir = makeRoot();
    try {
      await writeApprovedState(dir);
      await makeCell(dir, 'race-h2');
      const { claimCellCrossSession } = await import(CELLS_LIB_PATH);
      const { DEFAULT_CLAIM_TTL_SECONDS, DEFAULT_HEARTBEAT_STALE_SECONDS } = await import(CLAIMS_LIB_PATH);

      const claimed = await claimCellCrossSession(dir, { sessionId: 'sess-h2-original', worker: 'worker-h2-original', cellId: 'race-h2' });
      if (!claimed.ok) failures.push(`(h2) setup claim should succeed, got ${JSON.stringify(claimed)}`);

      // Simulate "a fresher claim already won it": hand-edit the cell's
      // trace.claim_session to a DIFFERENT session than the one whose claim
      // file the sweep below is about to remove — the ONLY way this happens
      // for real is a fresh claim already replacing it; hand-editing pins the
      // guard's behavior deterministically without needing a genuine race.
      const cellPathH2 = path.join(dir, '.bee', 'cells', 'race-h2.json');
      const cellH2 = readJson(cellPathH2, null);
      cellH2.trace.claim_session = 'sess-h2-fresher';
      writeJsonAtomic(cellPathH2, cellH2);

      const futureNow = Date.now() + (DEFAULT_CLAIM_TTL_SECONDS + DEFAULT_HEARTBEAT_STALE_SECONDS + 60) * 1000;
      const sweepResult = await sweepExpiredClaimsFrom(dir, futureNow);
      if (!sweepResult.ok || !sweepResult.swept.includes('race-h2')) {
        failures.push(`(h2) the expired, stale claim FILE should still sweep regardless of the mismatch, got ${JSON.stringify(sweepResult)}`);
      }
      if (Array.isArray(sweepResult.reset) && sweepResult.reset.includes('race-h2')) {
        failures.push(`(h2) a claim_session MISMATCH must never reset the cell — got ${JSON.stringify(sweepResult)}`);
      }
      const afterSweep = readJson(cellPathH2, null);
      if (!afterSweep || afterSweep.status !== 'claimed' || afterSweep.trace.claim_session !== 'sess-h2-fresher') {
        failures.push(`(h2) the cell must be left exactly as the fresher claimant left it — untouched by the mismatched sweep, got ${JSON.stringify(afterSweep)}`);
      }
    } finally {
      fs.rmSync(dir, { recursive: true, force: true });
    }
  }

  if (failures.length) {
    console.error('FAIL test_claim_race:');
    for (const f of failures) console.error(`  - ${f}`);
    process.exit(1);
  }

  // The (b)/(f)/(g-unsafe) negative controls are env-skipped on win32 (see
  // WIN32_UNGUARDED_RENAME above) — the summary must never claim a detector
  // bit when its scenario printed a SKIP line instead.
  const negativeControlsRan = process.platform !== 'win32';
  console.log(
    `PASS test_claim_race: (a) ${RACERS} real-path racers on one cell -> exactly 1 winner, ${RACERS - 1} typed CLAIMED ` +
      "refusals naming the winner's session + expiry; " +
      (negativeControlsRan
        ? '(b) deliberate-red unguarded proxy showed multiple racers believing they won (detector bites); '
        : '(b) env-skipped on win32 (see SKIP line above); ') +
      '(c) claim -> block -> reopen -> claim same-session round trip succeeded ' +
      `with no self-refusal; (d) ${RACERS} racers against an already-exhausted budget -> 0 winners, every loss typed ` +
      'CELL_BUDGET_EXHAUSTED or CLAIMED, no orphaned claim-store file (D2+Δ2 unwind holds under real concurrency); ' +
      `(e) ${RACERS} real recordVerify racers on one cell -> zero lost trace.attempts entries (GH #27.2 ledger lock); ` +
      (negativeControlsRan
        ? '(f) deliberate-red unsafe proxy demonstrated lost updates (detector bites); '
        : '(f) env-skipped on win32 (see SKIP line above); ') +
      '(g) hardening-4b: real claimCell vs real updateCell cross-mutator race never reverts the claim' +
      (negativeControlsRan
        ? ', deliberate-red unsafe-update proxy demonstrated the revert (detector bites); '
        : ' (its deliberate-red proxy env-skipped on win32, see SKIP line above); ') +
      '(h) sweepExpiredClaims resets a swept claimed ' +
      'cell back to open (audited, re-claimable); (h2) a claim_session mismatch never clobbers a fresher claim.',
  );
}

// hardening-4b (h): thin wrapper so scenarios (h)/(h2) share one import line
// for sweepExpiredClaims instead of repeating the dynamic import.
async function sweepExpiredClaimsFrom(root, now) {
  const { sweepExpiredClaims } = await import(CLAIMS_LIB_PATH);
  return sweepExpiredClaims(root, { now });
}
