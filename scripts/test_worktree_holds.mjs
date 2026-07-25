#!/usr/bin/env node
// test_worktree_holds.mjs — cell xwh-2: proves the cross-worktree holds
// ledger (worktree-holds.mjs, xwh-1) is actually WIRED into the reservation
// seam (bee.mjs handleReservationsReserve/Release/Sweep/List) and into the
// worktree lifecycle (worktree-store.mjs performCleanup), not just a proven
// but unwired primitive.
//
// Runs the REAL `bee` CLI via spawnSync (no mocking of the dispatcher, same
// convention as scripts/test_worktree_cli.mjs) against `.bee/bin/bee.mjs` —
// the self-onboarded copy the standing verify chain actually spawns, same as
// every other worktree-* suite in scripts/. Two shapes of fixture, matching
// the cell's guidance ("fabricate the topology inputs ... rather than
// creating real git worktrees where practical"):
//   - "ordinary checkout" cases (mirror, foreign-refusal, release, sweep):
//     a PLAIN temp repo (fake `.git` directory marker, same as
//     scripts/lib/test-fixture.mjs's makeTempRepo — resolveRoots only checks
//     `.git` is a directory vs. a file, so no real git init is needed) with
//     a hand-written ledger file where a foreign hold needs to already
//     exist — no real worktree, no real git.
//   - "linked worktree" cases (cleanup releases a holder's holds; an
//     ungranted worktree never double-mirrors): a REAL git repo + REAL
//     `git worktree add`, because resolveRoots' linked-valid classification
//     needs genuine bidirectional gitdir pointer files that only `git`
//     itself produces correctly.
//
// Exit 0 iff every case passes.

import { spawnSync } from 'node:child_process';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

import { writeJsonAtomic } from '../.bee/bin/lib/fsutil.mjs';
import { check, assert, printSummaryAndExit } from './lib/test-fixture.mjs';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const REPO_ROOT = path.join(__dirname, '..');
const BEE_MJS = path.join(REPO_ROOT, '.bee', 'bin', 'bee.mjs');

// ─── helpers ─────────────────────────────────────────────────────────────

function bee(cwd, args) {
  return spawnSync(process.execPath, [BEE_MJS, ...args], { cwd, encoding: 'utf8' });
}

function git(cwd, args) {
  const r = spawnSync('git', args, { cwd, encoding: 'utf8' });
  if (r.status !== 0) throw new Error(`git ${args.join(' ')} (cwd=${cwd}) failed: ${r.stderr}`);
  return r.stdout;
}

function beeJson(cwd, args) {
  const r = bee(cwd, [...args, '--json']);
  let parsed = null;
  try {
    parsed = JSON.parse(r.stdout);
  } catch {
    // left null — caller asserts on r.status / r.stdout / r.stderr instead.
  }
  return { r, json: parsed };
}

/** Plain temp repo: fake `.git` DIRECTORY marker (resolveRoots only checks
 * isFile() vs isDirectory(), never runs real git here) + a bootstrapped
 * `.bee/onboarding.json` — same minimal fixture shape as
 * scripts/lib/test-fixture.mjs's makeTempRepo, inlined here so this suite
 * has zero dependency on real git for its "ordinary checkout" cases. */
function makeOrdinaryRepo() {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), 'bee-holds-ord-'));
  fs.mkdirSync(path.join(root, '.git'), { recursive: true });
  fs.mkdirSync(path.join(root, '.bee'), { recursive: true });
  writeJsonAtomic(path.join(root, '.bee', 'onboarding.json'), { schema_version: '1.0', bee_version: '0.0.0' });
  fs.mkdirSync(path.join(root, 'src'), { recursive: true });
  return root;
}

function ledgerPath(mainRoot) {
  return path.join(mainRoot, '.bee', 'runtime', 'cross-worktree-holds.json');
}

function readLedger(mainRoot) {
  if (!fs.existsSync(ledgerPath(mainRoot))) return { holds: [] };
  return JSON.parse(fs.readFileSync(ledgerPath(mainRoot), 'utf8'));
}

function reservationsFilePath(root) {
  return path.join(root, '.bee', 'reservations.json');
}

/** Directly writes a fabricated foreign hold into `mainRoot`'s ledger — the
 * "fabricate the topology inputs" path for the foreign-refusal case: no real
 * second checkout is needed, only a ledger row that claims one exists. */
