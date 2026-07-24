#!/usr/bin/env node
// Regression test for cell 1710-8 (feature hardening-1-7-10, decision D8).
//
// Incident: during 2026-07-20/21 release-1.7.9 close-out, a git clone of this
// repo nested inside its own tree at `.bee/spikes/<clone>` (built for a
// hermetic "clean clone" verify pass) was followed, at some later point, by
// the PARENT checkout's own `remote.origin.url` being rewritten to the
// nested clone's path (`git push` afterwards went to that phantom local
// path instead of GitHub). A leftover fixture from that exact incident is
// still sitting at `.bee/spikes/clean-clone/.git/config` in this repo, with
// `remote.origin.url` pointing at the parent checkout — proof the scenario
// is real, not hypothetical.
//
// Hunt result (see docs/history/hardening-1-7-10/reports/1710-8.md for the
// full writeup): an exhaustive grep across every .mjs file in this repo
// (scripts/, skills/, .bee/bin/, hooks/) for `remote`, `set-url`, `config
// remote`, in any case, found ZERO matches — no script anywhere issues a
// `git remote` command. A full 53-suite `node scripts/run_verify.mjs` run
// executed FROM WITHIN a fresh nested clone (built the same way as the real
// incident: `git clone` of a parent fixture into `<parent>/.bee/spikes/
// <name>`) left the parent fixture's `.git/config` byte-identical, and the
// nested clone directory survived. No culprit *script* reproduces the
// corruption against current code — the leftover fixture's timing (created
// exactly at the release-1.7.9 finalization commit, before the origin
// P1 was filed) points instead to a PROCEDURAL hazard: manual, multi-step
// git setup commands (e.g. `cd <nested>` in one shell call, `git remote add
// origin <path>` in a later, separate call) where the `cd` did not carry
// into the later command, so the remote-mutating command landed on the
// parent's cwd instead of the nested clone's.
//
// This test cannot regression-guard a procedural mistake, so instead it
// pins the actual invariant the cell's must_haves ask for: every
// git-mutating, root-resolving bee code path this repo ships (resolveRoots,
// createFeatureWorktree, mergeFeatureWorktree, and a plain `git` invocation
// with cwd pinned to the nested clone) leaves a PARENT repo's remotes
// byte-identical when run from a clone nested inside that parent's own
// tree — today, and as a tripwire if that ever regresses.

import { spawnSync } from 'node:child_process';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

import { resolveRoots } from '../.bee/bin/lib/state.mjs';
import { createFeatureWorktree, mergeFeatureWorktree } from '../.bee/bin/lib/worktree-store.mjs';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const REPO_ROOT = path.join(__dirname, '..');
const BEE_MJS = path.join(REPO_ROOT, '.bee', 'bin', 'bee.mjs');

const results = [];
function record(desc, passed, detail) {
  results.push({ desc, passed });
  console.log((passed ? 'PASS ' : 'FAIL ') + desc + (passed ? '' : ` -- ${detail}`));
}

function git(cwd, args) {
  const r = spawnSync('git', args, { cwd, encoding: 'utf8' });
  if (r.status !== 0) throw new Error(`git ${args.join(' ')} (cwd=${cwd}) failed: ${r.stderr}`);
  return r.stdout;
}

function readParentConfig(parentGitDir) {
  return fs.readFileSync(path.join(parentGitDir, 'config'), 'utf8');
}

function assertParentUntouched(label, parentDir, parentGitDir, before) {
  const after = readParentConfig(parentGitDir);
  record(`${label}: parent .git/config byte-identical`, after === before, 'parent .git/config changed');
  const originAfter = git(parentDir, ['remote', 'get-url', 'origin']).trim();
  record(`${label}: parent origin still points at the known bare fixture`, originAfter.endsWith('fake-origin.git'), `origin is now "${originAfter}"`);
}

