#!/usr/bin/env node
// test_worktree_store_merge.mjs — lib/worktree-store.mjs merge-path tests:
// mergeFeatureWorktree's checkProcessorLease/onVerifyTick hooks
// (multisession-native-22, D8 stage 5, advisor condition C) and the
// companion-teardown-ordering fix (gfb-3, GH #84).
//
// foundation-fixes fx-2 (CONTEXT.md D3): split VERBATIM out of
// packages/bee/tests/test_worktree_store.mjs at the topology/merge boundary
// — the store/topology tests (createFeatureWorktree registration,
// resolveContext workspaceId) stayed in the sibling suite. The
// git/gitText/makeOrdinaryRepoFixture fixture helpers are duplicated here
// rather than imported from the sibling test file, so the runner never
// double-executes either half (this repo has no existing cross-suite
// fixture-module pattern to follow instead — see scripts/lib/test-fixture.mjs,
// which is a generic runner shared by many suites, not a per-pair fixture).
// Windows runs git-heavy suites 2-4x slower (.github/workflows/windows.yml);
// halving the original suite's runtime keeps both halves well under the CI
// timeout ceiling. Same PASS/FAIL/exit-1 contract as every other suite here.

import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { spawnSync } from 'node:child_process';
import { check, assert, printSummaryAndExit } from '../../../scripts/lib/test-fixture.mjs';
import { createFeatureWorktree, mergeFeatureWorktree } from '../lib/worktree-store.mjs';

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
