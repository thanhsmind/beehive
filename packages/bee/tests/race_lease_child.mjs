#!/usr/bin/env node
// race_lease_child.mjs — self-contained multi-worker race orchestrator for
// lease-store.mjs (multisession-native-23, CONTEXT.md D9 invariants 5/6).
//
// Same HARNESS CONSTRAINT as race_claims_child.mjs (see that file's own
// header): test-fixture.mjs's check() runner is synchronous and never
// awaits, so a genuine race cannot live inside a check() body — it lives
// HERE, in a self-contained orchestrator that starts its own
// barrier-synchronized Worker racers, asserts internally, prints ONE
// summary line, and exits 0 (pass) / 1 (fail). test_msn_invariants.mjs runs
// this through the shared module Worker per scenario and asserts exit code
// + summary line — same reuse shape race_claims_child.mjs already
// established for test_claims.mjs.
//
// Two scenarios:
//   'same-resource'      (invariant 6): N racers target the EXACT SAME
//                         {type:'path', id} lease resource — exactly one
//                         O_EXCL winner per round, everyone else typed
//                         LEASE_HELD.
//   'disjoint-resources'  (invariant 5): 2 racers target DIFFERENT lease
//                         resources under the SAME barrier trip — BOTH must
//                         win every round (acquireLeases's create path never
//                         shares a lock across resources, so two unrelated
//                         resources can never contend).
//
// Barrier files (go-*) are the correctness mechanism, same discipline as
// race_claims_child.mjs: any setTimeout below is a scheduling nudge that
// lets forked Workers reach the barrier before it trips, never load-bearing
// for the actual exclusion/non-contention result.

import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { Worker, workerData } from 'node:worker_threads';
import { acquireLeases } from '../lib/lease-store.mjs';

const self = fileURLToPath(import.meta.url);

if (workerData?.raceRole) {
  runRacer(workerData.raceRole);
} else {
  main();
}

function spinUntil(goFile) {
  while (!fs.existsSync(goFile)) { /* spin — no sleep, trips the instant the barrier file appears */ }
}

function runRacer(role) {
  spinUntil(role.goFile);
  try {
    acquireLeases(role.root, [
      { type: 'path', id: role.resourceId, mode: 'write', workflow_id: role.workflowId, session_id: role.sessionId, workspace_id: 'main', epoch: 1, ttl: 60 },
    ]);
    process.exit(0); // won
  } catch (error) {
    if (error && error.code === 'LEASE_HELD') process.exit(1); // lost cleanly
    process.exit(2); // unexpected — a bug
  }
}

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

// Scenario: invariant 6 — N racers all target the SAME path resource.
// Truth: exactly one O_EXCL winner every round, never zero, never more than one.
async function sameResource() {
  const root = freshRoot('bee-race-lease-same-');
  const ROUNDS = 6;
  const RACERS = 5;
  const failures = [];
  for (let r = 0; r < ROUNDS; r += 1) {
    const resourceId = `race/shared-${r}.txt`;
    const goFile = path.join(root, `go-${r}`);
    const children = [];
    for (let i = 0; i < RACERS; i += 1) {
      children.push(startRacer({ root, resourceId, workflowId: `wf-same-${r}`, sessionId: `race-sess-${r}-${i}`, goFile }));
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
    console.log(`FAIL  same-resource: ${failures.join(' | ')}`);
    return false;
  }
  console.log(`PASS  same-resource: ${ROUNDS} rounds x ${RACERS} racers on the SAME lease resource, exactly one O_EXCL winner every round`);
  return true;
}

// Scenario: invariant 5 — 2 racers target DIFFERENT path resources under the
// SAME barrier trip. Truth: BOTH win every round — a held lease on one
// resource never blocks acquiring an unrelated one (zero lock contention,
// proven under genuine concurrent Worker execution, not merely a sequential
// same-process call).
async function disjointResources() {
  const root = freshRoot('bee-race-lease-disjoint-');
  const ROUNDS = 6;
  const failures = [];
  for (let r = 0; r < ROUNDS; r += 1) {
    const goFile = path.join(root, `go-${r}`);
    const children = [
      startRacer({ root, resourceId: `race/disjoint-a-${r}.txt`, workflowId: `wf-disjoint-a-${r}`, sessionId: `race-sess-a-${r}`, goFile }),
      startRacer({ root, resourceId: `race/disjoint-b-${r}.txt`, workflowId: `wf-disjoint-b-${r}`, sessionId: `race-sess-b-${r}`, goFile }),
    ];
    const exits = Promise.all(children.map(waitExit));
    await sleep(120);
    fs.writeFileSync(goFile, '1');
    const codes = await exits;
    const winners = codes.filter((c) => c === 0).length;
    if (winners !== 2) {
      failures.push(`round ${r}: winners=${winners} codes=${JSON.stringify(codes)} — a disjoint resource contended with an unrelated one`);
    }
  }
  fs.rmSync(root, { recursive: true, force: true });
  if (failures.length) {
    console.log(`FAIL  disjoint-resources: ${failures.join(' | ')}`);
    return false;
  }
  console.log(`PASS  disjoint-resources: ${ROUNDS} rounds x 2 racers on DISJOINT lease resources under the same barrier trip, both won every round — zero lock contention`);
  return true;
}

async function main() {
  const scenario = process.argv[2];
  const scenarios = {
    'same-resource': sameResource,
    'disjoint-resources': disjointResources,
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
