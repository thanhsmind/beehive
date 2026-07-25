#!/usr/bin/env node
// test_workspace_store.mjs — lib/workspace-store.mjs contract tests
// (multisession-native-19, CONTEXT.md D2/D3: workspace registry + single
// write owner). Same PASS/FAIL/exit-1 contract as every other suite here —
// see scripts/lib/test-fixture.mjs. Mirrors test_workflow_store.mjs's /
// test_lease_store.mjs's per-test mkdtemp fixture style.

import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { check, assert, printSummaryAndExit } from '../../../scripts/lib/test-fixture.mjs';
import {
  WorkspaceStoreError,
  workspacesDir,
  workspacePath,
  registerWorkspace,
  unregisterWorkspace,
  readWorkspace,
  listWorkspaces,
  claimWriteOwnership,
  attachWorkspace,
  releaseWriteOwnership,
} from '../lib/workspace-store.mjs';
import { decideWorktreeStore } from '../lib/worktree-store.mjs';
import { createSession, heartbeatStale, DEFAULT_HEARTBEAT_STALE_SECONDS } from '../lib/claims.mjs';

function makeRoot() {
  return fs.mkdtempSync(path.join(os.tmpdir(), 'bee-workspace-store-'));
}

function mainReq(overrides = {}) {
  return { id: 'main', type: 'main', root: '/repo', ...overrides };
}

// This module's typed refusals carry a human-readable `.message` that does
// not always repeat the raw `.code` token verbatim (matching the codebase's
// existing convention — see e.g. lease-store.mjs's LEASE_HELD message) — so
// asserting on the CODE means catching and inspecting `.code` directly
// rather than test-fixture.mjs's assertRejects (which substring-matches
// `.message`).
async function assertRejectsCode(fn, expectedCode, message) {
  try {
    await fn();
  } catch (error) {
    assert(
      error instanceof WorkspaceStoreError && error.code === expectedCode,
      `${message} — expected typed WorkspaceStoreError code "${expectedCode}", got ${error instanceof WorkspaceStoreError ? `code "${error.code}"` : String(error)}`,
    );
    return;
  }
  throw new Error(`${message} — expected a rejection, none thrown`);
}

// ─── registration ────────────────────────────────────────────────────────

await check('registerWorkspace writes .bee/runtime/workspaces/<id>.json with the full schema; readWorkspace round-trips it', async () => {
  const root = makeRoot();
  try {
    const record = await registerWorkspace(root, { id: 'main', type: 'main', root: '/checkout', branch: 'main', base_sha: 'deadbeef' });
    assert(record.id === 'main', 'record carries id');
    assert(record.type === 'main', 'record carries type');
    assert(record.root === '/checkout', 'record carries root');
    assert(record.branch === 'main', 'record carries branch');
    assert(record.base_sha === 'deadbeef', 'record carries base_sha');
    assert(record.write_owner_session === null, 'fresh registration starts with no write owner');
    assert(record.fence_epoch === 0, 'fresh registration starts at fence_epoch 0');
    assert(Array.isArray(record.attached_sessions) && record.attached_sessions.length === 0, 'fresh registration starts with no attached sessions');
    assert(typeof record.created_at === 'string' && !Number.isNaN(Date.parse(record.created_at)), 'created_at is a timestamp');
    assert(fs.existsSync(workspacePath(root, 'main')), 'record exists on disk at the documented path');
    assert(workspacePath(root, 'main') === path.join(workspacesDir(root), 'main.json'), 'workspacePath matches workspacesDir/<id>.json');

    const readBack = readWorkspace(root, 'main');
    assert(readBack.id === 'main' && readBack.root === '/checkout', 'readWorkspace round-trips the record');
  } finally {
    fs.rmSync(root, { recursive: true, force: true });
  }
});

