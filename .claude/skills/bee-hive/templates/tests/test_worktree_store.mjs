#!/usr/bin/env node
// test_worktree_store.mjs — lib/worktree-store.mjs create/merge lifecycle
// tests, focused on the workspace-registry wiring added in
// multisession-native-19 (CONTEXT.md D2/D3): createFeatureWorktree registers
// a workspace record alongside its grant; mergeFeatureWorktree's cleanup path
// unregisters it. No prior suite exercised createFeatureWorktree/
// mergeFeatureWorktree directly against a real git fixture (worktree.new/
// worktree.merge were previously covered only through the CLI examples
// registry in test_bee_cli.mjs) — this file adds that direct coverage
// specifically for the workspace-registration lifecycle. Same PASS/FAIL/
// exit-1 contract as every other suite here — see scripts/lib/test-fixture.mjs.

import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { spawnSync } from 'node:child_process';
import { check, assert, printSummaryAndExit } from '../../../../scripts/lib/test-fixture.mjs';
import { createFeatureWorktree, mergeFeatureWorktree, readGrants } from '../lib/worktree-store.mjs';
import { WorkspaceStoreError, readWorkspace } from '../lib/workspace-store.mjs';
import { resolveContext } from '../lib/state.mjs';
import { createSession } from '../lib/claims.mjs';
import { acquireLeases } from '../lib/lease-store.mjs';

function git(cwd, args, { allowFailure = false } = {}) {
  const result = spawnSync('git', args, { cwd, encoding: 'utf8' });
  if (!allowFailure && result.status !== 0) {
    throw new Error(`git ${args.join(' ')} failed (${result.status}): ${result.stderr || result.stdout}`);
  }
  return result;
}

function gitText(cwd, args) {
  return git(cwd, args).stdout.trim();
}

function makeOrdinaryRepoFixture() {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), 'bee-worktree-store-main-'));
  git(root, ['init', '-b', 'main']);
  git(root, ['config', 'user.email', 'bee@example.invalid']);
  git(root, ['config', 'user.name', 'Bee Test']);
  // .bee/ (grant registry, workspace registry, state) must be gitignored —
  // same as any real bee-onboarded repo — so createFeatureWorktree's own
  // writes there never make `git status --porcelain` see MAIN as dirty; a
  // merge test needs isTreeDirty(mainRoot) to stay clean after create.
  fs.writeFileSync(path.join(root, '.gitignore'), '.bee/\n');
  fs.writeFileSync(path.join(root, 'README.md'), 'demo\n');
  git(root, ['add', '.gitignore', 'README.md']);
  git(root, ['commit', '-m', 'base']);
  return root;
}

await check('createFeatureWorktree registers a workspace record (type: worktree, matching root/branch/base_sha) alongside its grant', async () => {
  const mainRoot = makeOrdinaryRepoFixture();
  try {
    const created = await createFeatureWorktree(mainRoot, { feature: 'demo-feature' });
    assert(typeof created.id === 'string' && created.id, 'created worktree has a git-verified id');

    const grants = readGrants(path.join(mainRoot, '.bee'));
    assert(grants[created.id] === true, 'the grant registry is written (store-topology side, unchanged)');

    const workspace = readWorkspace(mainRoot, created.id);
    assert(workspace.type === 'worktree', 'workspace record type is "worktree"');
    assert(workspace.root === created.worktreeRoot, 'workspace record root matches the created worktree root');
    assert(workspace.branch === created.branch, 'workspace record branch matches the created branch ("wt/demo-feature")');
    assert(typeof workspace.base_sha === 'string' && workspace.base_sha.length > 0, 'workspace record carries a base_sha');
    assert(workspace.write_owner_session === null, 'a freshly created worktree workspace starts with no write owner');
  } finally {
    git(mainRoot, ['worktree', 'prune'], { allowFailure: true });
    fs.rmSync(mainRoot, { recursive: true, force: true });
  }
});

