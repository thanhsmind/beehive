#!/usr/bin/env node
// Proves worktree-companion-hook end-to-end against a REAL temp git repo +
// real `git worktree add`/`git merge`, mirroring test_worktree_cli.mjs's
// fixture pattern: spawns the real `bee.mjs` dispatcher via spawnSync, no
// mocking of worktree-store.mjs itself. The "companion tool" under test is a
// small fixture script standing in for fgos (or anything else) — bee's own
// code never names it; only the fixture's start/end scripts and the
// `.bee/config.json` values wiring them in know what they are.

import { spawnSync } from 'node:child_process';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
// worktree-concurrency-guard-controlroot-port (Port-D4): hasAnySharedNestedCheckout
// imported directly (in-process), from the SAME vendored location every other
// fixture in this file exercises the real bee CLI against, so the
// controlRoot-vs-root differential test below (Cases 12/13) can pass an
// opts.controlRoot that is a genuinely different directory than root — a
// scenario handleWorktreeNew's own controlRootFor(mainRoot) can never
// construct through the CLI itself (it only ever runs from an "ordinary"
// checkout, where controlRootFor(mainRoot) === mainRoot by construction), so
// this is the only way to prove the primitive's own controlRoot-vs-root
// scoping end to end rather than asserting it only indirectly through wiring
// that happens to never exercise the differing-root case.
import { hasAnySharedNestedCheckout } from '../../.bee/bin/lib/guards.mjs';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const REPO_ROOT = path.join(__dirname, '..', '..');
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

function bee(cwd, args, { env } = {}) {
  return spawnSync('node', [BEE_MJS, ...args], {
    cwd,
    encoding: 'utf8',
    env: env ? { ...process.env, ...env } : process.env,
  });
}

// Plants a foreign session record with a fresh heartbeat so isConcurrentMode()
// reads TRUE for `main` — the concurrency half of the worktree-new refusal
// (wcg-3). The bee CLI never registers a session itself (only the session-init
// hook does), so a fixture fully controls concurrency by planting/omitting this.
function plantLiveSession(main, id = 'other-live-session') {
  const dir = path.join(main, '.bee', 'sessions');
  fs.mkdirSync(dir, { recursive: true });
  const now = new Date().toISOString();
  fs.writeFileSync(path.join(dir, `${id}.json`), `${JSON.stringify({ id, started_at: now, last_heartbeat: now }, null, 2)}\n`);
}

// Plants a PLAIN nested git repo physically inside `main` (its own `.git` dir,
// not a symlink, not a .gitmodules-registered submodule) — STR65's exact
// unguarded incident shape (D2 shape (b)), the structural half of the refusal.
function plantNestedRepo(main, name = 'repo') {
  const nested = path.join(main, name);
  fs.mkdirSync(nested, { recursive: true });
  git(nested, ['init', '-q', '-b', 'main']);
  fs.writeFileSync(path.join(nested, 'nested-file'), 'nested');
  return nested;
}

// A fixture "companion tool": `start` creates its own throwaway directory
// (standing in for a real nested-repo session worktree) and prints
// {worktreePath, sessionId} JSON to stdout, exactly the contract
// runCompanionStart requires. `end` just records that it ran (writes a
// marker file under companionHome, keyed by the id it was given) so tests
// can assert the real command ran with the real substituted session id.
function writeCompanionFixture(main, companionHome) {
  fs.mkdirSync(companionHome, { recursive: true });
  const startScript = path.join(main, 'fixture-companion-start.mjs');
  fs.writeFileSync(
    startScript,
    [
      "import fs from 'node:fs';",
      "import path from 'node:path';",
      `const home = ${JSON.stringify(companionHome)};`,
      'const sessionId = `sess-${Date.now()}-${Math.random().toString(36).slice(2, 8)}`;',
      'const worktreePath = path.join(home, sessionId);',
      'fs.mkdirSync(worktreePath, { recursive: true });',
      "fs.writeFileSync(path.join(worktreePath, 'marker.txt'), 'i am the companion worktree');",
      'process.stdout.write(JSON.stringify({ worktreePath, sessionId }));',
    ].join('\n'),
  );
  const endScript = path.join(main, 'fixture-companion-end.mjs');
  fs.writeFileSync(
    endScript,
    [
      "import fs from 'node:fs';",
      "import path from 'node:path';",
      `const home = ${JSON.stringify(companionHome)};`,
      'const sessionId = process.argv[2];',
      "fs.writeFileSync(path.join(home, `ended-${sessionId}.txt`), 'ended');",
      'process.stdout.write(JSON.stringify({ ok: true, sessionId }));',
    ].join('\n'),
  );
  return { startScript, endScript };
}