await check('registerWorkspace is IDEMPOTENT — a second register for the same id returns the EXISTING record unchanged, never throws, never overwrites', async () => {
  const root = makeRoot();
  try {
    const first = await registerWorkspace(root, mainReq({ branch: 'main', base_sha: 'sha-1' }));
    // A second register with DIFFERENT field values must still return the
    // ORIGINAL record — proves "auto-registered lazily on first touch" is
    // safe to call on every session-init, not just the first.
    const second = await registerWorkspace(root, mainReq({ root: '/different-root', branch: 'other', base_sha: 'sha-2' }));
    assert(second.root === '/repo', 'second register does not overwrite root');
    assert(second.branch === 'main', 'second register does not overwrite branch');
    assert(second.base_sha === 'sha-1', 'second register does not overwrite base_sha');
    assert(second.created_at === first.created_at, 'created_at is untouched by a repeat register');
  } finally {
    fs.rmSync(root, { recursive: true, force: true });
  }
});

await check('unregisterWorkspace removes the record; idempotent on an already-absent id (ok:true, removed:false, never an error)', async () => {
  const root = makeRoot();
  try {
    await registerWorkspace(root, mainReq());
    const removed = await unregisterWorkspace(root, 'main');
    assert(removed.ok === true && removed.removed === true, 'first unregister removes the file');
    assert(!fs.existsSync(workspacePath(root, 'main')), 'record is gone from disk');

    const removedAgain = await unregisterWorkspace(root, 'main');
    assert(removedAgain.ok === true && removedAgain.removed === false, 'a second unregister on an absent record is a no-op, not an error');

    const neverExisted = await unregisterWorkspace(root, 'wt-never-existed');
    assert(neverExisted.ok === true && neverExisted.removed === false, 'unregistering an id that never existed is likewise a no-op');
  } finally {
    fs.rmSync(root, { recursive: true, force: true });
  }
});

await check('readWorkspace refuses typed WORKSPACE_MISSING for an unregistered id; listWorkspaces fail-opens to {workspaces:[],skipped:[]} with none registered and lists every registered record', async () => {
  const root = makeRoot();
  try {
    let threw = null;
    try {
      readWorkspace(root, 'ghost');
    } catch (err) {
      threw = err;
    }
    assert(threw instanceof WorkspaceStoreError && threw.code === 'WORKSPACE_MISSING', 'readWorkspace throws typed WORKSPACE_MISSING, never guesses');

    const empty = listWorkspaces(root);
    assert(Array.isArray(empty.workspaces) && empty.workspaces.length === 0, 'no runtime/workspaces dir yet -> empty list, never a throw');

    await registerWorkspace(root, mainReq());
    await registerWorkspace(root, { id: 'wt-1', type: 'worktree', root: '/wt-1' });
    const listed = listWorkspaces(root);
    const ids = listed.workspaces.map((w) => w.id).sort();
    assert(ids.length === 2 && ids[0] === 'main' && ids[1] === 'wt-1', `listWorkspaces returns both records, got ${JSON.stringify(ids)}`);
  } finally {
    fs.rmSync(root, { recursive: true, force: true });
  }
});

// ─── write ownership: O_EXCL-fenced claim ───────────────────────────────

await check('claimWriteOwnership refuses typed WORKSPACE_NOT_REGISTERED for an unregistered id — the prohibition: no unregistered workspace ever gains write ownership', async () => {
  const root = makeRoot();
  try {
    await assertRejectsCode(
      () => claimWriteOwnership(root, 'never-registered', 'sess-a'),
      'WORKSPACE_NOT_REGISTERED',
      'claiming ownership of an unregistered workspace must refuse, never auto-register',
    );
  } finally {
    fs.rmSync(root, { recursive: true, force: true });
  }
});

await check('claimWriteOwnership: first claimant becomes owner (fence_epoch bumps to 1); the SAME session re-claiming is idempotent (owner:true, reclaimed:false, epoch unchanged)', async () => {
  const root = makeRoot();
  try {
    await registerWorkspace(root, mainReq());
    const first = await claimWriteOwnership(root, 'main', 'sess-a');
    assert(first.ok === true && first.owner === true, 'first claim wins ownership');
    assert(first.reclaimed === false, 'a fresh claim (no prior owner) is not a "reclaim"');
    assert(first.record.write_owner_session === 'sess-a', 'record now names the owner');
    assert(first.record.fence_epoch === 1, 'fence_epoch bumps from 0 to 1 on first ownership');

    const again = await claimWriteOwnership(root, 'main', 'sess-a');
    assert(again.ok === true && again.owner === true, 'same-session re-claim still succeeds');
    assert(again.record.fence_epoch === 1, 'idempotent re-claim by the SAME owner never bumps fence_epoch again');
  } finally {
    fs.rmSync(root, { recursive: true, force: true });
  }
});

