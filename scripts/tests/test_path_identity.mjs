#!/usr/bin/env node
// test_path_identity.mjs — windows-path-identity wpi-1.
//
// Read-first (test-economy D5): the nearest existing coverage of this area is
// packages/bee/tests/test_worktree_store.mjs (createFeatureWorktree +
// mergeFeatureWorktree against a real git fixture) and
// scripts/tests/test_worktree_merge_queue.mjs (spawns the CLI end-to-end).
// Neither exercises a MISMATCHED gitdir pointer (case/separator drift) —
// both fixtures always produce byte-identical forward/reverse pointers on
// this Linux dev box, which is exactly why the bug they'd need to catch
// only ever showed up on real Windows CI. This file adds that missing case,
// plus direct unit coverage of the new canonical comparison helper itself
// (packages/bee/lib/path-identity.mjs) — a new, permanent suite, not a
// row appended to either of those, because it proves a distinct mechanism
// (a comparison helper's platform-branch logic) that neither existing file
// has any fixture for.
//
// Two layers:
//   1. Direct unit tests of canonicalPathsEqual/detectCaseInsensitiveVolume
//      against constructed inputs (real fixtures where possible; injected
//      statFn/detectCaseFold only to pin a platform branch this Linux box
//      cannot produce for real — no case-insensitive volume, no
//      zero-inode filesystem).
//   2. Site tests that drive the REAL resolveWorktreeById through its one
//      real call site — mergeFeatureWorktree(mainRoot, options) ->
//      mergeFeatureWorktreeStage -> resolveWorktreeById — using the
//      `pathsEqual` injection option threaded there for exactly this
//      purpose (see worktree-store.mjs's doc comment on both functions).
//
// Red-first proof (recorded in the cap trace, not re-derived here): with
// packages/bee/lib/worktree-store.mjs's :1110 comparison reverted to the
// pre-fix raw `!==`, test F below fails (the mismatched fixture refuses to
// resolve); with the fix applied, it passes. Test G is the permanent,
// always-on regression guard for the same fact — it does not depend on
// reverting any source, it substitutes the pre-fix comparison via the same
// injection seam so the distinction stays provable on every future run.

import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { spawnSync } from 'node:child_process';
import { check, assert, printSummaryAndExit } from '../lib/test-fixture.mjs';
import {
  canonicalPathsEqual,
  detectCaseInsensitiveVolume,
  _resetCaseFoldCacheForTests,
} from '../../packages/bee/lib/path-identity.mjs';
import { createFeatureWorktree, mergeFeatureWorktree } from '../../packages/bee/lib/worktree-store.mjs';

function git(cwd, args, { allowFailure = false } = {}) {
  const result = spawnSync('git', args, { cwd, encoding: 'utf8' });
  if (!allowFailure && result.status !== 0) {
    throw new Error(`git ${args.join(' ')} failed (${result.status}): ${result.stderr || result.stdout}`);
  }
  return result;
}

function makeOrdinaryRepoFixture() {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), 'bee-path-identity-main-'));
  git(root, ['init', '-b', 'main']);
  git(root, ['config', 'user.email', 'bee@example.invalid']);
  git(root, ['config', 'user.name', 'Bee Test']);
  fs.writeFileSync(path.join(root, '.gitignore'), '.bee/\n');
  fs.writeFileSync(path.join(root, 'README.md'), 'demo\n');
  git(root, ['add', '.gitignore', 'README.md']);
  git(root, ['commit', '-m', 'base']);
  return root;
}

/** Flips ASCII letter case in every alphabetic character of a string. */
function flipCase(s) {
  return [...s].map((ch) => (ch === ch.toUpperCase() ? ch.toLowerCase() : ch.toUpperCase())).join('');
}

/**
 * Rewrites a fresh worktree's reverse gitdir pointer (worktreeRoot/.git) so
 * its target path spelling is `transform`ed, while still parsing as a valid
 * "gitdir: <path>" pointer. Returns the untransformed original target path
 * (the real `<mainRoot>/.git/worktrees/<id>` directory) for assertions.
 */