function seedForeignHold(mainRoot, hold) {
  fs.mkdirSync(path.dirname(ledgerPath(mainRoot)), { recursive: true });
  const store = readLedger(mainRoot);
  store.holds.push({
    path: hold.path,
    holder: hold.holder,
    feature: hold.feature ?? null,
    session: hold.session ?? null,
    cell: hold.cell ?? null,
    ttl_seconds: hold.ttl_seconds ?? 3600,
    mirrored_at: hold.mirrored_at ?? new Date().toISOString(),
    released_at: hold.released_at ?? null,
  });
  writeJsonAtomic(ledgerPath(mainRoot), store);
}

/** Real git repo, real linked worktree — `mergeNewWorktree`'s sibling for
 * this suite, trimmed to just "create main + one linked worktree" (no merge
 * fixture bootstrap needed here). Returns { main, worktreeRoot, id }. */
function makeLinkedWorktree(feature) {
  const tmp = fs.mkdtempSync(path.join(os.tmpdir(), 'bee-holds-wt-'));
  const main = path.join(tmp, 'main');
  fs.mkdirSync(main);
  git(main, ['init', '-q', '-b', 'main']);
  git(main, ['config', 'user.email', 's@e']);
  git(main, ['config', 'user.name', 's']);

  // Same .gitignore shape as scripts/test_worktree_cli.mjs's own
  // BEE_GITIGNORE/initMergeFixtureMain: every bee-runtime path (.bee/runtime/
  // is where the cross-worktree ledger and worktree-grants registry live,
  // .bee/cache/ is written on every single CLI invocation via
  // checkManifestDrift) must be ignored, or `git status --porcelain` sees
  // them as untracked dirt and "worktree merge" refuses
  // WORKTREE_MERGE_MAIN_DIRTY before cleanup ever runs.
  fs.writeFileSync(
    path.join(main, '.gitignore'),
    [
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
      // hardening-4b: withStoreLock's lockfiles — same rationale as
      // test_worktree_cli.mjs's own BEE_GITIGNORE update.
      '.bee/locks/',
      '',
    ].join('\n'),
  );
  const mainBeeDir = path.join(main, '.bee');
  fs.mkdirSync(mainBeeDir, { recursive: true });
  fs.writeFileSync(path.join(mainBeeDir, 'onboarding.json'), JSON.stringify({ schema_version: '1.0', bee_version: '0.0.0' }));
  fs.writeFileSync(path.join(mainBeeDir, 'config.json'), JSON.stringify({ commands: {} }));
  fs.writeFileSync(path.join(main, 'f'), 'x');
  git(main, ['add', '.']);
  git(main, ['commit', '-q', '-m', 'init']);

  const worktreeRoot = path.join(tmp, `wt-${feature}`);
  git(main, ['worktree', 'add', '-q', '-b', `wt/${feature}`, worktreeRoot]);
  const gitFile = fs.readFileSync(path.join(worktreeRoot, '.git'), 'utf8').trim();
  const m = gitFile.match(/^gitdir:\s*(.+)$/);
  const gitdir = path.resolve(worktreeRoot, m[1].trim());
  const id = path.basename(gitdir);

  return { tmpRoot: tmp, main, worktreeRoot, id };
}

const cleanupDirs = [];
function trackCleanup(dir) {
  cleanupDirs.push(dir);
  return dir;
}

// ─── case 1: reserve-mirrors-to-ledger ──────────────────────────────────────

check('reserve from an ordinary checkout mirrors into the shared ledger under holder "main"', () => {
  const root = trackCleanup(makeOrdinaryRepo());
  const { r, json } = beeJson(root, ['reservations', 'reserve', '--agent', 'a1', '--cell', 'c1', '--path', 'src/foo']);
  assert(r.status === 0, `reserve exited ${r.status}: ${r.stdout}${r.stderr}`);
  assert(json && json.ok === true, `expected ok:true, got ${r.stdout}`);

  const ledger = readLedger(root);
  assert(ledger.holds.length === 1, `expected exactly 1 mirrored hold, got ${ledger.holds.length}`);
  const hold = ledger.holds[0];
  assert(hold.holder === 'main', `expected holder "main", got ${JSON.stringify(hold.holder)}`);
  assert(hold.path === 'src/foo', `expected path "src/foo", got ${JSON.stringify(hold.path)}`);
  assert(hold.cell === 'c1', `expected cell "c1", got ${JSON.stringify(hold.cell)}`);
  assert(hold.released_at === null, 'freshly mirrored hold should not be released');
});