await check('exactly-one-owner (red-first): two sessions race claimWriteOwnership on the SAME workspace through the real per-workspace lock — exactly one wins, the loser gets a typed WORKSPACE_OWNED refusal naming the winner', async () => {
  const root = makeRoot();
  try {
    await registerWorkspace(root, mainReq());
    const results = await Promise.allSettled([
      claimWriteOwnership(root, 'main', 'sess-a'),
      claimWriteOwnership(root, 'main', 'sess-b'),
    ]);
    const fulfilled = results.filter((r) => r.status === 'fulfilled');
    const rejected = results.filter((r) => r.status === 'rejected');
    assert(fulfilled.length === 1, `exactly one racer must win ownership, got ${fulfilled.length} winners`);
    assert(rejected.length === 1, `exactly one racer must lose, got ${rejected.length} losers`);
    const winnerSession = fulfilled[0].value.record.write_owner_session;
    assert(['sess-a', 'sess-b'].includes(winnerSession), 'the winner is one of the two racing sessions');
    const loserError = rejected[0].reason;
    assert(loserError instanceof WorkspaceStoreError && loserError.code === 'WORKSPACE_OWNED', 'the loser gets a typed WORKSPACE_OWNED refusal, never a silent failure');
    assert(loserError.holder === winnerSession, `the refusal names the ACTUAL winner as holder, got "${loserError.holder}" vs winner "${winnerSession}"`);

    const finalRecord = readWorkspace(root, 'main');
    assert(finalRecord.write_owner_session === winnerSession, 'on-disk record agrees with the winner — exactly one owner persists');
    assert(finalRecord.fence_epoch === 1, 'only ONE ownership change actually landed — fence_epoch bumped exactly once, not twice');
  } finally {
    fs.rmSync(root, { recursive: true, force: true });
  }
});

await check('claimWriteOwnership refuses WORKSPACE_OWNED while the current owner is LIVE (default isOwnerLive assumption); reclaims when isOwnerLive says the owner is stale — the heartbeat-staleness reclaim path', async () => {
  const root = makeRoot();
  try {
    await registerWorkspace(root, mainReq());
    await claimWriteOwnership(root, 'main', 'sess-owner');

    await assertRejectsCode(
      () => claimWriteOwnership(root, 'main', 'sess-other'),
      'WORKSPACE_OWNED',
      'a live owner (default assumption with no isOwnerLive predicate) blocks a competing claim',
    );

    const reclaimed = await claimWriteOwnership(root, 'main', 'sess-other', { isOwnerLive: () => false });
    assert(reclaimed.ok === true && reclaimed.owner === true, 'a stale owner (isOwnerLive: false) is reclaimable');
    assert(reclaimed.reclaimed === true, 'the outcome is explicitly flagged as a reclaim, not a fresh claim');
    assert(reclaimed.record.write_owner_session === 'sess-other', 'ownership transferred to the reclaiming session');
    assert(reclaimed.record.fence_epoch === 2, 'fence_epoch bumps again on a reclaim (1 -> 2), fencing a stale in-flight write from the old owner');
  } finally {
    fs.rmSync(root, { recursive: true, force: true });
  }
});

await check('claimWriteOwnership: the production-shaped isOwnerLive predicate built from claims.mjs readSession + heartbeatStale reclaims a genuinely dead session', async () => {
  const root = makeRoot();
  try {
    await registerWorkspace(root, mainReq());
    const longAgo = Date.now() - (DEFAULT_HEARTBEAT_STALE_SECONDS + 60) * 1000;
    createSession(root, { id: 'sess-dead', now: longAgo });
    await claimWriteOwnership(root, 'main', 'sess-dead', { now: longAgo });

    // The production-shaped predicate a real caller (bee.mjs) would build:
    // read the owner's OWN session record and apply the SAME staleness
    // window activeWorkers uses — this module never imports claims.mjs
    // itself (structural isolation, see workspace-store.mjs header), so the
    // caller composes it.
    const { readSession } = await import('../lib/claims.mjs');
    const isOwnerLive = (ownerSessionId, now) => {
      const session = readSession(root, ownerSessionId);
      return session ? !heartbeatStale(session, now) : false;
    };

    const claimed = await claimWriteOwnership(root, 'main', 'sess-fresh', { isOwnerLive });
    assert(claimed.ok === true && claimed.reclaimed === true, 'a dead session (heartbeat past the staleness window) is reclaimed by the production-shaped predicate');
  } finally {
    fs.rmSync(root, { recursive: true, force: true });
  }
});

