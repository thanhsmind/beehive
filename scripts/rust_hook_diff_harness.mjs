#!/usr/bin/env node
// rust_hook_diff_harness.mjs — feeds identical stdin payloads to the Node
// hook wrapper (`node packages/bee/hooks/bee-<name>.mjs`) and the Rust
// subcommand (`bee hook <name>`), byte-comparing stdout/stderr/exit
// (contract C2/C3). Hooks write only append-only telemetry, so running both
// in-place against this checkout is safe.
//
// Usage: node scripts/rust_hook_diff_harness.mjs [--bin <bee-exe>] [--hook <name>]

import { spawnSync } from 'node:child_process';
import { existsSync } from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');

const argv = process.argv.slice(2);
let bin = null;
let onlyHook = null;
for (let i = 0; i < argv.length; i++) {
  if (argv[i] === '--bin') bin = path.resolve(argv[++i]);
  if (argv[i] === '--hook') onlyHook = argv[++i];
}
if (!bin) {
  const exe = process.platform === 'win32' ? 'bee.exe' : 'bee';
  for (const profile of ['release', 'debug']) {
    const candidate = path.join(repoRoot, 'packages', 'bee-rs', 'target', profile, exe);
    if (existsSync(candidate)) { bin = candidate; break; }
  }
}
if (!bin || !existsSync(bin)) {
  console.error('hook-diff: bee binary not found — build packages/bee-rs first or pass --bin');
  process.exit(2);
}

// Shared adversarial payload set every hook must normalize identically.
const COMMON_PAYLOADS = [
  { label: 'empty-stdin', stdin: '' },
  { label: 'junk-stdin', stdin: 'not json at all {{{' },
  { label: 'top-level-null', stdin: 'null' },
  { label: 'top-level-array', stdin: '[1,2]' },
  { label: 'invalid-cwd', stdin: JSON.stringify({ cwd: 42, hook_event_name: 'PostToolUse' }) },
];

// Per-hook realistic payloads.
const HOOK_CASES = {
  'tools-logger': [
    { label: 'plain-tool', stdin: JSON.stringify({ hook_event_name: 'PostToolUse', tool_name: 'Edit', cwd: repoRoot }) },
    { label: 'subagent-tool', stdin: JSON.stringify({ hook_event_name: 'PostToolUse', tool_name: 'Bash', agent_id: 'a1', agent_type: 'worker', duration_ms: 12.5, tool_status: 'ok', cwd: repoRoot }) },
  ],
  'codex-subagent-audit': [
    { label: 'start', stdin: JSON.stringify({ hook_event_name: 'SubagentStart', session_id: 's1', agent_id: 'a1', agent_name: 'n', agent_type: 't', cwd: repoRoot }) },
    { label: 'stop', stdin: JSON.stringify({ hook_event_name: 'SubagentStop', session_id: 's1', cwd: repoRoot }) },
    { label: 'wrong-event', stdin: JSON.stringify({ hook_event_name: 'PostToolUse', cwd: repoRoot }) },
    { label: 'oversize-fields', stdin: JSON.stringify({ hook_event_name: 'SubagentStart', session_id: 'x'.repeat(500), agent_type: 'y'.repeat(200), cwd: repoRoot }) },
  ],
  'prompt-context': [
    { label: 'prompt', stdin: JSON.stringify({ hook_event_name: 'UserPromptSubmit', prompt: 'hello', cwd: repoRoot }) },
  ],
  'state-sync': [
    { label: 'posttool', stdin: JSON.stringify({ hook_event_name: 'PostToolUse', tool_name: 'Edit', cwd: repoRoot }) },
  ],
  'chain-nudge': [
    { label: 'stop', stdin: JSON.stringify({ hook_event_name: 'Stop', cwd: repoRoot }) },
  ],
  'session-init': [
    { label: 'startup', stdin: JSON.stringify({ hook_event_name: 'SessionStart', source: 'startup', session_id: 'diff-harness-session', cwd: repoRoot }) },
  ],
  'session-close': [
    { label: 'stop', stdin: JSON.stringify({ hook_event_name: 'Stop', session_id: 'diff-harness-session', cwd: repoRoot }) },
  ],
  'model-guard': [
    { label: 'task-ok', stdin: JSON.stringify({ hook_event_name: 'PreToolUse', tool_name: 'Task', tool_input: { prompt: '[bee-tier: extraction] read files', subagent_type: 'general-purpose' }, cwd: repoRoot }) },
  ],
  'write-guard': [
    { label: 'read-tool', stdin: JSON.stringify({ hook_event_name: 'PreToolUse', tool_name: 'Read', tool_input: { file_path: path.join(repoRoot, 'README.md') }, cwd: repoRoot }) },
    { label: 'edit-docs', stdin: JSON.stringify({ hook_event_name: 'PreToolUse', tool_name: 'Edit', tool_input: { file_path: path.join(repoRoot, 'docs', 'x.md') }, cwd: repoRoot }) },
    { label: 'bash-echo', stdin: JSON.stringify({ hook_event_name: 'PreToolUse', tool_name: 'Bash', tool_input: { command: 'echo hi' }, cwd: repoRoot }) },
  ],
};

function run(cmd, args, stdin) {
  const r = spawnSync(cmd, args, {
    cwd: repoRoot,
    input: Buffer.from(stdin, 'utf8'),
    encoding: 'buffer',
    windowsHide: true,
    env: { ...process.env, BEE_SESSION_ID: 'diff-harness-session' },
  });
  return { status: r.status, stdout: r.stdout ?? Buffer.alloc(0), stderr: r.stderr ?? Buffer.alloc(0) };
}

let failures = 0;
let ran = 0;
const hookNames = onlyHook ? [onlyHook] : Object.keys(HOOK_CASES);
for (const hook of hookNames) {
  const mjs = path.join(repoRoot, 'packages', 'bee', 'hooks', `bee-${hook}.mjs`);
  if (!existsSync(mjs)) {
    console.error(`skip  ${hook}: no ${mjs}`);
    continue;
  }
  // Only diff hooks the Rust side claims to have ported.
  const info = JSON.parse(run(bin, ['rs-info'], '').stdout.toString());
  if (!info.ported.includes(`hook ${hook}`)) {
    console.log(`todo  ${hook}: not ported yet`);
    continue;
  }
  for (const { label, stdin } of [...COMMON_PAYLOADS, ...(HOOK_CASES[hook] ?? [])]) {
    ran++;
    const js = run(process.execPath, [mjs], stdin);
    const rs = run(bin, ['hook', hook], stdin);
    const diffs = [];
    if (js.status !== rs.status) diffs.push(`exit ${js.status} != ${rs.status}`);
    if (!js.stdout.equals(rs.stdout)) diffs.push(`stdout ${JSON.stringify(js.stdout.toString().slice(0, 120))} != ${JSON.stringify(rs.stdout.toString().slice(0, 120))}`);
    if (!js.stderr.equals(rs.stderr)) diffs.push(`stderr ${JSON.stringify(js.stderr.toString().slice(0, 120))} != ${JSON.stringify(rs.stderr.toString().slice(0, 120))}`);
    if (diffs.length) {
      failures++;
      console.error(`FAIL  ${hook}/${label}: ${diffs.join('; ')}`);
    } else {
      console.log(`ok    ${hook}/${label}`);
    }
  }
}
console.log(failures === 0 ? `hook-diff: ${ran}/${ran} byte-identical` : `hook-diff: ${failures}/${ran} FAILED`);
process.exit(failures === 0 ? 0 : 1);
