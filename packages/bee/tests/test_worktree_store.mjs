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
import { check, assert, printSummaryAndExit } from '../../../scripts/lib/test-fixture.mjs';
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

// lstat-based existence check (unlike fs.existsSync, this reports a broken
// symlink as PRESENT rather than absent — exactly what the companion-mount
// tests below need: they're distinguishing "the symlink entry itself is
// still there" from "it's gone", not whether its target resolves).
function lstatExists(p) {
  try {
    fs.lstatSync(p);
    return true;
  } catch {
    return false;
  }
}

// Builds a `commands.worktree_companion_start`-shaped shell command that
// prints `payload` (as JSON) to stdout, WITHOUT embedding the JSON in the
// shell command string itself — the payload is written to a fixture file and
// a tiny script reads + echoes it back. Embedding JSON with nested quotes
// directly in a `spawnSync(..., { shell: true })` command string is fragile
// across POSIX shells and cmd.exe alike (see the existing plain `node -e
// "process.exit(0)"` commands elsewhere in this file for the ceiling on what
// stays portable inline); routing through a file sidesteps all of that.
function makeCompanionStartCommand(payload) {
  const scriptDir = fs.mkdtempSync(path.join(os.tmpdir(), 'bee-companion-script-'));
  const payloadPath = path.join(scriptDir, 'payload.json');
  fs.writeFileSync(payloadPath, JSON.stringify(payload));
  const scriptPath = path.join(scriptDir, 'start.mjs');
  fs.writeFileSync(scriptPath, `import fs from 'node:fs';\nprocess.stdout.write(fs.readFileSync(${JSON.stringify(payloadPath)}, 'utf8'));\n`);
  return `node ${JSON.stringify(scriptPath)}`;
}

// companionWorktreeFixture — like mergeableWorktreeFixture (below) but
// created `--with-companion`: `companionStartCommand` is wired directly via
// createFeatureWorktree's own options (bypassing config.json resolution,
// which is the CLI handler's job, not worktree-store.mjs's), pointing at a
// real target directory so the symlink it creates resolves to something.
async function companionWorktreeFixture(feature, { mountPath = 'companion' } = {}) {
  const mainRoot = makeOrdinaryRepoFixture();
  const targetDir = fs.mkdtempSync(path.join(os.tmpdir(), 'bee-companion-target-'));
  fs.writeFileSync(path.join(targetDir, 'marker.txt'), 'companion\n');
  const companionStartCommand = makeCompanionStartCommand({ worktreePath: targetDir, sessionId: 'sess-companion-1' });
  const created = await createFeatureWorktree(mainRoot, { feature, companionStartCommand, companionMountPath: mountPath });
  return { mainRoot, targetDir, created };
}

