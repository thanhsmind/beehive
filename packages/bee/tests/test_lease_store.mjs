#!/usr/bin/env node
// test_lease_store.mjs — lib/lease-store.mjs contract tests
// (multisession-native-11, CONTEXT.md D4: "sharded leases replace
// reservations.json"). Same PASS/FAIL/exit-1 contract as every other suite
// here — see scripts/lib/test-fixture.mjs.
//
// Covers: acquire/release/renew/sweep round-trip, partial-acquire rollback
// (zero residue), two disjoint resources never contending, hash-sort
// determinism (deadlock-free regardless of caller order), TTL
// never-expires semantics, a typed LEASE_HELD refusal naming the
// conflicting holder, and a static source-scan proving this module never
// imports session/state/reservations code or names any lock other than its
// own per-resource `lease:<hash>` ones (the structural isolation half of
// the C4 pattern workflow-store.mjs already established).

import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { check, assert, assertThrows, printSummaryAndExit } from '../../../scripts/lib/test-fixture.mjs';
import {
  acquireLeases,
  releaseLease,
  renewLease,
  renewLeasesBySession,
  sweepExpiredLeases,
  listLeases,
  leasesRoot,
  leaseCellsDir,
  leasePathsDir,
  LeaseStoreError,
} from '../lib/lease-store.mjs';

function makeRoot() {
  return fs.mkdtempSync(path.join(os.tmpdir(), 'bee-lease-store-'));
}

function baseRequest(overrides = {}) {
  return {
    type: 'cell',
    id: 'demo-cell-1',
    mode: 'write',
    workflow_id: 'wf-demo',
    session_id: 'session-a',
    workspace_id: 'workspace-a',
    epoch: 1,
    ...overrides,
  };
}

// ─── acquire/release/renew/sweep round-trip ─────────────────────────────────

await check('acquireLeases writes a lease file under leaseCellsDir; releaseLease removes it; re-acquire succeeds after release', async () => {
  const root = makeRoot();
  try {
    const [leased] = acquireLeases(root, [baseRequest()]);
    assert(leased.resource === 'cell:demo-cell-1', 'record carries the type-prefixed resource key');
    assert(leased.mode === 'write', 'record carries mode');
    assert(leased.workflow_id === 'wf-demo', 'record carries workflow_id');
    assert(leased.session_id === 'session-a', 'record carries session_id');
    assert(leased.workspace_id === 'workspace-a', 'record carries workspace_id');
    assert(leased.epoch === 1, 'record carries epoch verbatim');
    assert(typeof leased.acquired_at === 'string' && !Number.isNaN(Date.parse(leased.acquired_at)), 'acquired_at is a timestamp');
    assert(typeof leased.expires_at === 'string' && !Number.isNaN(Date.parse(leased.expires_at)), 'expires_at is a timestamp for a positive ttl');

    const file = path.join(leaseCellsDir(root), 'demo-cell-1.json');
    assert(fs.existsSync(file), 'lease record exists on disk at the documented cells/<cell-id>.json path');

    const released = await releaseLease(root, { type: 'cell', id: 'demo-cell-1' });
    assert(released.ok === true && released.released === true, 'releaseLease reports success for an existing lease');
    assert(!fs.existsSync(file), 'released lease file is gone from disk');

    const secondReleased = await releaseLease(root, { type: 'cell', id: 'demo-cell-1' });
    assert(secondReleased.ok === true && secondReleased.released === false, 'releasing an already-absent lease is idempotent, not an error');

    const [reacquired] = acquireLeases(root, [baseRequest({ session_id: 'session-b' })]);
    assert(reacquired.session_id === 'session-b', 'the resource is free again after release and a new session can acquire it');
  } finally {
    fs.rmSync(root, { recursive: true, force: true });
  }
});

await check('acquireLeases for a path request writes a lease under leasePathsDir, keyed by the resource hash', () => {
  const root = makeRoot();
  try {
    const [leased] = acquireLeases(root, [baseRequest({ type: 'path', id: './src/api/router.ts' })]);
    assert(leased.resource === 'path:src/api/router.ts', 'path id is canonicalized (leading "./" stripped) before becoming the resource key');
    const entries = fs.readdirSync(leasePathsDir(root));
    assert(entries.length === 1 && entries[0].endsWith('.json'), 'exactly one lease file exists under paths/');
    assert(!fs.existsSync(leaseCellsDir(root)), 'a path-only acquire never creates the cells/ directory');
  } finally {
    fs.rmSync(root, { recursive: true, force: true });
  }
});

