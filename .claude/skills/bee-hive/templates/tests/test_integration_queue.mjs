#!/usr/bin/env node
// test_integration_queue.mjs — lib/integration-queue.mjs contract tests
// (multisession-native-22, CONTEXT.md D8 stage 5, D9 invariant 12; advisor
// consult slice 5 conditions A/B/C — see docs/history/multisession-native/
// reports/advisor-digest-slice5.md). Same PASS/FAIL/exit-1 contract as every
// other suite here — see scripts/lib/test-fixture.mjs.
//
// Covers, all via deterministic seams (virtual `now`, controllable
// deferreds, small intervals) — NO real sleeps for the assertions that
// matter:
//   - enqueue/position/status round-trip.
//   - tryBecomeProcessor: acquire-when-free, refuse-when-held, strictly
//     positive ttl enforcement (condition C).
//   - dead-processor takeover via a virtual future `now` (no real TTL wait):
//     epoch bumps, checkProcessorLeaseEpoch flags the old epoch as drift.
//   - runThroughQueue: solo/empty-queue resolves with no sleep at all
//     (byte-identical-timing proof at this layer); two requests serialize
//     (second's runMerge only starts after the first's finishes); the
//     'integration-queue' store lock is never held while runMerge is
//     in flight (invariant 12, at this module's own boundary); the bounded
//     wait times out with a typed, unambiguous non-success result
//     (condition B, "never reads as success"); a throwing/failing runMerge
//     still releases the lease and marks the record terminal.
//
// The FULL merge-pipeline wiring (mergeFeatureWorktree's own
// checkProcessorLease/onVerifyTick hooks) is covered separately in
// test_worktree_store.mjs (worktree-store.mjs has zero lease-store
// knowledge — see that module's header — so this file never imports it);
// the real multi-OS-process CLI-level proof (two concurrent "bee worktree
// merge" invocations) lives in scripts/test_worktree_merge_queue.mjs.

import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { check, assert, assertRejects, printSummaryAndExit } from '../../../../scripts/lib/test-fixture.mjs';
import { lockFilePath } from '../lib/lock.mjs';
import {
  enqueueMergeRequest,
  queuePosition,
  queueStatusFor,
  integrationQueueDir,
  tryBecomeProcessor,
  renewProcessorLease,
  releaseProcessorLease,
  checkProcessorLeaseEpoch,
  runThroughQueue,
  IntegrationQueueError,
  DEFAULT_PROCESSOR_TTL_SECONDS,
} from '../lib/integration-queue.mjs';

function makeRoot() {
  const dir = fs.mkdtempSync(path.join(os.tmpdir(), 'bee-integration-queue-'));
  fs.mkdirSync(path.join(dir, '.bee'), { recursive: true });
  return dir;
}

function deferred() {
  let resolve;
  let reject;
  const promise = new Promise((res, rej) => {
    resolve = res;
    reject = rej;
  });
  return { promise, resolve, reject };
}

async function untilTrue(fn, { timeoutMs = 2000, intervalMs = 5 } = {}) {
  const deadline = Date.now() + timeoutMs;
  for (;;) {
    if (fn()) return;
    if (Date.now() >= deadline) throw new Error('untilTrue: condition never became true within timeout');
    // eslint-disable-next-line no-await-in-loop
    await new Promise((resolve) => setTimeout(resolve, intervalMs));
  }
}

// ─── enqueue / position / status ────────────────────────────────────────────