check('reservations list --json surfaces the mirrored entry under cross_worktree', () => {
  const root = trackCleanup(makeOrdinaryRepo());
  beeJson(root, ['reservations', 'reserve', '--agent', 'a1', '--cell', 'c1', '--path', 'src/foo']);
  const { r, json } = beeJson(root, ['reservations', 'list']);
  assert(r.status === 0, `list exited ${r.status}: ${r.stdout}${r.stderr}`);
  assert(Array.isArray(json.cross_worktree), 'expected a cross_worktree array in the JSON result');
  assert(json.cross_worktree.length === 1, `expected 1 cross_worktree entry, got ${json.cross_worktree.length}`);
  assert(json.cross_worktree[0].holder === 'main', 'cross_worktree entry should be holder "main"');
});

check('a repo that never reserves anything has no cross-worktree ledger file (byte-identical to today)', () => {
  const root = trackCleanup(makeOrdinaryRepo());
  assert(!fs.existsSync(ledgerPath(root)), 'ledger file should not exist until something mirrors into it');
  const { r, json } = beeJson(root, ['reservations', 'list']);
  assert(r.status === 0, `list exited ${r.status}: ${r.stdout}${r.stderr}`);
  assert(Array.isArray(json.cross_worktree) && json.cross_worktree.length === 0, 'missing ledger should read as an empty cross_worktree list, not a crash');
});

// ─── case 2: foreign-hold typed refusal shape ──────────────────────────────

check('reserve against a path a DIFFERENT checkout holds is refused as a typed FOREIGN_HOLD naming holder + expiry', () => {
  const root = trackCleanup(makeOrdinaryRepo());
  seedForeignHold(root, {
    path: 'src/shared',
    holder: 'wt-foreign-1234',
    feature: 'other-feature',
    cell: 'other-cell',
    ttl_seconds: 3600,
  });

  const { r, json } = beeJson(root, ['reservations', 'reserve', '--agent', 'a2', '--cell', 'c2', '--path', 'src/shared']);
  assert(r.status !== 0, `expected a nonzero exit on a foreign hold, got ${r.status}`);
  assert(json && json.ok === false, `expected ok:false, got ${r.stdout}`);
  assert(json.code === 'FOREIGN_HOLD', `expected code FOREIGN_HOLD, got ${JSON.stringify(json.code)}`);
  assert(json.holder === 'wt-foreign-1234', `refusal should name the foreign holder, got ${JSON.stringify(json.holder)}`);
  assert(json.cell === 'other-cell', `refusal should name the foreign cell, got ${JSON.stringify(json.cell)}`);
  assert(typeof json.expires === 'string' && json.expires.includes('expires'), `refusal should name an expiry, got ${JSON.stringify(json.expires)}`);

  // the human-text form (no --json) mirrors guards.mjs's cross-session-hold
  // phrasing convention and names holder + expiry directly in prose.
  const textResult = bee(root, ['reservations', 'reserve', '--agent', 'a2', '--cell', 'c2', '--path', 'src/shared']);
  assert(textResult.status !== 0, 'text-mode refusal should also exit nonzero');
  assert(textResult.stdout.includes('wt-foreign-1234'), `text refusal should name the holder, got: ${textResult.stdout}`);
  assert(textResult.stdout.includes('expires'), `text refusal should name an expiry, got: ${textResult.stdout}`);
  assert(/cross-worktree hold/i.test(textResult.stdout), `text refusal should call itself a cross-worktree hold, got: ${textResult.stdout}`);

  // zero-mutation before the local reserve: no local reservation row exists.
  assert(!fs.existsSync(reservationsFilePath(root)), 'a refused reserve must never write a local reservation row');
});

check('a reserve on a NON-overlapping path is unaffected by an unrelated foreign hold', () => {
  const root = trackCleanup(makeOrdinaryRepo());
  seedForeignHold(root, { path: 'src/other', holder: 'wt-foreign-1234' });
  const { r, json } = beeJson(root, ['reservations', 'reserve', '--agent', 'a3', '--cell', 'c3', '--path', 'src/unrelated']);
  assert(r.status === 0, `expected exit 0, got ${r.status}: ${r.stdout}${r.stderr}`);
  assert(json && json.ok === true, `expected ok:true, got ${r.stdout}`);
});

// ─── case 3: release-clears ─────────────────────────────────────────────────