function rewriteReversePointer(worktreeRoot, transform) {
  const gitFile = path.join(worktreeRoot, '.git');
  const raw = fs.readFileSync(gitFile, 'utf8').trim();
  const match = raw.match(/^gitdir:\s*(.+)$/);
  assert(match, `fixture .git file at ${gitFile} is not a "gitdir: ..." pointer`);
  const original = match[1].trim();
  fs.writeFileSync(gitFile, `gitdir: ${transform(original)}\n`);
  return original;
}

/**
 * Creates a real directory symlink alongside `gitWorktreeDir` that resolves
 * to the SAME real directory under a DIFFERENT path string — a real-fs
 * analog of Windows 8.3 short-name aliasing / a case-insensitive lookup:
 * two genuinely different strings, one physical directory. Unlike a case
 * flip (which this box's case-SENSITIVE filesystem cannot actually make
 * resolve), a symlink alias is real: `fs.statSync` follows it to the same
 * device+inode, AND a real `git` subprocess can navigate through it, so the
 * fixture below can prove the fix through an actually-succeeding merge,
 * not just a resolution-site assertion.
 */
function createGitdirAlias(gitWorktreeDir) {
  const aliasPath = `${gitWorktreeDir}-alias`;
  fs.symlinkSync(gitWorktreeDir, aliasPath, 'dir');
  return aliasPath;
}

// ─── Layer 1: canonicalPathsEqual / detectCaseInsensitiveVolume unit tests ──

await check('detectCaseInsensitiveVolume probes the nearest EXISTING ancestor (never assumes from process.platform) and caches per root — a second call for the same root does not re-probe', async () => {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), 'bee-path-identity-probe-'));
  try {
    const calls = [];
    const probe = (dir) => {
      calls.push(dir);
      return false;
    };
    const cache = new Map();
    const notYetCreated = path.join(root, 'sub', 'deep'); // does not exist -> ancestor is `root`
    const first = detectCaseInsensitiveVolume(notYetCreated, { probe, cache });
    const second = detectCaseInsensitiveVolume(notYetCreated, { probe, cache });
    assert(calls.length === 1, `probe must run exactly once across two calls for the same root, ran ${calls.length} time(s)`);
    assert(calls[0] === root, `probe must run against the nearest existing ancestor (${root}), got ${calls[0]}`);
    assert(first === false && second === false, 'both calls must return the (cached) probe result');
  } finally {
    fs.rmSync(root, { recursive: true, force: true });
  }
});

await check('NEGATIVE CONTROL (real filesystem, no injection): this dev box is case-sensitive — two directories differing only by case are genuinely distinct, and canonicalPathsEqual must say so via real filesystem identity', async () => {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), 'bee-path-identity-casefs-'));
  try {
    const lower = path.join(root, 'samplecase');
    const upper = path.join(root, 'SampleCase');
    fs.mkdirSync(lower);
    fs.mkdirSync(upper);
    assert(fs.statSync(lower).ino !== fs.statSync(upper).ino, 'fixture precondition: the two directories must be genuinely distinct inodes on this filesystem');
    assert(canonicalPathsEqual(lower, upper) === false, 'two really-distinct-case directories must compare UNEQUAL — folding case here would be the "EQUAL for two paths that must stay distinct" bug');
  } finally {
    fs.rmSync(root, { recursive: true, force: true });
  }
});

await check('NEGATIVE CONTROL, string-fallback branch: two case-differing, NON-EXISTENT paths (identity unusable) stay UNEQUAL under this box\'s real (case-sensitive) detected behaviour — case is never folded by default', async () => {
  _resetCaseFoldCacheForTests();
  const root = fs.mkdtempSync(path.join(os.tmpdir(), 'bee-path-identity-nofold-'));
  try {
    const a = path.join(root, 'NoSuchDir', 'child');
    const b = path.join(root, 'nosuchdir', 'child');
    assert(!fs.existsSync(a) && !fs.existsSync(b), 'fixture precondition: neither path exists, forcing the string-fallback branch');
    assert(canonicalPathsEqual(a, b) === false, 'with no case-insensitive volume detected, a case-mismatched pair must stay UNEQUAL');
  } finally {
    fs.rmSync(root, { recursive: true, force: true });
  }
});

