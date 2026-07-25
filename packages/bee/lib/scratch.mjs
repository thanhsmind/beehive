// scratch.mjs — tree-hygiene D1/D2: the canonical scratch home (.bee/tmp/ +
// .bee/spikes/) and its broom (`bee tmp sweep`, wired in bee.mjs). See
// docs/history/tree-hygiene/CONTEXT.md for the locked decisions this module
// implements.
//
// SAFETY (the point of this module, per cell th-4's directive — an over-eager
// sweep that eats a deliverable is far worse than the garbage it removes):
// the sweep may ONLY EVER remove a path that has been canonically resolved
// (fs.realpathSync.native) and re-checked to be contained inside
// <repo>/.bee/tmp/ or <repo>/.bee/spikes/ IMMEDIATELY before removal. This
// reuses the exact containment idiom .bee/bin/hooks/bee-write-guard.mjs's
// canonicalRelPath/isUnderRoot already use for write-guard denial — a
// candidate whose realpath escapes both roots (including a symlink pointing
// outside) is REFUSED, never followed, never deleted.
//
// ROOT ITSELF is never trusted to be a real directory just because something
// answers at that path (rework, th4-scratch-root-symlink-escape): the two
// scratch ROOTS — the literal <repo>/.bee/tmp and <repo>/.bee/spikes — are
// lstat'd component-by-component from the repo's own realpath. If `.bee/tmp`
// or `.bee/spikes` (or the `.bee` segment itself) is a symlink, that root is
// REFUSED wholesale — excluded from scratchRoots(), reported in
// refused_roots — never realpath-resolved to "follow" it to whatever it
// points at. Earlier code realpathed the ROOT path itself and used the
// resolved TARGET as the authority root, so a symlinked `.bee/spikes ->
// <repo>` or `.bee/tmp -> docs/` silently relocated the entire delete
// authority onto the repo root or a deliverables tree — reproduced end to
// end through the shipped CLI (judge verdict, .bee/tmp/tree-hygiene/
// judge-th-4.json). Only a LITERAL, non-symlinked <repo>/.bee/tmp or
// <repo>/.bee/spikes is ever treated as a root.
//
// Default target set (D2, no --feature/--all given): a feature/session-named
// scratch dir is swept when it is NOT live — either its lane record (or the
// default pipeline's own state.feature) is at a terminal phase ("closed"),
// or no record names it anywhere ("absent"). An absent name only qualifies
// once an explicit --before cutoff says it's old enough (no default age
// window — same "no default purge" discipline `decisions archive` already
// uses for its own mandatory --before); a closed name qualifies immediately,
// no age check needed, since its closure is already the definitive signal.
// A LIVE feature's scratch is swept only when named explicitly via
// --feature; --all sweeps everything, live or not — D2's "clears the lot".
//
// WHAT COUNTS AS AN ENTRY (issues-46-53 D7, issue #53's adjacent finding):
// every top-level entry under a scratch root — directories, symlinks, AND
// plain files. Agents write loose evidence dumps and helper scripts straight
// into `.bee/tmp/` because that is the directory bee's own write guard tells
// them to use; a broom that only saw directories left them there forever. See
// listEntries below for the measurement and the reasoning.
//
// WHAT --feature MATCHES: the exact name (the documented live-scratch
// override), plus `<feature><sep>...` prefixes — bee's own `<feature>-<n>`
// cell-id convention, which is how the per-cell dirs and evidence files of
// one feature are actually named. See matchFeature below, including why a
// prefix match refuses to eat a LIVE sibling and an exact match does not.

import fs from 'node:fs';
import path from 'node:path';
import { readState, readLane } from './state.mjs';

export const SCRATCH_TMP_REL = '.bee/tmp';
export const SCRATCH_SPIKES_REL = '.bee/spikes';
const SCRATCH_ROOT_RELS = [SCRATCH_TMP_REL, SCRATCH_SPIKES_REL];
const TERMINAL_PHASES = new Set(['idle', 'compounding-complete']);

function realpathOrNull(p) {
  try {
    return fs.realpathSync.native(p);
  } catch {
    return null;
  }
}

// Same containment idiom as bee-write-guard.mjs's isUnderRoot: the real root
// itself counts as contained; anything else must be a strict descendant.
function isUnderRoot(parentReal, childReal) {
  if (!parentReal || !childReal) return false;
  const rel = path.relative(parentReal, childReal);
  return rel === '' || (rel !== '..' && !rel.startsWith(`..${path.sep}`) && !path.isAbsolute(rel));
}