await check('enqueueMergeRequest assigns increasing seq and a queued status; queuePosition/queueStatusFor round-trip', async () => {
  const root = makeRoot();
  try {
    const a = await enqueueMergeRequest(root, { worktreeId: 'wt-a', requestedBySession: 'sess-1' });
    const b = await enqueueMergeRequest(root, { worktreeId: 'wt-b', feature: 'demo', requestedBySession: 'sess-2' });
    assert(a.seq === 1, `first record must be seq 1, got ${a.seq}`);
    assert(b.seq === 2, `second record must be seq 2, got ${b.seq}`);
    assert(a.status === 'queued' && b.status === 'queued', 'both records start "queued"');
    assert(b.feature === 'demo', 'feature is recorded when given');
    assert(a.feature === null, 'feature defaults to null when omitted');

    assert(fs.existsSync(integrationQueueDir(root)), 'the queue directory exists after enqueueing');

    const posA = queuePosition(root, a.seq);
    const posB = queuePosition(root, b.seq);
    assert(posA.position === 1 && posA.ahead === 0, `a must be position 1 (0 ahead), got ${JSON.stringify(posA)}`);
    assert(posB.position === 2 && posB.ahead === 1, `b must be position 2 (1 ahead), got ${JSON.stringify(posB)}`);
    assert(posA.total_open === 2 && posB.total_open === 2, 'both open records are counted in total_open');

    assert(queueStatusFor(root, a.seq).worktree_id === 'wt-a', 'queueStatusFor reads back the same record enqueueMergeRequest wrote');
    assert(queueStatusFor(root, 999) === null, 'queueStatusFor returns null for a seq that was never enqueued');
    assert(queuePosition(root, 999) === null, 'queuePosition returns null for a seq that was never enqueued');
  } finally {
    fs.rmSync(root, { recursive: true, force: true });
  }
});

await check('enqueueMergeRequest refuses (typed QUEUE_INVALID_REQUEST) without worktreeId or requestedBySession', async () => {
  const root = makeRoot();
  try {
    let threw = null;
    try {
      enqueueMergeRequest(root, { requestedBySession: 'sess-1' });
    } catch (err) {
      threw = err;
    }
    assert(threw instanceof IntegrationQueueError && threw.code === 'QUEUE_INVALID_REQUEST', `missing worktreeId must refuse typed, got ${threw}`);

    threw = null;
    try {
      enqueueMergeRequest(root, { worktreeId: 'wt-a' });
    } catch (err) {
      threw = err;
    }
    assert(threw instanceof IntegrationQueueError && threw.code === 'QUEUE_INVALID_REQUEST', `missing requestedBySession must refuse typed, got ${threw}`);
  } finally {
    fs.rmSync(root, { recursive: true, force: true });
  }
});

// ─── processor lease (condition C) ──────────────────────────────────────────

await check('tryBecomeProcessor acquires when free (epoch 1, tookOver:false); a second live attempt refuses ok:false with the holder', async () => {
  const root = makeRoot();
  try {
    const first = await tryBecomeProcessor(root, { sessionId: 'sess-1', ttlSeconds: 120 });
    assert(first.ok === true, `first acquire must succeed, got ${JSON.stringify(first)}`);
    assert(first.lease.epoch === 1, `a fresh processor lease starts at epoch 1, got ${first.lease.epoch}`);
    assert(first.tookOver === false, 'no prior record existed — this is not a takeover');

    const second = await tryBecomeProcessor(root, { sessionId: 'sess-2', ttlSeconds: 120 });
    assert(second.ok === false, `a second attempt while the first is still live must refuse, got ${JSON.stringify(second)}`);
    assert(second.holder && second.holder.session_id === 'sess-1', `the refusal must name the live holder, got ${JSON.stringify(second.holder)}`);
  } finally {
    fs.rmSync(root, { recursive: true, force: true });
  }
});

await check('tryBecomeProcessor refuses typed QUEUE_INVALID_TTL for a non-positive ttl (condition C: never-expires would deadlock the queue)', async () => {
  const root = makeRoot();
  try {
    for (const badTtl of [0, -1, NaN, Infinity]) {
      let threw = null;
      try {
        await tryBecomeProcessor(root, { sessionId: 'sess-1', ttlSeconds: badTtl });
      } catch (err) {
        threw = err;
      }
      assert(threw instanceof IntegrationQueueError && threw.code === 'QUEUE_INVALID_TTL', `ttlSeconds=${badTtl} must refuse typed QUEUE_INVALID_TTL, got ${threw}`);
    }
  } finally {
    fs.rmSync(root, { recursive: true, force: true });
  }
});

