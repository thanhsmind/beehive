// herding.mjs — the human-facing "bee herding enable/disable/status" verbs
// (herding-dispatch-lock-toggle, decisions D1-D5). These perform BYTE-FOR-
// BYTE the same filesystem operation as today's manual `touch`/`rm` of the
// dispatch loop's owner enable marker: resolveHerdingMainRoot mirrors
// dispatch-interlock.mjs's resolveMainRoot EXACTLY (same git command, same
// strip-trailing-.git logic), and ENABLE_BASENAME is the identical constant
// — so this module and dispatch-interlock.mjs always agree on the same file
// (`.claude/skills/bee-herding/scripts/dispatch-interlock.mjs`, the sole
// reader; never modified or called from here, per D4).
//
// D4: these functions are a convenience for the human owner's own terminal
// action only. Never call them from dispatch-interlock.mjs, bootstrap,
// dispatch, merge, or any other bee automation/skill/agent code.

import fs from 'node:fs';
import path from 'node:path';
import { execFileSync } from 'node:child_process';

export const ENABLE_BASENAME = 'bee-herding.enable';

// TRACKING (multisession-native-17, advisor digest slice4 binding condition
// 5): state.mjs's resolveContext(cwd) is the canonical single git-common-dir
// resolver everywhere else in this codebase. resolveHerdingMainRoot below is
// deliberately left as its own independent `git rev-parse --git-common-dir`
// call rather than reconciled onto resolveContext, because its entire reason
// to exist (see the module header above) is byte-for-byte agreement with
// dispatch-interlock.mjs — a standalone, zero-bee-dependency script this
// cell does not touch and that itself shells out to git directly rather than
// importing state.mjs (state.mjs's import graph pulls in claims.mjs,
// reservations.mjs, worktree-store.mjs, lock.mjs, decisions.mjs, and
// workflow-store.mjs — real hook/process-startup cost for a script invoked
// every dispatch-loop iteration, and out of scope to change here). Making
// THIS module delegate to resolveContext while dispatch-interlock.mjs keeps
// its own inline copy would trade today's proven agreement for a NEW drift
// risk (resolveContext's walk-up-and-validate algorithm is not proven
// byte-identical to a raw `git rev-parse --git-common-dir` call in every
// edge case — e.g. the onboarding-marker-without-.git fallback resolveRoots
// already tolerates has no git-CLI equivalent at all). Reconciling this
// properly means changing dispatch-interlock.mjs too, which is out of this
// cell's scope (D4 forbids calling into it from bee automation code in
// either direction). Revisit together, in one cell, if that changes.
export function resolveHerdingMainRoot(explicit) {
  if (explicit) return explicit;
  try {
    const gitCommonDir = execFileSync(
      'git',
      ['rev-parse', '--path-format=absolute', '--git-common-dir'],
      { encoding: 'utf8' },
    ).trim();
    if (!gitCommonDir) return null;
    return path.dirname(gitCommonDir);
  } catch {
    return null;
  }
}

function markerPath(mainRoot) {
  return path.join(mainRoot, '.bee', 'tmp', ENABLE_BASENAME);
}

function requireMainRoot(explicit) {
  const mainRoot = resolveHerdingMainRoot(explicit);
  if (!mainRoot) {
    throw new Error(
      'could not resolve the MAIN checkout root (`git rev-parse --path-format=absolute --git-common-dir` failed) — run this from inside a git checkout.',
    );
  }
  return mainRoot;
}

// D3: idempotent — enabling an already-enabled marker is not an error.
export function enableHerding(explicit) {
  const mainRoot = requireMainRoot(explicit);
  const marker = markerPath(mainRoot);
  fs.mkdirSync(path.dirname(marker), { recursive: true });
  fs.writeFileSync(marker, '');
  return { enabled: true, marker, main_root: mainRoot };
}

// D3: idempotent — disabling an already-absent marker is not an error.
export function disableHerding(explicit) {
  const mainRoot = requireMainRoot(explicit);
  const marker = markerPath(mainRoot);
  if (fs.existsSync(marker)) fs.rmSync(marker);
  return { enabled: false, marker, main_root: mainRoot };
}

export function herdingStatus(explicit) {
  const mainRoot = requireMainRoot(explicit);
  const marker = markerPath(mainRoot);
  return { enabled: fs.existsSync(marker), marker, main_root: mainRoot };
}
