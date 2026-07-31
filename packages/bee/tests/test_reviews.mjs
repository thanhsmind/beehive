#!/usr/bin/env node
// test_reviews.mjs — lib/reviews.mjs contract tests (session store + candidates
// ledger, bee.mjs reviews CLI, derived coverage/staleness engine, bee.mjs
// status review integration), split out of test_lib.mjs (cs-2a) to shrink the
// monolith. Same PASS/FAIL/exit-1 contract as every other suite here — see
// scripts/lib/test-fixture.mjs.

import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { spawnSync } from 'node:child_process';
import { fileURLToPath } from 'node:url';
import { runModuleWorker } from '../../../scripts/lib/run-module-worker.mjs';
import { check, assert, assertThrows, printSummaryAndExit } from '../../../scripts/lib/test-fixture.mjs';
import { defaultState, writeState } from '../lib/state.mjs';
import { addCell, readCell, claimCell, capCell } from '../lib/cells.mjs';
import {
  createReview,
  listReviews,
  readReview,
  readReviewStrict,
  recordOnReview,
  addCandidate,
  listCandidates,
  deriveCandidateStatus,
  CANDIDATE_STATUSES,
  reviewsDir,
  candidatesPath,
  REVIEW_MODES,
  SCOPE_ENTRY_TYPES,
} from '../lib/reviews.mjs';
import { readJson, writeJsonAtomic } from '../lib/fsutil.mjs';

// Self-containment fix (cs-2a split): makeStateRepo is defined in test_lib.mjs's
// "bee.mjs state CLI" section (a section that stays behind for cs-2b) and was
// only reachable here via function-declaration hoisting across the whole
// monolith. The reviews-CLI rows below need a throwaway repo distinct from
// any other fixture — verbatim copy of that helper, same shape, zero check
// weakened.
function makeStateRepo(prefix) {
  const dir = fs.mkdtempSync(path.join(os.tmpdir(), prefix));
  fs.mkdirSync(path.join(dir, '.bee'), { recursive: true });
  writeJsonAtomic(path.join(dir, '.bee', 'onboarding.json'), {
    schema_version: '1.0',
    bee_version: '0.1.0',
  });
  return dir;
}

// ─── reviews: session store + candidates ledger (review-od-1, decisions ─────
// 565e68d0/bb4bb18e) ───────────────────────────────────────────────────────
// Full review is user-invoked (565e68d0); this store freezes an immutable
// review scope (SPEC §8) and fails closed on missing verification evidence
// (A10) or in-progress work (A6) BEFORE any file is written. Mirrors the
// scribingDebt/frozen-judge sections above: fresh mkdtemp repo per test,
// direct lib calls (bee.mjs reviews is a thin CLI wrapper, covered
// separately below), gate execution approved by hand where claim is needed.

function makeReviewRepo(prefix) {
  const dir = fs.mkdtempSync(path.join(os.tmpdir(), prefix));
  fs.mkdirSync(path.join(dir, '.bee'), { recursive: true });
  writeJsonAtomic(path.join(dir, '.bee', 'onboarding.json'), {
    schema_version: '1.0',
    bee_version: '0.1.0',
  });
  writeState(dir, {
    ...defaultState(),
    phase: 'swarming',
    feature: 'demo',
    approved_gates: { context: true, shape: true, execution: true, review: false },
  });
  return dir;
}

function reviewCell(id, extra = {}) {
  return {
    id,
    feature: 'demo',
    title: `Cell ${id}`,
    lane: 'small',
    status: 'open',
    deps: [],
    action: 'Do the thing.',
    verify: 'node -e "process.exit(0)"',
    ...extra,
  };
}

/** A capped behavior_change cell (test-simple: no evidence machinery). */
async function seedCappedCellWithEvidence(dir, id) {
  addCell(dir, reviewCell(id, { behavior_change: true }));
  await claimCell(dir, id, 'worker-rev');
  await capCell(dir, id, {
    files_changed: ['a.js'],
    outcome: 'done',
  });
}

function baseScope(overrides = {}) {
  return {
    id: 'rev-1',
    requested_by: 'user',
    scope_description: 'review the demo feature',
    included: [{ type: 'cell', id: 'ok-1' }],
    baseline: 'sha-base',
    head: 'sha-head',
    ...overrides,
  };
}

await check('createReview: session roundtrip carries every SPEC §8 field, and show/readReview round-trips it', async () => {
  const dir = makeReviewRepo('bee-reviews-roundtrip-');
  try {
    await seedCappedCellWithEvidence(dir, 'ok-1');
    const session = createReview(dir, baseScope());
    for (const field of [
      'id', 'requested_by', 'requested_at', 'scope_description', 'included', 'excluded',
      'baseline', 'head', 'reviewer_manifest', 'verification_preflight', 'findings', 'uat',
      'decision', 'created_at', 'updated_at',
    ]) {
      assert(field in session, `session is missing SPEC §8 field "${field}"`);
    }
    assert(session.decision.status === 'pending', 'new session decision starts pending');
    assert(session.included.length === 1 && session.included[0].id === 'ok-1', 'included cell carried through');
    assert(fs.existsSync(path.join(reviewsDir(dir), 'rev-1.json')), 'session file written to .bee/reviews/<id>.json');
    const reread = readReview(dir, 'rev-1');
    assert(JSON.stringify(reread) === JSON.stringify(session), 'readReview round-trips the written session');
    const list = listReviews(dir);
    assert(list.length === 1 && list[0].id === 'rev-1', 'listReviews finds the new session');
  } finally {
    fs.rmSync(dir, { recursive: true, force: true });
  }
});

await check('createReview (test-simple): the A10 evidence preflight is deleted — a capped behavior_change cell with no evidence fields is normal scope, and the stored preflight counts it', async () => {
  const dir = makeReviewRepo('bee-reviews-a10-');
  try {
    await seedCappedCellWithEvidence(dir, 'legacy-1');
    const session = createReview(dir, baseScope({ id: 'rev-a10', included: [{ type: 'cell', id: 'legacy-1' }] }));
    assert(session.included.length === 1 && session.included[0].id === 'legacy-1', 'the evidence-less cap stays included — review never re-litigates the deleted evidence machinery');
    assert(Array.isArray(session.verification_preflight.cells_checked) && session.verification_preflight.cells_checked.includes('legacy-1'), `the stored preflight still counts behavior_change caps in scope, got ${JSON.stringify(session.verification_preflight)}`);
  } finally {
    fs.rmSync(dir, { recursive: true, force: true });
  }
});

await check('createReview: A6 auto-excludes an open/claimed included cell with reason "in progress", never silently reviewed-in', async () => {
  const dir = makeReviewRepo('bee-reviews-a6-');
  try {
    await seedCappedCellWithEvidence(dir, 'ok-1');
    addCell(dir, reviewCell('open-1')); // stays open — never claimed
    addCell(dir, reviewCell('claimed-1'));
    await claimCell(dir, 'claimed-1', 'worker-rev');

    const session = createReview(
      dir,
      baseScope({
        id: 'rev-a6',
        included: [
          { type: 'cell', id: 'ok-1' },
          { type: 'cell', id: 'open-1' },
          { type: 'cell', id: 'claimed-1' },
        ],
      }),
    );
    const includedIds = session.included.map((e) => e.id);
    assert(includedIds.length === 1 && includedIds[0] === 'ok-1', `only the capped cell stays included, got ${JSON.stringify(includedIds)}`);
    const excludedOpen = session.excluded.find((e) => e.id === 'open-1');
    const excludedClaimed = session.excluded.find((e) => e.id === 'claimed-1');
    assert(excludedOpen && excludedOpen.reason === 'in progress', 'open-1 auto-excluded with reason "in progress"');
    assert(excludedClaimed && excludedClaimed.reason === 'in progress', 'claimed-1 auto-excluded with reason "in progress"');
    // A6 must never leave the underlying cell's own state touched.
    assert(readCell(dir, 'open-1').status === 'open', 'excluding from review scope does not touch the cell itself');
    assert(readCell(dir, 'claimed-1').status === 'claimed', 'excluding from review scope does not touch the cell itself');
  } finally {
    fs.rmSync(dir, { recursive: true, force: true });
  }
});

