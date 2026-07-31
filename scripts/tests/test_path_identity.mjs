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
  canSymlink,
  envSkipLine,
  SYMLINK_SKIP_REASON,
  tmpdirIsCaseSensitive,
  CASE_SENSITIVE_FS_SKIP_REASON,
} from '../lib/env-capabilities.mjs';
import {
  canonicalPathsEqual,
  detectCaseInsensitiveVolume,
  _resetCaseFoldCacheForTests,
  _readOnlyCaseProbeForTests,
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
  // Unlink before rewriting: on Windows git marks a worktree's `.git` pointer
  // file hidden, and opening a hidden file for truncating write fails EPERM —
  // a fresh create carries no hidden attribute, so the rewrite works on every
  // platform. (Test-fixture concern only; the content contract is unchanged.)
  fs.rmSync(gitFile, { force: true });
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

/**
 * Hygiene (residual #4 from the rework round): `createFeatureWorktree`
 * places the worktree as a SIBLING of `mainRoot`, not nested inside it, so
 * removing `mainRoot` alone never removes it. `git worktree prune` only
 * cleans up git's own administrative records for an already-manually-deleted
 * worktree — it never deletes an existing worktree directory itself. A test
 * whose merge attempt THROWS before `mergeFeatureWorktree`'s own cleanup
 * path ever runs (every refusal-path test below) therefore leaked the
 * sibling worktree directory on every run until this helper explicitly
 * removes it too, best-effort, alongside `mainRoot`.
 */
function cleanupFixture(mainRoot, worktreeRoot) {
  spawnSync('git', ['worktree', 'prune'], { cwd: mainRoot, encoding: 'utf8' });
  if (worktreeRoot) fs.rmSync(worktreeRoot, { recursive: true, force: true });
  fs.rmSync(mainRoot, { recursive: true, force: true });
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

// The two NEGATIVE CONTROLs below premise a case-SENSITIVE volume ("two
// case-differing directories are genuinely distinct" / "detection correctly
// reports case-sensitive, so no fold"). On a case-insensitive volume (default
// NTFS) the premise itself is false — mkdir of the second spelling EEXISTs,
// and folding IS the correct detected behaviour — so the controls cannot
// fire. Skip loudly; the injected-fold checks below still pin both branches.
if (!tmpdirIsCaseSensitive()) {
  console.log(envSkipLine(CASE_SENSITIVE_FS_SKIP_REASON, 'NEGATIVE CONTROL (real filesystem, no injection): case-differing directories stay UNEQUAL'));
  console.log(envSkipLine(CASE_SENSITIVE_FS_SKIP_REASON, 'NEGATIVE CONTROL, string-fallback branch: case-mismatched non-existent pair stays UNEQUAL'));
} else {
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
}

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

await check('an injected case-insensitive volume detection folds case in the string fallback (BOTH sides must agree, not just side A) — a mixed-case spelling of the SAME non-existent path resolves EQUAL only when both sides probe case-insensitive', async () => {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), 'bee-path-identity-fold-'));
  try {
    const a = path.join(root, 'Worktrees', 'Feature-1');
    const bMixedCase = path.join(root, 'worktrees', 'feature-1'); // same segments, case only
    assert(!fs.existsSync(a), 'fixture precondition: identity must be unusable (path does not exist) so the string fallback runs');
    assert(canonicalPathsEqual(a, bMixedCase, { detectCaseFold: () => true }) === true, 'a mixed-case spelling of the same path must resolve EQUAL once BOTH sides are (here, pinned) case-insensitive');
    assert(canonicalPathsEqual(a, bMixedCase, { detectCaseFold: () => false }) === false, 'the SAME mixed-case pair must stay UNEQUAL when the volume is (correctly) detected as case-sensitive — the fold is conditional on real volume behaviour, never automatic');
    // Straddling pair: side A probes case-insensitive, side B probes case-sensitive (a
    // real scenario for two paths on different volumes/mounts). The safe default is
    // to NOT fold — folding on a mismatched pair could only ever widen a "not equal"
    // into a wrong "equal", never the reverse, so disagreement must refuse to fold.
    let call = 0;
    const straddling = (p) => {
      call += 1;
      return call === 1; // side A: true (case-insensitive); side B: false (case-sensitive)
    };
    assert(canonicalPathsEqual(a, bMixedCase, { detectCaseFold: straddling }) === false, 'a pair straddling a case-insensitive side A and a case-sensitive side B must NOT fold — the fold requires BOTH sides to agree, never just side A (residual #1)');
  } finally {
    fs.rmSync(root, { recursive: true, force: true });
  }
});