await check('renewLease extends expires_at; renewLeasesBySession renews every lease owned by that session and skips others', async () => {
  const root = makeRoot();
  try {
    const [a] = acquireLeases(root, [baseRequest({ id: 'cell-a', session_id: 'session-a', ttl: 60 })]);
    const [b] = acquireLeases(root, [baseRequest({ id: 'cell-b', session_id: 'session-b', ttl: 60 })]);

    const renewed = await renewLease(root, { type: 'cell', id: 'cell-a' }, { ttl: 999, now: Date.parse(a.acquired_at) + 500 });
    assert(Date.parse(renewed.expires_at) > Date.parse(a.expires_at), 'renewLease pushes expires_at further out');
    assert(renewed.resource === a.resource && renewed.session_id === a.session_id, 'renewLease preserves every other field');

    const before = { a: JSON.parse(fs.readFileSync(path.join(leaseCellsDir(root), 'cell-a.json'), 'utf8')), b };
    const result = await renewLeasesBySession(root, 'session-a', { ttl: 1234, now: Date.now() + 10_000 });
    assert(result.ok === true && result.renewed === 1, `expected exactly 1 renewed for session-a, got ${JSON.stringify(result)}`);

    const afterA = JSON.parse(fs.readFileSync(path.join(leaseCellsDir(root), 'cell-a.json'), 'utf8'));
    const afterB = JSON.parse(fs.readFileSync(path.join(leaseCellsDir(root), 'cell-b.json'), 'utf8'));
    assert(Date.parse(afterA.expires_at) > Date.parse(before.a.expires_at), 'session-a\'s own lease was renewed');
    assert(afterB.expires_at === b.expires_at, 'session-b\'s lease was left untouched by a session-a renewal sweep');
  } finally {
    fs.rmSync(root, { recursive: true, force: true });
  }
});

await check('renewLease throws typed LEASE_MISSING for an absent lease; renewLeasesBySession for an unknown session is a no-op', async () => {
  const root = makeRoot();
  try {
    let threw = null;
    try {
      await renewLease(root, { type: 'cell', id: 'ghost-cell' });
    } catch (error) {
      threw = error;
    }
    assert(threw instanceof LeaseStoreError && threw.code === 'LEASE_MISSING', `expected LEASE_MISSING, got ${threw && threw.code}`);

    const result = await renewLeasesBySession(root, 'nobody-home');
    assert(result.ok === true && result.renewed === 0, 'renewing an unknown session renews nothing and never throws');
  } finally {
    fs.rmSync(root, { recursive: true, force: true });
  }
});

await check('sweepExpiredLeases deletes only expired leases; never-expiring (ttl<=0) leases are never swept (TTL semantics)', () => {
  const root = makeRoot();
  try {
    acquireLeases(root, [baseRequest({ id: 'expired-cell', ttl: 60 })], { now: Date.now() - 3600_000 }); // acquired an hour ago, 60s ttl -> long expired
    acquireLeases(root, [baseRequest({ id: 'fresh-cell', ttl: 3600 })]); // acquired now, 1h ttl -> still active
    acquireLeases(root, [baseRequest({ id: 'forever-cell', ttl: 0 })]); // ttl<=0 -> never expires
    acquireLeases(root, [baseRequest({ id: 'forever-cell-negative', ttl: -5 })]); // negative ttl -> also never expires

    const forever = JSON.parse(fs.readFileSync(path.join(leaseCellsDir(root), 'forever-cell.json'), 'utf8'));
    assert(forever.expires_at === null, 'ttl<=0 stores expires_at: null, matching reservations.mjs\'s non-positive-ttl-never-expires semantics');
    const foreverNegative = JSON.parse(fs.readFileSync(path.join(leaseCellsDir(root), 'forever-cell-negative.json'), 'utf8'));
    assert(foreverNegative.expires_at === null, 'a negative ttl also stores expires_at: null');

    const swept = sweepExpiredLeases(root);
    assert(swept === 1, `expected exactly 1 swept lease, got ${swept}`);
    assert(!fs.existsSync(path.join(leaseCellsDir(root), 'expired-cell.json')), 'the expired lease is gone');
    assert(fs.existsSync(path.join(leaseCellsDir(root), 'fresh-cell.json')), 'the still-active lease survives the sweep');
    assert(fs.existsSync(path.join(leaseCellsDir(root), 'forever-cell.json')), 'the never-expiring (ttl=0) lease survives the sweep');
    assert(fs.existsSync(path.join(leaseCellsDir(root), 'forever-cell-negative.json')), 'the never-expiring (negative ttl) lease survives the sweep');

    const { leases, skipped } = listLeases(root);
    assert(leases.length === 3 && skipped.length === 0, `listLeases reports the 3 survivors, got ${leases.length} leases / ${skipped.length} skipped`);
  } finally {
    fs.rmSync(root, { recursive: true, force: true });
  }
});

