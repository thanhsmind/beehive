#!/usr/bin/env node
// race_claims_child.mjs — self-contained multi-worker race orchestrator for
// claims.mjs (fresh-session-handoff fsh-2).
//
// HARNESS CONSTRAINT (validating repair, cell-review CRITICAL): test_lib.mjs's
// check() runner is synchronous and never awaits — an async fn passed to
// check() would report PASS before its assertions ran. So the ENTIRE race
// lives HERE, inside a self-contained orchestrator that:
//   - is invoked with a scenario argument (process.argv[2]),
//   - starts its own barrier-synchronized Worker racers (adapting the pattern proven
//     in .bee/spikes/fresh-session-handoff/probe_atomic_claim.mjs),
//   - asserts winner counts / typed-failure shapes internally,
//   - prints ONE summary line, and
//   - exits 0 (pass) / 1 (fail).
// test_lib.mjs runs this through the shared module Worker per scenario and
// asserts exit code + summary line.
//
// Barrier files (go-*, stop-*) are the correctness mechanism for who-wins-the-
// race. Any setTimeout below is either (a) a scheduling nudge that lets forked
// children reach the barrier before it is tripped — never load-bearing for
// the actual exclusion result (the probe's 150ms nudge is the allowed
// pattern) — or (b) an inherent part of the heartbeat/TTL staleness window
// itself in the sweep-heartbeat scenario, where wall-clock duration IS the
// thing under test (staleness is defined in terms of elapsed time), used with
// a generous safety margin over the heartbeat interval.

import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { Worker, workerData } from 'node:worker_threads';
import {
  createSession,
  claimCellFile,
  readClaim,
  adoptClaim,
  sweepExpiredClaims,
  heartbeatSession,
  claimPath,
} from '../lib/claims.mjs';
import { writeJsonAtomic } from '../lib/fsutil.mjs';

const self = fileURLToPath(import.meta.url);

// ─── racer entry point (this file re-execs itself in a Worker) ─────────────

if (workerData?.raceRole) {
  runRacer(workerData.raceRole);
} else {
  main();
}

function runRacer(role) {
  switch (role.kind) {
    case 'claim':
      return raceClaim(role);
    case 'adopt':
      return raceAdopt(role);
    case 'steal':
      return raceSteal(role);
    case 'sweep':
      return raceSweep(role);
    case 'heartbeat':
      return raceHeartbeat(role);
    default:
      process.exit(2);
  }
}

function spinUntil(goFile) {
  // Barrier spin (no sleep): every racer trips the instant the file appears.
  while (!fs.existsSync(goFile)) { /* spin */ }
}

function busySleepMs(ms) {
  const end = Date.now() + ms;
  while (Date.now() < end) { /* pace without yielding a real timer */ }
}

// scenario: claim-contention — N racers, one cell, exactly one 'wx' winner.
function raceClaim({ root, sessionId, cellId, goFile, ttl }) {
  spinUntil(goFile);
  const result = claimCellFile(root, sessionId, cellId, ttl);
  if (result.ok === true) process.exit(0); // won
  if (result.ok === false && result.code === 'CLAIMED') process.exit(1); // lost cleanly
  process.exit(2); // unexpected
}

// scenario: adoption-steal — the sole adopter racing against thieves.
function raceAdopt({ root, cellId, sessionId, goFile }) {
  spinUntil(goFile);
  const result = adoptClaim(root, cellId, sessionId);
  process.exit(result.ok === true ? 0 : 2);
}

function raceSteal({ root, sessionId, cellId, goFile, ttl }) {
  spinUntil(goFile);
  const result = claimCellFile(root, sessionId, cellId, ttl);
  if (result.ok === false && result.code === 'CLAIMED') process.exit(1); // expected: steal denied
  process.exit(result.ok === true ? 3 : 2); // 3 = BUG (steal succeeded), 2 = unexpected code
}

// scenario: sweep-heartbeat — sweepers hammering while a heartbeat renewer
// keeps one claim alive; nobody may reclaim a live claim.
async function raceSweep({ root, cellId, goFile, stopFile, staleSeconds }) {
  spinUntil(goFile);
  let sawErroneousSweep = false;
  while (!fs.existsSync(stopFile)) {
    const result = await sweepExpiredClaims(root, { staleSeconds });
    if (result.ok && result.swept.includes(cellId)) {
      sawErroneousSweep = true;
      break;
    }
    busySleepMs(10);
  }
  process.exit(sawErroneousSweep ? 1 : 0);
}