// Walk `rel` (e.g. ".bee/tmp") one path segment at a time from `rootReal`,
// lstat-ing each segment WITHOUT resolving symlinks. Any segment that is a
// symlink — including the final one — REFUSES the whole root: the root is
// never realpath-resolved to "follow" it to a symlink target. Returns
// { abs, real: abs } only when every segment exists, is not a symlink, and
// the final segment is a real directory; otherwise { abs, refused: <reason> }
// with reason 'missing' (nothing to sweep there yet — not a safety issue) or
// 'symlinked_root' (a real safety refusal, reported to callers).
function literalRootInfo(rootReal, rel) {
  const segments = rel.split('/');
  let cursor = rootReal;
  const abs = path.join(rootReal, rel);
  for (const seg of segments) {
    cursor = path.join(cursor, seg);
    let stat;
    try {
      stat = fs.lstatSync(cursor);
    } catch {
      return { rel, abs, refused: 'missing' };
    }
    if (stat.isSymbolicLink()) {
      return { rel, abs, refused: 'symlinked_root' };
    }
    if (!stat.isDirectory()) {
      return { rel, abs, refused: 'not_a_directory' };
    }
  }
  // No segment was a symlink, so the literal path IS its own real path —
  // recorded as such (rather than re-realpathing) so `real` can never end up
  // pointing anywhere other than the literal <repo>/.bee/tmp|spikes prefix.
  return { rel, abs: cursor, real: cursor };
}

/** Resolve the two allowed scratch roots to their LITERAL, non-symlinked
 * absolute paths, skipping any that don't exist on disk yet (nothing to
 * sweep there — this never creates them). A root whose path (or any
 * intermediate segment, including `.bee` itself) is a symlink is EXCLUDED
 * here — see inspectScratchRoots for the same result plus the refusal
 * reason, used for reporting. */
export function scratchRoots(root) {
  return inspectScratchRoots(root).roots;
}

/** Same as scratchRoots, but also returns `refused` — every scratch root
 * that exists on disk but was refused because it (or an intermediate
 * segment) is a symlink, never a missing root (that's the ordinary "nothing
 * to sweep there yet" case, not a safety refusal). Callers report `refused`
 * in the sweep result (`refused_roots`) so a symlinked root is always
 * visible, never silently treated as "just empty". */
export function inspectScratchRoots(root) {
  const rootReal = realpathOrNull(root);
  if (!rootReal) return { roots: [], refused: [] };
  const infos = SCRATCH_ROOT_RELS.map((rel) => literalRootInfo(rootReal, rel));
  return {
    roots: infos.filter((entry) => !entry.refused).map(({ rel, abs, real }) => ({ rel, abs, real })),
    refused: infos
      .filter((entry) => entry.refused && entry.refused !== 'missing')
      .map(({ rel, abs, refused }) => ({ rel, path: abs, reason: refused })),
  };
}

/** Canonically resolve `candidateAbsPath` (must already exist) and confirm it
 * is contained inside one of `roots` (from scratchRoots). Returns the
 * matching root entry on success, or null when containment cannot be proved
 * — including a symlink whose target escapes both roots, which is refused
 * here rather than followed. Never throws. */
export function containedRoot(candidateAbsPath, roots) {
  const real = realpathOrNull(candidateAbsPath);
  if (!real) return null;
  return roots.find((r) => isUnderRoot(r.real, real)) || null;
}

// dirSize/countFiles never dereference a symlink for accounting (isSymbolicLink
// short-circuits to 0 for both) — a nested symlink is structurally removed
// along with its containing directory by fs.rmSync (which does not follow
// symlinks during recursive removal, the same "rm -rf never chases links"
// convention), but it is never sized or counted as a "file".
function dirSize(absPath) {
  let stat;
  try {
    stat = fs.lstatSync(absPath);
  } catch {
    return 0;
  }
  if (stat.isSymbolicLink()) return 0;
  if (!stat.isDirectory()) return stat.size;
  let entries = [];
  try {
    entries = fs.readdirSync(absPath, { withFileTypes: true });
  } catch {
    return 0;
  }
  let total = 0;
  for (const entry of entries) total += dirSize(path.join(absPath, entry.name));
  return total;
}

function countFiles(absPath) {
  let stat;
  try {
    stat = fs.lstatSync(absPath);
  } catch {
    return 0;
  }
  if (stat.isSymbolicLink()) return 0;
  if (!stat.isDirectory()) return 1;
  let entries = [];
  try {
    entries = fs.readdirSync(absPath, { withFileTypes: true });
  } catch {
    return 0;
  }
  let total = 0;
  for (const entry of entries) total += countFiles(path.join(absPath, entry.name));
  return total;
}

/** True when `name` (a feature-or-session scratch dir name) is a currently
 * LIVE feature — either bound as the default pipeline's active feature at a
 * non-terminal phase, or backed by a lane record at a non-terminal phase.
 * Anything else (a closed record, or no record found anywhere — an "absent"
 * feature/lane, or a bare session id) is NOT live. */