await check('createReview: a pre-declared "excluded" entry in the scope input is preserved alongside auto-exclusions', async () => {
  const dir = makeReviewRepo('bee-reviews-preexcl-');
  try {
    await seedCappedCellWithEvidence(dir, 'ok-1');
    const session = createReview(
      dir,
      baseScope({
        id: 'rev-preexcl',
        included: [{ type: 'cell', id: 'ok-1' }],
        excluded: [{ type: 'commit', id: 'deadbeef', reason: 'unrelated hotfix' }],
      }),
    );
    const pre = session.excluded.find((e) => e.id === 'deadbeef');
    assert(pre && pre.reason === 'unrelated hotfix', 'pre-declared exclusion reason preserved verbatim');
  } finally {
    fs.rmSync(dir, { recursive: true, force: true });
  }
});

await check('createReview: refuses an already-existing session id with non-zero-equivalent throw and leaves the file byte-unchanged (id non-reuse, §8)', async () => {
  const dir = makeReviewRepo('bee-reviews-idreuse-');
  try {
    await seedCappedCellWithEvidence(dir, 'ok-1');
    createReview(dir, baseScope({ id: 'rev-dup' }));
    const before = fs.readFileSync(path.join(reviewsDir(dir), 'rev-dup.json'), 'utf8');
    assertThrows(
      () => createReview(dir, baseScope({ id: 'rev-dup', scope_description: 'a different description' })),
      'already exists',
      'duplicate id refused',
    );
    const after = fs.readFileSync(path.join(reviewsDir(dir), 'rev-dup.json'), 'utf8');
    assert(before === after, 'the existing session file is byte-unchanged after a refused duplicate create');
  } finally {
    fs.rmSync(dir, { recursive: true, force: true });
  }
});

await check('createReview: rejects missing required scope fields and an empty "included" array before any write', async () => {
  const dir = makeReviewRepo('bee-reviews-validate-');
  try {
    assertThrows(() => createReview(dir, baseScope({ requested_by: '' })), 'requested_by', 'requested_by required');
    assertThrows(() => createReview(dir, baseScope({ baseline: undefined })), 'baseline', 'baseline required');
    assertThrows(() => createReview(dir, baseScope({ included: [] })), 'included', 'non-empty included required');
    assert(!fs.existsSync(reviewsDir(dir)), 'no session dir created by any rejected create');
  } finally {
    fs.rmSync(dir, { recursive: true, force: true });
  }
});

await check('recordOnReview: refuses any payload touching baseline/head/included/excluded — exits via throw, file byte-unchanged (R5 immutability)', async () => {
  const dir = makeReviewRepo('bee-reviews-immutable-');
  try {
    await seedCappedCellWithEvidence(dir, 'ok-1');
    createReview(dir, baseScope({ id: 'rev-immut' }));
    const before = fs.readFileSync(path.join(reviewsDir(dir), 'rev-immut.json'), 'utf8');
    for (const field of ['baseline', 'head', 'included', 'excluded']) {
      assertThrows(
        () => recordOnReview(dir, 'rev-immut', { kind: 'manifest', payload: { [field]: 'nope' } }),
        'immutable',
        `record must refuse a payload touching "${field}"`,
      );
    }
    const after = fs.readFileSync(path.join(reviewsDir(dir), 'rev-immut.json'), 'utf8');
    assert(before === after, 'session file byte-unchanged after every refused immutability attempt');
  } finally {
    fs.rmSync(dir, { recursive: true, force: true });
  }
});

await check('recordOnReview: manifest/preflight/decision SET the field; finding/uat APPEND one entry per call', async () => {
  const dir = makeReviewRepo('bee-reviews-record-');
  try {
    await seedCappedCellWithEvidence(dir, 'ok-1');
    createReview(dir, baseScope({ id: 'rev-record' }));

    let session = recordOnReview(dir, 'rev-record', {
      kind: 'manifest',
      payload: { reviewers: ['a', 'b'] },
    });
    assert(JSON.stringify(session.reviewer_manifest) === JSON.stringify({ reviewers: ['a', 'b'] }), 'manifest set');

    session = recordOnReview(dir, 'rev-record', { kind: 'finding', payload: { severity: 'P1', description: 'x' } });
    session = recordOnReview(dir, 'rev-record', { kind: 'finding', payload: { severity: 'P2', description: 'y' } });
    assert(session.findings.length === 2, `findings append, got ${session.findings.length}`);
    assert(session.findings[0].severity === 'P1' && session.findings[1].severity === 'P2', 'append order preserved');

    session = recordOnReview(dir, 'rev-record', { kind: 'uat', payload: { item: 'login flow', result: 'pass' } });
    assert(session.uat.length === 1 && session.uat[0].item === 'login flow', 'uat appended');

    session = recordOnReview(dir, 'rev-record', {
      kind: 'decision',
      payload: { status: 'approved', gate4: { approved_by: 'user', at: 'now' } },
    });
    assert(session.decision.status === 'approved', 'decision set (replace)');

    assertThrows(
      () => recordOnReview(dir, 'rev-record', { kind: 'decision', payload: { status: 'shipped' } }),
      'pending, blocked, approved',
      'an invalid decision.status is rejected',
    );
  } finally {
    fs.rmSync(dir, { recursive: true, force: true });
  }
});

await check('recordOnReview: rejects an unknown kind before touching the file', async () => {
  const dir = makeReviewRepo('bee-reviews-badkind-');
  try {
    await seedCappedCellWithEvidence(dir, 'ok-1');
    createReview(dir, baseScope({ id: 'rev-badkind' }));
    assertThrows(
      () => recordOnReview(dir, 'rev-badkind', { kind: 'sparkles', payload: {} }),
      'invalid kind',
      'unknown record kind rejected',
    );
  } finally {
    fs.rmSync(dir, { recursive: true, force: true });
  }
});

await check('reviews: strict read fails loud on a corrupt session (write verbs fail closed) — readReview/list stay fail-open', async () => {
  const dir = makeReviewRepo('bee-reviews-corrupt-');
  try {
    await seedCappedCellWithEvidence(dir, 'ok-1');
    createReview(dir, baseScope({ id: 'rev-good' }));
    fs.mkdirSync(reviewsDir(dir), { recursive: true });
    fs.writeFileSync(path.join(reviewsDir(dir), 'rev-corrupt.json'), 'not json', 'utf8');

    // write verb: readReviewStrict throws loud on the corrupt file
    assertThrows(
      () => recordOnReview(dir, 'rev-corrupt', { kind: 'decision', payload: { status: 'blocked' } }),
      'not valid json',
      'record refuses to mutate a present-but-corrupt session',
    );
    const stillCorrupt = fs.readFileSync(path.join(reviewsDir(dir), 'rev-corrupt.json'), 'utf8');
    assert(stillCorrupt === 'not json', 'corrupt file left untouched by the refused write');
    assertThrows(
      () => readReviewStrict(dir, 'rev-corrupt'),
      'not valid json',
      'readReviewStrict itself throws on corrupt JSON',
    );

    // read verbs: fail open, corrupt file skipped rather than crashing the sweep
    const list = listReviews(dir);
    assert(list.length === 1 && list[0].id === 'rev-good', 'listReviews skips the corrupt file and returns the good session');
    assert(readReview(dir, 'rev-corrupt') === null, 'readReview fails open to null on corrupt JSON');
  } finally {
    fs.rmSync(dir, { recursive: true, force: true });
  }
});

