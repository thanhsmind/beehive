#!/usr/bin/env node
// Proves the worktree-feature-parallelism wire-in end-to-end against BOTH real
// resolveRoots implementations: a worktree whose git-verified id is registered
// in the MAIN store's grant registry resolves to its OWN local store; an
// unregistered (or revoked) worktree resolves to the main store (P40 default).
// Uses a real temp git repo + real `git worktree add`.

import { resolveRoots as resolveState } from '../../.bee/bin/lib/state.mjs';
import { resolveRoots as resolveAdapter } from '../../.bee/bin/hooks/adapter.mjs';
import { spawnSync } from 'node:child_process';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';

const results = [];
function record(desc, passed, detail) {
  results.push({ desc, passed });
  console.log((passed ? 'PASS ' : 'FAIL ') + desc + (passed ? '' : ` -- ${detail}`));
}
function git(cwd, args) {
  const r = spawnSync('git', args, { cwd, encoding: 'utf8' });
  if (r.status !== 0) throw new Error(`git ${args.join(' ')} (cwd=${cwd}): ${r.stderr}`);
  return r.stdout;
}
// Derive the git-verified worktree id exactly as the resolvers do.
function verifiedId(wtPath) {
  const gitFile = fs.readFileSync(path.join(wtPath, '.git'), 'utf8').trim();
  const m = gitFile.match(/^gitdir:\s*(.+)$/);
  const gitdir = path.resolve(wtPath, m[1].trim());
  return path.basename(gitdir);
}
const both = (fn) => [['state', resolveState], ['adapter', resolveAdapter]].map(([name, r]) => [name, fn(r)]);

const tmp = fs.mkdtempSync(path.join(os.tmpdir(), 'bee-grant-'));
try {
  const main = path.join(tmp, 'main');
  fs.mkdirSync(main);
  git(main, ['init', '-q', '-b', 'main']);
  git(main, ['config', 'user.email', 's@e']);
  git(main, ['config', 'user.name', 's']);
  fs.writeFileSync(path.join(main, 'f'), 'x');
  git(main, ['add', '.']);
  git(main, ['commit', '-q', '-m', 'init']);

  const wtA = path.join(tmp, 'wtA');
  const wtB = path.join(tmp, 'wtB');
  git(main, ['worktree', 'add', '-q', '-b', 'fa', wtA]);
  git(main, ['worktree', 'add', '-q', '-b', 'fb', wtB]);

  const idA = verifiedId(wtA);
  const mainReal = fs.realpathSync(main);
  const wtAReal = fs.realpathSync(wtA);
  const fileInA = path.join(wtA, 'src.txt');
  const fileInB = path.join(wtB, 'src.txt');

  const grantsFile = path.join(main, '.bee', 'runtime', 'worktree-grants.json');
  fs.mkdirSync(path.dirname(grantsFile), { recursive: true });

  // Register A only.
  fs.writeFileSync(grantsFile, JSON.stringify({ [idA]: true }));

  // Case 1: granted A -> its own local store (both resolvers).
  for (const [name, r] of both((fn) => fn(fileInA))) {
    const ok =
      r.worktreeResolution === 'linked-valid' &&
      r.storeRoot &&
      fs.realpathSync(r.storeRoot) === wtAReal &&
      fs.realpathSync(r.storeRoot) !== mainReal &&
      r.id === idA;
    record(`${name}: granted worktree A resolves to its OWN local store (not main)`, ok, JSON.stringify(r));
  }

  // Case 2: unregistered B -> main (P40 default), both resolvers.
  for (const [name, r] of both((fn) => fn(fileInB))) {
    const ok = r.worktreeResolution === 'linked-valid' && r.storeRoot && fs.realpathSync(r.storeRoot) === mainReal;
    record(`${name}: unregistered worktree B resolves to main (P40 default preserved)`, ok, JSON.stringify(r));
  }

  // Case 3: revoke A's grant -> A falls back to main (both).
  fs.writeFileSync(grantsFile, JSON.stringify({}));
  for (const [name, r] of both((fn) => fn(fileInA))) {
    const ok = r.storeRoot && fs.realpathSync(r.storeRoot) === mainReal;
    record(`${name}: after revoking the grant, A falls back to main`, ok, JSON.stringify(r));
  }

  // Case 4: a self-written grant INSIDE the worktree grants nothing (the
  // security property): B writes its own .bee/runtime/worktree-grants.json
  // claiming itself, main registry stays empty -> B still resolves to main.
  const bSelf = path.join(wtB, '.bee', 'runtime', 'worktree-grants.json');
  fs.mkdirSync(path.dirname(bSelf), { recursive: true });
  fs.writeFileSync(bSelf, JSON.stringify({ [verifiedId(wtB)]: true }));
  for (const [name, r] of both((fn) => fn(fileInB))) {
    const ok = r.storeRoot && fs.realpathSync(r.storeRoot) === mainReal;
    record(`${name}: worktree B's self-written grant marker is ignored (resolves to main)`, ok, JSON.stringify(r));
  }
} finally {
  fs.rmSync(tmp, { recursive: true, force: true });
}

const failed = results.filter((r) => !r.passed);
console.log(`SUMMARY: ${results.length - failed.length}/${results.length} passed`);
process.exit(failed.length ? 7 : 0);
