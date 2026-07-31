#!/usr/bin/env node
// test_reservation_race.mjs — proves reservations.mjs's reserve()/release()/
// sweepExpired() (bee.mjs handleReservationsReserve/.../Sweep) are race-safe
// end to end (CONTEXT.md D2+D3, cell msh-3).
//
// multisession-native-16 (D4/D5, advisor consult slice 3 conditions B/C/E):
// reserve() is now a shim over lease-store.mjs's SHARDED per-resource lease
// files (`.bee/runtime/leases/paths/<hash>.json`), never a single shared
// `.bee/reservations.json` read-check-write under a store-wide
// withStoreLock('reservations') lock (D2's original mechanism, retired by
// this cell's own hot-path requirement: "no global 'reservations' store
// lock ... on reserve/release/renew"). Race-safety for the identical-path
// case (scenario (b) below, the one that matters most) now comes from
// lease-store.mjs's acquireLeases: an O_EXCL ('wx') file CREATE, which the
// filesystem itself guarantees only one racer can ever win for a given path
// — see reservations.mjs's own module header ("overlap-conflict race
// window") for the one narrowed guarantee this migration accepts (two
// racers on DIFFERENT-but-overlapping, non-identical paths).
//
// Self-contained child-orchestrator (fork racers, assert internally, exit
// 0/1) invoked by ONE blocking row (critical-patterns 20260714 "Async
// assertions under a non-awaiting runner pass vacuously" — fs writes are
// synchronous inside one process, so "concurrent" async calls fired inside a
// single event loop never exercise a genuine race). Every racer is its own OS
// process (this same file, re-invoked with --role=), same shape as
// scripts/tests/test_claim_race.mjs / scripts/tests/test_store_lock.mjs.
//
// Three scenarios:
//   (a) SAFE, distinct paths — N racers through the REAL reserve() on N
//       distinct paths, same cell: every racer's row must survive (N active
//       rows at the end, read through listReservations() — the legacy
//       reservations.json is a rebuildable projection nothing writes
//       synchronously anymore, see reservations.mjs's module header). Each
//       racer's lease now lives at its OWN sharded file, so there is
//       structurally nothing left to lose an update ON for this scenario —
//       it stays green as a regression guard, not because a lock protects a
//       shared file that no longer exists.
//   (b) SAFE, same path — N racers through the REAL reserve() on ONE shared
//       path: exactly one ok:true winner, N-1 typed conflict refusals naming
//       the holder agent — this is the scenario acquireLeases' O_EXCL
//       protection is actually load-bearing for.
//   (c) DELIBERATE RED (falsifiability, critical-patterns 20260714) —
//       redesigned for the sharded world: N racers through a test-owned
//       proxy that mimics the PRE-O_EXCL shape (check if the SAME shared
//       lease file exists, WIDEN the window with a short sleep, then
//       unconditionally overwrite it — no exclusive-create in front of it).
//       Demonstrates the double-grant hazard acquireLeases' 'wx' flag exists
//       to kill: MORE THAN ONE racer believes it exclusively acquired the
//       identical resource, because the naive existence check and the write
//       are not atomic. Runs in its own throwaway temp dir — the real (safe)
//       store is never touched by this scenario, so nothing needs
//       "restoring" afterward.

import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { spawn } from 'node:child_process';
import { fileURLToPath, pathToFileURL } from 'node:url';

const __filename = fileURLToPath(import.meta.url);
const REPO_ROOT = path.join(path.dirname(__filename), '..', '..');
// pathToFileURL: dynamic import() of a bare absolute path breaks on win32
// (a `d:\...` specifier parses as protocol "d:") — a file:// URL string is
// valid on every platform.
const RESERVATIONS_LIB_PATH = pathToFileURL(path.join(REPO_ROOT, '.bee', 'bin', 'lib', 'reservations.mjs')).href;

const RACERS = 8;
const UNSAFE_WIDEN_MS = 30;

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

// ─── worker roles (each its own OS process) ─────────────────────────────────