await check('recordOnReview: refuses a session id that does not exist (nothing to mutate)', async () => {
  const dir = makeReviewRepo('bee-reviews-noexist-');
  try {
    assertThrows(
      () => recordOnReview(dir, 'no-such-review', { kind: 'decision', payload: { status: 'blocked' } }),
      'not found',
      'record on a missing session id is refused',
    );
  } finally {
    fs.rmSync(dir, { recursive: true, force: true });
  }
});

await check('candidate ledger: addCandidate requires --mode from the closing feature\'s lane, appends exactly one JSONL line, never rewrites prior lines', async () => {
  const dir = makeReviewRepo('bee-reviews-candidates-');
  try {
    assertThrows(() => addCandidate(dir, { feature: 'demo', head: 'sha1', mode: '' }), 'mode', 'mode is required');
    assertThrows(() => addCandidate(dir, { feature: 'demo', head: 'sha1', mode: 'urgent' }), 'mode', 'mode must be a known lane');
    assert(!fs.existsSync(candidatesPath(dir)), 'no ledger file created by any rejected addCandidate call');

    const first = addCandidate(dir, { feature: 'demo', head: 'sha1', mode: 'standard', cells: ['c1', 'c2'] });
    assert(first.feature === 'demo' && first.head === 'sha1' && first.mode === 'standard', 'first entry carries feature/head/mode');
    const beforeSecondLine = fs.readFileSync(candidatesPath(dir), 'utf8');

    const second = addCandidate(dir, { feature: 'other', head: 'sha2', mode: 'tiny' });
    const lines = fs.readFileSync(candidatesPath(dir), 'utf8').split(/\r?\n/).filter(Boolean);
    assert(lines.length === 2, `ledger has exactly 2 lines, got ${lines.length}`);
    assert(lines[0] === beforeSecondLine.trim(), 'the first line is byte-unchanged after the second append — never rewritten');
    assert(JSON.parse(lines[1]).id === second.id, 'the second line is the new entry, appended after the first');

    const all = listCandidates(dir);
    assert(all.length === 2 && all[0].feature === 'demo' && all[1].feature === 'other', 'listCandidates returns both in append order');
    for (const mode of REVIEW_MODES) {
      assert(typeof mode === 'string' && mode.length > 0, 'REVIEW_MODES entries are non-empty strings');
    }
    assert(SCOPE_ENTRY_TYPES.includes('cell') && SCOPE_ENTRY_TYPES.includes('feature') && SCOPE_ENTRY_TYPES.includes('commit'), 'SCOPE_ENTRY_TYPES covers feature/cell/commit per SPEC §8');
  } finally {
    fs.rmSync(dir, { recursive: true, force: true });
  }
});

await check('candidate ledger: a corrupt line is skipped on read (fail-open), good lines still returned', async () => {
  const dir = makeReviewRepo('bee-reviews-candidates-corrupt-');
  try {
    addCandidate(dir, { feature: 'demo', head: 'sha1', mode: 'standard' });
    fs.appendFileSync(candidatesPath(dir), 'not json at all\n', 'utf8');
    addCandidate(dir, { feature: 'demo', head: 'sha2', mode: 'standard' });
    const all = listCandidates(dir);
    assert(all.length === 2, `corrupt line skipped, 2 good entries remain, got ${all.length}`);
  } finally {
    fs.rmSync(dir, { recursive: true, force: true });
  }
});

// ─── bee.mjs reviews CLI (thin wrapper contract) ─────────────────────────────

function beeReviewsModulePath() {
  return fileURLToPath(new URL('../bee.mjs', import.meta.url));
}

function runBeeReviews(cwd, args) {
  return runModuleWorker(beeReviewsModulePath(), { args: ['reviews', ...args], cwd });
}

function writeTempJson(dir, name, obj) {
  const file = path.join(dir, name);
  fs.writeFileSync(file, JSON.stringify(obj), 'utf8');
  return file;
}

await check('bee.mjs reviews create/show/list/record/candidate round-trip through the CLI, --file and --stdin both work', async () => {
  const dir = makeReviewRepo('bee-reviews-cli-');
  try {
    await seedCappedCellWithEvidence(dir, 'ok-1');

    const scopeFile = writeTempJson(dir, 'scope.json', baseScope({ id: 'rev-cli' }));
    const created = await runBeeReviews(dir, ['create', '--file', scopeFile, '--json']);
    assert(created.status === 0, `create should succeed, got ${created.status}: ${created.stderr}`);
    const session = JSON.parse(created.stdout);
    assert(session.id === 'rev-cli', 'created session id echoed back');

    const shown = await runBeeReviews(dir, ['show', '--id', 'rev-cli', '--json']);
    assert(shown.status === 0 && JSON.parse(shown.stdout).id === 'rev-cli', 'show returns the session');

    const listed = await runBeeReviews(dir, ['list']);
    assert(listed.status === 0 && /rev-cli/.test(listed.stdout), 'list mentions the session id');

    const recordFile = writeTempJson(dir, 'finding.json', { severity: 'P2', description: 'nit' });
    const recorded = await runBeeReviews(dir, ['record', '--id', 'rev-cli', '--kind', 'finding', '--file', recordFile]);
    assert(recorded.status === 0, `record should succeed, got ${recorded.status}: ${recorded.stderr}`);

    // --stdin path for create, on a second id
    const scope2 = JSON.stringify(baseScope({ id: 'rev-cli-2' }));
    const createdStdin = await runModuleWorker(beeReviewsModulePath(), {
      args: ['reviews', 'create', '--stdin', '--json'],
      cwd: dir,
      input: scope2,
    });
    assert(createdStdin.status === 0, `create --stdin should succeed, got ${createdStdin.status}: ${createdStdin.stderr}`);

    const candAdd = await runBeeReviews(dir, ['candidate', 'add', '--feature', 'demo', '--head', 'sha9', '--mode', 'standard', '--cells', 'ok-1']);
    assert(candAdd.status === 0, `candidate add should succeed, got ${candAdd.status}: ${candAdd.stderr}`);
    const cands = await runBeeReviews(dir, ['candidates', '--json']);
    assert(cands.status === 0, 'candidates list should succeed');
    const candList = JSON.parse(cands.stdout);
    assert(candList.length === 1 && candList[0].feature === 'demo' && candList[0].mode === 'standard', 'candidate ledger entry recorded via CLI');
  } finally {
    fs.rmSync(dir, { recursive: true, force: true });
  }
});

await check('bee.mjs reviews create exits non-zero and writes nothing when the preflight cannot resolve an included cell', async () => {
  const dir = makeReviewRepo('bee-reviews-cli-a10-');
  try {
    const scopeFile = writeTempJson(dir, 'scope.json', baseScope({ id: 'rev-cli-a10', included: [{ type: 'cell', id: 'no-such-cell' }] }));
    const result = await runBeeReviews(dir, ['create', '--file', scopeFile]);
    assert(result.status !== 0, 'an unresolvable included cell exits non-zero via the CLI');
    assert(/no such cell/.test(result.stderr), `error names the unresolvable cell, got ${result.stderr}`);
    assert(!fs.existsSync(path.join(dir, '.bee', 'reviews')), 'no session file written on a fail-closed CLI create');
  } finally {
    fs.rmSync(dir, { recursive: true, force: true });
  }
});

