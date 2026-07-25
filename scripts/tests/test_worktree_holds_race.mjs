#!/usr/bin/env node
// test_worktree_holds_race.mjs — proves worktree-holds.mjs's mirrorHold()
// (skills/bee-hive/templates/lib/worktree-holds.mjs, cell xwh-1) is
// race-safe end to end, the exact same shape scripts/tests/test_reservation_race.mjs
// already proves for reservations.mjs's reserve(): the read-check-write body
// (fresh read, append, write) runs inside withStoreLock(mainRoot,
// 'cross-worktree-holds'), so two concurrent mirrorHold() calls — from two
// separate OS processes, e.g. two different worktree checkouts racing to
// mirror a hold into the SAME shared ledger — can never both read the same
// pre-mutation snapshot and have the later write silently drop the earlier
// one.
//
// Self-contained child-orchestrator (fork racers, assert internally, exit
// 0/1), same pattern as scripts/tests/test_reservation_race.mjs / test_claim_race.mjs
// / test_store_lock.mjs: every racer is its own OS process (this same file,
// re-invoked with --role=), because fs writes are synchronous inside one
// process — "concurrent" async calls fired inside a single event loop never
// exercise a genuine cross-process race (critical-patterns 20260714).
//
// Five scenarios:
//   (a) SAFE, distinct paths/holders — N racers through the REAL mirrorHold()
//       on N distinct paths, each its own holder, into the SAME ledger: every
//       racer's entry must survive (N entries at the end) — no lost update
//       even though every racer read-check-writes the SAME
//       cross-worktree-holds.json.
//   (b) DELIBERATE RED (falsifiability, critical-patterns 20260714) — N
//       racers through a test-owned proxy that mimics the PRE-FIX shape
//       (read the store, WIDEN the window with a short sleep, append, write)
//       with NO store lock in front of it, each on a distinct path in the
//       same store. Demonstrates the exact lost-update hazard withStoreLock
//       exists to kill: fewer than N rows survive because a later writer's
//       read predates an earlier writer's write and clobbers it on save.
//       Runs in its own throwaway temp dir — the real (safe) store is never
//       touched by this scenario, so nothing needs "restoring" afterward.
//   (c) hardening-1-7-10 D3 — SAME-PATH negative control against the OLD
//       handleReservationsReserve shape (check-then-act: an unlocked
//       findForeignHolds read, then reserve(), then mirrorHold(), no shared
//       critical section across the three). N distinct holders (distinct
//       simulated checkouts, each its own local reservation root) race the
//       SAME path against ONE shared mainRoot ledger: 2+ racers must land an
//       active grant on the identical path (a double grant) — the exact
//       hazard D3's atomic section (scenario (d)) fixes. This is the
//       red_failure_evidence for cell 1710-3.
//   (d) hardening-1-7-10 D3 FIX — same same-path/distinct-holder shape as
//       (c), but each racer composes findForeignHolds + reserve() +
//       insertHold() inside ONE withHoldsLock(mainRoot, ...) section, the
//       exact atomic composition bee.mjs's fixed handleReservationsReserve
//       now uses: must yield EXACTLY ONE winner.
//   (e) renewHolds (D3) — pushes mirrored_at forward for a session's
//       still-active holds so the hold stays active past what would have
//       been its original TTL expiry; never touches an unrelated session's
//       holds. Deterministic via timestamp math, no sleeping.

import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { spawn } from 'node:child_process';
import { fileURLToPath } from 'node:url';

const __filename = fileURLToPath(import.meta.url);
const REPO_ROOT = path.join(path.dirname(__filename), '..', '..');
const WORKTREE_HOLDS_LIB_PATH = path.join(REPO_ROOT, '.bee', 'bin', 'lib', 'worktree-holds.mjs');
const RESERVATIONS_LIB_PATH = path.join(REPO_ROOT, '.bee', 'bin', 'lib', 'reservations.mjs');
const FSUTIL_LIB_PATH = path.join(REPO_ROOT, '.bee', 'bin', 'lib', 'fsutil.mjs');