await check('zero inode/device is treated as UNUSABLE and falls back to string comparison — two distinct real directories, forced to report a zero inode/device, still compare UNEQUAL (never accepted as identity)', async () => {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), 'bee-path-identity-zeroid-'));
  try {
    const a = path.join(root, 'alpha');
    const b = path.join(root, 'beta');
    fs.mkdirSync(a);
    fs.mkdirSync(b);
    const zeroStat = (p) => ({ ...fs.statSync(p), ino: 0, dev: 0 });
    assert(canonicalPathsEqual(a, a, { statFn: zeroStat }) === true, 'the SAME path, even with a zero inode/device, must still compare EQUAL via the string fallback');
    assert(canonicalPathsEqual(a, b, { statFn: zeroStat }) === false, 'two DISTINCT real directories, forced to report a zero inode/device, must still compare UNEQUAL — a zero identity must never be accepted as proof');
  } finally {
    fs.rmSync(root, { recursive: true, force: true });
  }
});

await check('an injected case-insensitive volume detection folds case (and separators) in the string fallback — a mixed-separator, mixed-case spelling of the SAME non-existent path resolves EQUAL', async () => {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), 'bee-path-identity-fold-'));
  try {
    const a = path.join(root, 'Worktrees', 'Feature-1');
    const bMixed = `${root}${path.sep}worktrees\\feature-1`; // mixed case + a literal backslash segment
    assert(!fs.existsSync(a), 'fixture precondition: identity must be unusable (path does not exist) so the string fallback runs');
    const detectCaseFold = () => true;
    assert(canonicalPathsEqual(a, bMixed, { detectCaseFold }) === true, 'a mixed-separator, mixed-case spelling of the same path must resolve EQUAL once the volume is detected (or, here, pinned) as case-insensitive');
    assert(canonicalPathsEqual(a, bMixed, { detectCaseFold: () => false }) === false, 'the SAME mixed-case pair must stay UNEQUAL when the volume is (correctly) detected as case-sensitive — the fold is conditional on real volume behaviour, never automatic');
  } finally {
    fs.rmSync(root, { recursive: true, force: true });
  }
});

// ─── Layer 2: the resolveWorktreeById call site, through the real mergeFeatureWorktree entrypoint ──

await check('SITE FIX (real, no injection): a reverse gitdir pointer that is a DIFFERENT STRING but the SAME real directory (via a symlink alias — the real-fs shape of 8.3 short-name aliasing) resolves correctly through the REAL resolveWorktreeById, end to end, with the DEFAULT pathsEqual (no test override) — filesystem identity is what decides it', async () => {
  const mainRoot = makeOrdinaryRepoFixture();
  try {
    const created = await createFeatureWorktree(mainRoot, { feature: 'wpi-symlink-alias' });
    const gitWorktreeDir = path.join(mainRoot, '.git', 'worktrees', created.id);
    const aliasPath = createGitdirAlias(gitWorktreeDir);
    rewriteReversePointer(created.worktreeRoot, () => aliasPath);
    assert(fs.statSync(aliasPath).ino === fs.statSync(gitWorktreeDir).ino, 'fixture precondition: the alias must be a real symlink resolving to the SAME inode as the real gitdir');

    const merged = await mergeFeatureWorktree(mainRoot, { id: created.id, cleanup: true });
    assert(merged.cleanup && merged.cleanup.ok === true, `expected the symlink-aliased worktree to resolve and merge cleanly with the default (real) pathsEqual, got ${JSON.stringify(merged)}`);
  } finally {
    git(mainRoot, ['worktree', 'prune'], { allowFailure: true });
    fs.rmSync(mainRoot, { recursive: true, force: true });
  }
});