function raceHeartbeat({ root, sessionId, goFile, stopFile, intervalMs }) {
  spinUntil(goFile);
  let ok = true;
  while (!fs.existsSync(stopFile)) {
    const result = heartbeatSession(root, sessionId);
    if (!result.ok) {
      ok = false;
      break;
    }
    busySleepMs(intervalMs);
  }
  process.exit(ok ? 0 : 2);
}

// ─── orchestrator (parent) side ─────────────────────────────────────────────

function startRacer(role) {
  return new Worker(self, { workerData: { raceRole: role } });
}

function waitExit(child) {
  return new Promise((resolve) => child.on('exit', (code) => resolve(code)));
}

function sleep(ms) {
  return new Promise((resolve) => setTimeout(resolve, ms));
}

function freshRoot(prefix) {
  return fs.mkdtempSync(path.join(os.tmpdir(), prefix));
}

// Scenario 1: N workers race claimCellFile for one cell across
// repeated rounds. Truth: exactly one winner each round.
async function claimContention() {
  const root = freshRoot('bee-race-claim-');
  const ROUNDS = 8;
  const RACERS = 5;
  const failures = [];
  for (let r = 0; r < ROUNDS; r += 1) {
    const cellId = `race-cell-${r}`;
    const goFile = path.join(root, `go-${r}`);
    const children = [];
    for (let i = 0; i < RACERS; i += 1) {
      const sessionId = `race-sess-${r}-${i}`;
      const created = createSession(root, { id: sessionId });
      if (!created.ok) failures.push(`round ${r}: session ${sessionId} setup failed`);
      children.push(startRacer({ kind: 'claim', root, sessionId, cellId, goFile, ttl: 60 }));
    }
    const exits = Promise.all(children.map(waitExit));
    await sleep(120); // scheduling nudge only — the goFile barrier below is the correctness mechanism
    fs.writeFileSync(goFile, '1');
    const codes = await exits;
    const winners = codes.filter((c) => c === 0).length;
    const unexpected = codes.filter((c) => c !== 0 && c !== 1).length;
    if (winners !== 1 || unexpected !== 0) {
      failures.push(`round ${r}: winners=${winners} unexpected=${unexpected} codes=${JSON.stringify(codes)}`);
    }
  }
  fs.rmSync(root, { recursive: true, force: true });
  if (failures.length) {
    console.log(`FAIL  claim-contention: ${failures.join(' | ')}`);
    return false;
  }
  console.log(`PASS  claim-contention: ${ROUNDS} rounds x ${RACERS} racers, exactly one O_EXCL winner every round`);
  return true;
}

// Scenario 2: while a fresh session adopts a handoff-carried claim, a third
// process (repeatedly, several of them) attempts to claim the same cell.
// Truth: the adopter always wins; every thief loses with typed CLAIMED —
// the claim file is never observably absent (in-place rewrite, never
// delete-then-recreate).
async function adoptionSteal() {
  const root = freshRoot('bee-race-adopt-');
  const ROUNDS = 6;
  const THIEVES = 3;
  const failures = [];
  for (let r = 0; r < ROUNDS; r += 1) {
    const cellId = `adopt-cell-${r}`;
    const ownerSession = `owner-${r}`;
    const adopterSession = `adopter-${r}`;
    createSession(root, { id: ownerSession });
    createSession(root, { id: adopterSession });
    const initial = claimCellFile(root, ownerSession, cellId, 3600);
    if (!initial.ok) {
      failures.push(`round ${r}: setup claim failed`);
      continue;
    }
    const goFile = path.join(root, `go-${r}`);
    const children = [startRacer({ kind: 'adopt', root, cellId, sessionId: adopterSession, goFile })];
    for (let t = 0; t < THIEVES; t += 1) {
      const thiefSession = `thief-${r}-${t}`;
      createSession(root, { id: thiefSession });
      children.push(startRacer({ kind: 'steal', root, sessionId: thiefSession, cellId, goFile, ttl: 60 }));
    }
    const exits = Promise.all(children.map(waitExit));
    await sleep(120);
    fs.writeFileSync(goFile, '1');
    const codes = await exits;
    const adoptCode = codes[0];
    const thiefCodes = codes.slice(1);
    const bugSteals = thiefCodes.filter((c) => c === 3).length;
    const unexpected = thiefCodes.filter((c) => c !== 1 && c !== 3).length;
    const finalOwner = readClaim(root, cellId)?.session;
    if (adoptCode !== 0 || bugSteals > 0 || unexpected > 0 || finalOwner !== adopterSession) {
      failures.push(
        `round ${r}: adoptCode=${adoptCode} bugSteals=${bugSteals} unexpected=${unexpected} finalOwner=${finalOwner} thiefCodes=${JSON.stringify(thiefCodes)}`,
      );
    }
  }
  fs.rmSync(root, { recursive: true, force: true });
  if (failures.length) {
    console.log(`FAIL  adoption-steal: ${failures.join(' | ')}`);
    return false;
  }
  console.log(
    `PASS  adoption-steal: ${ROUNDS} rounds x ${THIEVES} thieves, adoption always wins and every steal attempt loses with typed CLAIMED`,
  );
  return true;
}

