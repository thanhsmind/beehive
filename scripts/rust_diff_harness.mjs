#!/usr/bin/env node
// rust_diff_harness.mjs — byte-compares `node bee.mjs <args>` against the Rust
// front door `bee <args>` (contract C2, plans/rust-port.md).
//
// R0 scope: read-only verbs, run in-place against this checkout. Mutating
// verbs get fixture-copy semantics when R1 lands serde state types.
//
// Usage:
//   node scripts/rust_diff_harness.mjs [--bin <path-to-bee-exe>]
//
// Exit 0 = every case byte-identical (stdout, stderr, exit code); 1 otherwise.

import { spawnSync } from 'node:child_process';
import { existsSync } from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const jsEntry = path.join(repoRoot, 'packages', 'bee', 'bee.mjs');

const argv = process.argv.slice(2);
let bin = null;
for (let i = 0; i < argv.length; i++) {
  if (argv[i] === '--bin') bin = path.resolve(argv[++i]);
}
if (!bin) {
  const exe = process.platform === 'win32' ? 'bee.exe' : 'bee';
  for (const profile of ['release', 'debug']) {
    const candidate = path.join(repoRoot, 'packages', 'bee-rs', 'target', profile, exe);
    if (existsSync(candidate)) { bin = candidate; break; }
  }
}
if (!bin || !existsSync(bin)) {
  console.error('diff-harness: bee binary not found — build packages/bee-rs first or pass --bin');
  process.exit(2);
}

// Read-only porcelain cases. Every entry must be safe to run twice in-place.
const CASES = [
  ['--help'],
  ['--help', '--all'],
  ['--help', '--json'],
  ['status'],
  ['status', '--json'],
  ['status', '--brief', '--json'],
  ['status', '--lanes-full', '--json'],
  ['orient'],
  ['orient', '--json'],
  ['cells', 'list', '--json'],
  ['cells', 'ready', '--json'],
  ['reservations', 'list', '--json'],
  ['decisions', 'active', '--json'],
  ['backlog', 'counts', '--json'],
  ['capture', 'count', '--json'],
  ['reviews', 'list', '--json'],
  ['feedback', 'count', '--json'],
  ['knowledge', 'list', '--json'],
  ['nonexistent-verb'], // error paths must match too
];

function run(cmd, args) {
  const r = spawnSync(cmd, args, {
    cwd: repoRoot,
    encoding: 'buffer',
    env: { ...process.env, BEE_JS_ENTRY: jsEntry },
    windowsHide: true,
  });
  return { status: r.status, stdout: r.stdout ?? Buffer.alloc(0), stderr: r.stderr ?? Buffer.alloc(0) };
}

let failures = 0;
for (const args of CASES) {
  const label = `bee ${args.join(' ')}`;
  const js = run(process.execPath, [jsEntry, ...args]);
  const rs = run(bin, args);
  const diffs = [];
  // Perf lines like "[bee] status 983ms" are wall-clock and differ run to
  // run even Node-vs-Node — normalize the duration token before comparing.
  const normalizeMs = (buf) => buf.toString('binary').replace(/\b\d+ms\b/g, 'Nms');
  if (js.status !== rs.status) diffs.push(`exit ${js.status} != ${rs.status}`);
  if (!js.stdout.equals(rs.stdout)) diffs.push(`stdout differs (${js.stdout.length}B vs ${rs.stdout.length}B)`);
  if (normalizeMs(js.stderr) !== normalizeMs(rs.stderr)) diffs.push(`stderr differs (${js.stderr.length}B vs ${rs.stderr.length}B)`);
  if (diffs.length) {
    failures++;
    console.error(`FAIL  ${label}: ${diffs.join('; ')}`);
    const firstDiff = (a, b) => {
      const n = Math.min(a.length, b.length);
      for (let i = 0; i < n; i++) if (a[i] !== b[i]) return i;
      return n;
    };
    if (!js.stdout.equals(rs.stdout)) {
      const i = firstDiff(js.stdout, rs.stdout);
      console.error(`      stdout@${i}: js=${JSON.stringify(js.stdout.subarray(i, i + 40).toString())} rs=${JSON.stringify(rs.stdout.subarray(i, i + 40).toString())}`);
    }
  } else {
    console.log(`ok    ${label}`);
  }
}

console.log(failures === 0
  ? `diff-harness: ${CASES.length}/${CASES.length} byte-identical`
  : `diff-harness: ${failures}/${CASES.length} FAILED`);
process.exit(failures === 0 ? 0 : 1);
