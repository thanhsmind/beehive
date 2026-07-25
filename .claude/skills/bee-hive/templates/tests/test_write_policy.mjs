#!/usr/bin/env node
// test_write_policy.mjs — lib/state.mjs's applyWritePolicy contract tests
// (multisession-native-20, CONTEXT.md D2/D3, advisor-digest-slice4 condition
// 6). Same PASS/FAIL/exit-1 contract as every other suite here — see
// scripts/lib/test-fixture.mjs. Mirrors test_workspace_store.mjs's /
// test_worktree_store.mjs's fixture style (plain onboarding-marker root for
// the non-worktree paths; a real git fixture, same helper shape as
// test_worktree_store.mjs's makeOrdinaryRepoFixture, for the isolate-create
// paths that must actually invoke createFeatureWorktree).

import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { spawnSync } from 'node:child_process';
import { check, assert, printSummaryAndExit } from '../../../../scripts/lib/test-fixture.mjs';
import { applyWritePolicy, WritePolicyRefusalError } from '../lib/state.mjs';
import { createSession } from '../lib/claims.mjs';
import { reserve } from '../lib/reservations.mjs';
import { readWorkspace, workspacesDir } from '../lib/workspace-store.mjs';
import { readJson, writeJsonAtomic } from '../lib/fsutil.mjs';

function makeRoot(prefix) {
  const dir = fs.mkdtempSync(path.join(os.tmpdir(), prefix));
  fs.mkdirSync(path.join(dir, '.bee'), { recursive: true });
  writeJsonAtomic(path.join(dir, '.bee', 'onboarding.json'), { schema_version: '1.0', bee_version: '0.1.0' });
  return dir;
}

function writeConfig(root, config) {
  writeJsonAtomic(path.join(root, '.bee', 'config.json'), config);
}

function git(cwd, args, { allowFailure = false } = {}) {
  const result = spawnSync('git', args, { cwd, encoding: 'utf8' });
  if (!allowFailure && result.status !== 0) {
    throw new Error(`git ${args.join(' ')} failed (${result.status}): ${result.stderr || result.stdout}`);
  }
  return result;
}

// Same shape as test_worktree_store.mjs's makeOrdinaryRepoFixture — a real
// git repo is required for the isolate-create path (createFeatureWorktree
// shells out to `git worktree add`).
function makeGitRoot(prefix) {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), prefix));
  git(root, ['init', '-b', 'main']);
  git(root, ['config', 'user.email', 'bee@example.invalid']);
  git(root, ['config', 'user.name', 'Bee Test']);
  fs.writeFileSync(path.join(root, '.gitignore'), '.bee/\n');
  fs.writeFileSync(path.join(root, 'README.md'), 'demo\n');
  git(root, ['add', '.gitignore', 'README.md']);
  git(root, ['commit', '-m', 'base']);
  return root;
}

async function assertRejectsCode(fn, expectedCode, message) {
  try {
    await fn();
  } catch (error) {
    assert(
      error instanceof WritePolicyRefusalError && error.code === expectedCode,
      `${message} — expected typed WritePolicyRefusalError code "${expectedCode}", got ${
        error instanceof WritePolicyRefusalError ? `code "${error.code}"` : String(error)
      }`,
    );
    return;
  }
  throw new Error(`${message} — expected a rejection, none thrown`);
}

// ─── observe ────────────────────────────────────────────────────────────

await check('observe: config.guards.write_policy="observe" — two different live sessions never contend, and nothing is written to the workspace registry at all', async () => {
  const root = makeRoot('bee-wp-observe-');
  try {
    writeConfig(root, { guards: { write_policy: 'observe' } });
    createSession(root, { id: 'sess-a' });
    createSession(root, { id: 'sess-b' });
    const a = await applyWritePolicy(root, { sessionId: 'sess-a' });
    assert(a.ok === true && a.mode === 'observe', `sess-a proceeds under observe, got ${JSON.stringify(a)}`);
    const b = await applyWritePolicy(root, { sessionId: 'sess-b' });
    assert(b.ok === true && b.mode === 'observe', `sess-b (a second live session) is never blocked under observe, got ${JSON.stringify(b)}`);
    assert(!fs.existsSync(workspacesDir(root)), 'observe never registers/touches the workspace registry at all');
  } finally {
    fs.rmSync(root, { recursive: true, force: true });
  }
});