await check('bee.mjs reviews candidate add requires --mode and rejects an unrecognized mode, leaving the ledger untouched', async () => {
  const dir = makeStateRepo('bee-reviews-cli-mode-');
  try {
    const missing = await runBeeReviews(dir, ['candidate', 'add', '--feature', 'demo', '--head', 'sha1']);
    assert(missing.status !== 0, 'missing --mode is rejected');
    const bad = await runBeeReviews(dir, ['candidate', 'add', '--feature', 'demo', '--head', 'sha1', '--mode', 'urgent']);
    assert(bad.status !== 0, 'an unrecognized --mode is rejected');
    assert(!fs.existsSync(path.join(dir, '.bee', 'review-candidates.jsonl')), 'ledger file never created by a rejected candidate add');
  } finally {
    fs.rmSync(dir, { recursive: true, force: true });
  }
});

await check('bee.mjs reviews with no command prints a Use: line listing all verbs and exits non-zero', async () => {
  const dir = makeStateRepo('bee-reviews-cli-noverb-');
  try {
    const result = await runBeeReviews(dir, []);
    assert(result.status !== 0, 'no-command invocation exits non-zero');
    assert(/Use:/.test(result.stderr), `expected a "Use:" line, got stderr="${result.stderr}"`);
    assert(
      /create/.test(result.stderr) &&
        /list/.test(result.stderr) &&
        /show/.test(result.stderr) &&
        /record/.test(result.stderr) &&
        /candidate add/.test(result.stderr) &&
        /candidates/.test(result.stderr) &&
        /status/.test(result.stderr),
      `Use: line should list all verbs, got ${result.stderr}`,
    );
  } finally {
    fs.rmSync(dir, { recursive: true, force: true });
  }
});

// ─── reviews: derived coverage/staleness engine (review-od-2, SPEC §5/§8/ ───
// R6/R10, A7/A8, decision 565e68d0) ─────────────────────────────────────────
// Status is NEVER stored — deriveCandidateStatus always recomputes from
// session records + git at read time. Fixtures below layer a real git
// repo on top of makeReviewRepo's bee scaffolding since coverage/staleness
// is defined over actual commit ancestry (git rev-list / merge-base
// --is-ancestor), mirroring test_onboard_bee.mjs:872-875's runGit helper —
// its env isolation there is HOME-override; the git-unavailable case below
// is a NEW variation (PATH-strip), proven in .bee/spikes/review-on-demand/RESULT.md.

function runGit(cwd, args, { requireOutput = false } = {}) {
  const result = spawnSync('git', args, { cwd, encoding: 'utf8' });
  assert(result.status !== null, `git ${args.join(' ')} returned no concrete status: ${result.error ?? ''}`);
  assert(result.status === 0, `git ${args.join(' ')} exited ${result.status}: ${result.stderr ?? ''}`);
  if (requireOutput) {
    assert(typeof result.stdout === 'string' && result.stdout.trim().length > 0, `git ${args.join(' ')} returned no output`);
  }
  return result;
}
const gitVersion = spawnSync('git', ['--version'], { encoding: 'utf8' });
const gitAvailable =
  gitVersion.status === 0 && typeof gitVersion.stdout === 'string' && gitVersion.stdout.trim().length > 0;

function makeReviewGitRepo(prefix) {
  const dir = makeReviewRepo(prefix);
  runGit(dir, ['init', '-q']);
  runGit(dir, ['config', 'user.email', 'bee-review-od-2@example.com']);
  runGit(dir, ['config', 'user.name', 'bee review-od-2 tests']);
  fs.writeFileSync(path.join(dir, 'seed.txt'), 'seed\n', 'utf8');
  runGit(dir, ['add', 'seed.txt']);
  runGit(dir, ['commit', '-q', '-m', 'seed']);
  return dir;
}

function gitHead(dir) {
  return runGit(dir, ['rev-parse', 'HEAD'], { requireOutput: true }).stdout.trim();
}

function gitCommit(dir, file, content, message) {
  fs.writeFileSync(path.join(dir, file), content, 'utf8');
  runGit(dir, ['add', file]);
  runGit(dir, ['commit', '-q', '-m', message]);
  return gitHead(dir);
}

await check('deriveCandidateStatus: a legacy candidate with no covering session derives "unreviewed" — no fake session records fabricated (SPEC §11.3)', async () => {
  const dir = makeReviewRepo('bee-cand-legacy-');
  try {
    const candidate = addCandidate(dir, { feature: 'legacy-feature', head: 'sha-legacy', mode: 'standard' });
    const derived = deriveCandidateStatus(dir, candidate);
    assert(derived.status === 'unreviewed', `legacy candidate with no session derives unreviewed, got ${derived.status}`);
    assert(derived.session === undefined, 'unreviewed carries no session reference');
  } finally {
    fs.rmSync(dir, { recursive: true, force: true });
  }
});

await check('deriveCandidateStatus: a non-approved (pending) session whose scope includes the candidate\'s feature derives "in review"', async () => {
  const dir = makeReviewRepo('bee-cand-inreview-');
  try {
    await seedCappedCellWithEvidence(dir, 'ok-1');
    createReview(dir, baseScope({
      id: 'rev-open',
      included: [{ type: 'feature', id: 'demo' }],
      baseline: 'sha0',
      head: 'sha1',
    }));
    const candidate = addCandidate(dir, { feature: 'demo', head: 'sha1', mode: 'standard' });
    const derived = deriveCandidateStatus(dir, candidate);
    assert(derived.status === 'in review', `open covering session derives in review, got ${derived.status}`);
    assert(derived.session === 'rev-open', 'in review carries the covering session id');
  } finally {
    fs.rmSync(dir, { recursive: true, force: true });
  }
});

await check('deriveCandidateStatus: a blocked (P1-pending) session still derives "in review", never "reviewed" (R8 — P1 blocks approval)', async () => {
  const dir = makeReviewRepo('bee-cand-blocked-');
  try {
    await seedCappedCellWithEvidence(dir, 'ok-1');
    createReview(dir, baseScope({
      id: 'rev-blocked',
      included: [{ type: 'feature', id: 'demo' }],
      baseline: 'sha0',
      head: 'sha1',
    }));
    recordOnReview(dir, 'rev-blocked', { kind: 'decision', payload: { status: 'blocked', gate4: null } });
    const candidate = addCandidate(dir, { feature: 'demo', head: 'sha1', mode: 'standard' });
    const derived = deriveCandidateStatus(dir, candidate);
    assert(derived.status === 'in review', `blocked session derives in review, got ${derived.status}`);
  } finally {
    fs.rmSync(dir, { recursive: true, force: true });
  }
});

