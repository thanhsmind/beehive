// test-runner.mjs — the ONE deterministic test path (docs/specs/test-simple.md).
//
// Three pieces, like fluent's tester:
//   1. Declaration: `.bee/config.json` `commands.test` (string or array) is
//      the single place a project declares how it is tested. Nothing else
//      declares test obligations.
//   2. Deterministic runner: runDeclaredTests runs the declared commands in
//      order, captures per-command output, and writes ONE normalized record —
//      .bee/logs/test-results.json:
//        {ran_at, green, commands: [{command, exit, duration_ms, failure_excerpt}]}
//      failure_excerpt is the last ≤500 chars of a failing command's combined
//      output (null on pass). The runner is a program; an agent's word is
//      never the record.
//   3. The record is the evidence: `bee test` surfaces it, `bee cells finish`
//      caps on it, `bee close` re-runs it fresh, and Prior-rounds re-dispatch
//      prompts cite the failure_excerpt directly.
//
// A red run is a NORMAL RESULT, never a throw — the record is still written
// and the caller decides what a red means (bee test exits 1; finish refuses
// the cap; close reports the door red).

import path from 'node:path';
import { spawnSync } from 'node:child_process';
import { readConfig, isNoTestCommand } from './state.mjs';
import { writeJsonAtomic } from './fsutil.mjs';

// The pointer recorded on cell traces and printed in text output — always the
// repo-relative form, so a trace read from another checkout still names the
// right file.
export const TEST_RESULTS_RELATIVE = '.bee/logs/test-results.json';

// Last ≤500 chars of a failing command's combined stdout+stderr — enough to
// carry the actual failure line(s) into refusals and re-dispatch prompts
// without ever inlining a whole log.
const FAILURE_EXCERPT_MAX_CHARS = 500;

export function testResultsPath(root) {
  return path.join(root, '.bee', 'logs', 'test-results.json');
}

// ─── cross-platform command spawning ───────────────────────────────────────
// Declared commands are commonly written in POSIX shell syntax (env prefixes
// like `FOO=1 node ...`). On Windows, node's `shell: true` is cmd.exe, which
// cannot parse that shape ("'FOO' is not recognized as an internal or
// external command"), so the runner prefers a POSIX-capable shell when one
// is on PATH (Git Bash ships one on most Windows dev boxes) and falls back
// to the platform shell otherwise — a declared command behaves the same
// cross-platform. On POSIX, `shell: true` already IS a POSIX sh, so no
// probe is spent there. The probe result is cached per process.
let cachedPosixShell; // undefined = unprobed; null = none available
function posixShell() {
  if (cachedPosixShell !== undefined) return cachedPosixShell;
  if (process.platform !== 'win32') {
    cachedPosixShell = null; // shell: true is already /bin/sh
    return cachedPosixShell;
  }
  try {
    const probe = spawnSync('bash', ['-c', 'exit 0'], { encoding: 'utf8', timeout: 10_000 });
    cachedPosixShell = !probe.error && probe.status === 0 ? 'bash' : null;
  } catch {
    cachedPosixShell = null;
  }
  return cachedPosixShell;
}

/**
 * spawnDeclaredCommand(command, {cwd}) — one declared command, one spawn,
 * POSIX semantics wherever possible. Exported so tests can exercise the
 * shell-selection seam directly.
 */
export function spawnDeclaredCommand(command, { cwd }) {
  const options = { cwd, encoding: 'utf8', maxBuffer: 64 * 1024 * 1024 };
  const shell = posixShell();
  if (shell) return spawnSync(shell, ['-c', command], options);
  return spawnSync(command, { ...options, shell: true });
}

/**
 * declaredTestCommands(root) — the declaration, normalized. Returns the
 * ordered command list, or null when the repo declares no test path: absent
 * `commands.test`, an empty string/array, or the no-test sentinel ("none",
 * decision 55b951e1 — a deliberate declaration that there is nothing to run).
 */
export function declaredTestCommands(root) {
  const commands = readConfig(root).commands || {};
  const raw = commands.test;
  if (raw == null) return null;
  const list = Array.isArray(raw) ? raw : [raw];
  const cleaned = list
    .filter((c) => typeof c === 'string' && c.trim() && !isNoTestCommand(c))
    .map((c) => c.trim());
  return cleaned.length > 0 ? cleaned : null;
}

/**
 * runDeclaredTests(root) — the deterministic runner.
 *
 * Undeclared: returns {undeclared: true, green: null, commands: []} and
 * writes nothing — there is no run to record.
 *
 * Declared: runs every command sequentially (shell spawn, cwd = root), never
 * short-circuiting on a red — the record always covers the whole declared
 * path. Writes the normalized record to .bee/logs/test-results.json and
 * returns it (plus undeclared:false and results_path). Never throws on a red
 * command; a spawn failure (command not runnable at all) records exit null
 * with the spawn error in the excerpt — still a red result, not an exception.
 */
export function runDeclaredTests(root) {
  const commands = declaredTestCommands(root);
  if (!commands) return { undeclared: true, green: null, commands: [] };
  const ran_at = new Date().toISOString();
  const results = [];
  let green = true;
  for (const command of commands) {
    const started = Date.now();
    const run = spawnDeclaredCommand(command, { cwd: root });
    const duration_ms = Date.now() - started;
    let output = `${run.stdout || ''}${run.stderr || ''}`;
    if (run.error) output += `\n[bee test] spawn error: ${run.error.message}`;
    const exit = run.error ? null : (run.status ?? null);
    const passed = !run.error && run.status === 0;
    if (!passed) green = false;
    results.push({
      command,
      exit,
      duration_ms,
      failure_excerpt: passed
        ? null
        : output.trim().slice(-FAILURE_EXCERPT_MAX_CHARS) || `(no output; exit ${exit})`,
    });
  }
  const record = { ran_at, green, commands: results };
  writeJsonAtomic(testResultsPath(root), record);
  return { undeclared: false, results_path: TEST_RESULTS_RELATIVE, ...record };
}

/**
 * firstFailureLine(run) — the first line of the first failing command's
 * failure_excerpt, for refusal heads and Prior-rounds one-liners. Null when
 * nothing failed (or nothing ran).
 */
export function firstFailureLine(run) {
  const failing = Array.isArray(run && run.commands)
    ? run.commands.find((c) => c && typeof c.failure_excerpt === 'string' && c.failure_excerpt)
    : null;
  if (!failing) return null;
  const line = failing.failure_excerpt
    .split('\n')
    .map((l) => l.trim())
    .find((l) => l);
  return line || null;
}