await check('dead-processor takeover via a virtual future `now` (no real sleep): epoch bumps, and checkProcessorLeaseEpoch flags the old epoch as drift', async () => {
  const root = makeRoot();
  try {
    const acquiredAtMs = Date.now() - 3600_000; // acquired "an hour ago"
    const first = await tryBecomeProcessor(root, { sessionId: 'sess-zombie', ttlSeconds: 5, now: acquiredAtMs }); // 5s ttl, long expired by real now
    assert(first.ok === true && first.lease.epoch === 1, `sanity: zombie acquires epoch 1, got ${JSON.stringify(first)}`);

    // Before the takeover, the zombie's own epoch (1) still reads as current
    // — no drift YET, because nobody has taken over.
    assert(checkProcessorLeaseEpoch(root, 1) === null, 'before any takeover, the zombie\'s own epoch (1) still matches the on-disk record');

    // A second caller, using a virtual `now` PAST the zombie's expiry (no
    // real sleep), takes over: epoch bumps from 1 to 2.
    const second = await tryBecomeProcessor(root, { sessionId: 'sess-fresh', ttlSeconds: 120, now: Date.now() });
    assert(second.ok === true, `takeover must succeed once the zombie's lease is expired, got ${JSON.stringify(second)}`);
    assert(second.tookOver === true, 'this acquire is flagged as a takeover (a prior record existed)');
    assert(second.lease.epoch === 2, `epoch must bump from the zombie's 1 to 2, got ${second.lease.epoch}`);

    // The zombie's OWN epoch (1) is now stale — checkProcessorLeaseEpoch must
    // flag it as drift; the fresh processor's epoch (2) is clean.
    const zombieDrift = checkProcessorLeaseEpoch(root, 1);
    assert(typeof zombieDrift === 'string' && /epoch 1 -> 2/.test(zombieDrift), `zombie's stale epoch 1 must read as drift naming the new epoch, got ${JSON.stringify(zombieDrift)}`);
    assert(checkProcessorLeaseEpoch(root, 2) === null, 'the fresh processor\'s own epoch (2) reads clean');
  } finally {
    fs.rmSync(root, { recursive: true, force: true });
  }
});

await check('checkProcessorLeaseEpoch flags drift when the lease record is entirely absent (released/swept, never re-acquired)', async () => {
  const root = makeRoot();
  try {
    const acquired = await tryBecomeProcessor(root, { sessionId: 'sess-1', ttlSeconds: 120 });
    await releaseProcessorLease(root, { presentedEpoch: acquired.lease.epoch });
    const drift = checkProcessorLeaseEpoch(root, acquired.lease.epoch);
    assert(typeof drift === 'string' && /disappeared/.test(drift), `an absent lease must read as drift mentioning it disappeared, got ${JSON.stringify(drift)}`);
  } finally {
    fs.rmSync(root, { recursive: true, force: true });
  }
});

await check('renewProcessorLease extends expiry and is fenced on presentedEpoch (refuses stale)', async () => {
  const root = makeRoot();
  try {
    const acquired = await tryBecomeProcessor(root, { sessionId: 'sess-1', ttlSeconds: 60 });
    const renewed = await renewProcessorLease(root, { ttlSeconds: 999, presentedEpoch: acquired.lease.epoch });
    assert(renewed.expires_at !== acquired.lease.expires_at, 'renewal pushes expires_at forward');

    await assertRejects(
      () => renewProcessorLease(root, { ttlSeconds: 60, presentedEpoch: acquired.lease.epoch - 1 }),
      'behind',
      'renewing with a stale (behind-current) presentedEpoch must refuse',
    );
  } finally {
    fs.rmSync(root, { recursive: true, force: true });
  }
});

// ─── the drainer (condition B) ──────────────────────────────────────────────

await check('runThroughQueue: an empty queue resolves on the FIRST iteration with no sleep at all (the solo/byte-identical-timing case)', async () => {
  const root = makeRoot();
  try {
    let runMergeCalls = 0;
    const startedAt = Date.now();
    const result = await runThroughQueue(root, {
      worktreeId: 'wt-solo',
      sessionId: 'sess-solo',
      pollIntervalMs: 1000, // if the loop ever actually slept, this test would take >=1s
      runMerge: async (hooks) => {
        runMergeCalls += 1;
        assert(typeof hooks.checkProcessorLease === 'function', 'runMerge is handed a checkProcessorLease hook');
        assert(typeof hooks.onVerifyTick === 'function', 'runMerge is handed an onVerifyTick hook');
        return { ok: true, merged: true };
      },
    });
    const elapsedMs = Date.now() - startedAt;
    assert(runMergeCalls === 1, `runMerge must be called exactly once for a solo request, got ${runMergeCalls}`);
    assert(result.ok === true && result.merged === true, `expected the solo runMerge result to pass through unchanged, got ${JSON.stringify(result)}`);
    assert(elapsedMs < 500, `a solo/uncontended request must never hit the poll-sleep path (1000ms interval) — took ${elapsedMs}ms`);

    const records = fs.readdirSync(integrationQueueDir(root)).filter((n) => n.endsWith('.json'));
    assert(records.length === 1, `exactly one (transient) queue record must exist, got ${records.length}`);
    assert(queueStatusFor(root, 1).status === 'done', 'the solo request\'s queue record ends "done"');
  } finally {
    fs.rmSync(root, { recursive: true, force: true });
  }
});