await check('SITE FIX (case-fold branch, injected): a case-flipped reverse gitdir pointer is NOT refused at the resolveWorktreeById site itself once pathsEqual simulates a case-insensitive volume — this box\'s real git subprocess still cannot itself navigate a fake-cased path afterward (no real case-insensitive filesystem is available here), so the ONLY acceptable failure past this point is that unrelated, untyped git-invocation error, never the typed WORKTREE_MERGE_UNKNOWN_ID the pre-fix comparison would have produced right at resolution', async () => {
  const mainRoot = makeOrdinaryRepoFixture();
  try {
    const created = await createFeatureWorktree(mainRoot, { feature: 'wpi-case-mismatch' });
    const original = rewriteReversePointer(created.worktreeRoot, flipCase);
    assert(original !== flipCase(original), 'fixture precondition: flipping case must actually change the pointer spelling (otherwise this proves nothing)');

    let threw = null;
    try {
      await mergeFeatureWorktree(mainRoot, {
        id: created.id,
        cleanup: true,
        pathsEqual: (a, b) => canonicalPathsEqual(a, b, { detectCaseFold: () => true }),
      });
    } catch (err) {
      threw = err;
    }
    assert(!(threw && threw.code === 'WORKTREE_MERGE_UNKNOWN_ID'), `resolveWorktreeById itself must accept the case-mismatched pointer once pathsEqual folds case — got a WORKTREE_MERGE_UNKNOWN_ID refusal instead, meaning resolution failed: ${threw ? threw.message : '(no throw)'}`);
    if (threw) {
      assert(threw.code === undefined && /git status --porcelain/.test(threw.message), `any failure past resolution must be the known, untyped git-subprocess artifact of faking a case-insensitive path on this case-sensitive box, got: ${threw.code || ''} ${threw.message}`);
    }
  } finally {
    git(mainRoot, ['worktree', 'prune'], { allowFailure: true });
    fs.rmSync(mainRoot, { recursive: true, force: true });
  }
});

await check('RED-FIRST REGRESSION GUARD: the SAME case-mismatched fixture is refused (WORKTREE_MERGE_UNKNOWN_ID) when pathsEqual is the pre-fix raw `===` comparison — proves the injected comparison function, not some other effect, is what makes the case-fold test above get past resolution', async () => {
  const mainRoot = makeOrdinaryRepoFixture();
  try {
    const created = await createFeatureWorktree(mainRoot, { feature: 'wpi-case-mismatch-prefix' });
    rewriteReversePointer(created.worktreeRoot, flipCase);

    let threw = null;
    try {
      await mergeFeatureWorktree(mainRoot, {
        id: created.id,
        cleanup: true,
        pathsEqual: (a, b) => a === b, // the exact pre-fix behavior at worktree-store.mjs:1110
      });
    } catch (err) {
      threw = err;
    }
    assert(threw && threw.code === 'WORKTREE_MERGE_UNKNOWN_ID', `expected the pre-fix raw comparison to refuse the case-mismatched fixture with WORKTREE_MERGE_UNKNOWN_ID, got ${threw ? threw.code || threw.message : '(no throw — the fixture is not actually mismatched)'}`);
  } finally {
    git(mainRoot, ['worktree', 'prune'], { allowFailure: true });
    fs.rmSync(mainRoot, { recursive: true, force: true });
  }
});

await check('a GENUINELY wrong reverse pointer (pointing at a totally different worktrees/<id> directory) still returns null and refuses WORKTREE_MERGE_UNKNOWN_ID even with the generous case-insensitive-simulating pathsEqual — the fix never accepts an actually-wrong pointer', async () => {
  const mainRoot = makeOrdinaryRepoFixture();
  try {
    const created = await createFeatureWorktree(mainRoot, { feature: 'wpi-wrong-pointer' });
    rewriteReversePointer(created.worktreeRoot, (original) => path.join(path.dirname(original), 'totally-different-id'));

    let threw = null;
    try {
      await mergeFeatureWorktree(mainRoot, {
        id: created.id,
        cleanup: true,
        pathsEqual: (a, b) => canonicalPathsEqual(a, b, { detectCaseFold: () => true }),
      });
    } catch (err) {
      threw = err;
    }
    assert(threw && threw.code === 'WORKTREE_MERGE_UNKNOWN_ID', `a genuinely wrong pointer must still refuse as WORKTREE_MERGE_UNKNOWN_ID, got ${threw ? threw.code || threw.message : '(no throw — the fixture did not actually diverge)'}`);
  } finally {
    git(mainRoot, ['worktree', 'prune'], { allowFailure: true });
    fs.rmSync(mainRoot, { recursive: true, force: true });
  }
});

printSummaryAndExit();