const RACERS = 8;
const UNSAFE_WIDEN_MS = 30;

function argVal(flag) {
  const found = process.argv.find((a) => a.startsWith(`${flag}=`));
  return found ? found.slice(flag.length + 1) : undefined;
}

// ─── deterministic barrier (rel1710rc-2, same technique scripts/tests/test_claim_race.mjs
// uses for its (g-unsafe) negative control) ─────────────────────────────────
// Scenario (c) below used to order its racers with a fixed UNSAFE_WIDEN_MS
// sleep between each racer's unlocked findForeignHolds read and its later
// reserve+mirrorHold write. On a fast/many-core box every racer's sleep
// overlaps every other racer's, so all 8 checks land on the still-empty
// ledger before any write commits — the double grant is reliable. On a slow
// 2-core CI runner, real OS-process scheduling can let the FIRST racer's
// entire check-sleep-reserve-mirror sequence complete before the SECOND
// racer even starts its check; that second racer's findForeignHolds then
// correctly sees the first racer's already-mirrored hold and refuses,
// collapsing the negative control to only 1 winner ("DETECTOR DID NOT BITE"
// on Linux CI, all Node versions — reproduced locally here under `taskset -c
// 0,1`, run 4/5 flaky). An fs-based ready-file handshake between the racer
// processes replaces the sleep with a real ordering guarantee: every racer
// signals its own `check-<id>.ready` file the instant its findForeignHolds
// read completes (seeing the empty ledger), then waits for every OTHER
// racer's ready file before proceeding to reserve+mirrorHold — so ALL 8
// checks are proven to happen-before ANY of the 8 writes, making the double
// grant structural (every racer's own mirrorHold never re-checks for a
// foreign hold — see worktree-holds.mjs's insertHold comment — so once every
// racer has passed the check on the same empty ledger, every one of their
// writes lands unconditionally). That makes scenario (c) 10/10, not
// "usually".
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
    if (workerRole === 'distinct-racer') {
      const root = argVal('--root');
      const id = argVal('--id');
      const holdPath = argVal('--path');
      const { mirrorHold } = await import(WORKTREE_HOLDS_LIB_PATH);
      const result = await mirrorHold(root, { path: holdPath, holder: `wt-${id}`, cell: 'race-a' });
      process.stdout.write(`${JSON.stringify(result)}\n`);
      process.exit(0);
    } else if (workerRole === 'unsafe-racer') {
      const root = argVal('--root');
      const id = argVal('--id');
      const holdPath = argVal('--path');
      const barrier = argVal('--barrier');
      const barrierRacers = Number(argVal('--racers') || 0);
      const { readJson, writeJsonAtomic } = await import(FSUTIL_LIB_PATH);
      const storePath = path.join(root, '.bee', 'runtime', 'cross-worktree-holds.json');
      // Pre-fix shape: read, WIDEN the window, then append+write — no store
      // lock in front of it (the exact hazard withStoreLock removes from the
      // real mirrorHold() path).
      //
      // rel1710rc-3: a fixed UNSAFE_WIDEN_MS sleep is only a probabilistic
      // ordering — same class of scheduler-timing-dependent assertion as
      // this file's own old scenario (c) (fixed in rel1710rc-2 with a
      // deterministic fs-based barrier). On a slow/contended 2-core runner
      // all RACERS read-sleep-write sequences can fully serialize instead of
      // overlapping, collapsing "fewer than RACERS survive" down to "all
      // RACERS survive" ("DETECTOR DID NOT BITE"). The same ready-file
      // handshake proves every racer's read happens-before every racer's
      // write, making the lost-update collapse structural rather than
      // timing-dependent.
      const store = readJson(storePath, { holds: [] });
      if (barrier && barrierRacers > 0) {
        touchFile(path.join(barrier, `read-${id}.ready`));
        for (let i = 0; i < barrierRacers; i += 1) {
          // eslint-disable-next-line no-await-in-loop
          await waitForFile(path.join(barrier, `read-${i}.ready`));
        }
      } else {
        await new Promise((resolve) => setTimeout(resolve, UNSAFE_WIDEN_MS));
      }
      store.holds.push({
        path: holdPath,
        holder: `wt-unsafe-${id}`,
        feature: null,
        session: null,
        cell: 'race-c',
        ttl_seconds: 3600,
        mirrored_at: new Date().toISOString(),
        released_at: null,
      });
      writeJsonAtomic(storePath, store);
      process.stdout.write(`${JSON.stringify({ id, path: holdPath })}\n`);
      process.exit(0);
    } else if (workerRole === 'same-path-old-racer') {
      // hardening-1-7-10 (D3) RED reproduction — a faithful replay of
      // bee.mjs's PRE-D3 handleReservationsReserve sequence (check-then-act,
      // real exported functions, NOT a reimplementation): an UNLOCKED
      // findForeignHolds read against the shared mainRoot ledger, a widen
      // sleep (same falsifiability convention as UNSAFE_WIDEN_MS above — it
      // makes the real-world race window deterministic under test instead of
      // depending on process-scheduling luck), THEN the local reserve() into
      // THIS racer's own root (simulating a distinct checkout), THEN
      // mirrorHold() into the shared ledger. No re-check ever runs between
      // the widen and the mirror — exactly today's gap.
      const mainRoot = argVal('--main-root');
      const ownRoot = argVal('--own-root');
      const id = argVal('--id');
      const holdPath = argVal('--path');
      const barrier = argVal('--barrier');
      const barrierRacers = Number(argVal('--racers') || 0);
      const holder = `wt-old-${id}`;
      const { findForeignHolds, mirrorHold } = await import(WORKTREE_HOLDS_LIB_PATH);
      const { reserve } = await import(RESERVATIONS_LIB_PATH);
      const foreign = findForeignHolds(mainRoot, holder, [holdPath]);
      if (foreign.length > 0) {
        process.stdout.write(`${JSON.stringify({ id, ok: false, reason: 'foreign-hold-seen' })}\n`);
        process.exit(0);
      }
      // Deterministic barrier (rel1710rc-2): signal this racer's
      // findForeignHolds check is done (it saw the empty ledger), then wait
      // for every other racer to also finish its own check before ANY racer
      // proceeds to reserve+mirrorHold. This makes the double-grant
      // structural — every racer is PROVEN to have passed its check on the
      // SAME empty ledger before any write lands — instead of merely hoping
      // a fixed sleep widened the window enough under real OS scheduling.
      if (barrier && barrierRacers > 0) {
        touchFile(path.join(barrier, `check-${id}.ready`));
        for (let i = 0; i < barrierRacers; i += 1) {
          // eslint-disable-next-line no-await-in-loop
          await waitForFile(path.join(barrier, `check-${i}.ready`));
        }
      } else {
        await new Promise((resolve) => setTimeout(resolve, UNSAFE_WIDEN_MS));
      }
      const reserveResult = await reserve(ownRoot, { agent: `agent-old-${id}`, cell: 'race-old', path: holdPath });
      if (reserveResult.ok) {
        await mirrorHold(mainRoot, { path: holdPath, holder, cell: 'race-old', session: `sess-old-${id}` });
      }
      process.stdout.write(`${JSON.stringify({ id, ok: reserveResult.ok === true })}\n`);
      process.exit(0);
    } else if (workerRole === 'same-path-atomic-racer') {
      // D3 FIX regression — the SAME check/reserve/mirror sequence as above,
      // but composed exactly as bee.mjs's fixed handleReservationsReserve
      // now does: findForeignHolds, reserve(), and the unlocked insertHold()
      // core all run INSIDE one withHoldsLock(mainRoot, ...) critical
      // section, so a second racer can only ever observe the first racer's
      // mirrored row as either fully absent or fully committed — never the
      // in-between gap the old-racer role above exploits.
      const mainRoot = argVal('--main-root');
      const ownRoot = argVal('--own-root');
      const id = argVal('--id');
      const holdPath = argVal('--path');
      const holder = `wt-atomic-${id}`;
      const { withHoldsLock, findForeignHolds, insertHold } = await import(WORKTREE_HOLDS_LIB_PATH);
      const { reserve } = await import(RESERVATIONS_LIB_PATH);
      const outcome = await withHoldsLock(mainRoot, async () => {
        const foreign = findForeignHolds(mainRoot, holder, [holdPath]);
        if (foreign.length > 0) return { ok: false, reason: 'foreign-hold-seen' };
        const reserveResult = await reserve(ownRoot, { agent: `agent-atomic-${id}`, cell: 'race-atomic', path: holdPath });
        if (!reserveResult.ok) return { ok: false, reason: 'local-conflict' };
        insertHold(mainRoot, { path: holdPath, holder, cell: 'race-atomic', session: `sess-atomic-${id}` });
        return { ok: true };
      });
      process.stdout.write(`${JSON.stringify({ id, ok: outcome.ok === true })}\n`);
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

// ─── fixture helpers (mirrors test_reservation_race.mjs's makeRoot) ───────

function makeRoot() {
  const dir = fs.mkdtempSync(path.join(os.tmpdir(), 'test-worktree-holds-race-'));
  fs.mkdirSync(path.join(dir, '.bee'), { recursive: true });
  return dir;
}

// ─── orchestrator ────────────────────────────────────────────────────────────

async function runOrchestrator() {
  const { readJson, writeJsonAtomic } = await import(FSUTIL_LIB_PATH);
  const failures = [];

  // (a) SAFE, distinct paths/holders — N racers through the REAL
  // mirrorHold(), each on its own path and holder, same ledger: every entry
  // must survive.
  {
    const dir = makeRoot();
    try {
      const results = await spawnRacers(RACERS, (i) => [
        '--role=distinct-racer',
        `--root=${dir}`,
        `--id=${i}`,
        `--path=src/lib/file-${i}.ts`,
      ]);
      const crashed = results.filter((r) => r.code !== 0);
      if (crashed.length) {
        failures.push(
          `(a) ${crashed.length}/${RACERS} distinct-path racer(s) crashed:\n` +
            crashed.map((c) => `  exit=${c.code} stderr=${c.stderr.trim()}`).join('\n'),
        );
      }
      const parsed = results.map((r) => lastJsonLine(r.stdout));
      const winners = parsed.filter((p) => p && p.ok === true);
      if (winners.length !== RACERS) {
        failures.push(`(a) expected all ${RACERS} distinct-path mirrorHold() calls to succeed, got ${winners.length}: ${JSON.stringify(parsed)}`);
      }

      const store = readJson(path.join(dir, '.bee', 'runtime', 'cross-worktree-holds.json'), { holds: [] });
      const active = store.holds.filter((h) => h.released_at === null && h.cell === 'race-a');
      if (active.length !== RACERS) {
        failures.push(
          `(a) LOST UPDATE: expected ${RACERS} surviving active hold(s), got ${active.length} — ` +
            `store: ${JSON.stringify(store.holds)}`,
        );
      }
      const paths = new Set(active.map((h) => h.path));
      if (paths.size !== RACERS) {
        failures.push(`(a) surviving holds must cover all ${RACERS} distinct paths, got ${paths.size}: ${[...paths].join(', ')}`);
      }
      const holders = new Set(active.map((h) => h.holder));
      if (holders.size !== RACERS) {
        failures.push(`(a) surviving holds must cover all ${RACERS} distinct holders, got ${holders.size}: ${[...holders].join(', ')}`);
      }
    } finally {
      fs.rmSync(dir, { recursive: true, force: true });
    }
  }

  // (b) DELIBERATE RED — widened-window read-check-write with NO store lock,
  // proving the pre-fix hazard is real (falsifiability: the safe result above
  // is not simply "nothing ever races here"). Runs in its own throwaway dir.
  // rel1710rc-3: a deterministic fs-based barrier (same technique as the old
  // scenario (c) fix below) proves every racer's stale read happens-before
  // every racer's write, making the lost-update collapse structural rather
  // than timing-dependent.
  {
    const dir = makeRoot();
    const barrier = path.join(dir, '.race-barrier-b-unsafe');
    fs.mkdirSync(barrier, { recursive: true });
    try {
      const results = await spawnRacers(RACERS, (i) => [
        '--role=unsafe-racer',
        `--root=${dir}`,
        `--id=${i}`,
        `--path=src/lib/unsafe-${i}.ts`,
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
      const store = readJson(path.join(dir, '.bee', 'runtime', 'cross-worktree-holds.json'), { holds: [] });
      const survived = store.holds.length;
      if (survived >= RACERS) {
        failures.push(
          `(b) DETECTOR DID NOT BITE: all ${survived}/${RACERS} unguarded racer holds survived — this negative ` +
            'control must show FEWER than N surviving holds (a lost update), or the (a) green result proves nothing.',
        );
      }
    } finally {
      fs.rmSync(dir, { recursive: true, force: true });
    }
  }

  // (c) hardening-1-7-10 D3 — SAME-PATH negative control against the OLD
  // (pre-D3) handleReservationsReserve shape: check-then-act with real gaps
  // between an unlocked findForeignHolds read, the local reserve(), and
  // mirrorHold() — never composed under one lock. RACERS distinct holders
  // (simulating RACERS distinct checkouts, each its own local reservation
  // root) race the SAME path against ONE shared mainRoot ledger. This is the
  // red_failure_evidence cell 1710-3 captures: the old flow must let 2+
  // racers land an active grant on the identical path (a double grant) —
  // exactly the hazard the D3 atomic section (scenario (d) below) closes.
  //
  // rel1710rc-2: this negative control used to rely on a fixed
  // UNSAFE_WIDEN_MS sleep to order every racer's check before every racer's
  // write, which is only a probabilistic ordering — on slow 2-core CI
  // runners the racers can serialize naturally instead (the first racer's
  // full check+reserve+mirror sequence finishes before the second racer even
  // starts its check), so only 1 of 8 wins and the detector "does not bite"
  // (observed on Linux CI, all Node versions; reproduced locally under
  // `taskset -c 0,1`). The fs-barrier below (touchFile/waitForFile, same
  // technique scripts/tests/test_claim_race.mjs's (g-unsafe) control uses) proves
  // ALL racers complete their findForeignHolds check — seeing the SAME empty
  // ledger — BEFORE ANY racer performs its reserve+mirrorHold write, making
  // the double grant structural rather than timing-dependent.
  {
    const mainRoot = makeRoot();
    const ownRoots = [];
    const barrier = path.join(mainRoot, '.race-barrier-c-old');
    fs.mkdirSync(barrier, { recursive: true });
    const contestedPath = 'src/shared/contested-old.ts';
    try {
      const results = await spawnRacers(RACERS, (i) => {
        const ownRoot = makeRoot();
        ownRoots.push(ownRoot);
        return [
          '--role=same-path-old-racer',
          `--main-root=${mainRoot}`,
          `--own-root=${ownRoot}`,
          `--id=${i}`,
          `--path=${contestedPath}`,
          `--barrier=${barrier}`,
          `--racers=${RACERS}`,
        ];
      });
      const crashed = results.filter((r) => r.code !== 0);
      if (crashed.length) {
        failures.push(
          `(c) ${crashed.length}/${RACERS} old-flow same-path racer(s) crashed:\n` +
            crashed.map((c) => `  exit=${c.code} stderr=${c.stderr.trim()}`).join('\n'),
        );
      }
      const parsed = results.map((r) => lastJsonLine(r.stdout));
      const granted = parsed.filter((p) => p && p.ok === true).length;
      if (granted < 2) {
        failures.push(
          `(c) DETECTOR DID NOT BITE: expected the OLD check-then-act flow to double-grant the same path to ` +
            `2+ of ${RACERS} racers, got ${granted} — this negative control must demonstrate the hazard D3 fixes, ` +
            'or the fixed-flow result in (d) proves nothing.',
        );
      }
      const ledger = readJson(path.join(mainRoot, '.bee', 'runtime', 'cross-worktree-holds.json'), { holds: [] });
      const activeForPath = ledger.holds.filter((h) => h.released_at === null && h.path === contestedPath);
      if (activeForPath.length < 2) {
        failures.push(`(c) expected 2+ active mirrored holds on the contested path (double grant), got ${activeForPath.length}: ${JSON.stringify(ledger.holds)}`);
      }
    } finally {
      fs.rmSync(mainRoot, { recursive: true, force: true });
      for (const dir of ownRoots) fs.rmSync(dir, { recursive: true, force: true });
    }
  }

  // (d) hardening-1-7-10 D3 FIX — same shape as (c) (RACERS distinct
  // holders/own-roots racing the SAME path against ONE shared mainRoot
  // ledger), but each racer composes findForeignHolds + reserve() +
  // insertHold() inside ONE withHoldsLock(mainRoot, ...) section — the exact
  // atomic composition bee.mjs's fixed handleReservationsReserve now uses.
  // Must yield EXACTLY ONE winner: every other racer's atomic section
  // re-checks foreign holds AFTER acquiring the lock and must see the
  // winner's already-committed row.
  {
    const mainRoot = makeRoot();
    const ownRoots = [];
    const contestedPath = 'src/shared/contested-atomic.ts';
    try {
      const results = await spawnRacers(RACERS, (i) => {
        const ownRoot = makeRoot();
        ownRoots.push(ownRoot);
        return ['--role=same-path-atomic-racer', `--main-root=${mainRoot}`, `--own-root=${ownRoot}`, `--id=${i}`, `--path=${contestedPath}`];
      });
      const crashed = results.filter((r) => r.code !== 0);
      if (crashed.length) {
        failures.push(
          `(d) ${crashed.length}/${RACERS} atomic-flow same-path racer(s) crashed:\n` +
            crashed.map((c) => `  exit=${c.code} stderr=${c.stderr.trim()}`).join('\n'),
        );
      }
      const parsed = results.map((r) => lastJsonLine(r.stdout));
      const winners = parsed.filter((p) => p && p.ok === true);
      if (winners.length !== 1) {
        failures.push(
          `(d) LOST MUTUAL EXCLUSION: expected exactly ONE winner among ${RACERS} same-path atomic racers, got ` +
            `${winners.length}: ${JSON.stringify(parsed)}`,
        );
      }
      const ledger = readJson(path.join(mainRoot, '.bee', 'runtime', 'cross-worktree-holds.json'), { holds: [] });
      const activeForPath = ledger.holds.filter((h) => h.released_at === null && h.path === contestedPath);
      if (activeForPath.length !== 1) {
        failures.push(`(d) expected exactly 1 active mirrored hold on the contested path, got ${activeForPath.length}: ${JSON.stringify(ledger.holds)}`);
      }
    } finally {
      fs.rmSync(mainRoot, { recursive: true, force: true });
      for (const dir of ownRoots) fs.rmSync(dir, { recursive: true, force: true });
    }
  }

  // (e) renewHolds — pushes mirrored_at forward for a session's still-active
  // holds so isExpired stays false past the ORIGINAL ttl window (D3's
  // heartbeat-renewal primitive). Deterministic via timestamp math, no
  // sleeping: backdate mirrored_at into the recent past (still active, not
  // yet expired), call renewHolds, then confirm the NEW mirrored_at's expiry
  // point is later than what the ORIGINAL mirrored_at's expiry would have
  // been.
  {
    const mainRoot = makeRoot();
    try {
      const { mirrorHold, renewHolds } = await import(WORKTREE_HOLDS_LIB_PATH);
      const ttlSeconds = 5;
      const session = 'sess-renew-1';
      await mirrorHold(mainRoot, { path: 'src/renew/target.ts', holder: 'wt-renew', session, cell: 'renew-cell', ttl: ttlSeconds });

      const ledgerFile = path.join(mainRoot, '.bee', 'runtime', 'cross-worktree-holds.json');
      const before = readJson(ledgerFile, { holds: [] });
      if (before.holds.length !== 1) {
        failures.push(`(e) sanity: expected exactly 1 hold before renewal, got ${before.holds.length}`);
      }

      // Backdate mirrored_at to 3s ago (still ACTIVE: 3s < 5s ttl) so the
      // hold's ORIGINAL expiry point is 2s in the future from real "now".
      const backdatedMs = Date.now() - 3000;
      before.holds[0].mirrored_at = new Date(backdatedMs).toISOString();
      writeJsonAtomic(ledgerFile, before);
      const originalExpiryMs = backdatedMs + ttlSeconds * 1000;

      const renewResult = await renewHolds(mainRoot, session);
      if (!renewResult || renewResult.renewed !== 1) {
        failures.push(`(e) expected renewHolds to renew exactly 1 hold, got ${JSON.stringify(renewResult)}`);
      }

      const after = readJson(ledgerFile, { holds: [] });
      const renewedHold = after.holds[0];
      const renewedMirroredMs = Date.parse(renewedHold.mirrored_at);
      const newExpiryMs = renewedMirroredMs + ttlSeconds * 1000;
      if (!(renewedMirroredMs > backdatedMs)) {
        failures.push(`(e) expected renewHolds to push mirrored_at forward, got backdated=${before.holds[0].mirrored_at} renewed=${renewedHold.mirrored_at}`);
      }
      if (!(newExpiryMs > originalExpiryMs)) {
        failures.push(
          `(e) expected the renewed expiry (${new Date(newExpiryMs).toISOString()}) to be later than the ` +
            `original expiry (${new Date(originalExpiryMs).toISOString()})`,
        );
      }
      // The concrete claim the cell asks for: isExpired must read false at a
      // virtual "now" that is already PAST the original expiry point, since
      // renewal moved the clock forward past it.
      const virtualNowPastOriginalExpiry = originalExpiryMs + 500;
      const stillActiveAtThatMoment = renewedHold.released_at == null && !(newExpiryMs <= virtualNowPastOriginalExpiry);
      if (!stillActiveAtThatMoment) {
        failures.push(
          '(e) expected the renewed hold to still read as NOT expired at a virtual time past the original TTL window, ' +
            `got mirrored_at=${renewedHold.mirrored_at} ttl=${ttlSeconds}s virtualNow=${new Date(virtualNowPastOriginalExpiry).toISOString()}`,
        );
      }

      // A renewal never touches a DIFFERENT session's holds.
      const untouchedResult = await renewHolds(mainRoot, 'sess-does-not-exist');
      if (!untouchedResult || untouchedResult.renewed !== 0) {
        failures.push(`(e) expected renewHolds for an unrelated session to renew 0 holds, got ${JSON.stringify(untouchedResult)}`);
      }
    } finally {
      fs.rmSync(mainRoot, { recursive: true, force: true });
    }
  }

  if (failures.length) {
    console.error('FAIL test_worktree_holds_race:');
    for (const f of failures) console.error(`  - ${f}`);
    process.exit(1);
  }

  console.log(
    `PASS test_worktree_holds_race: (a) ${RACERS} real mirrorHold() racers on ${RACERS} distinct paths/holders -> all ${RACERS} entries survived ` +
      "(no lost update); (b) deliberate-red unguarded proxy lost at least one racer's entry (detector bites, lock removed); " +
      `(c) OLD check-then-act flow double-granted the SAME path (detector bites, hazard proven real); ` +
      `(d) NEW withHoldsLock-atomic flow on the SAME path yielded exactly 1 winner among ${RACERS} racers (no double grant); ` +
      '(e) renewHolds pushed mirrored_at forward, keeping a live session\'s hold active past its original TTL window, ' +
      "without touching an unrelated session's holds.",
  );
}