// ─── typed refusal names the holder (D4) ────────────────────────────────────

await check('acquireLeases on an already-held resource throws typed LEASE_HELD naming the conflicting holder', () => {
  const root = makeRoot();
  try {
    acquireLeases(root, [baseRequest({ id: 'contended-cell', session_id: 'first-session' })]);
    let threw = null;
    try {
      acquireLeases(root, [baseRequest({ id: 'contended-cell', session_id: 'second-session' })]);
    } catch (error) {
      threw = error;
    }
    assert(threw instanceof LeaseStoreError && threw.code === 'LEASE_HELD', `expected LEASE_HELD, got ${threw && threw.code}`);
    assert(threw.resource === 'cell:contended-cell', 'refusal names the contended resource');
    assert(threw.holder && threw.holder.session_id === 'first-session', 'refusal names the actual holder session, not the requester');
    assert(threw.message.includes('first-session'), `refusal message should name the holder — got "${threw.message}"`);
  } finally {
    fs.rmSync(root, { recursive: true, force: true });
  }
});

// ─── partial-acquire rollback (D4 must_have) ────────────────────────────────

await check('partial acquire rolls back fully: the 2nd of 3 leases collides -> ALL are rolled back, zero residue', () => {
  const root = makeRoot();
  try {
    // Pre-seed a lease for "middle-cell" so a batch acquire touching
    // first/middle/last collides on the middle one, no matter the sort order.
    acquireLeases(root, [baseRequest({ id: 'middle-cell', session_id: 'blocker' })]);

    let threw = null;
    try {
      acquireLeases(root, [
        baseRequest({ id: 'first-cell', session_id: 'batch' }),
        baseRequest({ id: 'middle-cell', session_id: 'batch' }),
        baseRequest({ id: 'last-cell', session_id: 'batch' }),
      ]);
    } catch (error) {
      threw = error;
    }
    assert(threw instanceof LeaseStoreError && threw.code === 'LEASE_HELD', `expected the batch to refuse with LEASE_HELD, got ${threw && threw.code}`);

    // RED-FIRST NEGATIVE CONTROL (captured, not left to assertion alone):
    // this exact scenario was run once against a lease-store.mjs variant
    // with the `for (const won of acquired) rollbackOne(won.file, won.record);`
    // line in acquireLeases' collision branch commented out — that variant
    // left first-cell.json on disk after the same LEASE_HELD throw (a
    // negative-space residue), which the assertions below caught
    // immediately (fs.existsSync(firstCellFile) was true). Restoring the
    // rollback line makes them pass. This proves the assertions are
    // load-bearing, not vacuously true.
    const firstCellFile = path.join(leaseCellsDir(root), 'first-cell.json');
    const lastCellFile = path.join(leaseCellsDir(root), 'last-cell.json');
    assert(!fs.existsSync(firstCellFile), 'first-cell (acquired before the collision) must be rolled back — zero residue');
    assert(!fs.existsSync(lastCellFile), 'last-cell must never have been created past the collision point');

    // The pre-existing blocker lease is untouched by the failed batch's rollback.
    const middle = JSON.parse(fs.readFileSync(path.join(leaseCellsDir(root), 'middle-cell.json'), 'utf8'));
    assert(middle.session_id === 'blocker', 'the original blocking lease survives the failed batch\'s rollback unchanged');

    // The resource is fully free again for a fresh single acquire.
    const [refirst] = acquireLeases(root, [baseRequest({ id: 'first-cell', session_id: 'someone-else' })]);
    assert(refirst.session_id === 'someone-else', 'first-cell is acquirable again after the rolled-back batch — no orphaned lease left behind');
  } finally {
    fs.rmSync(root, { recursive: true, force: true });
  }
});

// ─── two disjoint resources never contend ───────────────────────────────────