await check('runThroughQueue: two requests serialize — the second\'s runMerge only starts after the first\'s finishes', async () => {
  const root = makeRoot();
  try {
    const order = [];
    const gateA = deferred();

    const pA = runThroughQueue(root, {
      worktreeId: 'wt-a',
      sessionId: 'sess-a',
      pollIntervalMs: 5,
      waitBoundMs: 5000,
      runMerge: async () => {
        order.push('A-start');
        await gateA.promise;
        order.push('A-done');
        return { ok: true, merged: true };
      },
    });

    await untilTrue(() => order.includes('A-start'));
    assert(queueStatusFor(root, 1).status === 'processing', 'request A\'s own record is "processing" once its runMerge has started');

    const pB = runThroughQueue(root, {
      worktreeId: 'wt-b',
      sessionId: 'sess-b',
      pollIntervalMs: 5,
      waitBoundMs: 5000,
      runMerge: async () => {
        order.push('B-start');
        return { ok: true, merged: true };
      },
    });

    await untilTrue(() => queueStatusFor(root, 2) !== null && queueStatusFor(root, 2).status === 'queued');
    assert(!order.includes('B-start'), 'B must NOT have started while A still holds the processor lease');

    gateA.resolve();
    const [resultA, resultB] = await Promise.all([pA, pB]);
    assert(resultA.ok === true && resultB.ok === true, `both requests must eventually succeed, got A=${JSON.stringify(resultA)} B=${JSON.stringify(resultB)}`);
    assert(order.indexOf('A-done') < order.indexOf('B-start'), `expected strict serialization (A-done before B-start), got order=${JSON.stringify(order)}`);
    assert(queueStatusFor(root, 1).status === 'done' && queueStatusFor(root, 2).status === 'done', 'both queue records end "done"');
  } finally {
    fs.rmSync(root, { recursive: true, force: true });
  }
});

await check('runThroughQueue: the "integration-queue" store lock is never held while runMerge is in flight (invariant 12, this module\'s own boundary)', async () => {
  const root = makeRoot();
  try {
    const gate = deferred();
    let observedLockDuringRunMerge = 'never observed';
    const pRun = runThroughQueue(root, {
      worktreeId: 'wt-lockcheck',
      sessionId: 'sess-lockcheck',
      pollIntervalMs: 5,
      runMerge: async () => {
        observedLockDuringRunMerge = fs.existsSync(lockFilePath(root, 'integration-queue'));
        await gate.promise;
        return { ok: true, merged: true };
      },
    });
    await untilTrue(() => observedLockDuringRunMerge !== 'never observed');
    assert(observedLockDuringRunMerge === false, `the 'integration-queue' lock file must be ABSENT while runMerge is in flight, got ${observedLockDuringRunMerge}`);
    gate.resolve();
    await pRun;
  } finally {
    fs.rmSync(root, { recursive: true, force: true });
  }
});

