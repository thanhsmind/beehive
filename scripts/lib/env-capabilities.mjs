#!/usr/bin/env node
// env-capabilities.mjs — shared, probe-once environment-capability checks for
// test suites, so a check whose FIXTURE cannot exist on this machine skips
// LOUDLY (naming the missing capability) instead of failing or crashing the
// rest of its suite.
//
// Doctrine (test-simple): `commands.test` is the cap door — a red must mean a
// real defect. A machine limit (e.g. Windows without Developer Mode cannot
// create symlinks) is not a defect, but it must never be a SILENT pass
// either: every gated check prints one `SKIP (env: <capability>) — <check>`
// line so a reader can count exactly what did not run and why.
//
// Rules for callers:
//   - Gate ONLY the checks (or sub-sections) whose fixture genuinely needs
//     the capability — the rest of the suite must still run.
//   - Never gate a check that CAN run here; never weaken the assertion
//     inside a gated check.
//   - Probes are real (they try the operation), cached per process, and an
//     UNEXPECTED probe failure still throws — only the known
//     capability-denied error codes turn into `false`.
//
// Location note: this sits beside scripts/lib/test-fixture.mjs and
// scripts/lib/run-module-worker.mjs, both already imported relatively from
// scripts/tests/*, packages/bee/tests/*, packages/bee/hooks/* and
// packages/bee/scripts/* — the established shared-helper spot for suites.

import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';

/** One consistent, greppable skip line: `SKIP (env: <reason>) — <check>`. */
export function envSkipLine(reason, checkName) {
  return `SKIP (env: ${reason}) — ${checkName}`;
}

// ─── symlink creation ───────────────────────────────────────────────────────
// Windows requires SeCreateSymbolicLinkPrivilege (admin or Developer Mode)
// for BOTH file and directory symlinks; without it fs.symlinkSync throws
// EPERM. One dir+file probe covers both shapes (same privilege).

export const SYMLINK_SKIP_REASON = 'symlink requires elevation on win32';

let symlinkProbe; // undefined = not yet probed
export function canSymlink() {
  if (symlinkProbe !== undefined) return symlinkProbe;
  const dir = fs.mkdtempSync(path.join(os.tmpdir(), 'bee-env-cap-symlink-'));
  try {
    const dirTarget = path.join(dir, 'dir-target');
    fs.mkdirSync(dirTarget);
    const fileTarget = path.join(dir, 'file-target');
    fs.writeFileSync(fileTarget, '');
    fs.symlinkSync(dirTarget, path.join(dir, 'dir-link'), 'dir');
    fs.symlinkSync(fileTarget, path.join(dir, 'file-link'));
    symlinkProbe = true;
  } catch (error) {
    const code = error && error.code;
    if (code === 'EPERM' || code === 'EACCES' || code === 'ENOSYS') {
      symlinkProbe = false; // the capability is genuinely denied here
    } else {
      throw error; // anything else is a real defect, never a skip
    }
  } finally {
    fs.rmSync(dir, { recursive: true, force: true });
  }
  return symlinkProbe;
}

// ─── filesystem case sensitivity (of the tmpdir volume) ─────────────────────
// Checks written on a case-sensitive box ("two dirs differing only by case
// are distinct") cannot fire on a case-insensitive volume (default NTFS):
// the premise itself is false there, not the code under test.

export const CASE_SENSITIVE_FS_SKIP_REASON =
  'case-insensitive filesystem (default NTFS) — a case-sensitivity-dependent fixture cannot exist here';

let caseProbe; // undefined = not yet probed; true = tmpdir volume IS case-sensitive
export function tmpdirIsCaseSensitive() {
  if (caseProbe !== undefined) return caseProbe;
  const dir = fs.mkdtempSync(path.join(os.tmpdir(), 'bee-env-cap-case-'));
  try {
    fs.mkdirSync(path.join(dir, 'CaseProbe'));
    caseProbe = !fs.existsSync(path.join(dir, 'caseprobe'));
  } finally {
    fs.rmSync(dir, { recursive: true, force: true });
  }
  return caseProbe;
}