// ─── shared-disjoint ────────────────────────────────────────────────────

await check('shared-disjoint: a write without an exact-path lease is denied (LEASE_REQUIRED); the same call flows once the lease is held', async () => {
  const root = makeRoot('bee-wp-shared-disjoint-');
  try {
    writeConfig(root, { guards: { write_policy: 'shared-disjoint' } });
    createSession(root, { id: 'sess-a' });

    await assertRejectsCode(
      () => applyWritePolicy(root, { sessionId: 'sess-a', paths: ['src/app.ts'] }),
      'LEASE_REQUIRED',
      'shared-disjoint refuses a write with no exact-path lease held',
    );

    const reserved = await reserve(root, { agent: 'worker-a', cell: 'cell-1', path: 'src/app.ts', session: 'sess-a' });
    assert(reserved.ok === true, `precondition: the lease must actually be acquired, got ${JSON.stringify(reserved)}`);

    const flowed = await applyWritePolicy(root, { sessionId: 'sess-a', paths: ['src/app.ts'] });
    assert(flowed.ok === true && flowed.mode === 'shared-disjoint', `with the lease held, the write flows, got ${JSON.stringify(flowed)}`);
    assert(flowed.leased.includes('src/app.ts'), 'the leased path is reported back');
  } finally {
    fs.rmSync(root, { recursive: true, force: true });
  }
});

await check('shared-disjoint: a BROAD/glob reservation never satisfies the exact-path requirement', async () => {
  const root = makeRoot('bee-wp-shared-disjoint-glob-');
  try {
    writeConfig(root, { guards: { write_policy: 'shared-disjoint' } });
    createSession(root, { id: 'sess-a' });
    const reserved = await reserve(root, { agent: 'worker-a', cell: 'cell-1', path: 'src/*', session: 'sess-a' });
    assert(reserved.ok === true, 'precondition: the glob reservation is created');
    await assertRejectsCode(
      () => applyWritePolicy(root, { sessionId: 'sess-a', paths: ['src/app.ts'] }),
      'LEASE_REQUIRED',
      'a broad/glob reservation never satisfies shared-disjoint\'s exact-path requirement',
    );
  } finally {
    fs.rmSync(root, { recursive: true, force: true });
  }
});

// ─── isolated (default): no-consent refusal ────────────────────────────

await check('isolated (default): a solo session always becomes the workspace write owner — never blocked, byte-identical prohibition', async () => {
  const root = makeRoot('bee-wp-solo-');
  try {
    createSession(root, { id: 'sess-solo' });
    const first = await applyWritePolicy(root, { sessionId: 'sess-solo' });
    assert(first.ok === true && first.mode === 'isolated' && first.workspace === 'owner', `solo session becomes owner, got ${JSON.stringify(first)}`);
    // Re-claiming its own ownership a second time is a no-op success, not a refusal.
    const again = await applyWritePolicy(root, { sessionId: 'sess-solo' });
    assert(again.ok === true && again.workspace === 'owner', 'the SAME session re-claiming is never blocked by its own ownership');
  } finally {
    fs.rmSync(root, { recursive: true, force: true });
  }
});

await check('isolated (default): a sessionless caller (legacy convention) proceeds untouched — zero new files written (prohibition: no silent default change for solo single-session repos)', async () => {
  const root = makeRoot('bee-wp-sessionless-');
  try {
    const result = await applyWritePolicy(root, { sessionId: null });
    assert(result.ok === true && result.workspace === 'unbound', `sessionless call proceeds untouched, got ${JSON.stringify(result)}`);
    assert(!fs.existsSync(workspacesDir(root)), 'a sessionless call never registers a workspace record');
  } finally {
    fs.rmSync(root, { recursive: true, force: true });
  }
});