await check('runThroughQueue: bounded wait times out with a typed, unambiguous non-success result (condition B) — never blocks past waitBoundMs', async () => {
  const root = makeRoot();
  try {
    const gateForever = deferred(); // never resolved within this test — simulates a still-busy processor
    const pBlocker = runThroughQueue(root, {
      worktreeId: 'wt-blocker',
      sessionId: 'sess-blocker',
      pollIntervalMs: 5,
      waitBoundMs: 60_000,
      runMerge: async () => {
        await gateForever.promise;
        return { ok: true, merged: true };
      },
    });
    await untilTrue(() => queueStatusFor(root, 1)?.status === 'processing');

    const startedAt = Date.now();
    const timedOut = await runThroughQueue(root, {
      worktreeId: 'wt-impatient',
      sessionId: 'sess-impatient',
      pollIntervalMs: 10,
      waitBoundMs: 80,
    runMerge: async () => {
        throw new Error('runMerge must NEVER be called on the timeout path — the merge did not get its turn');
      },
    });
    const elapsedMs = Date.now() - startedAt;

    assert(timedOut.ok === false, `a timed-out request must report ok:false, got ${JSON.stringify(timedOut)}`);
    assert(timedOut.merged === false, 'a timed-out request must report merged:false — never readable as success');
    assert(timedOut.code === 'INTEGRATION_QUEUE_TIMEOUT', `expected code INTEGRATION_QUEUE_TIMEOUT, got ${timedOut.code}`);
    assert(/did NOT run/.test(timedOut.message), `the message must unambiguously say the merge did NOT run, got: ${timedOut.message}`);
    assert(timedOut.queue && timedOut.queue.seq === 2 && typeof timedOut.queue.position === 'number', `expected a queue position on timeout, got ${JSON.stringify(timedOut.queue)}`);
    assert(elapsedMs < 2000, `the bounded wait (80ms) must never balloon into a long real wait — took ${elapsedMs}ms`);
    assert(queueStatusFor(root, 2).status === 'queued', 'the timed-out request\'s own record is left "queued" (still eligible for a later retry), never silently dropped');

    gateForever.resolve();
    await pBlocker;
  } finally {
    fs.rmSync(root, { recursive: true, force: true });
  }
});

await check('runThroughQueue: a throwing runMerge still marks the record "failed" and releases the lease for the next caller', async () => {
  const root = makeRoot();
  try {
    let threw = null;
    try {
      await runThroughQueue(root, {
        worktreeId: 'wt-throws',
        sessionId: 'sess-throws',
        runMerge: async () => {
          throw new Error('boom');
        },
      });
    } catch (err) {
      threw = err;
    }
    assert(threw instanceof Error && threw.message === 'boom', `the original error must propagate to the caller, got ${threw}`);
    assert(queueStatusFor(root, 1).status === 'failed', 'a throwing runMerge marks its own record "failed"');

    // The lease must have been released — a fresh caller can become
    // processor immediately, with no leftover lock/lease from the throw.
    const next = await tryBecomeProcessor(root, { sessionId: 'sess-next', ttlSeconds: 60 });
    assert(next.ok === true, `the processor lease must be free after a throwing runMerge released it, got ${JSON.stringify(next)}`);
  } finally {
    fs.rmSync(root, { recursive: true, force: true });
  }
});

await check('runThroughQueue: a runMerge that resolves ok:false (e.g. MERGE_CONFLICT) marks the record "failed" too, and still releases the lease', async () => {
  const root = makeRoot();
  try {
    const result = await runThroughQueue(root, {
      worktreeId: 'wt-conflict',
      sessionId: 'sess-conflict',
      runMerge: async () => ({ ok: false, code: 'MERGE_CONFLICT', merged: false }),
    });
    assert(result.ok === false && result.code === 'MERGE_CONFLICT', 'the original ok:false result passes through unchanged');
    assert(queueStatusFor(root, 1).status === 'failed', 'an ok:false runMerge result marks its own record "failed"');

    const next = await tryBecomeProcessor(root, { sessionId: 'sess-next', ttlSeconds: 60 });
    assert(next.ok === true, 'the processor lease is free after an ok:false runMerge outcome');
  } finally {
    fs.rmSync(root, { recursive: true, force: true });
  }
});

await check('sanity: DEFAULT_PROCESSOR_TTL_SECONDS is strictly positive (condition C invariant, exported so callers can rely on the same default)', () => {
  assert(Number.isFinite(DEFAULT_PROCESSOR_TTL_SECONDS) && DEFAULT_PROCESSOR_TTL_SECONDS > 0, `DEFAULT_PROCESSOR_TTL_SECONDS must be strictly positive, got ${DEFAULT_PROCESSOR_TTL_SECONDS}`);
});

printSummaryAndExit();