// Mirrors test_worktree_cli.mjs's BEE_GITIGNORE fixture exactly: without it,
// bee's own runtime writes (.bee/state.json et al) make "main" itself read
// as dirty to the D8a `git status --porcelain` checks these tests exercise —
// nothing to do with worktree-companion-hook, just fixture hygiene.
const BEE_GITIGNORE = [
  '.bee/state.json',
  '.bee/reservations.json',
  '.bee/workers/',
  '.bee/logs/',
  '.bee/capture-queue.jsonl',
  '.bee/feedback-digest.json',
  '.bee/.inject-cache.json',
  '.bee/HANDOFF.json',
  '.bee/spikes/',
  '.bee/manifest-hash.json',
  '.bee/sessions/',
  '.bee/claims/',
  '.bee/runtime/',
  '.bee/cache/',
  '.bee/locks/',
  '',
].join('\n');

function initMain(main, { withCompanion = true, brokenStart = false } = {}) {
  fs.mkdirSync(main, { recursive: true });
  git(main, ['init', '-q', '-b', 'main']);
  git(main, ['config', 'user.email', 's@e']);
  git(main, ['config', 'user.name', 's']);
  fs.writeFileSync(path.join(main, '.gitignore'), BEE_GITIGNORE);
  fs.mkdirSync(path.join(main, '.bee'), { recursive: true });
  fs.writeFileSync(path.join(main, '.bee', 'onboarding.json'), JSON.stringify({ schema_version: '1.0', bee_version: '0.0.0' }));

  const companionHome = path.join(main, '..', `companion-home-${path.basename(main)}`);
  const commands = {};
  if (withCompanion) {
    const { startScript, endScript } = writeCompanionFixture(main, companionHome);
    commands.worktree_companion_start = brokenStart ? 'node -e "process.exit(1)"' : `node ${JSON.stringify(path.basename(startScript))}`;
    commands.worktree_companion_end = `node ${JSON.stringify(path.basename(endScript))} <id>`;
    commands.worktree_companion_mount = 'companion';
  }
  fs.writeFileSync(path.join(main, '.bee', 'config.json'), JSON.stringify({ commands }));
  fs.writeFileSync(path.join(main, 'f'), 'x');
  git(main, ['add', '.']);
  git(main, ['commit', '-q', '-m', 'init']);
  return { companionHome };
}

// The companion MOUNT is a real symlink (worktree-store.mjs mounts it with
// fs.symlinkSync). Windows denies symlink creation with EPERM unless the
// process holds SeCreateSymbolicLinkPrivilege — i.e. Developer Mode is on or
// the shell is elevated; junctions still work, symlinks do not. On such a host
// every case that expects a SUCCESSFUL `--with-companion` mount is
// unrepresentable, so those cases skip loudly naming the capability instead of
// failing (the env-limited-tests-skip-loudly rule). The no-companion,
// refusal, and rollback cases never mount, so they still run.
const SYMLINK_CAPABLE = (() => {
  const probeDir = fs.mkdtempSync(path.join(os.tmpdir(), 'bee-symlink-probe-'));
  try {
    fs.symlinkSync(probeDir, path.join(probeDir, 'link'));
    return true;
  } catch {
    return false;
  } finally {
    fs.rmSync(probeDir, { recursive: true, force: true });
  }
})();
const SYMLINK_SKIP_REASON =
  'symlink creation denied (EPERM) — needs Developer Mode or an elevated shell on win32';

function skipNoSymlink(desc) {
  console.log(`SKIP (env: ${SYMLINK_SKIP_REASON}) — ${desc}`);
}