export function isLiveFeature(root, name) {
  const state = readState(root);
  if (state && state.feature === name && !TERMINAL_PHASES.has(state.phase)) return true;
  const lane = readLane(root, name);
  if (lane && !TERMINAL_PHASES.has(lane.phase)) return true;
  return false;
}

// hasRecord — true when `name` is named ANYWHERE (default state.feature or a
// lane record), regardless of live/closed. Used to distinguish "closed"
// (a record exists, but its phase is terminal) from "absent" (no record at
// all), which only "absent" gates behind an explicit --before cutoff.
function hasRecord(root, name) {
  const state = readState(root);
  if (state && state.feature === name) return true;
  return readLane(root, name) != null;
}

// A top-level scratch entry is ANY entry sitting directly under a scratch
// root — a directory, a symlink, or a PLAIN FILE.
//
// Files used to be filtered out here, and that is the whole of issue #53's
// adjacent finding (issues-46-53 D7). The documented promise is that
// `.bee/tmp/` is the scratch home and `bee tmp sweep` clears it — guards.mjs
// tells every agent, in its own refusal message, to "write it to .bee/tmp/
// instead ... and let `bee tmp sweep` clear it later", naming the ROOT, not a
// per-feature subdirectory. Agents did exactly that. The broom then never saw
// any of it: measured on this repo, 58 of 76 scratch entries were loose files
// at the root, permanently unsweepable — not by `--feature`, not by
// `--before`, and not even by `--all`, the flag D2 documents as "clears the
// lot". The documented behavior and the real behavior disagreed, and the
// documented one is the one that was right.
//
// Nothing about the safety doctrine changes: every entry returned here is
// still proved canonically contained (containedRoot) at plan time AND
// re-proved immediately before removal, so a loose file gets exactly the same
// treatment a directory always got.
function listEntries(rootInfo) {
  let dirents = [];
  try {
    dirents = fs.readdirSync(rootInfo.abs, { withFileTypes: true });
  } catch {
    return [];
  }
  return dirents.map((d) => ({ scratchRoot: rootInfo.rel, name: d.name, absPath: path.join(rootInfo.abs, d.name) }));
}

// Separators that end a feature prefix. A boundary character is REQUIRED, so
// `--feature rel1` never matches `rel1100`.
const FEATURE_PREFIX_SEPARATORS = new Set(['-', '.', '_']);

/** Does scratch entry `name` belong to `feature`?
 *
 * Exact name — always, unchanged: that is the one documented override that
 * sweeps even a LIVE feature's scratch.
 *
 * `<feature><sep>...` prefix — also yes, because that is bee's own cell-id
 * convention (`<feature>-<n>`) and it is how agents actually name what they
 * leave behind: `f3-1/`, `f3-3-evidence.json`, `okf6-run-verify.log`. Without
 * it, `tmp sweep --feature <f>` at close swept a single directory and left
 * every per-cell artifact of the same feature sitting at the root.
 *
 * A prefix match is an INFERENCE, never the documented override — so unlike
 * an exact match it refuses to eat a sibling that is itself a live feature or
 * lane (`--feature auth` must not delete `auth-v2/` while auth-v2 is live).
 * Returns {qualifies, reason} so that refusal is reported, never silent. */
function matchFeature(root, name, feature) {
  if (name === feature) return { qualifies: true, reason: null };
  if (!name.startsWith(feature)) return { qualifies: false, reason: null };
  if (!FEATURE_PREFIX_SEPARATORS.has(name.charAt(feature.length))) return { qualifies: false, reason: null };
  if (isLiveFeature(root, name)) return { qualifies: false, reason: 'live_sibling' };
  return { qualifies: true, reason: null };
}

/** Compute the sweep plan: which top-level scratch entries (feature-or-
 * session dirs directly under .bee/tmp/ or .bee/spikes/) qualify for removal
 * under the given flags, without deleting anything. Every entry in
 * `included` has already been proved canonically contained in one of the two
 * allowed roots (containedRoot) — the raw directory listing alone is never
 * trusted. `refusedEscapes` names every candidate whose realpath could not
 * be proved contained (symlink escapes included) — always skipped, never
 * deleted, regardless of any flag. */