await check('two disjoint resources never contend: a held cell lease never blocks acquiring an unrelated cell or path lease', () => {
  const root = makeRoot();
  try {
    acquireLeases(root, [baseRequest({ id: 'held-cell', session_id: 'holder' })]);

    const [otherCell] = acquireLeases(root, [baseRequest({ id: 'unrelated-cell', session_id: 'someone-else' })]);
    assert(otherCell.resource === 'cell:unrelated-cell', 'an unrelated cell resource acquires cleanly while held-cell stays leased');

    const [otherPath] = acquireLeases(root, [baseRequest({ type: 'path', id: 'src/unrelated.ts', session_id: 'someone-else' })]);
    assert(otherPath.resource === 'path:src/unrelated.ts', 'an unrelated path resource acquires cleanly while held-cell stays leased');

    const held = JSON.parse(fs.readFileSync(path.join(leaseCellsDir(root), 'held-cell.json'), 'utf8'));
    assert(held.session_id === 'holder', 'held-cell itself was never touched by acquiring the two unrelated resources');
  } finally {
    fs.rmSync(root, { recursive: true, force: true });
  }
});

// ─── hash-sort determinism (D4 must_have: deadlock-free regardless of order) ─

await check('hash-sorted multi-resource acquire is deadlock-free: [A,B] and [B,A] both process resources in the identical sorted order', async () => {
  const root = makeRoot();
  try {
    const orderForward = [];
    acquireLeases(
      root,
      [baseRequest({ id: 'order-a', session_id: 'caller-1' }), baseRequest({ id: 'order-b', session_id: 'caller-1' })],
      { _orderSeam: (resourceKey) => orderForward.push(resourceKey) },
    );
    await releaseLease(root, { type: 'cell', id: 'order-a' });
    await releaseLease(root, { type: 'cell', id: 'order-b' });

    const orderReversed = [];
    acquireLeases(
      root,
      [baseRequest({ id: 'order-b', session_id: 'caller-2' }), baseRequest({ id: 'order-a', session_id: 'caller-2' })],
      { _orderSeam: (resourceKey) => orderReversed.push(resourceKey) },
    );

    assert(orderForward.length === 2 && orderReversed.length === 2, 'both calls processed exactly 2 resources');
    assert(
      JSON.stringify(orderForward) === JSON.stringify(orderReversed),
      `expected identical processing order regardless of caller request order — forward=${JSON.stringify(orderForward)} reversed=${JSON.stringify(orderReversed)}`,
    );

    // NEGATIVE CONTROL, captured: this assertion was run once with the
    // `.sort((a, b) => (a.hash < b.hash ? -1 : a.hash > b.hash ? 1 : 0))`
    // call in acquireLeases replaced with a no-op identity pass-through
    // (`[...normalized]` unsorted) — orderForward then read
    // ["cell:order-a","cell:order-b"] while orderReversed read
    // ["cell:order-b","cell:order-a"], so the JSON.stringify equality above
    // failed immediately. Restoring the sort call makes it pass, proving
    // this assertion is load-bearing rather than vacuous.
  } finally {
    fs.rmSync(root, { recursive: true, force: true });
  }
});

// ─── request validation ─────────────────────────────────────────────────────

await check('acquireLeases refuses malformed requests with typed LEASE_INVALID_REQUEST (missing fields, bad type, duplicate resource in one batch)', () => {
  const root = makeRoot();
  try {
    assertThrows(() => acquireLeases(root, []), 'non-empty array', 'refuses an empty requests array');
    assertThrows(() => acquireLeases(root, [baseRequest({ type: 'bogus' })]), 'type must be one of', 'refuses an unknown resource type');
    assertThrows(() => acquireLeases(root, [baseRequest({ mode: '' })]), 'mode is required', 'refuses a missing mode');
    assertThrows(() => acquireLeases(root, [baseRequest({ workflow_id: '' })]), 'workflow_id is required', 'refuses a missing workflow_id');
    assertThrows(() => acquireLeases(root, [baseRequest({ epoch: 'not-a-number' })]), 'epoch must be a finite number', 'refuses a non-numeric epoch');
    assertThrows(
      () => acquireLeases(root, [baseRequest({ id: 'dup-cell' }), baseRequest({ id: 'dup-cell' })]),
      'more than once',
      'refuses the same resource requested twice in one batch',
    );
    assert(!fs.existsSync(leaseCellsDir(root)), 'no lease file is ever created by a batch that is refused up front');
  } finally {
    fs.rmSync(root, { recursive: true, force: true });
  }
});