await check('POSIX REGRESSION GUARD (real fixture, the exact blocker a rework-round judge found): a literal backslash is DATA on POSIX, never a separator — a real directory named `a\\b` and a genuinely different real directory `a/b` (two nested segments) must compare UNEQUAL', async () => {
  if (path.sep === '\\') return; // this guard is meaningless (and its fixture unbuildable) on an actual Windows runtime
  const root = fs.mkdtempSync(path.join(os.tmpdir(), 'bee-path-identity-posix-backslash-'));
  try {
    const backslashDir = path.join(root, 'a\\b'); // ONE segment, literal backslash character in the name
    fs.mkdirSync(backslashDir);
    const nestedDir = path.join(root, 'a', 'b'); // TWO segments, genuinely different real directory
    fs.mkdirSync(path.dirname(nestedDir), { recursive: true });
    fs.mkdirSync(nestedDir);
    assert(fs.statSync(backslashDir).ino !== fs.statSync(nestedDir).ino, 'fixture precondition: the two are genuinely distinct real directories');

    assert(canonicalPathsEqual(backslashDir, nestedDir) === false, 'a directory literally named "a\\b" and the genuinely different "a/b" must compare UNEQUAL on POSIX — folding the backslash to a separator here (BEFORE resolve, BEFORE stat) is exactly the false-equal a rework judge caught: it would have stat\'d "a/b" for BOTH sides and reported them equal');
  } finally {
    fs.rmSync(root, { recursive: true, force: true });
  }
});

await check('WINDOWS-SHAPED FOLD (platformPath: path.win32 — Node\'s own real win32 path semantics, not a mock): a backslash-form and a forward-slash-form of "the same" path, differing only in separator style and case, resolve EQUAL once case-insensitivity is (pinned) true — proving separator handling is delegated to the platform\'s OWN resolve, never hand-rolled', async () => {
  const backslashForm = 'C:\\Users\\Demo\\Worktrees\\Feature-1';
  const slashForm = 'C:/Users/Demo/worktrees/feature-1'; // same path, forward slashes + case differs
  const alwaysThrow = () => {
    throw new Error('no real C:\\ filesystem on this box — force the string-fallback branch');
  };
  assert(
    canonicalPathsEqual(backslashForm, slashForm, { platformPath: path.win32, statFn: alwaysThrow, detectCaseFold: () => true }) === true,
    'a backslash-form and a forward-slash-form of the same Windows path (differing only in separator style + case) must resolve EQUAL — path.win32.resolve normalizes the separator style on its own, and case-fold handles the rest',
  );
  assert(
    canonicalPathsEqual(backslashForm, slashForm, { platformPath: path.win32, statFn: alwaysThrow, detectCaseFold: () => false }) === false,
    'the SAME pair must stay UNEQUAL when the (pinned) volume is case-sensitive — separator normalization alone is not a license to also fold case',
  );
});

await check('detectCaseInsensitiveVolume caches by VOLUME (device number), not by the ancestor\'s path string — two different, genuinely existing directories on the SAME device share one probe result (residual #2)', async () => {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), 'bee-path-identity-devcache-'));
  try {
    const dirA = path.join(root, 'alpha');
    const dirB = path.join(root, 'beta');
    fs.mkdirSync(dirA);
    fs.mkdirSync(dirB);
    assert(fs.statSync(dirA).dev === fs.statSync(dirB).dev, 'fixture precondition: both directories must be on the same device (same tmp filesystem)');
    const calls = [];
    const probe = (dir) => {
      calls.push(dir);
      return false;
    };
    const cache = new Map();
    detectCaseInsensitiveVolume(dirA, { probe, cache });
    detectCaseInsensitiveVolume(dirB, { probe, cache });
    assert(calls.length === 1, `two DIFFERENT ancestor directories on the SAME device must share one cached probe result (keyed by device, not by ancestor path), ran ${calls.length} time(s)`);
  } finally {
    fs.rmSync(root, { recursive: true, force: true });
  }
});