// Scenario 3: concurrent sweepExpiredClaims and heartbeat renewal. Truth:
// while the owner heartbeats, sweepers (hammering concurrently) never
// reclaim a TTL-expired-but-heartbeat-fresh claim (pattern 20260710) — AND,
// as a no-op guard, once the heartbeat stops and staleness truly elapses, a
// direct sweep call DOES reclaim it (so "never swept" can't be a vacuous
// pass from a broken/no-op sweep).
async function sweepHeartbeat() {
  const root = freshRoot('bee-race-sweep-');
  const ROUNDS = 2;
  const SWEEPERS = 3;
  const STALE_SECONDS = 1;
  const WINDOW_MS = 500; // duration of the concurrent race window
  const HEARTBEAT_INTERVAL_MS = 60; // well under STALE_SECONDS*1000 — generous margin
  const failures = [];
  for (let r = 0; r < ROUNDS; r += 1) {
    const cellId = `sweep-cell-${r}`;
    const ownerSession = `sweep-owner-${r}`;
    createSession(root, { id: ownerSession });
    const claimed = claimCellFile(root, ownerSession, cellId, 60);
    if (!claimed.ok) {
      failures.push(`round ${r}: setup claim failed`);
      continue;
    }
    // Backdate claimed_at so TTL is already expired — only the live heartbeat
    // still protects this claim from sweepExpiredClaims.
    const claim = readClaim(root, cellId);
    writeJsonAtomic(claimPath(root, cellId), {
      ...claim,
      claimed_at: new Date(Date.now() - 7200 * 1000).toISOString(),
      ttl_seconds: 60,
    });

    const goFile = path.join(root, `go-${r}`);
    const stopFile = path.join(root, `stop-${r}`);
    const children = [
      startRacer({ kind: 'heartbeat', root, sessionId: ownerSession, goFile, stopFile, intervalMs: HEARTBEAT_INTERVAL_MS }),
    ];
    for (let s = 0; s < SWEEPERS; s += 1) {
      children.push(startRacer({ kind: 'sweep', root, cellId, goFile, stopFile, staleSeconds: STALE_SECONDS }));
    }
    const exits = Promise.all(children.map(waitExit));
    await sleep(120);
    fs.writeFileSync(goFile, '1');
    await sleep(WINDOW_MS);
    fs.writeFileSync(stopFile, '1');
    const codes = await exits;
    const heartbeatCode = codes[0];
    const sweepCodes = codes.slice(1);
    const erroneous = sweepCodes.filter((c) => c !== 0).length;
    const stillPresent = fs.existsSync(claimPath(root, cellId));
    if (heartbeatCode !== 0 || erroneous > 0 || !stillPresent) {
      failures.push(
        `round ${r}: heartbeatCode=${heartbeatCode} erroneousSweeps=${erroneous} stillPresent=${stillPresent} sweepCodes=${JSON.stringify(sweepCodes)}`,
      );
      continue;
    }
    // Negative control: heartbeat has stopped; once staleness truly elapses,
    // a direct sweep call must reclaim it — proving sweep is not a no-op.
    await sleep(STALE_SECONDS * 1000 + 300);
    const finalSweep = await sweepExpiredClaims(root, { staleSeconds: STALE_SECONDS });
    if (!finalSweep.ok || !finalSweep.swept.includes(cellId) || fs.existsSync(claimPath(root, cellId))) {
      failures.push(`round ${r}: negative control failed — sweep did not reclaim after heartbeat stopped: ${JSON.stringify(finalSweep)}`);
    }
  }
  fs.rmSync(root, { recursive: true, force: true });
  if (failures.length) {
    console.log(`FAIL  sweep-heartbeat: ${failures.join(' | ')}`);
    return false;
  }
  console.log(
    `PASS  sweep-heartbeat: ${ROUNDS} rounds x ${SWEEPERS} sweepers, a live heartbeat is never reclaimed and a truly stale claim IS reclaimed`,
  );
  return true;
}

async function main() {
  const scenario = process.argv[2];
  const scenarios = {
    'claim-contention': claimContention,
    'adoption-steal': adoptionSteal,
    'sweep-heartbeat': sweepHeartbeat,
  };
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