if (gitAvailable) {
  await check('deriveCandidateStatus: an approved session covers the candidate\'s exact head as "reviewed"; one extra commit after that head flips the SAME candidate to "review stale" while the session file stays byte-unchanged (A8)', async () => {
    const dir = makeReviewGitRepo('bee-cand-stale-flip-');
    try {
      const sha1 = gitHead(dir);
      await seedCappedCellWithEvidence(dir, 'ok-1');
      createReview(dir, baseScope({
        id: 'rev-reviewed',
        included: [{ type: 'feature', id: 'demo' }],
        baseline: sha1,
        head: sha1,
      }));
      recordOnReview(dir, 'rev-reviewed', { kind: 'decision', payload: { status: 'approved', gate4: { approved_by: 'user', at: 'now' } } });
      const candidate = addCandidate(dir, { feature: 'demo', head: sha1, mode: 'standard' });

      const reviewed = deriveCandidateStatus(dir, candidate);
      assert(reviewed.status === 'reviewed', `exact-head coverage derives reviewed, got ${reviewed.status}`);
      assert(reviewed.session === 'rev-reviewed', 'reviewed carries the covering session id');

      const sessionFile = path.join(reviewsDir(dir), 'rev-reviewed.json');
      const before = fs.readFileSync(sessionFile, 'utf8');
      gitCommit(dir, 'unrelated.txt', 'unrelated change\n', 'unrelated commit after review head');
      const after = fs.readFileSync(sessionFile, 'utf8');
      assert(before === after, 'session file stays byte-unchanged across the new commit — audit trail preserved (A8)');

      const stale = deriveCandidateStatus(dir, candidate);
      assert(stale.status === 'review stale', `a commit after the covering session's head flips status to review stale, got ${stale.status}`);
      assert(stale.session === 'rev-reviewed', 'review stale still names the covering session');
      assert(!stale.note, 'a resolvable stale range carries no "range unresolvable" note');
    } finally {
      fs.rmSync(dir, { recursive: true, force: true });
    }
  });

  await check('deriveCandidateStatus: an unresolvable candidate head (unknown sha, simulating rebase/amend) with a covering approved session degrades to "review stale" with a "range unresolvable" note, never throws (plan open question 1)', async () => {
    const dir = makeReviewGitRepo('bee-cand-unresolvable-');
    try {
      const sha1 = gitHead(dir);
      await seedCappedCellWithEvidence(dir, 'ok-1');
      createReview(dir, baseScope({
        id: 'rev-unresolvable',
        included: [{ type: 'feature', id: 'demo' }],
        baseline: sha1,
        head: sha1,
      }));
      recordOnReview(dir, 'rev-unresolvable', { kind: 'decision', payload: { status: 'approved', gate4: { approved_by: 'user', at: 'now' } } });
      const fakeSha = 'a'.repeat(40);
      const candidate = addCandidate(dir, { feature: 'demo', head: fakeSha, mode: 'standard' });

      const derived = deriveCandidateStatus(dir, candidate);
      assert(derived.status === 'review stale', `unresolvable candidate head degrades to review stale, got ${derived.status}`);
      assert(derived.note === 'range unresolvable', `unresolvable range carries the "range unresolvable" note, got ${JSON.stringify(derived.note)}`);
      assert(derived.session === 'rev-unresolvable', 'degraded status still names the covering session');
    } finally {
      fs.rmSync(dir, { recursive: true, force: true });
    }
  });

  await check('deriveCandidateStatus: git binary unavailable (PATH stripped) never throws — a covering session degrades to "review stale"/"range unresolvable", read path stays usable', async () => {
    const dir = makeReviewGitRepo('bee-cand-nogit-');
    try {
      const sha1 = gitHead(dir);
      await seedCappedCellWithEvidence(dir, 'ok-1');
      createReview(dir, baseScope({
        id: 'rev-nogit',
        included: [{ type: 'feature', id: 'demo' }],
        baseline: sha1,
        head: sha1,
      }));
      recordOnReview(dir, 'rev-nogit', { kind: 'decision', payload: { status: 'approved', gate4: { approved_by: 'user', at: 'now' } } });
      const candidate = addCandidate(dir, { feature: 'demo', head: sha1, mode: 'standard' });

      const savedPath = process.env.PATH;
      let derived;
      let threw = null;
      try {
        process.env.PATH = '/nonexistent';
        try {
          derived = deriveCandidateStatus(dir, candidate);
        } catch (err) {
          threw = err;
        }
      } finally {
        process.env.PATH = savedPath;
      }
      assert(!threw, `deriveCandidateStatus must never throw on a missing git binary, threw: ${threw && threw.message}`);
      assert(derived.status === 'review stale', `git-unavailable degrades to review stale, got ${derived.status}`);
      assert(derived.note === 'range unresolvable', 'git-unavailable carries the range-unresolvable note');
    } finally {
      fs.rmSync(dir, { recursive: true, force: true });
    }
  });

  await check('deriveCandidateStatus: a candidate whose head postdates the covering approved session\'s frozen head (new work, same feature, no new session) derives "unreviewed" — not a stale re-labelling of unrelated new work', async () => {
    const dir = makeReviewGitRepo('bee-cand-newdelta-');
    try {
      const sha1 = gitHead(dir);
      await seedCappedCellWithEvidence(dir, 'ok-1');
      createReview(dir, baseScope({
        id: 'rev-old',
        included: [{ type: 'feature', id: 'demo' }],
        baseline: sha1,
        head: sha1,
      }));
      recordOnReview(dir, 'rev-old', { kind: 'decision', payload: { status: 'approved', gate4: { approved_by: 'user', at: 'now' } } });
      const sha2 = gitCommit(dir, 'more.txt', 'more work\n', 'new delta commit after review head');
      const newCandidate = addCandidate(dir, { feature: 'demo', head: sha2, mode: 'standard' });
      const derived = deriveCandidateStatus(dir, newCandidate);
      assert(derived.status === 'unreviewed', `new delta candidate not an ancestor of the old session's head derives unreviewed, got ${derived.status}`);
    } finally {
      fs.rmSync(dir, { recursive: true, force: true });
    }
  });
  await check('deriveCandidateStatus: a pass-local gitMemo dedupes repeated git invocations when multiple candidates share a covering session\'s (head,ref)/(ref) pair (D2) — derived statuses stay byte-identical to the unmemoized path (cp-2)', async () => {
    const dir = makeReviewGitRepo('bee-cand-memo-');
    try {
      const sha0 = gitHead(dir); // seed commit — becomes both candidates' head
      await seedCappedCellWithEvidence(dir, 'ok-1');
      const sha1 = gitCommit(dir, 'v1.txt', 'v1\n', 'commit1 (session\'s frozen head)');
      gitCommit(dir, 'v2.txt', 'v2\n', 'commit2 (advances HEAD past the session head)');
      createReview(dir, baseScope({
        id: 'rev-memo',
        included: [{ type: 'feature', id: 'demo' }],
        baseline: sha0,
        head: sha1,
      }));
      recordOnReview(dir, 'rev-memo', { kind: 'decision', payload: { status: 'approved', gate4: { approved_by: 'user', at: 'now' } } });

      const candidateA = addCandidate(dir, { feature: 'demo', head: sha0, mode: 'standard' });
      const candidateB = addCandidate(dir, { feature: 'demo', head: sha0, mode: 'standard' });

      // Baseline (unmemoized, no opts.gitMemo/opts.runGit at all) — frozen expectation.
      const baselineA = deriveCandidateStatus(dir, candidateA);
      const baselineB = deriveCandidateStatus(dir, candidateB);
      assert(baselineA.status === 'review stale', `sanity: candidate head predates the session head and HEAD has since advanced -> review stale, got ${baselineA.status}`);
      assert(baselineB.status === 'review stale', `sanity: same fixture, second candidate -> review stale, got ${baselineB.status}`);

      const calls = [];
      const countingRunner = (cwd, args) => {
        calls.push(args.join(' '));
        return spawnSync('git', args, { cwd, encoding: 'utf8' });
      };
      const gitMemo = new Map();
      const sessions = listReviews(dir);
      const memoA = deriveCandidateStatus(dir, candidateA, { sessions, gitMemo, runGit: countingRunner });
      const memoB = deriveCandidateStatus(dir, candidateB, { sessions, gitMemo, runGit: countingRunner });

      assert(memoA.status === baselineA.status && memoA.session === baselineA.session, `memoized derivation must match the unmemoized baseline for candidate A, got ${JSON.stringify(memoA)} vs ${JSON.stringify(baselineA)}`);
      assert(memoB.status === baselineB.status && memoB.session === baselineB.session, `memoized derivation must match the unmemoized baseline for candidate B, got ${JSON.stringify(memoB)} vs ${JSON.stringify(baselineB)}`);

      const uniqueKeys = new Set(calls);
      assert(calls.length === uniqueKeys.size, `each unique git invocation key must run at most once per pass (shared gitMemo across candidates A and B) — saw ${calls.length} calls for ${uniqueKeys.size} unique keys: ${JSON.stringify(calls)}`);
      assert(calls.some((c) => c.startsWith('merge-base')), `expected a merge-base invocation to have been recorded via the injected runner, saw: ${JSON.stringify(calls)}`);
      assert(calls.some((c) => c.startsWith('rev-list')), `expected a rev-list invocation to have been recorded via the injected runner, saw: ${JSON.stringify(calls)}`);
      assert(calls.length === 2, `two candidates sharing one covering session's (head,ref)/(ref) pair must produce exactly one merge-base + one rev-list call total (not four) via a pass-local memo, saw ${calls.length}: ${JSON.stringify(calls)}`);
    } finally {
      fs.rmSync(dir, { recursive: true, force: true });
    }
  });

  await check('deriveCandidateStatus: unresolvable-ancestry degradation ("review stale" / "range unresolvable") is unchanged with a gitMemo threaded through opts, and the unresolvable git invocation is memoized across candidates sharing the same unresolvable head (cp-2)', async () => {
    const dir = makeReviewGitRepo('bee-cand-memo-unresolvable-');
    try {
      const sha1 = gitHead(dir);
      await seedCappedCellWithEvidence(dir, 'ok-1');
      createReview(dir, baseScope({
        id: 'rev-memo-unresolvable',
        included: [{ type: 'feature', id: 'demo' }],
        baseline: sha1,
        head: sha1,
      }));
      recordOnReview(dir, 'rev-memo-unresolvable', { kind: 'decision', payload: { status: 'approved', gate4: { approved_by: 'user', at: 'now' } } });
      const fakeSha = 'b'.repeat(40);
      const candidateA = addCandidate(dir, { feature: 'demo', head: fakeSha, mode: 'standard' });
      const candidateB = addCandidate(dir, { feature: 'demo', head: fakeSha, mode: 'standard' });

      const baseline = deriveCandidateStatus(dir, candidateA);
      assert(baseline.status === 'review stale' && baseline.note === 'range unresolvable', `sanity baseline must match the existing unresolvable-ancestry fixture behavior, got ${JSON.stringify(baseline)}`);

      const calls = [];
      const countingRunner = (cwd, args) => {
        calls.push(args.join(' '));
        return spawnSync('git', args, { cwd, encoding: 'utf8' });
      };
      const gitMemo = new Map();
      const sessions = listReviews(dir);
      const derivedA = deriveCandidateStatus(dir, candidateA, { sessions, gitMemo, runGit: countingRunner });
      const derivedB = deriveCandidateStatus(dir, candidateB, { sessions, gitMemo, runGit: countingRunner });

      assert(derivedA.status === baseline.status && derivedA.note === baseline.note, `degraded status must be unchanged for candidate A, got ${JSON.stringify(derivedA)}`);
      assert(derivedB.status === baseline.status && derivedB.note === baseline.note, `degraded status must be unchanged for candidate B, got ${JSON.stringify(derivedB)}`);
      assert(derivedA.session === 'rev-memo-unresolvable' && derivedB.session === 'rev-memo-unresolvable', 'degraded status still names the covering session for both candidates');

      const uniqueKeys = new Set(calls);
      assert(calls.length === uniqueKeys.size, `unresolvable-key git invocation must also be memoized across candidates sharing the same unresolvable head, saw ${calls.length} calls for ${uniqueKeys.size} unique keys: ${JSON.stringify(calls)}`);
      assert(calls.length === 1, `two candidates sharing the same unresolvable (head,ref) pair must invoke git exactly once total via the pass-local memo, saw ${calls.length}: ${JSON.stringify(calls)}`);
    } finally {
      fs.rmSync(dir, { recursive: true, force: true });
    }
  });
} else {
  console.log('SKIP  deriveCandidateStatus git-fixture tests (git binary not available in this environment)');
}