async function runWorker(workerRole) {
  try {
    if (workerRole === 'distinct-racer' || workerRole === 'same-path-racer') {
      const root = argVal('--root');
      const cellId = argVal('--cell');
      const id = argVal('--id');
      const reservedPath = argVal('--path');
      const { reserve } = await import(RESERVATIONS_LIB_PATH);
      const result = await reserve(root, { agent: `worker-${id}`, cell: cellId, path: reservedPath });
      process.stdout.write(`${JSON.stringify(result)}\n`);
      process.exit(0);
    } else if (workerRole === 'unsafe-racer') {
      const root = argVal('--root');
      const id = argVal('--id');
      const reservedPath = argVal('--path'); // every unsafe racer targets the SAME shared path
      const crypto = await import('node:crypto');
      // multisession-native-16: reserve() no longer read-check-writes ONE
      // shared reservations.json (that whole store-wide hazard is gone —
      // there is no longer any shared mutable file for distinct-path racers
      // to lose an update on at all, since each resource now lives at its
      // own path). The hazard this negative control must now reproduce is
      // lease-store.mjs's OWN protection: acquireLeases' O_EXCL ('wx') create
      // is what makes two racers targeting the SAME resource key mutually
      // exclusive (proven safe by scenario (b) above, through the REAL
      // reserve()). This mimics the PRE-O_EXCL shape instead: a naive
      // check-if-file-exists, WIDEN the window, then unconditionally
      // overwrite — no exclusivity at all. Each racer reports what IT
      // observed at check time (`believedWon: true` means it saw no
      // pre-existing file) — a TOCTOU race lets MORE THAN ONE racer believe
      // it exclusively acquired the same resource, the double-grant hazard
      // O_EXCL exists to kill.
      const canonical = reservedPath.replace(/\\/g, '/').replace(/^\.\/+/, '').replace(/\/+$/, '');
      const resourceKey = `path:${canonical}`;
      const hash = crypto.createHash('sha256').update(resourceKey).digest('hex');
      const leasePathsDir = path.join(root, '.bee', 'runtime', 'leases', 'paths');
      fs.mkdirSync(leasePathsDir, { recursive: true });
      const file = path.join(leasePathsDir, `${hash}.json`);
      const believedWon = !fs.existsSync(file);
      await new Promise((resolve) => setTimeout(resolve, UNSAFE_WIDEN_MS));
      fs.writeFileSync(
        file,
        `${JSON.stringify(
          {
            resource: resourceKey,
            mode: 'write',
            workflow_id: 'race-c',
            session_id: `unsafe-sess-${id}`,
            workspace_id: `agent:worker-unsafe-${id}`,
            epoch: 0,
            acquired_at: new Date().toISOString(),
            expires_at: null,
            kind: 'lease',
          },
          null,
          2,
        )}\n`,
      );
      process.stdout.write(`${JSON.stringify({ id, path: reservedPath, believedWon })}\n`);
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

// ─── fixture helpers (mirrors test_claim_race.mjs's makeRoot) ──────────────

function makeRoot() {
  const dir = fs.mkdtempSync(path.join(os.tmpdir(), 'test-reservation-race-'));
  fs.mkdirSync(path.join(dir, '.bee'), { recursive: true });
  return dir;
}

// ─── orchestrator ────────────────────────────────────────────────────────────

async function runOrchestrator() {
  // multisession-native-16: reserve() is now a shim over lease-store.mjs's
  // sharded per-resource files — `.bee/reservations.json` is a rebuildable
  // PROJECTION nothing writes synchronously (see reservations.mjs's own
  // module header), so scenarios (a)/(b) below must read the LIVE store
  // through listReservations(), never the legacy file directly.
  const { listReservations } = await import(RESERVATIONS_LIB_PATH);
  const failures = [];

  // (a) SAFE, distinct paths — N racers through the REAL reserve(), each on
  // its own path, same cell, same store: every row must survive.
  {
    const dir = makeRoot();
    try {
      const results = await spawnRacers(RACERS, (i) => [
        '--role=distinct-racer',
        `--root=${dir}`,
        '--cell=race-a',
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
        failures.push(`(a) expected all ${RACERS} distinct-path reserves to succeed, got ${winners.length}: ${JSON.stringify(parsed)}`);
      }

      const active = listReservations(dir, { activeOnly: true }).filter((r) => r.cell === 'race-a');
      if (active.length !== RACERS) {
        failures.push(
          `(a) LOST UPDATE: expected ${RACERS} surviving active rows, got ${active.length} — ` +
            `store: ${JSON.stringify(active)}`,
        );
      }
      const paths = new Set(active.map((r) => r.path));
      if (paths.size !== RACERS) {
        failures.push(`(a) surviving rows must cover all ${RACERS} distinct paths, got ${paths.size}: ${[...paths].join(', ')}`);
      }
    } finally {
      fs.rmSync(dir, { recursive: true, force: true });
    }
  }

  // (b) SAFE, same path — N racers through the REAL reserve() all targeting
  // ONE shared path: exactly one winner, N-1 typed conflict refusals.
  {
    const dir = makeRoot();
    try {
      const results = await spawnRacers(RACERS, (i) => [
        '--role=same-path-racer',
        `--root=${dir}`,
        '--cell=race-b',
        `--id=${i}`,
        '--path=src/api/router.ts',
      ]);
      const crashed = results.filter((r) => r.code !== 0);
      if (crashed.length) {
        failures.push(
          `(b) ${crashed.length}/${RACERS} same-path racer(s) crashed:\n` +
            crashed.map((c) => `  exit=${c.code} stderr=${c.stderr.trim()}`).join('\n'),
        );
      }
      const parsed = results.map((r) => lastJsonLine(r.stdout));
      const winners = parsed.filter((p) => p && p.ok === true);
      const losers = parsed.filter((p) => p && p.ok === false);

      if (winners.length !== 1) {
        failures.push(`(b) expected exactly 1 winner on the shared path, got ${winners.length}: ${JSON.stringify(winners)}`);
      }
      if (losers.length !== RACERS - 1) {
        failures.push(`(b) expected ${RACERS - 1} typed conflicts, got ${losers.length}: ${JSON.stringify(parsed)}`);
      }
      const winnerAgent = winners[0] && winners[0].reservation && winners[0].reservation.agent;
      for (const loser of losers) {
        if (!Array.isArray(loser.conflicts) || loser.conflicts.length === 0) {
          failures.push(`(b) loser must carry a non-empty typed conflicts array, got ${JSON.stringify(loser)}`);
        } else if (!loser.conflicts.some((c) => c.agent === winnerAgent)) {
          failures.push(`(b) loser's conflicts must name the actual winner "${winnerAgent}", got ${JSON.stringify(loser.conflicts)}`);
        }
      }

      const active = listReservations(dir, { activeOnly: true }).filter((r) => r.cell === 'race-b');
      if (active.length !== 1) {
        failures.push(`(b) exactly one row should survive on the shared path, got ${active.length}: ${JSON.stringify(active)}`);
      }
    } finally {
      fs.rmSync(dir, { recursive: true, force: true });
    }
  }

  // (c) DELIBERATE RED — widened-window check-then-write with NO O_EXCL
  // exclusivity, proving lease-store.mjs's real protection (acquireLeases'
  // 'wx' create, exercised for real by scenario (b) above) actually matters
  // (falsifiability: the safe single-winner result above is not simply
  // "nothing ever races here"). Runs in its own throwaway dir. Redesigned
  // for multisession-native-16 (see the unsafe-racer worker role's own
  // comment): the old shape raced N racers appending to ONE shared
  // reservations.json, which no longer exists as reserve()'s store at all;
  // this now races N racers on the SAME lease resource key with a naive
  // (non-exclusive) check-then-write, proving a TOCTOU double-grant is real
  // without O_EXCL.
  {
    const dir = makeRoot();
    try {
      const results = await spawnRacers(RACERS, (i) => [
        '--role=unsafe-racer',
        `--root=${dir}`,
        `--id=${i}`,
        '--path=src/lib/unsafe-shared.ts',
      ]);
      const crashed = results.filter((r) => r.code !== 0);
      if (crashed.length) {
        failures.push(
          `(c) ${crashed.length}/${RACERS} unsafe racer(s) crashed:\n` +
            crashed.map((c) => `  exit=${c.code} stderr=${c.stderr.trim()}`).join('\n'),
        );
      }
      const parsed = results.map((r) => lastJsonLine(r.stdout));
      const believedWonCount = parsed.filter((p) => p && p.believedWon === true).length;
      if (believedWonCount < 2) {
        failures.push(
          `(c) DETECTOR DID NOT BITE: only ${believedWonCount}/${RACERS} unguarded racers believed they exclusively ` +
            'acquired the SAME shared resource — this negative control must show MORE THAN ONE racer winning the ' +
            'naive check (a double grant), or the (b) green single-winner result proves nothing.',
        );
      }
    } finally {
      fs.rmSync(dir, { recursive: true, force: true });
    }
  }

  if (failures.length) {
    console.error('FAIL test_reservation_race:');
    for (const f of failures) console.error(`  - ${f}`);
    process.exit(1);
  }

  console.log(
    `PASS test_reservation_race: (a) ${RACERS} real-path racers on ${RACERS} distinct paths -> all ${RACERS} rows survived ` +
      `(no lost update); (b) ${RACERS} real-path racers on one shared path -> exactly 1 winner, ${RACERS - 1} typed conflicts ` +
      "naming the winner; (c) deliberate-red unguarded proxy lost at least one racer's row (detector bites, lock removed).",
  );
}