const tmp = fs.mkdtempSync(path.join(os.tmpdir(), 'bee-worktree-companion-'));
try {
  // -------------------------------------------------------------------
  // Case 1: new --with-companion mounts the symlink + writes the marker.
  // -------------------------------------------------------------------
  if (!SYMLINK_CAPABLE) {
    skipNoSymlink('new --with-companion succeeds');
    skipNoSymlink('new --with-companion reports companion {worktreePath, sessionId, mountPath}');
    skipNoSymlink('mounted path is a real symlink resolving into the companion worktree');
    skipNoSymlink('companion-session.json marker written inside the new worktree');
  } else {
    const main = path.join(tmp, 'case1-main');
    initMain(main);
    const r = bee(main, ['worktree', 'new', '--feature', 'demo-a', '--with-companion', '--json']);
    if (r.status !== 0) {
      record('new --with-companion succeeds', false, `status=${r.status} stdout=${r.stdout} stderr=${r.stderr}`);
    } else {
      const created = JSON.parse(r.stdout);
      const ok =
        created.companion &&
        typeof created.companion.worktreePath === 'string' &&
        typeof created.companion.sessionId === 'string' &&
        created.companion.mountPath === 'companion';
      record('new --with-companion reports companion {worktreePath, sessionId, mountPath}', ok, JSON.stringify(created));

      const mountPath = path.join(created.worktreeRoot, 'companion');
      const isSymlink = fs.lstatSync(mountPath).isSymbolicLink();
      const targetMarker = fs.existsSync(path.join(mountPath, 'marker.txt'));
      record('mounted path is a real symlink resolving into the companion worktree', isSymlink && targetMarker, `isSymlink=${isSymlink} targetMarker=${targetMarker}`);

      const markerPath = path.join(created.worktreeRoot, '.bee', 'companion-session.json');
      const markerExists = fs.existsSync(markerPath);
      record('companion-session.json marker written inside the new worktree', markerExists, markerPath);

      // Cleanup for a clean merge later isn't needed here — case 3 covers merge.
      git(main, ['worktree', 'remove', '--force', '--', created.worktreeRoot]);
    }
  }

  // -------------------------------------------------------------------
  // Case 2: new WITHOUT --with-companion is unaffected (backward-compat).
  // -------------------------------------------------------------------
  {
    const main = path.join(tmp, 'case2-main');
    initMain(main, { withCompanion: false });
    const r = bee(main, ['worktree', 'new', '--feature', 'demo-b', '--json']);
    const created = r.status === 0 ? JSON.parse(r.stdout) : null;
    const ok = r.status === 0 && created && created.companion === null && !fs.existsSync(path.join(created.worktreeRoot, '.bee', 'companion-session.json'));
    record('new without --with-companion: companion is null, no marker, ordinary worktree', ok, r.status === 0 ? JSON.stringify(created) : r.stderr);
  }

  // -------------------------------------------------------------------
  // Case 3: full lifecycle — new --with-companion, then merge --cleanup
  // succeeds (the untracked symlink must NOT trip WORKTREE_MERGE_WORKTREE_DIRTY),
  // the fixture end script actually runs with the real session id, and the
  // symlink + marker are gone before the dirty-check would ever see them.
  // -------------------------------------------------------------------
  if (!SYMLINK_CAPABLE) {
    skipNoSymlink('case 3 setup: new --with-companion succeeds');
    skipNoSymlink('merge --cleanup succeeds despite the companion symlink');
    skipNoSymlink('merge result reports companion.ended === true, no warning');
    skipNoSymlink('fixture end script actually ran with the real session id');
    skipNoSymlink('bee worktree itself was cleaned up (removed)');
  } else {
    const main = path.join(tmp, 'case3-main');
    const { companionHome } = initMain(main);
    const newR = bee(main, ['worktree', 'new', '--feature', 'demo-c', '--with-companion', '--json']);
    if (newR.status !== 0) {
      record('case 3 setup: new --with-companion succeeds', false, newR.stderr);
    } else {
      const created = JSON.parse(newR.stdout);
      const sessionId = created.companion.sessionId;

      // A worktree with nothing new for main hits ALREADY_UP_TO_DATE before
      // ever calling attachCleanupOutcome upstream — write a real commit in
      // the worktree so this exercises the actual merge+cleanup path.
      fs.writeFileSync(path.join(created.worktreeRoot, 'g'), 'y');
      git(created.worktreeRoot, ['add', 'g']);
      git(created.worktreeRoot, ['commit', '-q', '-m', 'feature commit']);

      const mergeR = bee(main, ['worktree', 'merge', '--id', created.id, '--cleanup', '--json']);
      if (mergeR.status !== 0) {
        record('merge --cleanup succeeds despite the companion symlink', false, `status=${mergeR.status} stdout=${mergeR.stdout} stderr=${mergeR.stderr}`);
      } else {
        const merged = JSON.parse(mergeR.stdout);
        record('merge --cleanup succeeds despite the companion symlink', merged.ok === true && merged.merged === true, JSON.stringify(merged));
        record('merge result reports companion.ended === true, no warning', merged.companion && merged.companion.ended === true && !merged.companion.warning, JSON.stringify(merged.companion));

        const endedMarker = path.join(companionHome, `ended-${sessionId}.txt`);
        record('fixture end script actually ran with the real session id', fs.existsSync(endedMarker), endedMarker);

        record('bee worktree itself was cleaned up (removed)', merged.cleanup && merged.cleanup.ok === true, JSON.stringify(merged.cleanup));
      }
    }
  }

  // -------------------------------------------------------------------
  // Case 4: --with-companion refuses cleanly when worktree_companion_start
  // isn't configured — zero mutation, no worktree created.
  // -------------------------------------------------------------------
  {
    const main = path.join(tmp, 'case4-main');
    initMain(main, { withCompanion: false });
    const before = fs.readdirSync(path.join(main, '..'));
    const r = bee(main, ['worktree', 'new', '--feature', 'demo-d', '--with-companion', '--json']);
    const after = fs.readdirSync(path.join(main, '..'));
    const refusedCleanly = r.status !== 0 && /worktree_companion_start/.test(r.stdout + r.stderr) && after.length === before.length;
    record('--with-companion without commands.worktree_companion_start refuses, zero mutation', refusedCleanly, `status=${r.status} stdout=${r.stdout} stderr=${r.stderr}`);
  }

  // -------------------------------------------------------------------
  // Case 5: a failing companion start rolls the whole worktree back —
  // same as any other post-`git worktree add` failure.
  // -------------------------------------------------------------------
  {
    const main = path.join(tmp, 'case5-main');
    initMain(main, { brokenStart: true });
    const before = fs.readdirSync(path.join(main, '..'));
    const r = bee(main, ['worktree', 'new', '--feature', 'demo-e', '--with-companion', '--json']);
    const after = fs.readdirSync(path.join(main, '..'));
    const branchGone = git(main, ['branch', '--list', 'wt/demo-e']).trim() === '';
    const rolledBack = r.status !== 0 && after.length === before.length && branchGone;
    record('a failing companion start rolls the worktree + branch back (no half-configured leftover)', rolledBack, `status=${r.status} before=${before} after=${after} branchGone=${branchGone} stdout=${r.stdout} stderr=${r.stderr}`);
  }
  // -------------------------------------------------------------------
  // Case 6 (wcg-3): concurrent + a shared nested checkout present, WITHOUT
  // --with-companion → hard fail-closed refusal, zero mutation (D1a/D3). This
  // is STR65's exact unguarded shape; red-first this row SUCCEEDS (worktree
  // created) before the handleWorktreeNew wiring lands.
  // -------------------------------------------------------------------
  {
    const main = path.join(tmp, 'case6-main');
    initMain(main, { withCompanion: false });
    plantLiveSession(main, 'other-live-session');
    plantNestedRepo(main, 'repo');
    const before = fs.readdirSync(path.join(main, '..'));
    // Acting session is a DISTINCT id from the planted foreign session, so the
    // self-exclusion in handleWorktreeNew leaves the foreign session genuinely
    // "other" — the refusal is the real second-session defense, not the acting
    // session tripping itself.
    const r = bee(main, ['worktree', 'new', '--feature', 'demo-f', '--json'], { env: { BEE_SESSION_ID: 'acting-session-6' } });
    const after = fs.readdirSync(path.join(main, '..'));
    const branchGone = git(main, ['branch', '--list', 'wt/demo-f']).trim() === '';
    const refused = r.status !== 0 && after.length === before.length && branchGone;
    record('concurrent + shared nested checkout, no --with-companion: refused, zero mutation', refused, `status=${r.status} stdout=${r.stdout} stderr=${r.stderr}`);
    record('refusal names --with-companion as the fix (paved road)', r.status !== 0 && /--with-companion/.test(r.stdout + r.stderr), r.stdout + r.stderr);
  }

  // -------------------------------------------------------------------
  // Case 7 (wcg-3): a shared nested checkout is present but NO other session
  // is live → isConcurrentMode() false → proceeds exactly as today (D6). The
  // structural half alone never trips the refusal.
  // -------------------------------------------------------------------
  {
    const main = path.join(tmp, 'case7-main');
    initMain(main, { withCompanion: false });
    plantNestedRepo(main, 'repo');
    const r = bee(main, ['worktree', 'new', '--feature', 'demo-g', '--json']);
    const created = r.status === 0 ? JSON.parse(r.stdout) : null;
    record('solo (no live session) + nested checkout: proceeds unchanged (D6)', r.status === 0 && !!created && created.companion === null, r.status === 0 ? JSON.stringify(created) : r.stderr);
  }

  // -------------------------------------------------------------------
  // Case 8 (wcg-3): another session is live but the checkout has NO shared
  // nested target → proceeds exactly as today (D6). The concurrency half alone
  // never trips the refusal either.
  // -------------------------------------------------------------------
  {
    const main = path.join(tmp, 'case8-main');
    initMain(main, { withCompanion: false });
    plantLiveSession(main);
    const r = bee(main, ['worktree', 'new', '--feature', 'demo-h', '--json']);
    const created = r.status === 0 ? JSON.parse(r.stdout) : null;
    record('concurrent but no shared nested checkout: proceeds unchanged (D6)', r.status === 0 && !!created && created.companion === null, r.status === 0 ? JSON.stringify(created) : r.stderr);
  }

  // -------------------------------------------------------------------
  // Case 9 (wcg-3): concurrent AND a shared nested checkout present, but the
  // session declared --with-companion → the new check must NEVER refuse it; the
  // full companion path runs and mounts as usual (the paved road).
  // -------------------------------------------------------------------
  if (!SYMLINK_CAPABLE) {
    skipNoSymlink('concurrent + nested checkout + --with-companion: never refused, mounts as usual');
  } else {
    const main = path.join(tmp, 'case9-main');
    initMain(main);
    plantLiveSession(main);
    plantNestedRepo(main, 'repo');
    const r = bee(main, ['worktree', 'new', '--feature', 'demo-i', '--with-companion', '--json']);
    const created = r.status === 0 ? JSON.parse(r.stdout) : null;
    const ok = r.status === 0 && !!created && !!created.companion && created.companion.mountPath === 'companion';
    record('concurrent + nested checkout + --with-companion: never refused, mounts as usual', ok, r.status === 0 ? JSON.stringify(created) : `status=${r.status} stdout=${r.stdout} stderr=${r.stderr}`);
    if (r.status === 0 && created) git(main, ['worktree', 'remove', '--force', '--', created.worktreeRoot]);
  }

  // -------------------------------------------------------------------
  // Case 10 (wcg-fix-1, P1 self-exclusion): the ONLY live session record is
  // the acting session's own — no genuine second session — yet a companion-
  // eligible nested checkout is present. handleWorktreeNew must exclude the
  // acting session from its own concurrency check, so this is NOT concurrency
  // and the worktree proceeds. Red-first this row REFUSES (false positive)
  // before the excludeSessionId wiring lands; green after.
  // -------------------------------------------------------------------
  {
    const main = path.join(tmp, 'case10-main');
    initMain(main, { withCompanion: false });
    plantLiveSession(main, 'self-session');
    plantNestedRepo(main, 'repo');
    const r = bee(main, ['worktree', 'new', '--feature', 'demo-j', '--json'], { env: { BEE_SESSION_ID: 'self-session' } });
    const created = r.status === 0 ? JSON.parse(r.stdout) : null;
    record('self-only live session (no genuine second session) + nested checkout: proceeds, not a false-positive refusal', r.status === 0 && !!created && created.companion === null, r.status === 0 ? JSON.stringify(created) : `status=${r.status} stdout=${r.stdout} stderr=${r.stderr}`);
  }

  // -------------------------------------------------------------------
  // Case 11 (wcg-fix-1, P1 regression guard): the acting session is excluded,
  // but a GENUINE second session is also live alongside the nested checkout —
  // self-exclusion must NOT disable the real defense. Still a hard refusal.
  // Green both before and after the fix (proves the fix narrows, not removes).
  // -------------------------------------------------------------------
  {
    const main = path.join(tmp, 'case11-main');
    initMain(main, { withCompanion: false });
    plantLiveSession(main, 'self-session');
    plantLiveSession(main, 'genuine-other-session');
    plantNestedRepo(main, 'repo');
    const before = fs.readdirSync(path.join(main, '..'));
    const r = bee(main, ['worktree', 'new', '--feature', 'demo-k', '--json'], { env: { BEE_SESSION_ID: 'self-session' } });
    const after = fs.readdirSync(path.join(main, '..'));
    const branchGone = git(main, ['branch', '--list', 'wt/demo-k']).trim() === '';
    const refused = r.status !== 0 && after.length === before.length && branchGone;
    record('acting session excluded but a genuine second session live: still refused (self-exclusion never disables the check)', refused, `status=${r.status} stdout=${r.stdout} stderr=${r.stderr}`);
  }

  // -------------------------------------------------------------------
  // Case 12/13 (worktree-concurrency-guard-controlroot-port, Port-D4):
  // hasAnySharedNestedCheckout's concurrency check must consult
  // opts.controlRoot, NOT the physical root the directory scan itself walks
  // — proved DIFFERENTIALLY (both directions), in-process (see the top-of-
  // file import note: handleWorktreeNew's own controlRootFor(mainRoot) is
  // architecturally pinned to mainRoot for every CLI-reachable call, so a
  // genuinely differing root/controlRoot pair can only be constructed by
  // calling the primitive directly).
  // -------------------------------------------------------------------
  {
    const physicalRoot = path.join(tmp, 'case12-root');
    const controlRootDir = path.join(tmp, 'case12-controlroot');
    fs.mkdirSync(physicalRoot, { recursive: true });
    fs.mkdirSync(controlRootDir, { recursive: true });
    plantNestedRepo(physicalRoot, 'repo');
    // Live session under controlRootDir ONLY — physicalRoot has none of its own.
    plantLiveSession(controlRootDir, 'other-live');
    const flagged = hasAnySharedNestedCheckout(physicalRoot, { controlRoot: controlRootDir });
    record('Case 12: a live session under controlRoot alone (root has none) IS flagged by hasAnySharedNestedCheckout (concurrency reads controlRoot)', flagged === true, `flagged=${flagged}`);
  }
  {
    const physicalRoot = path.join(tmp, 'case13-root');
    const controlRootDir = path.join(tmp, 'case13-controlroot');
    fs.mkdirSync(physicalRoot, { recursive: true });
    fs.mkdirSync(controlRootDir, { recursive: true });
    plantNestedRepo(physicalRoot, 'repo');
    // The SAME live session, but planted under physicalRoot instead —
    // controlRootDir has none. A bug that silently read `root` instead of
    // `opts.controlRoot` would flag this (false positive); the real code must not.
    plantLiveSession(physicalRoot, 'other-live');
    const flagged = hasAnySharedNestedCheckout(physicalRoot, { controlRoot: controlRootDir });
    record('Case 13: the SAME live session planted under root alone (controlRoot has none) is NOT flagged by hasAnySharedNestedCheckout (concurrency ignores root)', flagged === false, `flagged=${flagged}`);
  }

  // -------------------------------------------------------------------
  // Case 14 (review finding F1, worktree-concurrency-guard-controlroot-port):
  // a REAL hard fs error reading the controlRoot's session records (not a
  // synthetic stub) must propagate out of hasAnySharedNestedCheckout, not
  // silently read as "nobody else is live". `.bee/sessions` is made a FILE
  // instead of a directory — a genuine, portable (root-safe) ENOTDIR, the
  // exact shape F1 flagged as fail-open before this fix.
  // -------------------------------------------------------------------
  {
    const physicalRoot = path.join(tmp, 'case14-root');
    const controlRootDir = path.join(tmp, 'case14-controlroot');
    fs.mkdirSync(physicalRoot, { recursive: true });
    fs.mkdirSync(path.join(controlRootDir, '.bee'), { recursive: true });
    plantNestedRepo(physicalRoot, 'repo');
    fs.writeFileSync(path.join(controlRootDir, '.bee', 'sessions'), 'not a directory\n');
    let threw = null;
    try {
      hasAnySharedNestedCheckout(physicalRoot, { controlRoot: controlRootDir });
    } catch (err) {
      threw = err;
    }
    record(
      'Case 14: a real hard error reading controlRoot session records (ENOTDIR) propagates out of hasAnySharedNestedCheckout instead of silently returning false',
      threw !== null && /ENOTDIR/.test(threw.message || String(threw)),
      threw ? `threw: ${threw.message}` : 'did not throw',
    );
  }
} finally {
  fs.rmSync(tmp, { recursive: true, force: true });
}

const failed = results.filter((r) => !r.passed);
console.log(`\nSUMMARY: ${results.length - failed.length}/${results.length} passed`);
process.exit(failed.length ? 7 : 0);