check('release clears this checkout\'s mirrored ledger entries', () => {
  const root = trackCleanup(makeOrdinaryRepo());
  beeJson(root, ['reservations', 'reserve', '--agent', 'a4', '--cell', 'c4', '--path', 'src/rel']);
  assert(readLedger(root).holds.some((h) => h.released_at === null), 'sanity: the ledger should have an active entry before release');

  const { r, json } = beeJson(root, ['reservations', 'release', '--agent', 'a4', '--cell', 'c4']);
  assert(r.status === 0, `release exited ${r.status}: ${r.stdout}${r.stderr}`);
  assert(json.holds_released === 1, `expected holds_released:1, got ${JSON.stringify(json.holds_released)}`);

  const ledger = readLedger(root);
  assert(ledger.holds.length === 1, 'release should not delete the row, only mark it released');
  assert(ledger.holds[0].released_at !== null, 'the mirrored entry should be released after "reservations release"');

  const { json: listJson } = beeJson(root, ['reservations', 'list']);
  assert(listJson.cross_worktree.length === 0, 'a released hold must not appear in the active cross_worktree list');
});

check(
  'release --agent (no --cell) only clears THAT agent\'s own mirrored cell(s) — never a different agent/cell sharing the same holder (regression: caught live, a same-checkout agent-wide release once wiped out a concurrent agent\'s active mirrored holds too, because a ledger row has no agent field, only holder+cell)',
  () => {
    const root = trackCleanup(makeOrdinaryRepo());
    // Two DIFFERENT agents, two DIFFERENT cells, same ordinary checkout —
    // both mirror under the SAME holder ('main'), exactly the shared-holder
    // shape that made the original bug possible.
    beeJson(root, ['reservations', 'reserve', '--agent', 'agent-one', '--cell', 'cell-one', '--path', 'src/one']);
    beeJson(root, ['reservations', 'reserve', '--agent', 'agent-two', '--cell', 'cell-two', '--path', 'src/two']);
    const before = readLedger(root);
    assert(before.holds.length === 2, `sanity: expected 2 mirrored holds before release, got ${before.holds.length}`);

    // agent-one releases WITHOUT --cell (the exact shape that regressed).
    const { r, json } = beeJson(root, ['reservations', 'release', '--agent', 'agent-one']);
    assert(r.status === 0, `release exited ${r.status}: ${r.stdout}${r.stderr}`);
    assert(json.holds_released === 1, `expected holds_released:1 (only agent-one's own cell), got ${JSON.stringify(json.holds_released)}`);

    const after = readLedger(root);
    const cellOneHold = after.holds.find((h) => h.cell === 'cell-one');
    const cellTwoHold = after.holds.find((h) => h.cell === 'cell-two');
    assert(cellOneHold && cellOneHold.released_at !== null, 'agent-one\'s own mirrored hold (cell-one) should be released');
    assert(cellTwoHold && cellTwoHold.released_at === null, 'agent-two\'s mirrored hold (cell-two) must survive agent-one\'s release untouched');

    const { json: listJson } = beeJson(root, ['reservations', 'list']);
    assert(
      listJson.cross_worktree.some((h) => h.cell === 'cell-two'),
      `agent-two's cell-two hold should still be active in cross_worktree, got ${JSON.stringify(listJson.cross_worktree)}`,
    );
    assert(
      !listJson.cross_worktree.some((h) => h.cell === 'cell-one'),
      'agent-one\'s cell-one hold should no longer be active in cross_worktree',
    );
  },
);

// ─── case 4: sweep-prunes ───────────────────────────────────────────────────

check('sweep prunes TTL-expired ledger entries', () => {
  const root = trackCleanup(makeOrdinaryRepo());
  beeJson(root, ['reservations', 'reserve', '--agent', 'a5', '--cell', 'c5', '--path', 'src/exp', '--ttl', '1']);

  // Force expiry deterministically (no sleep): push mirrored_at into the
  // past far enough that ttl_seconds=1 has already elapsed.
  const store = readLedger(root);
  store.holds[0].mirrored_at = new Date(Date.now() - 60_000).toISOString();
  writeJsonAtomic(ledgerPath(root), store);

  const { r, json } = beeJson(root, ['reservations', 'sweep']);
  assert(r.status === 0, `sweep exited ${r.status}: ${r.stdout}${r.stderr}`);
  assert(json.holds_released === 1, `expected holds_released:1, got ${JSON.stringify(json.holds_released)}`);

  const after = readLedger(root);
  assert(after.holds[0].released_at !== null, 'the expired entry should be released after sweep');
});

// ─── case 5: cleanup-releases-all-for-holder ────────────────────────────────