await check('the volume probe is READ-ONLY when the sample directory itself exists and has a case-bearing basename (residual #3) — proven by calling the read-only probe directly and getting a CONCLUSIVE (non-null) answer, which means it never fell through to the write-based marker probe at all (a leftover-file check alone cannot prove this: the write probe cleans up after itself too)', async () => {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), 'bee-path-identity-readonly-'));
  try {
    const sample = path.join(root, 'ReadOnlySample');
    fs.mkdirSync(sample);
    const result = _readOnlyCaseProbeForTests(sample);
    assert(result !== null, 'the read-only probe must conclusively settle an existing, case-bearing directory on its own — a null here means it punted to the write-based fallback unnecessarily');
    // The expected answer is the volume's REAL behaviour, measured
    // independently (does the case-flipped spelling resolve to the same
    // entry?) — false on a case-sensitive box, true on default NTFS. The
    // probe must agree with the measurement either way.
    const measured = fs.existsSync(path.join(root, 'readonlysample'));
    assert(result === measured, `the read-only probe must report this volume's real case behaviour (measured ${measured} via a case-flipped lookup), got ${result}`);
    // Also confirm the module-level entrypoint (detectCaseInsensitiveVolume, no
    // injection) leaves the directory with no marker litter, as a second signal.
    detectCaseInsensitiveVolume(sample);
    const entries = fs.readdirSync(sample);
    assert(entries.length === 0, `no leftover entries expected either way — found: ${entries.join(', ')}`);
  } finally {
    fs.rmSync(root, { recursive: true, force: true });
  }
});

// ─── Layer 2: the resolveWorktreeById call site, through the real mergeFeatureWorktree entrypoint ──

if (!canSymlink()) {
  console.log(envSkipLine(SYMLINK_SKIP_REASON, 'SITE FIX (real, no injection): symlink-aliased reverse gitdir pointer resolves through the REAL resolveWorktreeById'));
} else {
await check('SITE FIX (real, no injection): a reverse gitdir pointer that is a DIFFERENT STRING but the SAME real directory (via a symlink alias — the real-fs shape of 8.3 short-name aliasing) resolves correctly through the REAL resolveWorktreeById, end to end, with the DEFAULT pathsEqual (no test override) — filesystem identity is what decides it', async () => {
  const mainRoot = makeOrdinaryRepoFixture();
  let created;
  try {
    created = await createFeatureWorktree(mainRoot, { feature: 'wpi-symlink-alias' });
    const gitWorktreeDir = path.join(mainRoot, '.git', 'worktrees', created.id);
    const aliasPath = createGitdirAlias(gitWorktreeDir);
    rewriteReversePointer(created.worktreeRoot, () => aliasPath);
    assert(fs.statSync(aliasPath).ino === fs.statSync(gitWorktreeDir).ino, 'fixture precondition: the alias must be a real symlink resolving to the SAME inode as the real gitdir');

    const merged = await mergeFeatureWorktree(mainRoot, { id: created.id, cleanup: true });
    assert(merged.cleanup && merged.cleanup.ok === true, `expected the symlink-aliased worktree to resolve and merge cleanly with the default (real) pathsEqual, got ${JSON.stringify(merged)}`);
  } finally {
    cleanupFixture(mainRoot, created && created.worktreeRoot);
  }
});
}

await check('SITE FIX (case-fold branch, injected): a case-flipped reverse gitdir pointer is NOT refused at the resolveWorktreeById site itself once pathsEqual simulates a case-insensitive volume — this box\'s real git subprocess still cannot itself navigate a fake-cased path afterward (no real case-insensitive filesystem is available here), so the ONLY acceptable failure past this point is that unrelated, untyped git-invocation error, never the typed WORKTREE_MERGE_UNKNOWN_ID the pre-fix comparison would have produced right at resolution', async () => {
  const mainRoot = makeOrdinaryRepoFixture();
  let created;
  try {
    created = await createFeatureWorktree(mainRoot, { feature: 'wpi-case-mismatch' });
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
    cleanupFixture(mainRoot, created && created.worktreeRoot);
  }
});

await check('RED-FIRST REGRESSION GUARD: the SAME case-mismatched fixture is refused (WORKTREE_MERGE_UNKNOWN_ID) when pathsEqual is the pre-fix raw `===` comparison — proves the injected comparison function, not some other effect, is what makes the case-fold test above get past resolution', async () => {
  const mainRoot = makeOrdinaryRepoFixture();
  let created;
  try {
    created = await createFeatureWorktree(mainRoot, { feature: 'wpi-case-mismatch-prefix' });
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
    cleanupFixture(mainRoot, created && created.worktreeRoot);
  }
});

await check('a GENUINELY wrong reverse pointer (pointing at a totally different worktrees/<id> directory) still returns null and refuses WORKTREE_MERGE_UNKNOWN_ID even with the generous case-insensitive-simulating pathsEqual — the fix never accepts an actually-wrong pointer', async () => {
  const mainRoot = makeOrdinaryRepoFixture();
  let created;
  try {
    created = await createFeatureWorktree(mainRoot, { feature: 'wpi-wrong-pointer' });
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
    cleanupFixture(mainRoot, created && created.worktreeRoot);
  }
});

printSummaryAndExit();
