#!/usr/bin/env node
// Guard: every `node <path>` invocation inside a `.github/workflows/*.yml`
// run step must resolve to a file that actually exists. windows.yml's
// "portable paths guard" and "config validate" steps invoked
// scripts/test_portable_paths.mjs and scripts/test_config_validate.mjs — the
// files live under scripts/tests/ — so neither step could ever succeed. Both
// were masked because the job's first step ("bee-hive template suites") was
// already failing on other, unrelated Windows causes, so a broken later step
// never surfaced on its own. Nothing in the repo checked workflow step paths
// before this suite (windows-path-identity wpi-3) — a validator that cannot
// fail is the thing this repeatedly slips past, so a negative-control
// fixture below proves this one actually discriminates.
//
// Not a real YAML parser (this repo carries no YAML-parsing dependency —
// plain node scripts throughout, see scripts/tests/test_config_validate.mjs
// for the same style). Narrow by design: it only recognizes a `run:` step
// key, inline or block-scalar, and only extracts `node <path>` tokens from
// its body — exactly the shape every workflow file here uses.

import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const REPO_ROOT = path.join(__dirname, '..', '..');
const WORKFLOWS_DIR = path.join(REPO_ROOT, '.github', 'workflows');

const results = [];
function record(desc, passed, detail) {
  results.push({ desc, passed });
  console.log((passed ? 'PASS ' : 'FAIL ') + desc + (passed ? '' : ` -- ${detail}`));
}

// A `run:` value is either inline (`run: <cmd>`) or a block scalar
// (`run: |` / `run: >`, optionally with chomping/comment) whose body is the
// following more-indented lines, up to (not including) a line back at or
// below the `run:` key's own indentation.
function collectNodePaths(text, lineNumber, out) {
  const nodeCallRe = /\bnode\s+([^\s"'|&;<>]+)/g;
  let match;
  while ((match = nodeCallRe.exec(text))) {
    out.push({ path: match[1], lineNumber });
  }
}

export function extractNodeInvocations(yamlText) {
  const lines = yamlText.split(/\r?\n/);
  const invocations = [];
  const inlineRunRe = /^(\s*)run:\s*(.*)$/;
  const blockScalarRe = /^[|>][+-]?\s*(#.*)?$/;

  for (let i = 0; i < lines.length; i++) {
    const m = lines[i].match(inlineRunRe);
    if (!m) continue;
    const indent = m[1].length;
    const rest = m[2].trim();
    if (rest === '' || blockScalarRe.test(rest)) {
      let j = i + 1;
      while (j < lines.length) {
        const bodyLine = lines[j];
        if (bodyLine.trim() === '') {
          j++;
          continue;
        }
        const bodyIndent = bodyLine.match(/^(\s*)/)[1].length;
        if (bodyIndent <= indent) break;
        collectNodePaths(bodyLine, j + 1, invocations);
        j++;
      }
      i = j - 1;
    } else {
      collectNodePaths(rest, i + 1, invocations);
    }
  }
  return invocations;
}

// Returns one entry per `node <path>` invocation across every workflow file
// under `<root>/.github/workflows` whose path does not resolve to a real
// file — the check every step runs relative to `root` (GitHub Actions runs
// every step from the repo root after actions/checkout).
export function findMissingWorkflowScripts(root) {
  const missing = [];
  let workflowFiles;
  try {
    workflowFiles = fs
      .readdirSync(path.join(root, '.github', 'workflows'))
      .filter((f) => f.endsWith('.yml') || f.endsWith('.yaml'));
  } catch {
    return missing;
  }
  for (const file of workflowFiles) {
    const text = fs.readFileSync(path.join(root, '.github', 'workflows', file), 'utf8');
    for (const { path: scriptPath, lineNumber } of extractNodeInvocations(text)) {
      if (!fs.existsSync(path.join(root, scriptPath))) {
        missing.push({ file, path: scriptPath, lineNumber });
      }
    }
  }
  return missing;
}

// ── real repo: every node <path> invocation across every workflow file resolves ──

{
  const missing = findMissingWorkflowScripts(REPO_ROOT);
  record(
    'every `node <path>` invocation in a .github/workflows/*.yml run step resolves to a file that exists',
    missing.length === 0,
    JSON.stringify(missing),
  );
}

// ── extractor sanity: the corrected scripts/tests/ paths are actually found ──

{
  const text = fs.readFileSync(path.join(WORKFLOWS_DIR, 'windows.yml'), 'utf8');
  const found = extractNodeInvocations(text).map((i) => i.path);
  const expected = [
    'scripts/run_verify.mjs',
    'scripts/tests/test_portable_paths.mjs',
    'scripts/tests/test_config_validate.mjs',
  ];
  record(
    'extractor finds the corrected scripts/tests/ paths in windows.yml (not the old broken scripts/*.mjs paths)',
    expected.every((p) => found.includes(p)),
    JSON.stringify(found),
  );
}

// ── negative control: a fixture step pointing at a nonexistent file is caught ──
// This is the discrimination proof: a validator that cannot fail is exactly
// the defect that let the two real broken paths sit unnoticed.

{
  const tmpRoot = fs.mkdtempSync(path.join(os.tmpdir(), 'workflow-step-paths-fixture-broken-'));
  fs.mkdirSync(path.join(tmpRoot, '.github', 'workflows'), { recursive: true });
  fs.writeFileSync(
    path.join(tmpRoot, '.github', 'workflows', 'broken.yml'),
    [
      'name: Fixture',
      'jobs:',
      '  test:',
      '    runs-on: ubuntu-latest',
      '    steps:',
      '      - name: broken step',
      '        run: node scripts/this-file-does-not-exist-xyz-fixture.mjs',
      '',
    ].join('\n'),
  );
  const missing = findMissingWorkflowScripts(tmpRoot);
  record(
    'negative control: a fixture step pointing at a nonexistent path is reported missing',
    missing.length === 1 && missing[0].path === 'scripts/this-file-does-not-exist-xyz-fixture.mjs',
    JSON.stringify(missing),
  );
  fs.rmSync(tmpRoot, { recursive: true, force: true });
}

// ── positive control: a fixture step whose path DOES exist is not reported ──

{
  const tmpRoot = fs.mkdtempSync(path.join(os.tmpdir(), 'workflow-step-paths-fixture-ok-'));
  fs.mkdirSync(path.join(tmpRoot, '.github', 'workflows'), { recursive: true });
  fs.mkdirSync(path.join(tmpRoot, 'scripts'), { recursive: true });
  fs.writeFileSync(path.join(tmpRoot, 'scripts', 'real.mjs'), '// exists\n');
  fs.writeFileSync(
    path.join(tmpRoot, '.github', 'workflows', 'ok.yml'),
    [
      'name: Fixture',
      'jobs:',
      '  test:',
      '    runs-on: ubuntu-latest',
      '    steps:',
      '      - name: ok step',
      '        run: node scripts/real.mjs',
      '',
    ].join('\n'),
  );
  const missing = findMissingWorkflowScripts(tmpRoot);
  record('a fixture step whose path exists is not reported missing', missing.length === 0, JSON.stringify(missing));
  fs.rmSync(tmpRoot, { recursive: true, force: true });
}

const failed = results.filter((r) => !r.passed);
console.log(`SUMMARY: ${results.length - failed.length}/${results.length} passed`);
process.exit(failed.length ? 7 : 0);