await check('deriveCandidateStatus: CANDIDATE_STATUSES exports exactly the four R10 labels', async () => {
  assert(
    JSON.stringify(CANDIDATE_STATUSES) === JSON.stringify(['unreviewed', 'in review', 'reviewed', 'review stale']),
    `CANDIDATE_STATUSES must be the four SPEC §5/R10 labels in order, got ${JSON.stringify(CANDIDATE_STATUSES)}`,
  );
});

if (gitAvailable) {
  await check('bee.mjs reviews status: --json renders verified + four-label counts and per-candidate coverage, "reviewed (covered by <id>)" answers A7', async () => {
    const dir = makeReviewGitRepo('bee-reviews-status-cli-');
    try {
      const sha1 = gitHead(dir);
      await seedCappedCellWithEvidence(dir, 'ok-1');
      createReview(dir, baseScope({
        id: 'rev-status',
        included: [{ type: 'feature', id: 'demo' }],
        baseline: sha1,
        head: sha1,
      }));
      recordOnReview(dir, 'rev-status', { kind: 'decision', payload: { status: 'approved', gate4: { approved_by: 'user', at: 'now' } } });
      addCandidate(dir, { feature: 'demo', head: sha1, mode: 'standard' }); // reviewed (exact head, zero commits since)
      addCandidate(dir, { feature: 'other', head: 'sha9', mode: 'tiny' }); // unreviewed (no covering session)

      const result = await runBeeReviews(dir, ['status', '--json']);
      assert(result.status === 0, `status --json should succeed, got ${result.status}: ${result.stderr}`);
      const summary = JSON.parse(result.stdout);
      assert(summary.counts.verified === 2, `verified counts every candidate, got ${summary.counts.verified}`);
      assert(summary.counts.reviewed === 1, `one candidate reviewed, got ${summary.counts.reviewed}`);
      assert(summary.counts.unreviewed === 1, `one candidate unreviewed, got ${summary.counts.unreviewed}`);
      assert(summary.counts['in review'] === 0 && summary.counts['review stale'] === 0, 'no in-review or stale candidates in this fixture');
      const demoRow = summary.candidates.find((c) => c.feature === 'demo');
      assert(demoRow.review_status === 'reviewed' && demoRow.review_session === 'rev-status', 'demo candidate row carries the derived status + covering session');

      const text = await runBeeReviews(dir, ['status']);
      assert(text.status === 0, 'status text mode succeeds');
      assert(/reviewed \(covered by rev-status\)/.test(text.stdout), `A7 answer surface names the covering review id, got ${text.stdout}`);
      assert(/unreviewed/.test(text.stdout), 'unreviewed candidate rendered in text output');
    } finally {
      fs.rmSync(dir, { recursive: true, force: true });
    }
  });
} else {
  console.log('SKIP  bee.mjs reviews status A7 covered-by test (git binary not available in this environment)');
}