await check('isolated (default), RED-FIRST: a second write-capable session finding a LIVE different owner, with no consent, refuses with a typed error naming the exact --isolate one-liner; the fuller message is shown once, a shorter one on repeat', async () => {
  const root = makeRoot('bee-wp-no-consent-');
  try {
    createSession(root, { id: 'sess-a' });
    createSession(root, { id: 'sess-b' });

    const owner = await applyWritePolicy(root, { sessionId: 'sess-a', verbHint: 'cells claim --id x --worker w' });
    assert(owner.ok === true && owner.workspace === 'owner', 'precondition: sess-a is the live write owner');

    let firstMessage = null;
    try {
      await applyWritePolicy(root, { sessionId: 'sess-b', verbHint: 'cells claim --id x --worker w' });
      throw new Error('expected a WritePolicyRefusalError, none thrown');
    } catch (error) {
      assert(error instanceof WritePolicyRefusalError && error.code === 'WORKSPACE_ISOLATION_REQUIRED', `typed refusal, got ${error}`);
      assert(error.message.includes('--isolate'), `refusal names the exact --isolate one-liner, got: ${error.message}`);
      assert(error.message.includes('cells claim --id x --worker w'), `the one-liner carries the caller's own verb, got: ${error.message}`);
      firstMessage = error.message;
    }

    // Second refusal, same session: surfaced once per session — the repeat is shorter.
    try {
      await applyWritePolicy(root, { sessionId: 'sess-b', verbHint: 'cells claim --id x --worker w' });
      throw new Error('expected a second WritePolicyRefusalError, none thrown');
    } catch (error) {
      assert(error instanceof WritePolicyRefusalError, 'still a typed refusal on repeat');
      assert(error.message.includes('--isolate'), 'the repeat refusal still names the one-liner');
      assert(error.message.length < firstMessage.length, `the repeat refusal is the SHORTER, not-spammed message (first=${firstMessage.length} chars, repeat=${error.message.length} chars)`);
    }
  } finally {
    fs.rmSync(root, { recursive: true, force: true });
  }
});

await check('isolated (default): a DEAD (heartbeat-stale) owner is silently reclaimed — never a refusal', async () => {
  const root = makeRoot('bee-wp-reclaim-');
  try {
    // sess-dead is never registered as a live session record at all — readSession
    // returns null, and applyWritePolicy's isOwnerLive predicate treats a session
    // with no record as not-live (same production-shaped predicate the
    // test_workspace_store.mjs msn-19 precedent uses).
    const first = await applyWritePolicy(root, { sessionId: 'sess-dead' });
    assert(first.ok === true && first.workspace === 'owner', 'precondition: sess-dead becomes owner');
    createSession(root, { id: 'sess-alive' });
    const second = await applyWritePolicy(root, { sessionId: 'sess-alive' });
    assert(second.ok === true && second.workspace === 'owner' && second.reclaimed === true, `a dead owner is reclaimed, never refused, got ${JSON.stringify(second)}`);
  } finally {
    fs.rmSync(root, { recursive: true, force: true });
  }
});

// ─── isolated (default): consented / configured auto-isolate ──────────

await check('isolated (default), --isolate: a fresh worktree is created, its workspace is registered, and the caller is told the new path', async () => {
  const root = makeGitRoot('bee-wp-isolate-flag-');
  try {
    createSession(root, { id: 'sess-a' });
    await applyWritePolicy(root, { sessionId: 'sess-a' }); // sess-a becomes owner

    createSession(root, { id: 'sess-b' });
    const isolated = await applyWritePolicy(root, { sessionId: 'sess-b', isolate: true, feature: 'demo-isolate' });
    assert(isolated.ok === true && isolated.redirect === true, `a consented isolate-create succeeds with redirect:true, got ${JSON.stringify(isolated)}`);
    assert(typeof isolated.worktreeRoot === 'string' && fs.existsSync(isolated.worktreeRoot), `the worktree physically exists on disk at ${isolated.worktreeRoot}`);
    assert(isolated.text.includes(isolated.worktreeRoot), 'the caller is told the new path in work-language text');
    assert(isolated.text.toLowerCase().includes('cost') || isolated.costDisclosure.toLowerCase().includes('cost'), 'a loud one-line cost disclosure names the disk cost');

    const workspace = readWorkspace(root, isolated.workspaceId);
    assert(workspace.type === 'worktree', 'the new workspace is registered (msn-19 createFeatureWorktree wiring)');
    assert(workspace.write_owner_session === 'sess-b', 'sess-b is attached as the new workspace\'s own write owner');
  } finally {
    git(root, ['worktree', 'prune'], { allowFailure: true });
    fs.rmSync(root, { recursive: true, force: true });
  }
});

