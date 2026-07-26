// path-identity.mjs — windows-path-identity wpi-1: a single canonical
// path-identity comparison, used wherever two independently-obtained path
// strings must be checked for "same directory" rather than "same bytes".
//
// Two sources routinely disagree on separators and case even when they name
// the same directory: git's own stdout (`git rev-parse ...`, gitdir pointer
// files) always emits `/`, while Node's path machinery (`path.join`,
// `path.resolve`, `fs.realpathSync`) emits the native separator, in whatever
// case the filesystem reports. A raw `===`/`!==` on those two strings works
// on every platform where the two sources happen to agree, and silently
// breaks the moment they don't.
//
// This module is deliberately narrow: it is the comparison, not a general
// path-normalization utility, and it makes no platform assumption. Four
// corrections from validation shape every decision below (see
// docs/history/windows-path-identity/plan.md CORRECTION section):
//
//   1. Case sensitivity is per-VOLUME, not per-platform. APFS can be either,
//      Windows supports per-directory case sensitivity (WSL sets it), Linux
//      can mount case-insensitive filesystems. A `process.platform ===
//      'win32'` fold is itself the "two distinct paths compare equal" bug —
//      so this module PROBES the actual volume behaviour (by creating a
//      marker file and checking whether a case-flipped name also "exists")
//      rather than assuming it, and caches the probe result per root
//      directory so repeated comparisons under the same tree don't re-probe.
//   2. Filesystem identity (device + inode) is a fast path, never the only
//      rule. Node reports `ino = 0` on filesystems with no file index
//      (FAT/exFAT, many SMB/UNC shares) — treating a zero inode or zero
//      device as proof of identity would make two genuinely distinct
//      directories compare EQUAL, exactly the wrong-pointer-accepted failure
//      this module exists to prevent. A zero inode/device (or either path
//      simply not existing) is treated as UNUSABLE and this module falls
//      back to the normalized-string comparison instead.
//   3. Hardlinks and junctions also compare EQUAL by filesystem identity
//      while remaining distinct path strings — identity is the right
//      confirmation of "same directory" once it is usable, but it never
//      substitutes for a fallback when it isn't.
//   4. Every comparison and every probe is injectable (a `statFn` and a
//      `detectCaseFold` option, both defaulting to the real filesystem) so
//      callers — and this module's own tests — can pin platform behaviour
//      explicitly without needing an actual case-insensitive or
//      no-file-index volume on hand.

import fs from 'node:fs';
import path from 'node:path';

// ---------------------------------------------------------------------------
// Volume case-behaviour detection, cached per probed root.
// ---------------------------------------------------------------------------

const caseFoldCache = new Map();

/**
 * Walks up from `startPath` to the nearest ancestor that actually exists on
 * disk — the probe needs a real, writable directory, and the two paths being
 * compared may themselves not exist yet (e.g. a stale gitdir pointer).
 */
function nearestExistingAncestor(startPath) {
  let current = path.resolve(startPath);
  // eslint-disable-next-line no-constant-condition
  while (true) {
    try {
      if (fs.existsSync(current)) return current;
    } catch {
      // fall through to parent
    }
    const parent = path.dirname(current);
    if (parent === current) return current; // reached the filesystem root
    current = parent;
  }
}

/**
 * Probes whether the directory `dir` sits on a case-insensitive volume by
 * writing a uniquely-named marker file and checking whether a case-flipped
 * spelling of that same name also resolves — the standard "create lower,
 * look up upper" technique. Best-effort: any failure to write/clean up the
 * probe file is treated as case-SENSITIVE (the safer default — it never
 * folds two paths together that a real case-sensitive volume would keep
 * distinct).
 */
function probeCaseInsensitive(dir) {
  const marker = `.bee-case-probe-${process.pid}-${Date.now()}-${Math.random().toString(36).slice(2)}`;
  const lower = marker.toLowerCase();
  const upper = marker.toUpperCase();
  const lowerPath = path.join(dir, lower);
  try {
    fs.writeFileSync(lowerPath, '');
    try {
      return fs.existsSync(path.join(dir, upper));
    } finally {
      try {
        fs.unlinkSync(lowerPath);
      } catch {
        // best-effort cleanup only
      }
    }
  } catch {
    return false;
  }
}

/**
 * Detects (and caches, per resolved root directory) whether the volume
 * containing `samplePath` is case-insensitive. Never derived from
 * `process.platform` — see module doc comment, correction 1.
 */
export function detectCaseInsensitiveVolume(samplePath, { probe = probeCaseInsensitive, cache = caseFoldCache } = {}) {
  const root = nearestExistingAncestor(samplePath);
  if (cache.has(root)) return cache.get(root);
  const result = probe(root);
  cache.set(root, result);
  return result;
}

/** Test-only: clears the per-root cache so a test can re-probe deliberately. */
export function _resetCaseFoldCacheForTests() {
  caseFoldCache.clear();
}

// ---------------------------------------------------------------------------
// Filesystem-identity fast path (device + inode).
// ---------------------------------------------------------------------------

/**
 * Returns `true`/`false` when filesystem identity is usable and decisive,
 * or `null` when it is not (either path missing, or either side reports a
 * zero inode/device — correction 2: never trust that as proof of anything).
 */
function tryIdentityEqual(resolvedA, resolvedB, statFn) {
  let statA;
  let statB;
  try {
    statA = statFn(resolvedA);
  } catch {
    return null;
  }
  try {
    statB = statFn(resolvedB);
  } catch {
    return null;
  }
  if (!statA || !statB) return null;
  if (!statA.ino || !statA.dev || !statB.ino || !statB.dev) return null;
  return statA.dev === statB.dev && statA.ino === statB.ino;
}

// ---------------------------------------------------------------------------
// Canonical comparison.
// ---------------------------------------------------------------------------

/** Separator normalization: a literal backslash is only ever a Windows
 * separator inside a path string (illegal in a POSIX filename), so this
 * fold is safe on every platform, not just win32. */
function normalizeSeparators(p) {
  return p.replace(/\\/g, path.sep);
}

/**
 * canonicalPathsEqual(a, b, opts?) — the single comparison this module
 * exists to provide. Resolves + normalizes separators on both sides, then:
 *
 *   1. tries filesystem identity (device + inode) when usable — the fast,
 *      authoritative path for the common case;
 *   2. falls back to a normalized-string comparison, case-folded ONLY when
 *      the volume in play is detected (never assumed) to be
 *      case-insensitive.
 *
 * `opts.statFn` (default `fs.statSync`) and `opts.detectCaseFold` (default
 * `detectCaseInsensitiveVolume`) are injectable so a caller can pin either
 * branch's behaviour explicitly — this is how the module's own tests prove
 * the zero-inode fallback and the case-insensitive fold without needing a
 * real FAT/exFAT volume or a real case-insensitive disk on hand.
 */
export function canonicalPathsEqual(a, b, opts = {}) {
  const { statFn = fs.statSync, detectCaseFold = detectCaseInsensitiveVolume } = opts;

  const resolvedA = path.resolve(normalizeSeparators(a));
  const resolvedB = path.resolve(normalizeSeparators(b));

  const identity = tryIdentityEqual(resolvedA, resolvedB, statFn);
  if (identity !== null) return identity;

  const caseInsensitive = detectCaseFold(resolvedA);
  const strA = caseInsensitive ? resolvedA.toLowerCase() : resolvedA;
  const strB = caseInsensitive ? resolvedB.toLowerCase() : resolvedB;
  return strA === strB;
}