// ─── attach: the forgiving read-only-attach wrapper ─────────────────────

await check('attachWorkspace becomes owner when unowned (same as claimWriteOwnership); records a READ-ONLY ATTACH instead of throwing when a live owner already holds it', async () => {
  const root = makeRoot();
  try {
    await registerWorkspace(root, mainReq());
    const asOwner = await attachWorkspace(root, 'main', 'sess-a');
    assert(asOwner.ok === true && asOwner.role === 'owner', 'attach becomes owner when the workspace is unowned');

    const asAttached = await attachWorkspace(root, 'main', 'sess-b');
    assert(asAttached.ok === true, 'attach never throws when the workspace is owned by someone else');
    assert(asAttached.role === 'read-only', 'a second session attaching to an owned workspace gets a read-only role, not an error');
    assert(asAttached.write_owner_session === 'sess-a', 'the read-only attach result names the current owner');
    assert(asAttached.record.attached_sessions.includes('sess-b'), 'sess-b is recorded in attached_sessions');
    assert(!asAttached.record.attached_sessions.includes('sess-a'), 'the owner itself is never ALSO listed as an attached session');

    // Idempotent: attaching again does not duplicate the entry.
    const attachAgain = await attachWorkspace(root, 'main', 'sess-b');
    assert(attachAgain.record.attached_sessions.filter((s) => s === 'sess-b').length === 1, 'a repeat attach never duplicates the session in attached_sessions');

    assert(readWorkspace(root, 'main').write_owner_session === 'sess-a', 'a read-only attach never changes who owns the workspace');
  } finally {
    fs.rmSync(root, { recursive: true, force: true });
  }
});

await check('attachWorkspace refuses typed WORKSPACE_NOT_REGISTERED on an unregistered id — attach never auto-registers either', async () => {
  const root = makeRoot();
  try {
    await assertRejectsCode(
      () => attachWorkspace(root, 'ghost', 'sess-a'),
      'WORKSPACE_NOT_REGISTERED',
      'attach on an unregistered workspace must refuse, never silently register-then-attach',
    );
  } finally {
    fs.rmSync(root, { recursive: true, force: true });
  }
});

await check('releaseWriteOwnership: the current owner releasing clears write_owner_session and bumps fence_epoch; a non-owner releasing is a no-op', async () => {
  const root = makeRoot();
  try {
    await registerWorkspace(root, mainReq());
    await claimWriteOwnership(root, 'main', 'sess-a');

    const notOwner = await releaseWriteOwnership(root, 'main', 'sess-b');
    assert(notOwner.ok === true && notOwner.released === false, 'a non-owner release is a no-op, never an error');
    assert(readWorkspace(root, 'main').write_owner_session === 'sess-a', 'ownership is untouched by a non-owner release');

    const released = await releaseWriteOwnership(root, 'main', 'sess-a');
    assert(released.ok === true && released.released === true, 'the actual owner can release');
    assert(released.record.write_owner_session === null, 'ownership is cleared');
    assert(released.record.fence_epoch === 2, 'fence_epoch bumps on release too (1 -> 2)');

    const reclaim = await claimWriteOwnership(root, 'main', 'sess-c');
    assert(reclaim.ok === true, 'the workspace is claimable again after an explicit release');
  } finally {
    fs.rmSync(root, { recursive: true, force: true });
  }
});

// ─── condition 4 (advisor digest slice4, F5): grant (store topology) and
// write ownership (session concurrency) are independent axes, never one
// subsuming the other — enumerate all four combinations. ──────────────────