await check('mergeFeatureWorktree --cleanup unregisters the workspace record on the already-up-to-date cleanup path (same trigger performCleanup already uses for the grant)', async () => {
  const mainRoot = makeOrdinaryRepoFixture();
  try {
    const created = await createFeatureWorktree(mainRoot, { feature: 'demo-merge' });
    assert(readWorkspace(mainRoot, created.id).type === 'worktree', 'workspace registered right after create');

    // No new commits on the worktree branch -> ALREADY_UP_TO_DATE, which
    // still runs cleanup when requested (issues-46-53 D3) — the SAME path
    // performCleanup's grant removal already relies on, now also exercising
    // the workspace unregister wired in alongside it.
    const merged = await mergeFeatureWorktree(mainRoot, { id: created.id, cleanup: true });
    assert(merged.cleanup && merged.cleanup.ok === true, `cleanup must succeed, got ${JSON.stringify(merged.cleanup)}`);

    const grantsAfter = readGrants(path.join(mainRoot, '.bee'));
    assert(grantsAfter[created.id] !== true, 'the grant is removed after cleanup');

    let threw = null;
    try {
      readWorkspace(mainRoot, created.id);
    } catch (err) {
      threw = err;
    }
    assert(threw instanceof WorkspaceStoreError && threw.code === 'WORKSPACE_MISSING', 'the workspace record is unregistered after cleanup — readWorkspace now refuses typed WORKSPACE_MISSING');
  } finally {
    git(mainRoot, ['worktree', 'prune'], { allowFailure: true });
    fs.rmSync(mainRoot, { recursive: true, force: true });
  }
});

await check('TOPOLOGY: a session created from inside a GRANTED linked worktree resolves workspaceId to the worktree\'s own id (not "main"); a lease it acquires carries that same workspace_id', async () => {
  const mainRoot = makeOrdinaryRepoFixture();
  try {
    const created = await createFeatureWorktree(mainRoot, { feature: 'demo-topology' });

    const context = resolveContext(created.worktreeRoot);
    assert(context.workspaceId === created.id, `resolveContext from inside the granted worktree must resolve workspaceId to the worktree's own id, got "${context.workspaceId}" vs expected "${created.id}"`);
    assert(context.controlRoot === mainRoot, 'controlRoot from inside the worktree still resolves to MAIN (control plane is shared)');

    const sessionResult = createSession(context.controlRoot, { id: 'sess-topology-1', workspace_id: context.workspaceId });
    assert(sessionResult.ok === true, 'session creation from the worktree succeeds, landing in the shared control store');
    assert(sessionResult.session.workspace_id === created.id, 'the session record carries the WORKTREE\'s workspace_id, not "main"');

    const [lease] = acquireLeases(context.controlRoot, [
      {
        type: 'cell',
        id: 'demo-topology-cell',
        mode: 'write',
        workflow_id: 'wf-demo-topology',
        session_id: sessionResult.session.id,
        workspace_id: sessionResult.session.workspace_id,
        epoch: 1,
      },
    ]);
    assert(lease.workspace_id === created.id, `a lease acquired from a session bound to this worktree must carry the worktree's workspace_id, got "${lease.workspace_id}" vs expected "${created.id}"`);
    assert(lease.workspace_id !== 'main', 'the worktree\'s workspace_id is never conflated with "main"');
  } finally {
    git(mainRoot, ['worktree', 'prune'], { allowFailure: true });
    fs.rmSync(mainRoot, { recursive: true, force: true });
  }
});

await check('TOPOLOGY: a session created from the MAIN checkout resolves workspaceId to "main"', async () => {
  const mainRoot = makeOrdinaryRepoFixture();
  try {
    const context = resolveContext(mainRoot);
    assert(context.workspaceId === 'main', `resolveContext from the main checkout must resolve workspaceId to "main", got "${context.workspaceId}"`);
    const sessionResult = createSession(context.controlRoot, { id: 'sess-main-1', workspace_id: context.workspaceId });
    assert(sessionResult.session.workspace_id === 'main', 'a session created in the main checkout carries workspace_id "main"');
  } finally {
    fs.rmSync(mainRoot, { recursive: true, force: true });
  }
});

printSummaryAndExit();