// ─── multisession-native-12: fence_epoch enforcement (D4/D9 invariant 10) ──
// lease-store.mjs has stored `epoch` verbatim from the caller since
// multisession-native-11 (ad19826) but never compared it against anything —
// enforcement is this cell's job. `renewLease`/`releaseLease` now accept an
// optional `presentedEpoch`: omitted, both are byte-unchanged from before
// this cell (no production caller exists yet — lease-store.mjs is not wired
// into any CLI verb until msn-16 — so this is a safe, non-breaking addition
// tested here at the primitive level). Present it stale (< the resource's
// stored epoch) and both refuse typed LEASE_FENCE_STALE instead of acting.
// There is no forced-takeover primitive in this module (by design — see the
// module header's condition-A/F notes), so a "takeover" is proven the way
// this module actually supports it: release, then a fresh acquire with a
// caller-bumped epoch (the caller, e.g. a future adopt-a-lease flow, owns
// deciding the new epoch value — this module only stores and now enforces
// it).

await check('invariant 10: renewLease refuses typed LEASE_FENCE_STALE when the presented epoch is behind the stored epoch; the stored epoch succeeds', async () => {
  const root = makeRoot();
  try {
    acquireLeases(root, [baseRequest({ id: 'fence-cell', session_id: 'holder-a', epoch: 1, ttl: 3600 })]);
    await releaseLease(root, { type: 'cell', id: 'fence-cell' });
    // "Takeover": a new holder acquires the same resource with a bumped epoch.
    acquireLeases(root, [baseRequest({ id: 'fence-cell', session_id: 'holder-b', epoch: 2, ttl: 3600 })]);

    let threw = null;
    try {
      await renewLease(root, { type: 'cell', id: 'fence-cell' }, { presentedEpoch: 1 });
    } catch (error) {
      threw = error;
    }
    assert(threw instanceof LeaseStoreError && threw.code === 'LEASE_FENCE_STALE', `expected typed LEASE_FENCE_STALE, got ${threw && threw.code}`);
    assert(threw.resource === 'cell:fence-cell', 'refusal names the resource');

    const untouched = JSON.parse(fs.readFileSync(path.join(leaseCellsDir(root), 'fence-cell.json'), 'utf8'));
    assert(untouched.epoch === 2 && untouched.session_id === 'holder-b', 'a fenced-stale renew never touches the lease record');

    const renewed = await renewLease(root, { type: 'cell', id: 'fence-cell' }, { presentedEpoch: 2, ttl: 999 });
    assert(renewed.epoch === 2, 'presenting the stored (current) epoch renews successfully');
  } finally {
    fs.rmSync(root, { recursive: true, force: true });
  }
});

await check('renewLease: omitting presentedEpoch is byte-unchanged legacy behavior (never fenced, even against a bumped epoch)', async () => {
  const root = makeRoot();
  try {
    acquireLeases(root, [baseRequest({ id: 'legacy-cell', session_id: 'holder-a', epoch: 1, ttl: 3600 })]);
    await releaseLease(root, { type: 'cell', id: 'legacy-cell' });
    acquireLeases(root, [baseRequest({ id: 'legacy-cell', session_id: 'holder-b', epoch: 5, ttl: 3600 })]);

    const renewed = await renewLease(root, { type: 'cell', id: 'legacy-cell' }, { ttl: 999 });
    assert(renewed.epoch === 5, 'a legacy (no presentedEpoch) renew succeeds regardless of epoch — enforcement engages only when an epoch is presented');
  } finally {
    fs.rmSync(root, { recursive: true, force: true });
  }
});

await check('invariant 10: releaseLease refuses typed LEASE_FENCE_STALE when the presented epoch is behind the stored epoch — the lease file is never removed on a fenced refusal; the stored epoch succeeds and removes it', async () => {
  const root = makeRoot();
  try {
    acquireLeases(root, [baseRequest({ id: 'fence-release-cell', session_id: 'holder-a', epoch: 3, ttl: 3600 })]);
    const file = path.join(leaseCellsDir(root), 'fence-release-cell.json');

    let threw = null;
    try {
      await releaseLease(root, { type: 'cell', id: 'fence-release-cell' }, { presentedEpoch: 2 });
    } catch (error) {
      threw = error;
    }
    assert(threw instanceof LeaseStoreError && threw.code === 'LEASE_FENCE_STALE', `expected typed LEASE_FENCE_STALE, got ${threw && threw.code}`);
    assert(fs.existsSync(file), 'a fenced-stale release must never remove the lease file');

    const released = await releaseLease(root, { type: 'cell', id: 'fence-release-cell' }, { presentedEpoch: 3 });
    assert(released.ok === true && released.released === true, 'presenting the stored (current) epoch releases successfully');
    assert(!fs.existsSync(file), 'lease file removed on a correctly-fenced release');
  } finally {
    fs.rmSync(root, { recursive: true, force: true });
  }
});