await check('bee.mjs reviews status: --feature filters the candidate set, and a repo with zero candidates still renders all-zero counts at exit 0', async () => {
  const dir = makeReviewRepo('bee-reviews-status-filter-');
  try {
    const empty = await runBeeReviews(dir, ['status', '--json']);
    assert(empty.status === 0, `status on an empty ledger still exits 0, got ${empty.status}: ${empty.stderr}`);
    const emptySummary = JSON.parse(empty.stdout);
    assert(emptySummary.counts.verified === 0 && emptySummary.candidates.length === 0, 'zero candidates renders all-zero counts, no crash');

    addCandidate(dir, { feature: 'feature-a', head: 'shaA', mode: 'standard' });
    addCandidate(dir, { feature: 'feature-b', head: 'shaB', mode: 'standard' });

    const filtered = await runBeeReviews(dir, ['status', '--feature', 'feature-a', '--json']);
    assert(filtered.status === 0, 'filtered status succeeds');
    const filteredSummary = JSON.parse(filtered.stdout);
    assert(filteredSummary.counts.verified === 1, `--feature filter narrows to one candidate, got ${filteredSummary.counts.verified}`);
    assert(filteredSummary.candidates.length === 1 && filteredSummary.candidates[0].feature === 'feature-a', 'only feature-a candidate present after filter');
  } finally {
    fs.rmSync(dir, { recursive: true, force: true });
  }
});

// ─── bee.mjs status review integration (review-od-3, SPEC R3/R7/R10/§9/§11.5,
// decision 565e68d0) ─────────────────────────────────────────────────────────
// The POST_REVIEW_PHASES staleness warning ("past reviewing but gate review
// still pending") is RETIRED — reaching scribing/compounding/compounding-
// complete without Gate 4 is the normal truthful close under review-on-demand
// (R3), not drift. In its place: a `review` block in --json (candidate counts
// sourced from lib/reviews.mjs's own derivation, no second implementation
// here), an informational §9 completion line in text render, a prominent R7
// high-risk warning line, and a candidate-aware recommended_next that never
// names bee-reviewing as an automatic next step (§11.5).

function beeStatusModulePath() {
  return fileURLToPath(new URL('../bee.mjs', import.meta.url));
}

function runBeeStatus(cwd, args) {
  return runModuleWorker(beeStatusModulePath(), { args: ['status', ...args], cwd });
}

if (gitAvailable) {
  await check('bee.mjs status --json review block distinguishes all four candidate statuses (unreviewed/in_review/reviewed/stale), lists open sessions, and flags a high-risk unreviewed candidate (R7/R10)', async () => {
    const dir = makeReviewGitRepo('bee-status-review-counts-');
    try {
      const sha1 = gitHead(dir);

      // reviewed-then-stale: session covers feature "demo-old" at sha1,
      // approved while sha1 is still the real HEAD (reviewed); a later
      // unrelated commit advances HEAD past sha1, flipping the SAME
      // candidate to stale without touching the session file (A8 mechanics).
      createReview(dir, baseScope({ id: 'rev-old', included: [{ type: 'feature', id: 'demo-old' }], baseline: sha1, head: sha1 }));
      recordOnReview(dir, 'rev-old', { kind: 'decision', payload: { status: 'approved', gate4: { approved_by: 'user', at: 'now' } } });
      addCandidate(dir, { feature: 'demo-old', head: sha1, mode: 'standard' });

      const sha2 = gitCommit(dir, 'unrelated.txt', 'unrelated\n', 'advance head past rev-old');

      // reviewed: a fresh session approved exactly at the current HEAD.
      createReview(dir, baseScope({ id: 'rev-new', included: [{ type: 'feature', id: 'demo-new' }], baseline: sha2, head: sha2 }));
      recordOnReview(dir, 'rev-new', { kind: 'decision', payload: { status: 'approved', gate4: { approved_by: 'user', at: 'now' } } });
      addCandidate(dir, { feature: 'demo-new', head: sha2, mode: 'standard' });

      // in review: a pending (never approved) covering session.
      createReview(dir, baseScope({ id: 'rev-open', included: [{ type: 'feature', id: 'demo-pending' }], baseline: sha2, head: sha2 }));
      addCandidate(dir, { feature: 'demo-pending', head: sha2, mode: 'standard' });

      // unreviewed: no covering session at all.
      addCandidate(dir, { feature: 'no-session', head: sha2, mode: 'standard' });

      // unreviewed + high-risk: no covering session, mode high-risk (R7).
      addCandidate(dir, { feature: 'demo-risk', head: sha2, mode: 'high-risk' });

      const result = await runBeeStatus(dir, ['--json']);
      assert(result.status === 0, `bee_status --json exited ${result.status} :: ${result.stderr}`);
      const payload = JSON.parse(result.stdout);
      assert(payload.review, 'status JSON carries a "review" block');
      const c = payload.review.candidates;
      assert(c.total === 5, `total counts every candidate, got ${c.total}`);
      assert(c.unreviewed === 2, `two unreviewed candidates (no-session + demo-risk), got ${c.unreviewed}`);
      assert(c.in_review === 1, `one in-review candidate, got ${c.in_review}`);
      assert(c.reviewed === 1, `one reviewed candidate, got ${c.reviewed}`);
      assert(c.stale === 1, `one stale candidate, got ${c.stale}`);
      assert(
        payload.review.open_sessions.includes('rev-open'),
        `open_sessions lists the pending session, got ${JSON.stringify(payload.review.open_sessions)}`,
      );
      assert(
        !payload.review.open_sessions.includes('rev-old') && !payload.review.open_sessions.includes('rev-new'),
        'approved sessions are never listed as open',
      );
      assert(payload.review.high_risk_unreviewed === 1, `one high-risk unreviewed candidate, got ${payload.review.high_risk_unreviewed}`);

      const text = await runBeeStatus(dir, []);
      assert(text.status === 0, 'text-mode status also exits 0');
      assert(
        /High-risk unreviewed: 1 high-risk candidate/.test(text.stdout),
        `text render carries the prominent R7 high-risk warning line, got:\n${text.stdout}`,
      );
    } finally {
      fs.rmSync(dir, { recursive: true, force: true });
    }
  });
} else {
  console.log('SKIP  bee.mjs status review candidate-count test (git binary not available in this environment)');
}

await check('bee.mjs status: a compounding-complete state with gate "review" pending produces NO staleness warning (R3 — the retired Gate-4-pending warning never fires); the §9 completion line renders in text instead, naming the unreviewed count', async () => {
  const dir = makeReviewRepo('bee-status-post-review-close-');
  try {
    writeState(dir, {
      ...defaultState(),
      phase: 'compounding-complete',
      feature: 'demo',
      approved_gates: { context: true, shape: true, execution: true, review: false },
    });
    addCandidate(dir, { feature: 'demo', head: 'sha-close', mode: 'standard' }); // unreviewed: no session at all

    const result = await runBeeStatus(dir, ['--json']);
    assert(result.status === 0, `bee_status --json exited ${result.status} :: ${result.stderr}`);
    const payload = JSON.parse(result.stdout);

    // Dry-run the negative regex against this fixture's own JSON text first
    // (critical pattern 20260712) — proves the assertion below is a real
    // negative, not an accidental match on unrelated fixture content.
    const fixtureText = JSON.stringify(payload);
    const retiredWarningPattern = /past reviewing but gate/;
    assert(!retiredWarningPattern.test(fixtureText), 'sanity: the fixture itself does not coincidentally contain the retired warning phrase');

    assert(
      !payload.staleness_warnings.some((w) => retiredWarningPattern.test(w)),
      `the retired Gate-4-pending warning must never fire again, got staleness_warnings=${JSON.stringify(payload.staleness_warnings)}`,
    );
    assert(payload.review.candidates.unreviewed === 1, `one unreviewed candidate in this fixture, got ${payload.review.candidates.unreviewed}`);

    const text = await runBeeStatus(dir, []);
    assert(text.status === 0, 'text-mode status exits 0');
    assert(
      /Completed and verified; independent review not requested; 1 candidate\(s\) awaiting review\./.test(text.stdout),
      `text render carries the exact §9 completion line, got:\n${text.stdout}`,
    );
    assert(!/past reviewing but gate/.test(text.stdout), 'text render never carries the retired warning phrase either');
  } finally {
    fs.rmSync(dir, { recursive: true, force: true });
  }
});