const tmp = fs.mkdtempSync(path.join(os.tmpdir(), 'bee-nested-clone-iso-'));
try {
  // ── build the PARENT fixture: a real repo with a KNOWN origin ──────────
  const bareOrigin = path.join(tmp, 'fake-origin.git');
  git(tmp, ['init', '--bare', '-q', bareOrigin]);

  const parentDir = path.join(tmp, 'parent');
  fs.mkdirSync(parentDir);
  git(parentDir, ['init', '-q', '-b', 'main']);
  git(parentDir, ['config', 'user.email', 's@e']);
  git(parentDir, ['config', 'user.name', 's']);
  fs.writeFileSync(path.join(parentDir, 'README.md'), 'parent fixture\n');
  // Mirrors this repo's own .gitignore for bee's runtime-derived state (grants,
  // locks, etc.) so writeGrant/bootstrapWorktreeStore writes below don't trip
  // isTreeDirty in this fixture the way they wouldn't in the real repo either.
  // multisession-native-3: .bee/logs/ joined this list once lock.mjs started
  // appending contention.jsonl telemetry on every withStoreLock acquire —
  // mergeFeatureWorktree below acquires 'worktree-admin' against THIS nested
  // clone's own root, so without this line the telemetry write itself made
  // "git status --porcelain" non-empty and tripped refuseMerge's dirty check.
  fs.writeFileSync(path.join(parentDir, '.gitignore'), '.bee/runtime/\n.bee/locks/\n.bee/cache/\n.bee/spikes/\n.bee/logs/\n');
  git(parentDir, ['add', '.']);
  git(parentDir, ['commit', '-q', '-m', 'init parent fixture']);
  git(parentDir, ['remote', 'add', 'origin', bareOrigin]);

  const parentGitDir = path.join(parentDir, '.git');
  const configBaseline = readParentConfig(parentGitDir);

  // ── nest a real, independent `git clone` of the parent INSIDE the
  // parent's own tree, at the exact incident layout: <parent>/.bee/spikes/
  // <name> — mirrors the leftover .bee/spikes/clean-clone fixture in this
  // repo (its own .git/config shows remote.origin.url pointing at the
  // parent checkout, proving that layout is exactly what happened). ──────
  const spikesDir = path.join(parentDir, '.bee', 'spikes');
  fs.mkdirSync(spikesDir, { recursive: true });
  const nested = path.join(spikesDir, 'clone1');
  git(parentDir, ['clone', '-q', parentDir, nested]);
  git(nested, ['config', 'user.email', 's@e']);
  git(nested, ['config', 'user.name', 's']);

  // Minimal bee store so resolveRoots sees the nested clone as onboarded.
  // Committed (not left untracked) so the nested clone itself starts clean —
  // mergeFeatureWorktree below refuses on ANY dirty main tree, tracked or not.
  const nestedBeeDir = path.join(nested, '.bee');
  fs.mkdirSync(nestedBeeDir, { recursive: true });
  fs.writeFileSync(path.join(nestedBeeDir, 'onboarding.json'), JSON.stringify({ schema_version: '1.0', bee_version: '0.0.0' }));
  fs.writeFileSync(path.join(nestedBeeDir, 'config.json'), JSON.stringify({ commands: {} }));
  git(nested, ['add', '.']);
  git(nested, ['commit', '-q', '-m', 'bootstrap nested .bee store']);

  record('nested clone was created inside the parent tree', fs.existsSync(path.join(nested, '.git')), 'nested clone .git missing');
  assertParentUntouched('after nesting a clone', parentDir, parentGitDir, configBaseline);

  // ── resolveRoots(nested) must resolve to the nested clone ITSELF, never
  // walk up past it into the parent. ──────────────────────────────────────
  const roots = resolveRoots(nested);
  record(
    'resolveRoots(nested clone) resolves storeRoot/workRoot to the nested clone, not the parent',
    path.resolve(roots.storeRoot || '') === path.resolve(nested) && path.resolve(roots.workRoot || '') === path.resolve(nested),
    `got storeRoot=${roots.storeRoot} workRoot=${roots.workRoot} (expected ${nested})`,
  );
  assertParentUntouched('after resolveRoots(nested)', parentDir, parentGitDir, configBaseline);

  // ── createFeatureWorktree(nested, ...) must only ever create a SIBLING
  // of the nested clone (still inside .bee/spikes/), never touch the
  // parent's own .git. ─────────────────────────────────────────────────────
  const created = await createFeatureWorktree(nested, { feature: 'probe' });
  const expectedWorktreeRoot = path.resolve(nested, '..', `${path.basename(nested)}--wt--probe`);
  record(
    'createFeatureWorktree(nested) placed the worktree as a sibling of the nested clone (still under .bee/spikes/), not under the parent root',
    path.resolve(created.worktreeRoot) === expectedWorktreeRoot,
    `got worktreeRoot=${created.worktreeRoot}`,
  );
  assertParentUntouched('after createFeatureWorktree(nested)', parentDir, parentGitDir, configBaseline);

  // ── mergeFeatureWorktree(nested, ...) must only touch the nested clone's
  // own HEAD/config, never the parent's. ───────────────────────────────────
  fs.writeFileSync(path.join(created.worktreeRoot, 'probe.txt'), 'probe\n');
  git(created.worktreeRoot, ['add', '.']);
  git(created.worktreeRoot, ['commit', '-q', '-m', 'probe commit']);
  const nestedConfigBeforeMerge = readParentConfig(path.join(nested, '.git'));
  await mergeFeatureWorktree(nested, { id: created.id, cleanup: true });
  const nestedConfigAfterMerge = readParentConfig(path.join(nested, '.git'));
  record(
    'mergeFeatureWorktree(nested) left the NESTED clone\'s own remote config untouched too',
    nestedConfigAfterMerge === nestedConfigBeforeMerge,
    'nested clone .git/config changed by its own merge',
  );
  assertParentUntouched('after mergeFeatureWorktree(nested)', parentDir, parentGitDir, configBaseline);

  // ── the actual regression scenario named in the cell: run a real verify
  // pass (a representative bee CLI invocation, `bee status`) with cwd
  // EXPLICITLY pinned to the nested clone, exactly as a "verify run inside
  // a nested clone" would invoke it. ───────────────────────────────────────
  const statusResult = spawnSync('node', [BEE_MJS, 'status', '--json'], { cwd: nested, encoding: 'utf8' });
  record('bee status --json run with cwd=nested clone exits 0', statusResult.status === 0, statusResult.stderr || statusResult.stdout);
  assertParentUntouched('after `bee status` run from the nested clone', parentDir, parentGitDir, configBaseline);

  // ── the nested clone directory itself must still be standing (the
  // incident also reported spike clone dirs vanishing after runs). ───────
  record('the nested clone directory still exists after every operation above', fs.existsSync(nested), 'nested clone directory disappeared');
} finally {
  fs.rmSync(tmp, { recursive: true, force: true });
}

const failed = results.filter((r) => !r.passed).length;
console.log(`\n${results.length - failed} passed, ${failed} failed`);
process.exit(failed === 0 ? 0 : 1);