await check('releaseLease: a presentedEpoch against an already-absent lease is a no-op (ok:true, released:false), never a fence error — nothing to fence against', async () => {
  const root = makeRoot();
  try {
    const result = await releaseLease(root, { type: 'cell', id: 'never-existed' }, { presentedEpoch: 99 });
    assert(result.ok === true && result.released === false, `releasing an absent resource must stay idempotent even with a presentedEpoch, got ${JSON.stringify(result)}`);
  } finally {
    fs.rmSync(root, { recursive: true, force: true });
  }
});

// ─── C4 structural guarantee: this module is a session/state-free leaf ─────

await check(
  'lease-store.mjs never imports claims.mjs/state.mjs/reservations.mjs and never names the sessions/state/reservations locks (C4 structural isolation)',
  () => {
    const modulePath = fileURLToPath(new URL('../lib/lease-store.mjs', import.meta.url));
    const source = fs.readFileSync(modulePath, 'utf8');
    assert(!/from ['"]\.\/claims\.mjs['"]/.test(source), 'must never import claims.mjs (session records)');
    assert(!/from ['"]\.\/state\.mjs['"]/.test(source), 'must never import state.mjs (transitively pulls session code)');
    assert(!/from ['"]\.\/reservations\.mjs['"]/.test(source), 'must never import reservations.mjs (itself imports claims.mjs — see module header)');
    assert(!/withStoreLock\(\s*root\s*,\s*['"]sessions['"]/.test(source), 'must never acquire the global "sessions" lock');
    assert(!/withStoreLock\(\s*root\s*,\s*['"]state['"]/.test(source), 'must never acquire the global "state" lock');
    assert(!/withStoreLock\(\s*root\s*,\s*['"]reservations['"]/.test(source), 'must never acquire the global "reservations" lock (must_haves prohibition: no global reservations lock taken)');
    assert(/leasesRoot\(root\)/.test(source), 'the leases directory is built through the single leasesRoot(root) seam (slice-4 re-root stays a one-line change)');
  },
);

await check('leasesRoot/leaseCellsDir/leasePathsDir all nest under .bee/runtime/leases (documented placement, condition A)', () => {
  const root = makeRoot();
  try {
    assert(leasesRoot(root) === path.join(root, '.bee', 'runtime', 'leases'), 'leasesRoot matches the documented .bee/runtime/leases placement');
    assert(leaseCellsDir(root) === path.join(leasesRoot(root), 'cells'), 'leaseCellsDir nests under leasesRoot');
    assert(leasePathsDir(root) === path.join(leasesRoot(root), 'paths'), 'leasePathsDir nests under leasesRoot');
  } finally {
    fs.rmSync(root, { recursive: true, force: true });
  }
});

// ─── multisession-native-13: kind field (D4) ────────────────────────────────
// Forward groundwork only in this cell (see module header) — no production
// consumer reads this field yet (msn-16's full shim is what wires guards.mjs
// onto this store). Proven here at the store level: default, explicit, and
// invalid-value handling.

await check('acquireLeases: kind defaults to "lease" when omitted; an explicit "intent" is stamped verbatim; an invalid kind refuses typed LEASE_INVALID_REQUEST before any file is created', async () => {
  const root = makeRoot();
  try {
    const [defaulted] = acquireLeases(root, [baseRequest({ id: 'kind-default' })]);
    assert(defaulted.kind === 'lease', `omitted kind defaults to 'lease', got ${JSON.stringify(defaulted)}`);

    const [declared] = acquireLeases(root, [baseRequest({ id: 'kind-intent', kind: 'intent' })]);
    assert(declared.kind === 'intent', `explicit kind: 'intent' is stamped, got ${JSON.stringify(declared)}`);

    assertThrows(
      () => acquireLeases(root, [baseRequest({ id: 'kind-bad', kind: 'exclusive' })]),
      'kind must be one of',
      'an invalid kind value refuses LEASE_INVALID_REQUEST',
    );
    assert(!fs.existsSync(path.join(leaseCellsDir(root), 'kind-bad.json')), 'a refused invalid-kind acquire never creates a lease file');
  } finally {
    fs.rmSync(root, { recursive: true, force: true });
  }
});

printSummaryAndExit();