function makeOrdinaryRepoFixture() {
  // realpath the fixture root — on Windows os.tmpdir() gives the 8.3 short form
  // while every resolver speaks the long form, so a raw mkdtemp root makes this
  // suite compare two spellings of one directory (see the same repair in
  // scripts/tests/test_worktree_merge_queue.mjs, whose marker wait it reuses).
  const root = fs.realpathSync.native(fs.mkdtempSync(path.join(os.tmpdir(), 'bee-worktree-store-main-')));
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

// ─── multisession-native-22 (D8 stage 5, advisor condition C): mergeFeatureWorktree's
// new options.checkProcessorLease / options.onVerifyTick hooks. This module
// (worktree-store.mjs) has zero lease-store knowledge (see its own header,
// "zero deps beyond node builtins" + the module header note added by this
// cell) — these tests exercise the hooks as plain caller-supplied callbacks,
// exactly the way integration-queue.mjs's runThroughQueue wires them in
// production, without importing integration-queue.mjs itself. ───────────────

// mergeableWorktreeFixture — a worktree with a REAL new commit on its branch
// (never ALREADY_UP_TO_DATE) so P2's verify actually runs and P3 actually
// reaches the fence checks these tests are about.
async function mergeableWorktreeFixture(feature) {
  const mainRoot = makeOrdinaryRepoFixture();
  const created = await createFeatureWorktree(mainRoot, { feature });
  fs.writeFileSync(path.join(created.worktreeRoot, 'work.txt'), 'x\n');
  git(created.worktreeRoot, ['add', 'work.txt']);
  git(created.worktreeRoot, ['commit', '-m', 'fixture work']);
  return { mainRoot, created };
}

await check('checkProcessorLease returning null (no drift) never affects an otherwise-clean merge — pure passthrough', async () => {
  const { mainRoot, created } = await mergeableWorktreeFixture('demo-lease-clean');
  try {
    let calls = 0;
    const merged = await mergeFeatureWorktree(mainRoot, {
      id: created.id,
      verifyCommand: 'node -e "process.exit(0)"',
      checkProcessorLease: () => {
        calls += 1;
        return null;
      },
    });
    assert(calls === 1, `checkProcessorLease must be called exactly once (P3), got ${calls}`);
    assert(merged.ok === true && merged.merged === true, `a clean merge with a null-returning checkProcessorLease must still succeed, got ${JSON.stringify(merged)}`);
  } finally {
    git(mainRoot, ['worktree', 'prune'], { allowFailure: true });
    fs.rmSync(mainRoot, { recursive: true, force: true });
  }
});

await check('checkProcessorLease returning a drift string aborts the merge untouched (WORKTREE_MERGE_FENCE_DRIFT) — the FIRST line of the P3 fence, independent of checkMergeFence', async () => {
  const { mainRoot, created } = await mergeableWorktreeFixture('demo-lease-drift');
  const preHead = gitText(mainRoot, ['rev-parse', 'HEAD']);
  try {
    let threw = null;
    try {
      await mergeFeatureWorktree(mainRoot, {
        id: created.id,
        verifyCommand: 'node -e "process.exit(0)"',
        // Simulates integration-queue.mjs's checkProcessorLeaseEpoch
        // detecting a takeover — nothing about the staged tree/HEAD is
        // tampered here, proving this check is independent of (and runs
        // ahead of) the existing checkMergeFence staged-tree/HEAD check.
        checkProcessorLease: () => 'simulated processor-lease takeover: epoch 1 -> 2',
      });
    } catch (err) {
      threw = err;
    }
    assert(threw && threw.code === 'WORKTREE_MERGE_FENCE_DRIFT', `expected a typed WORKTREE_MERGE_FENCE_DRIFT throw, got ${threw ? threw.code || threw.message : '(no throw)'}`);
    assert(/simulated processor-lease takeover/.test(threw.message), `the refusal message must carry the processor-lease drift text, got: ${threw.message}`);

    const headAfter = gitText(mainRoot, ['rev-parse', 'HEAD']);
    assert(headAfter === preHead, 'HEAD is unchanged after the processor-lease-drift abort');
    assert(!fs.existsSync(path.join(mainRoot, '.git', 'MERGE_HEAD')), 'no MERGE_HEAD lingers after the abort');
    const status = git(mainRoot, ['status', '--porcelain', '--untracked-files=no']).stdout;
    assert(status.trim() === '', `main tracked status must be clean after the abort, got: ${status}`);
  } finally {
    git(mainRoot, ['worktree', 'prune'], { allowFailure: true });
    fs.rmSync(mainRoot, { recursive: true, force: true });
  }
});

await check('onVerifyTick fires periodically DURING the verify child (P2) — proves the async spawn replacement for spawnSync actually interleaves a timer, not just accepts the option', async () => {
  const { mainRoot, created } = await mergeableWorktreeFixture('demo-verify-tick');
  try {
    const ticks = [];
    const merged = await mergeFeatureWorktree(mainRoot, {
      id: created.id,
      // Sleeps ~300ms before exiting 0 — long enough for several 40ms ticks
      // if (and only if) the child truly runs unblocked alongside a timer.
      verifyCommand: 'node -e "setTimeout(() => process.exit(0), 300)"',
      onVerifyTick: () => {
        ticks.push(Date.now());
      },
      verifyTickIntervalMs: 40,
    });
    assert(merged.ok === true && merged.verify === 'green', `expected a green merge, got ${JSON.stringify(merged)}`);
    assert(ticks.length >= 2, `expected onVerifyTick to fire at least twice during a ~300ms verify child with a 40ms interval, got ${ticks.length} tick(s) — spawnSync would have blocked the event loop and fired zero`);
  } finally {
    git(mainRoot, ['worktree', 'prune'], { allowFailure: true });
    fs.rmSync(mainRoot, { recursive: true, force: true });
  }
});

await check('a merge with NO onVerifyTick/checkProcessorLease given at all (every pre-existing caller) is unaffected — byte-identical result shape', async () => {
  const { mainRoot, created } = await mergeableWorktreeFixture('demo-no-hooks');
  try {
    const merged = await mergeFeatureWorktree(mainRoot, { id: created.id, verifyCommand: 'node -e "process.exit(0)"' });
    assert(merged.ok === true && merged.merged === true && merged.verify === 'green', `expected the ordinary green-merge shape with no queue hooks, got ${JSON.stringify(merged)}`);
  } finally {
    git(mainRoot, ['worktree', 'prune'], { allowFailure: true });
    fs.rmSync(mainRoot, { recursive: true, force: true });
  }
});

// ─── gfb-3 (GH #84): companion teardown must run only AFTER the merge's
// zero-mutation refusal checks pass — a merge refused for any reason must
// leave a --with-companion worktree's mount (symlink + marker) fully intact,
// and the worktree dirty-check must never falsely trip on the mount itself
// (a git pathspec exclusion, not text-filtering of porcelain output). ──────

await check('a merge refused by the worktree dirty-check (a genuinely dirty file, unrelated to the companion mount) preserves the companion mount — marker + symlink survive the refusal untouched', async () => {
  const { mainRoot, targetDir, created } = await companionWorktreeFixture('demo-companion-dirty-refuse');
  try {
    const markerPath = path.join(created.worktreeRoot, '.bee', 'companion-session.json');
    const mountFullPath = path.join(created.worktreeRoot, 'companion');
    assert(lstatExists(markerPath), 'sanity: companion marker exists before merge');
    assert(lstatExists(mountFullPath), 'sanity: companion mount symlink exists before merge');

    // A genuinely dirty file OTHER than the mount must still refuse the
    // merge — the exclusion is scoped to the mount path alone.
    fs.writeFileSync(path.join(created.worktreeRoot, 'dirty.txt'), 'oops\n');

    let threw = null;
    try {
      await mergeFeatureWorktree(mainRoot, { id: created.id, companionEndCommand: 'node -e "process.exit(0)"' });
    } catch (err) {
      threw = err;
    }
    assert(threw && threw.code === 'WORKTREE_MERGE_WORKTREE_DIRTY', `expected WORKTREE_MERGE_WORKTREE_DIRTY, got ${threw ? threw.code || threw.message : '(no throw)'}`);
    assert(lstatExists(markerPath), 'the companion marker must survive a refused merge');
    assert(lstatExists(mountFullPath), 'the companion mount symlink must survive a refused merge');
  } finally {
    git(mainRoot, ['worktree', 'prune'], { allowFailure: true });
    fs.rmSync(mainRoot, { recursive: true, force: true });
    fs.rmSync(created.worktreeRoot, { recursive: true, force: true });
    fs.rmSync(targetDir, { recursive: true, force: true });
  }
});

await check('a clean --with-companion merge with a NESTED mountPath ("vendor/companion") succeeds and tears down the marker + symlink — proves the dirty-check uses a git pathspec exclusion, not text-filtering of porcelain output (a nested mount collapses to "?? vendor/" in porcelain, which text-filtering for the exact mount path would never match)', async () => {
  const { mainRoot, targetDir, created } = await companionWorktreeFixture('demo-companion-nested-clean', { mountPath: 'vendor/companion' });
  try {
    // Give the branch something new to merge so this isn't ALREADY_UP_TO_DATE
    // (which skips verify and takes a different result shape entirely).
    fs.writeFileSync(path.join(created.worktreeRoot, 'work.txt'), 'x\n');
    git(created.worktreeRoot, ['add', 'work.txt']);
    git(created.worktreeRoot, ['commit', '-m', 'fixture work']);

    const markerPath = path.join(created.worktreeRoot, '.bee', 'companion-session.json');
    const mountFullPath = path.join(created.worktreeRoot, 'vendor', 'companion');
    assert(lstatExists(markerPath), 'sanity: companion marker exists before merge');
    assert(lstatExists(mountFullPath), 'sanity: nested companion mount symlink exists before merge');

    const merged = await mergeFeatureWorktree(mainRoot, { id: created.id, companionEndCommand: 'node -e "process.exit(0)"' });
    assert(merged.ok === true && merged.merged === true, `expected a clean merge, got ${JSON.stringify(merged)}`);
    assert(merged.companion && merged.companion.ended === true, `expected companion.ended === true (no warning), got ${JSON.stringify(merged.companion)}`);
    assert(!lstatExists(markerPath), 'the companion marker must be torn down after a clean merge');
    assert(!lstatExists(mountFullPath), 'the nested companion mount symlink must be torn down after a clean merge');
  } finally {
    git(mainRoot, ['worktree', 'prune'], { allowFailure: true });
    fs.rmSync(mainRoot, { recursive: true, force: true });
    fs.rmSync(created.worktreeRoot, { recursive: true, force: true });
    fs.rmSync(targetDir, { recursive: true, force: true });
  }
});

await check('a worktree with no companion marker merges exactly as before — no "companion" field on the result at all, byte-identical to pre-companion behavior', async () => {
  const { mainRoot, created } = await mergeableWorktreeFixture('demo-no-companion-marker');
  try {
    const merged = await mergeFeatureWorktree(mainRoot, { id: created.id, companionEndCommand: 'node -e "process.exit(0)"' });
    assert(merged.ok === true && merged.merged === true, `expected a clean merge, got ${JSON.stringify(merged)}`);
    assert(!('companion' in merged), 'no companion marker present -> no "companion" field on the result at all');
  } finally {
    git(mainRoot, ['worktree', 'prune'], { allowFailure: true });
    fs.rmSync(mainRoot, { recursive: true, force: true });
  }
});

await check('a merge refused for a DETACHED HEAD worktree preserves the companion mount too — the fix covers every zero-mutation refusal, not just the dirty-check', async () => {
  const { mainRoot, targetDir, created } = await companionWorktreeFixture('demo-companion-detached');
  try {
    const markerPath = path.join(created.worktreeRoot, '.bee', 'companion-session.json');
    const mountFullPath = path.join(created.worktreeRoot, 'companion');

    git(created.worktreeRoot, ['checkout', '--detach', 'HEAD']);

    let threw = null;
    try {
      await mergeFeatureWorktree(mainRoot, { id: created.id, companionEndCommand: 'node -e "process.exit(0)"' });
    } catch (err) {
      threw = err;
    }
    assert(threw && threw.code === 'WORKTREE_MERGE_DETACHED_HEAD', `expected WORKTREE_MERGE_DETACHED_HEAD, got ${threw ? threw.code || threw.message : '(no throw)'}`);
    assert(lstatExists(markerPath), 'the companion marker must survive a detached-HEAD refusal');
    assert(lstatExists(mountFullPath), 'the companion mount symlink must survive a detached-HEAD refusal');
  } finally {
    git(mainRoot, ['worktree', 'prune'], { allowFailure: true });
    fs.rmSync(mainRoot, { recursive: true, force: true });
    fs.rmSync(created.worktreeRoot, { recursive: true, force: true });
    fs.rmSync(targetDir, { recursive: true, force: true });
  }
});

// GH #84 incident replay: the old code tore the companion mount down BEFORE
// the dirty-check refusal, so a refused merge destroyed the mount, and any
// retry (after the caller cleaned the dirt) found no marker left to tear
// down again — the incident's exact complaint. gfb-3's fix reorders teardown
// to run only after every zero-mutation refusal clears (see
// mergeFeatureWorktreeStage above); this is the end-to-end proof that a
// refused-then-retried merge actually recovers.
await check('refused companion merge is retryable: refusal preserves the mount, and the retry after cleaning the dirt merges clean with companion teardown', async () => {
  const { mainRoot, targetDir, created } = await companionWorktreeFixture('demo-companion-retry');
  try {
    const markerPath = path.join(created.worktreeRoot, '.bee', 'companion-session.json');
    const mountFullPath = path.join(created.worktreeRoot, 'companion');
    assert(lstatExists(markerPath), 'sanity: companion marker exists before merge');
    assert(lstatExists(mountFullPath), 'sanity: companion mount symlink exists before merge');

    // (1) A genuinely dirty, uncommitted file refuses the merge.
    fs.writeFileSync(path.join(created.worktreeRoot, 'dirty.txt'), 'oops\n');

    let threw = null;
    try {
      await mergeFeatureWorktree(mainRoot, { id: created.id, companionEndCommand: 'node -e "process.exit(0)"' });
    } catch (err) {
      threw = err;
    }
    assert(threw && threw.code === 'WORKTREE_MERGE_WORKTREE_DIRTY', `expected WORKTREE_MERGE_WORKTREE_DIRTY on the first attempt, got ${threw ? threw.code || threw.message : '(no throw)'}`);
    assert(lstatExists(markerPath), 'the companion marker must survive the refused merge — the old bug destroyed it here');
    assert(lstatExists(mountFullPath), 'the companion mount symlink must survive the refused merge — the old bug destroyed it here');

    // (2) Clean the refusal cause: commit the dirty file in the worktree.
    git(created.worktreeRoot, ['add', 'dirty.txt']);
    git(created.worktreeRoot, ['commit', '-m', 'clean the dirt that blocked the first merge attempt']);

    // (3) Retry the SAME merge id — must now succeed, tear the companion
    // down, and land the merge commit on main. The old code could not reach
    // this: the mount was already gone from step (1), and teardownCompanionIfPresent
    // would have blown up (or silently no-op'd) trying to tear down a marker
    // that no longer existed.
    const merged = await mergeFeatureWorktree(mainRoot, { id: created.id, companionEndCommand: 'node -e "process.exit(0)"' });
    assert(merged.ok === true && merged.merged === true, `expected the retry to merge cleanly, got ${JSON.stringify(merged)}`);
    assert(merged.companion && merged.companion.ended === true, `expected companion.ended === true on the successful retry, got ${JSON.stringify(merged.companion)}`);
    assert(!lstatExists(markerPath), 'the companion marker must be torn down after the successful retry');
    assert(!lstatExists(mountFullPath), 'the companion mount symlink must be torn down after the successful retry');
    assert(fs.existsSync(path.join(mainRoot, 'dirty.txt')), 'the retried merge commit must actually be on main — the file that blocked the first attempt is now present');
  } finally {
    git(mainRoot, ['worktree', 'prune'], { allowFailure: true });
    fs.rmSync(mainRoot, { recursive: true, force: true });
    fs.rmSync(created.worktreeRoot, { recursive: true, force: true });
    fs.rmSync(targetDir, { recursive: true, force: true });
  }
});

printSummaryAndExit();