export function computeSweepPlan(root, { feature, before, all } = {}) {
  const { roots, refused: refusedRoots } = inspectScratchRoots(root);
  const beforeMs = before ? Date.parse(before) : null;
  if (before && Number.isNaN(beforeMs)) {
    throw new Error(`tmp sweep: --before "${before}" is not a valid ISO date.`);
  }

  const included = [];
  const skipped = [];
  const refusedEscapes = [];

  for (const rootInfo of roots) {
    for (const entry of listEntries(rootInfo)) {
      const proof = containedRoot(entry.absPath, roots);
      if (!proof) {
        // Cannot be proved contained after realpath resolution — e.g. a
        // symlink escaping both roots. Refused: never deleted, never
        // followed, regardless of --all/--feature.
        refusedEscapes.push({ scratchRoot: rootInfo.rel, name: entry.name, path: entry.absPath });
        continue;
      }

      let qualifies;
      let skipReason = null; // only meaningful when qualifies ends up false
      if (all) {
        qualifies = true;
      } else if (feature) {
        const match = matchFeature(root, entry.name, feature);
        qualifies = match.qualifies;
        skipReason = match.reason;
      } else {
        const live = isLiveFeature(root, entry.name);
        if (live) {
          qualifies = false;
          skipReason = 'live';
        } else if (hasRecord(root, entry.name)) {
          // Closed (terminal-phase) record: sweep unconditionally — closure
          // is already the definitive signal, no age gate needed.
          qualifies = true;
        } else if (beforeMs != null) {
          // No record anywhere (absent feature/lane, or a bare session id):
          // only sweep once an explicit --before cutoff says it's old
          // enough — mirrors decisions.archive's mandatory --before, no
          // default age window.
          let mtimeMs;
          try {
            mtimeMs = fs.statSync(entry.absPath).mtimeMs;
          } catch {
            mtimeMs = Date.now();
          }
          qualifies = mtimeMs < beforeMs;
          if (!qualifies) skipReason = 'absent_not_old_enough';
        } else {
          // Absent (no record anywhere) and no --before given at all: never
          // swept by default — same "no default age window" discipline as
          // the closed/live branches above, distinct from "live" (this
          // entry may well be closeable, it's just unproven without a
          // cutoff), so it is NOT mislabeled as a protected live feature.
          qualifies = false;
          skipReason = 'absent_no_before';
        }
      }

      if (!qualifies) {
        // A prefix match refused for liveness is a SAFETY refusal and is
        // always reported, even in --feature mode where the ordinary
        // "wasn't this feature" noise stays out of the result.
        if ((!all && !feature) || skipReason === 'live_sibling') {
          skipped.push({ scratchRoot: rootInfo.rel, name: entry.name, reason: skipReason });
        }
        continue;
      }

      included.push({
        scratchRoot: rootInfo.rel,
        name: entry.name,
        path: entry.absPath,
        bytes: dirSize(entry.absPath),
        files: countFiles(entry.absPath),
      });
    }
  }

  return { included, skipped, refusedEscapes, refusedRoots };
}

/** Execute (or, with dryRun true, merely report) the sweep plan. Every
 * removal re-resolves and re-checks containment immediately before
 * fs.rmSync — a belt-and-braces re-check against a TOCTOU where the target
 * changed between planning and removal (never forced through on a failed
 * re-check; the candidate is moved into refused_escapes instead).
 *
 * dry-run/real-run parity (rework, finding 2): the re-check runs
 * UNCONDITIONALLY for every candidate, dry-run or not — only the actual
 * fs.rmSync is gated on `!dryRun`. Earlier code skipped the re-check under
 * dryRun entirely, so a dry-run could advertise a path as sweepable that a
 * real run would refuse (e.g. a root that only became a symlink after the
 * plan was computed) — the preview must never promise more than the real
 * run will do. */
export function runSweep(root, { feature, before, all, dryRun } = {}) {
  const plan = computeSweepPlan(root, { feature, before, all });
  const { roots } = inspectScratchRoots(root);
  const removed = [];
  let bytesFreed = 0;
  let filesFreed = 0;

  for (const candidate of plan.included) {
    const proof = containedRoot(candidate.path, roots);
    if (!proof) {
      plan.refusedEscapes.push({ scratchRoot: candidate.scratchRoot, name: candidate.name, path: candidate.path });
      continue;
    }
    if (!dryRun) {
      fs.rmSync(candidate.path, { recursive: true, force: true });
    }
    removed.push({ scratchRoot: candidate.scratchRoot, name: candidate.name, bytes: candidate.bytes, files: candidate.files });
    bytesFreed += candidate.bytes;
    filesFreed += candidate.files;
  }

  return {
    dry_run: !!dryRun,
    removed,
    bytes_freed: bytesFreed,
    files_freed: filesFreed,
    // Each entry names WHY it was skipped ('live' | 'absent_no_before' |
    // 'absent_not_old_enough') — a name with no record at all is never
    // mislabeled as a protected live feature.
    skipped: plan.skipped,
    refused_escapes: plan.refusedEscapes,
    // A scratch root that exists on disk but is (or sits behind) a symlink —
    // e.g. `.bee/spikes -> <repo>` — is refused wholesale and reported here,
    // never silently treated as "empty" (rework, root cause fix).
    refused_roots: plan.refusedRoots,
  };
}