check('worktree merge --cleanup releases every mirrored hold for the removed worktree\'s id', () => {
  const { tmpRoot, main, worktreeRoot, id } = makeLinkedWorktree('holds-cleanup');
  trackCleanup(tmpRoot);

  const registerResult = bee(worktreeRoot, ['worktree', 'register', '--feature', 'holds-cleanup', '--json']);
  assert(registerResult.status === 0, `worktree register exited ${registerResult.status}: ${registerResult.stdout}${registerResult.stderr}`);

  const { r: reserveResult, json: reserveJson } = beeJson(worktreeRoot, [
    'reservations',
    'reserve',
    '--agent',
    'aw',
    '--cell',
    'cw',
    '--path',
    'src/wtfile',
  ]);
  assert(reserveResult.status === 0, `reserve inside the granted worktree exited ${reserveResult.status}: ${reserveResult.stdout}${reserveResult.stderr}`);
  assert(reserveJson && reserveJson.ok === true, `expected ok:true, got ${reserveResult.stdout}`);

  const beforeLedger = readLedger(main);
  const mirroredForId = beforeLedger.holds.find((h) => h.holder === id && h.released_at === null);
  assert(mirroredForId, `expected an active mirrored hold for holder ${JSON.stringify(id)} before cleanup, ledger: ${JSON.stringify(beforeLedger)}`);

  // give the worktree branch something to merge
  fs.writeFileSync(path.join(worktreeRoot, 'wt-change.txt'), 'hello');
  git(worktreeRoot, ['add', '.']);
  git(worktreeRoot, ['commit', '-q', '-m', 'wt change']);

  const mergeResult = bee(main, ['worktree', 'merge', '--id', id, '--cleanup', '--json']);
  assert(mergeResult.status === 0, `worktree merge --cleanup exited ${mergeResult.status}: ${mergeResult.stdout}${mergeResult.stderr}`);
  const mergeJson = JSON.parse(mergeResult.stdout);
  assert(mergeJson.ok === true, `expected merge ok:true, got ${mergeResult.stdout}`);
  assert(mergeJson.cleanup && mergeJson.cleanup.ok === true, `expected cleanup.ok:true, got ${JSON.stringify(mergeJson.cleanup)}`);

  const afterLedger = readLedger(main);
  const stillActiveForId = afterLedger.holds.filter((h) => h.holder === id && h.released_at === null);
  assert(stillActiveForId.length === 0, `expected every hold for holder ${JSON.stringify(id)} to be released after cleanup, still active: ${JSON.stringify(stillActiveForId)}`);
  const releasedForId = afterLedger.holds.filter((h) => h.holder === id && h.released_at !== null);
  assert(releasedForId.length >= 1, 'the mirrored hold for the removed worktree should now be marked released');
});

// ─── case 6: ungranted-no-double-mirror ─────────────────────────────────────

check('reserve from an UNGRANTED linked worktree lands in the shared main reservations store but never mirrors into the ledger', () => {
  const { tmpRoot, main, worktreeRoot } = makeLinkedWorktree('holds-ungranted');
  trackCleanup(tmpRoot);

  // deliberately never "worktree register" — storeRoot falls back to main
  // (P40 default).
  assert(!fs.existsSync(ledgerPath(main)), 'sanity: no ledger should exist yet in this fresh fixture');

  const { r, json } = beeJson(worktreeRoot, ['reservations', 'reserve', '--agent', 'au', '--cell', 'cu', '--path', 'src/ungranted']);
  assert(r.status === 0, `reserve from the ungranted worktree exited ${r.status}: ${r.stdout}${r.stderr}`);
  assert(json && json.ok === true, `expected ok:true, got ${r.stdout}`);

  // the local reservation landed in MAIN's own store (shared, since
  // storeRoot === mainRoot for an ungranted linked worktree) — multisession-
  // native-16: `.bee/reservations.json` is a rebuildable PROJECTION nothing
  // writes synchronously anymore (reservations.mjs's own module header), so
  // this is proven through the live `reservations list` CLI output (which
  // reads lease-store.mjs's files through the shim) rather than the legacy
  // file directly.
  const { json: mainList } = beeJson(main, ['reservations', 'list', '--active-only']);
  assert(
    Array.isArray(mainList.reservations) && mainList.reservations.some((res) => res.path === 'src/ungranted' && res.agent === 'au'),
    `expected the reservation to appear in main's live reservation list, got ${JSON.stringify(mainList)}`,
  );

  // ... but nothing was ever mirrored into the cross-worktree ledger — an
  // ungranted linked worktree already shares main's own store directly, so
  // mirroring it again would just be a duplicate.
  assert(!fs.existsSync(ledgerPath(main)), 'an ungranted linked worktree reserve must never create/populate the cross-worktree ledger');
});

// ─── cleanup ─────────────────────────────────────────────────────────────

for (const dir of cleanupDirs) {
  try {
    fs.rmSync(dir, { recursive: true, force: true });
  } catch {
    // best-effort cleanup only.
  }
}

printSummaryAndExit();