await check('condition 4 compose: grant (worktree-store.mjs decideWorktreeStore) and write ownership (workspace-store.mjs claimWriteOwnership) are independently computed and independently composable — all 4 combinations behave sensibly, no double-deny', async () => {
  const root = makeRoot();
  try {
    const worktreeId = 'wt-compose-1';
    const classification = { kind: 'linked-valid', id: worktreeId, mainRoot: root, worktreeRoot: path.join(root, '..', 'wt-compose-1') };

    // A local combinator — never production code, just the proof that
    // composing the two independent decisions is sensible and never
    // produces a double-deny/double-allow surprise.
    function canWrite(grantDecision, ownershipResult) {
      return Boolean(grantDecision.ok && grantDecision.kind === 'linked-valid-granted' && ownershipResult && ownershipResult.owner === true);
    }

    // ── Combo 1: GRANTED + OWNED (by the requesting session) -> writes flow.
    await registerWorkspace(root, { id: worktreeId, type: 'worktree', root: classification.worktreeRoot });
    const grantsA = { [worktreeId]: true };
    const grantDecisionA = decideWorktreeStore(classification, { grants: grantsA });
    const ownershipA = await claimWriteOwnership(root, worktreeId, 'sess-compose');
    assert(grantDecisionA.ok && grantDecisionA.kind === 'linked-valid-granted', 'combo 1: store topology is granted');
    assert(ownershipA.owner === true, 'combo 1: ownership is held by the requesting session');
    assert(canWrite(grantDecisionA, ownershipA) === true, 'combo 1 (granted + owned): compose is true — writes flow');

    // ── Combo 2: GRANTED + NOT OWNED (nobody has claimed it yet) -> grant
    // alone never implies ownership; a fresh session CAN still acquire it.
    await releaseWriteOwnership(root, worktreeId, 'sess-compose');
    const grantDecisionB = decideWorktreeStore(classification, { grants: grantsA });
    const unownedRecord = readWorkspace(root, worktreeId);
    assert(grantDecisionB.ok && grantDecisionB.kind === 'linked-valid-granted', 'combo 2: store topology is STILL granted (grant is topology, independent of ownership release)');
    assert(unownedRecord.write_owner_session === null, 'combo 2: nobody owns it right now');
    assert(canWrite(grantDecisionB, { owner: false }) === false, 'combo 2 (granted, unowned): compose is false — grant alone never authorizes a write');
    const acquiredAfterB = await attachWorkspace(root, worktreeId, 'sess-compose-2');
    assert(acquiredAfterB.role === 'owner', 'combo 2: "grant without owner" behaves sensibly — ownership is freely claimable, the grant never blocks acquiring it');

    // ── Combo 3: NOT GRANTED + OWNED -> ownership alone never implies grant;
    // the store still resolves to MAIN (fallback), never the worktree's own.
    const grantDecisionC = decideWorktreeStore(classification, { grants: {} });
    const ownershipC = readWorkspace(root, worktreeId);
    assert(grantDecisionC.ok && grantDecisionC.kind === 'linked-valid-default', 'combo 3: store topology falls back to main — NOT granted');
    assert(ownershipC.write_owner_session === 'sess-compose-2', 'combo 3: ownership is STILL held (ownership is independent of the grant registry)');
    assert(canWrite(grantDecisionC, { owner: true }) === false, 'combo 3 (owned, not granted): compose is false — ownership alone never overrides store topology; no double-allow');

    // ── Combo 4: NOT GRANTED + NOT REGISTERED/OWNED -> both independently deny.
    const otherWorktreeId = 'wt-compose-2';
    const otherClassification = { ...classification, id: otherWorktreeId, worktreeRoot: path.join(root, '..', 'wt-compose-2') };
    const grantDecisionD = decideWorktreeStore(otherClassification, { grants: {} });
    assert(grantDecisionD.ok && grantDecisionD.kind === 'linked-valid-default', 'combo 4: store topology denies (falls back to main)');
    await assertRejectsCode(
      () => claimWriteOwnership(root, otherWorktreeId, 'sess-compose-3'),
      'WORKSPACE_NOT_REGISTERED',
      'combo 4: ownership also denies (never registered) — both layers independently agree, no conflicting signal',
    );
  } finally {
    fs.rmSync(root, { recursive: true, force: true });
  }
});

printSummaryAndExit();