await check('isolated (default), config.guards.auto_isolate=true: the SAME auto-create flow fires without an explicit --isolate flag', async () => {
  const root = makeGitRoot('bee-wp-auto-isolate-config-');
  try {
    writeConfig(root, { guards: { auto_isolate: true } });
    createSession(root, { id: 'sess-a' });
    await applyWritePolicy(root, { sessionId: 'sess-a' });

    createSession(root, { id: 'sess-b' });
    const isolated = await applyWritePolicy(root, { sessionId: 'sess-b', feature: 'demo-config-isolate' });
    assert(isolated.ok === true && isolated.redirect === true, `config.guards.auto_isolate=true auto-creates with no --isolate flag, got ${JSON.stringify(isolated)}`);
    assert(fs.existsSync(isolated.worktreeRoot), 'the worktree physically exists');
  } finally {
    git(root, ['worktree', 'prune'], { allowFailure: true });
    fs.rmSync(root, { recursive: true, force: true });
  }
});

// ─── condition 6c: isolation cannot self-deadlock against the write guard ──

await check('condition 6c: isolate-create succeeds even under the MOST restrictive state (terminal phase, every gate unapproved) — proves this internal write path never consults checkWrite/the write guard, so isolation can never self-deadlock against the very guard that made it necessary', async () => {
  const root = makeGitRoot('bee-wp-no-deadlock-');
  try {
    writeJsonAtomic(path.join(root, '.bee', 'state.json'), {
      schema_version: '1.0',
      phase: 'idle',
      feature: null,
      approved_gates: { context: false, shape: false, execution: false, review: false },
      workers: [],
    });
    createSession(root, { id: 'sess-a' });
    await applyWritePolicy(root, { sessionId: 'sess-a' });
    createSession(root, { id: 'sess-b' });
    const isolated = await applyWritePolicy(root, { sessionId: 'sess-b', isolate: true, feature: 'demo-no-deadlock' });
    assert(isolated.ok === true && fs.existsSync(isolated.worktreeRoot), `isolate-create succeeds regardless of state.json's phase/gates, got ${JSON.stringify(isolated)}`);
    assert(readJson(path.join(root, '.bee', 'state.json'), null).phase === 'idle', 'the restrictive state.json itself is left completely untouched by this call');
  } finally {
    git(root, ['worktree', 'prune'], { allowFailure: true });
    fs.rmSync(root, { recursive: true, force: true });
  }
});

// ─── enforceIsolation: false (cells claim/claim-next's own opt-out) ──────

await check('enforceIsolation: false — a second live session is NEVER blocked (bee-swarming\'s own concurrent-claim mechanism stays byte-untouched, same reasoning as the lane path)', async () => {
  const root = makeRoot('bee-wp-enforce-false-');
  try {
    createSession(root, { id: 'sess-a' });
    createSession(root, { id: 'sess-b' });
    const a = await applyWritePolicy(root, { sessionId: 'sess-a', enforceIsolation: false });
    assert(a.ok === true && a.workspace === 'unbound', `enforceIsolation:false never claims ownership, got ${JSON.stringify(a)}`);
    const b = await applyWritePolicy(root, { sessionId: 'sess-b', enforceIsolation: false });
    assert(b.ok === true && b.workspace === 'unbound', `a second concurrent session is never blocked, got ${JSON.stringify(b)}`);
    assert(!fs.existsSync(workspacesDir(root)), 'enforceIsolation:false never touches the workspace registry at all');
  } finally {
    fs.rmSync(root, { recursive: true, force: true });
  }
});

printSummaryAndExit();