await check('bee.mjs status: the §9 completion line only renders in a post-execution phase — it stays silent mid-swarm even with unreviewed candidates present, and stays silent post-execution with zero candidates', async () => {
  const dir = makeReviewRepo('bee-status-post-review-silent-');
  try {
    // swarming (not post-execution) + an unreviewed candidate -> no line.
    addCandidate(dir, { feature: 'demo', head: 'sha-mid', mode: 'standard' });
    const midSwarm = await runBeeStatus(dir, []);
    assert(midSwarm.status === 0, 'mid-swarm status exits 0');
    assert(!/Completed and verified; independent review not requested/.test(midSwarm.stdout), 'the §9 line never renders outside a post-execution phase');

    // compounding-complete + zero candidates -> no line either.
    const emptyDir = makeReviewRepo('bee-status-post-review-silent-empty-');
    try {
      writeState(emptyDir, {
        ...defaultState(),
        phase: 'compounding-complete',
        feature: 'demo',
        approved_gates: { context: true, shape: true, execution: true, review: false },
      });
      const noCandidates = await runBeeStatus(emptyDir, []);
      assert(noCandidates.status === 0, 'compounding-complete with zero candidates exits 0');
      assert(!/Completed and verified; independent review not requested/.test(noCandidates.stdout), 'the §9 line never renders when there are zero unreviewed candidates');
    } finally {
      fs.rmSync(emptyDir, { recursive: true, force: true });
    }
  } finally {
    fs.rmSync(dir, { recursive: true, force: true });
  }
});

await check('bee.mjs status: the unknown-phase warning (decision 0004) still fires unchanged after the review-block wiring', async () => {
  const dir = makeReviewRepo('bee-status-unknown-phase-');
  try {
    writeState(dir, {
      ...defaultState(),
      phase: 'totally-invented-phase',
      feature: 'demo',
      approved_gates: { context: true, shape: true, execution: true, review: false },
    });
    const result = await runBeeStatus(dir, ['--json']);
    assert(result.status === 0, `bee_status --json exited ${result.status} :: ${result.stderr}`);
    const payload = JSON.parse(result.stdout);
    assert(
      payload.staleness_warnings.some((w) => /Unknown phase "totally-invented-phase"/.test(w)),
      `the decision-0004 unknown-phase warning must still fire, got ${JSON.stringify(payload.staleness_warnings)}`,
    );
    assert(payload.review && payload.review.candidates, 'the review block is still present alongside the unknown-phase warning (never crashes)');
  } finally {
    fs.rmSync(dir, { recursive: true, force: true });
  }
});

await check('bee.mjs status: recommended_next after compounding-complete with unreviewed candidates reports the candidate count and never names "Invoke bee-reviewing" as the automatic next step (§11.5), even overriding a stale state.next_action that did', async () => {
  const dir = makeReviewRepo('bee-status-recommended-next-');
  try {
    writeState(dir, {
      ...defaultState(),
      phase: 'compounding-complete',
      feature: 'demo',
      approved_gates: { context: true, shape: true, execution: true, review: false },
      next_action: 'Invoke bee-reviewing for independent review.',
    });
    addCandidate(dir, { feature: 'demo', head: 'sha-next', mode: 'standard' });

    const result = await runBeeStatus(dir, ['--json']);
    assert(result.status === 0, `bee_status --json exited ${result.status} :: ${result.stderr}`);
    const payload = JSON.parse(result.stdout);
    assert(!/Invoke bee-reviewing/.test(payload.recommended_next), `recommended_next must never propose bee-reviewing automatically, got "${payload.recommended_next}"`);
    assert(/candidate/i.test(payload.recommended_next), `recommended_next mentions review candidates, got "${payload.recommended_next}"`);
    assert(/1/.test(payload.recommended_next), `recommended_next carries the unreviewed count, got "${payload.recommended_next}"`);
  } finally {
    fs.rmSync(dir, { recursive: true, force: true });
  }
});

await check('bee.mjs status: a high-risk unreviewed candidate renders the prominent R7 warning line, and a repo with only non-high-risk candidates renders no such line', async () => {
  const dir = makeReviewRepo('bee-status-high-risk-');
  try {
    addCandidate(dir, { feature: 'demo', head: 'sha-risk', mode: 'high-risk' });
    const withRisk = await runBeeStatus(dir, ['--json']);
    assert(withRisk.status === 0, `bee_status --json exited ${withRisk.status} :: ${withRisk.stderr}`);
    const riskPayload = JSON.parse(withRisk.stdout);
    assert(riskPayload.review.high_risk_unreviewed === 1, `high_risk_unreviewed counts the candidate, got ${riskPayload.review.high_risk_unreviewed}`);

    const riskText = await runBeeStatus(dir, []);
    assert(
      /High-risk unreviewed: 1 high-risk candidate\(s\) have not passed independent review — bee will not auto-dispatch reviewers/.test(riskText.stdout),
      `text render carries the exact prominent R7 warning line, got:\n${riskText.stdout}`,
    );
  } finally {
    fs.rmSync(dir, { recursive: true, force: true });
  }
});

await check('bee.mjs status: a standard-mode candidate never triggers the R7 high-risk warning line, even when unreviewed', async () => {
  const dir = makeReviewRepo('bee-status-no-high-risk-');
  try {
    addCandidate(dir, { feature: 'demo', head: 'sha-std', mode: 'standard' });
    const result = await runBeeStatus(dir, []);
    assert(result.status === 0, `bee_status exited ${result.status} :: ${result.stderr}`);
    assert(!/High-risk unreviewed/.test(result.stdout), 'no high-risk warning line for a non-high-risk unreviewed candidate');
  } finally {
    fs.rmSync(dir, { recursive: true, force: true });
  }
});

await check('bee.mjs status: a corrupt .bee/reviews entry and an unreadable candidates ledger degrade the review block but leave bee_status exiting 0 (fail-open read path, never a hard dependency)', async () => {
  const dir = makeReviewRepo('bee-status-corrupt-reviews-');
  try {
    fs.mkdirSync(reviewsDir(dir), { recursive: true });
    fs.writeFileSync(path.join(reviewsDir(dir), 'broken.json'), '{ not valid json', 'utf8');
    // A directory in place of the append-only ledger file: readFileSync on a
    // directory throws EISDIR — the read path must still degrade, not crash.
    fs.mkdirSync(candidatesPath(dir), { recursive: true });

    const result = await runBeeStatus(dir, ['--json']);
    assert(result.status === 0, `bee_status --json must exit 0 on a corrupt reviews store, got ${result.status} :: ${result.stderr}`);
    const payload = JSON.parse(result.stdout);
    assert(payload.review && payload.review.candidates, 'the review block is still present (degraded, not absent) on a corrupt store');
    assert(payload.review.candidates.total === 0, `degraded review block reports zero candidates rather than throwing, got ${payload.review.candidates.total}`);

    const text = await runBeeStatus(dir, []);
    assert(text.status === 0, `bee_status text mode must also exit 0 on a corrupt reviews store, got ${text.status} :: ${text.stderr}`);
  } finally {
    fs.rmSync(dir, { recursive: true, force: true });
  }
});

printSummaryAndExit();
