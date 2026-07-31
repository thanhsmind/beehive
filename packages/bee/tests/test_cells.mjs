#!/usr/bin/env node
// test_cells.mjs — cells.mjs contract tests (add/update/claim/cap/budgets/judge/
// claim-next + cells-archive/hardening-1 satellites), split out of test_lib.mjs
// (cs-2a) to shrink the monolith. Same PASS/FAIL/exit-1 contract as every other
// suite here — see scripts/lib/test-fixture.mjs.

import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { runModuleWorker } from '../../../scripts/lib/run-module-worker.mjs';
import {
  makeTempRepo,
  makeCell,
  check,
  assert,
  assertThrows,
  assertRejects,
  printSummaryAndExit,
} from '../../../scripts/lib/test-fixture.mjs';
import {
  readState,
  writeState,
  isKnownPhase,
  bypassLevel,
} from '../lib/state.mjs';
import {
  addCell,
  addCells,
  previewAddCells,
  updateCell,
  readCell,
  listCells,
  cellsArchiveDir,
  resolveCellFile,
  archiveFeature,
  unarchiveFeature,
  archivedSummary,
  readyCells,
  claimCell,
  capCell,
  recordTestsRedAttempt,
  blockCell,
  dropCell,
  unclaimCell,
  reopenCell,
  setTier,
  resetCellBudget,
  recordJudgeVerdict,
  writeCell,
  claimNextCell,
  claimCellCrossSession,
  normalizeFailureSignature,
  resolveCellBudgets,
  checkCellBudgets,
  deriveChangeClass,
  CHANGE_CLASSES,
} from '../lib/cells.mjs';
import { claimCellFile, readClaim, claimPath } from '../lib/claims.mjs';
import { reserve } from '../lib/reservations.mjs';
import { activeDecisions } from '../lib/decisions.mjs';
import { writeJsonAtomic } from '../lib/fsutil.mjs';
import { acquireStoreLockOnceSync } from '../lib/lock.mjs';
import { JUDGE_VERDICT_SCHEMA } from '../lib/judge.mjs';

const root = makeTempRepo();

// Self-containment fix (cs-2a split): makeStateRepo is defined in test_lib.mjs's
// "bee.mjs state CLI" section (a section that stays behind for cs-2b) and was
// only reachable here via function-declaration hoisting across the whole
// monolith. The budget-audit-order / gate-bypass / claim-next-budget rows
// below need a throwaway repo distinct from the shared `root`, so this is a
// verbatim copy of that helper — same shape, same behavior, zero check
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

// jrt-1: a throwaway repo that HAS docs/decisions/taxonomy.json — the
// bootstrap/enforced boundary decisions.mjs's classifyDecisionTags checks
// (decision-propagation D7b). Every internal logDecision( call in
// cells.mjs/claims.mjs must survive being exercised against a repo shaped
// like this one; only two tag names are seeded (cells, judge) — enough for
// every cells.mjs call site this cell fixes, and deliberately NOT copied
// verbatim from the real docs/decisions/taxonomy.json so this fixture can
// never silently drift out of sync with it.
function makeTaxonomyRepo(prefix) {
  const dir = fs.mkdtempSync(path.join(os.tmpdir(), prefix));
  fs.mkdirSync(path.join(dir, '.bee', 'cells'), { recursive: true });
  writeJsonAtomic(path.join(dir, '.bee', 'onboarding.json'), {
    schema_version: '1.0',
    bee_version: '0.1.0',
  });
  fs.mkdirSync(path.join(dir, 'docs', 'decisions'), { recursive: true });
  writeJsonAtomic(path.join(dir, 'docs', 'decisions', 'taxonomy.json'), {
    schema_version: 1,
    tags: [
      { name: 'cells', description: 'Work cells: authoring, claims, caps' },
      { name: 'judge', description: 'Semantic goal-check judging' },
    ],
    candidates: [],
  });
  // Gate 3 pre-approved — claimCellCrossSession refuses on an unapproved
  // execution gate, which is orthogonal to what this fixture exercises.
  writeJsonAtomic(path.join(dir, '.bee', 'state.json'), {
    schema_version: '1.0',
    phase: 'swarming',
    feature: 'demo-feat',
    mode: 'standard',
    approved_gates: { context: true, shape: true, execution: true, review: false },
    workers: [],
  });
  return dir;
}

// Self-containment fix (cs-2a split): makeCellFile is defined in test_lib.mjs's
// "bee.mjs state start-feature" section (also staying behind for cs-2b) and,
// like makeStateRepo above, was only reachable here via hoisting. It fabricates
// a raw cell file (bypassing addCell's validation) so the budget/claim-next
// rows below can seed a specific trace.attempts shape directly — verbatim
// copy, same shape, zero check weakened.
// hardening-1-7-10 (D4): a small local helper — assertThrows/assertRejects
// (test-fixture.mjs) only check the error MESSAGE substring, but this file's
// new typed-error tests (CELL_ARCHIVED, CELLS_ARCHIVE_BUSY,
// ARCHIVE_DESTINATION_COLLISION) need to pin the actual `.code` a caller
// would branch on, not just wording that could drift. Local rather than
// added to the shared fixture — no other suite needs it yet.
function assertThrowsCode(fn, code, message) {
  try {
    fn();
  } catch (error) {
    assert(error && error.code === code, `${message} — threw, but error.code is "${error && error.code}", not "${code}"`);
    return;
  }
  throw new Error(`${message} — expected an error, none thrown`);
}

function makeCellFile(dir, id, extra = {}) {
  fs.mkdirSync(path.join(dir, '.bee', 'cells'), { recursive: true });
  const cell = {
    id,
    feature: 'old-feature',
    title: `Cell ${id}`,
    lane: 'tiny',
    status: 'open',
    deps: [],
    action: 'do it',
    verify: 'node -e "process.exit(0)"',
    trace: {},
    ...extra,
  };
  writeJsonAtomic(path.join(dir, '.bee', 'cells', `${id}.json`), cell);
  return cell;
}

// ─── cells: add validation ──────────────────────────────────────────────────

await check('addCell rejects an invalid lane', async () => {
  assertThrows(() => addCell(root, makeCell('bad-lane', { lane: 'huge' })), 'lane', 'invalid lane');
});

await check('addCell rejects standard lane without must_haves.truths', async () => {
  assertThrows(
    () => addCell(root, makeCell('std-1', { lane: 'standard' })),
    'must_haves',
    'standard lane needs truths',
  );
});

await check('addCell accepts a valid small cell and a standard cell with truths', async () => {
  addCell(root, makeCell('demo-1'));
  addCell(
    root,
    makeCell('demo-2', {
      lane: 'standard',
      deps: ['demo-1'],
      must_haves: { truths: ['Users see X'], artifacts: [], key_links: [], prohibitions: [] },
    }),
  );
  assert(readCell(root, 'demo-1') !== null, 'demo-1 should exist');
  assert(readCell(root, 'demo-2') !== null, 'demo-2 should exist');
});

// ─── cells: batch add (cells-batch-add) ─────────────────────────────────────

await check('addCells creates every cell of a valid batch in one call', async () => {
  const added = addCells(root, [makeCell('batch-1'), makeCell('batch-2'), makeCell('batch-3')]);
  assert(added.length === 3, 'three cells returned');
  for (const id of ['batch-1', 'batch-2', 'batch-3']) {
    assert(readCell(root, id) !== null, `${id} should exist`);
  }
});

await check('addCells is all-or-nothing: one invalid cell in the batch writes zero files', async () => {
  assertThrows(
    () => addCells(root, [makeCell('batch-x1'), makeCell('batch-x2', { lane: 'huge' }), makeCell('batch-x3')]),
    'lane',
    'invalid lane in the middle of the batch refuses',
  );
  for (const id of ['batch-x1', 'batch-x2', 'batch-x3']) {
    assert(readCell(root, id) === null, `${id} must not exist after a failed batch`);
  }
});

await check('addCells refuses a duplicate id within the batch, nothing written', async () => {
  assertThrows(
    () => addCells(root, [makeCell('batch-dup'), makeCell('batch-dup')]),
    'duplicate',
    'in-batch duplicate id refuses',
  );
  assert(readCell(root, 'batch-dup') === null, 'batch-dup must not exist');
});

await check('addCells refuses a non-array and an empty array', async () => {
  assertThrows(() => addCells(root, makeCell('batch-notarray')), 'array', 'plain object refused');
  assertThrows(() => addCells(root, []), 'array', 'empty array refused');
});

// ─── cells: whole-array batch validation report + dry-run (ce-2) ───────────
// A multi-cell payload never needs re-sending to discover the next error —
// EVERY cell's problem is collected before the batch refuses, not just the
// first one the old loop happened to reach.

await check('addCells aggregates EVERY failing cell in one refusal — two bad cells among three, both named with their own problem, nothing written', async () => {
  assertThrows(
    () =>
      addCells(root, [
        makeCell('batch-agg-1'),
        makeCell('batch-agg-2', { lane: 'huge' }),
        makeCell('batch-agg-3', { title: '' }),
      ]),
    'batch-agg-2',
    'the first bad cell (invalid lane) is named',
  );
  assertThrows(
    () =>
      addCells(root, [
        makeCell('batch-agg-1b'),
        makeCell('batch-agg-2b', { lane: 'huge' }),
        makeCell('batch-agg-3b', { title: '' }),
      ]),
    'batch-agg-3b',
    'the batch does NOT stop at the first bad cell — the second bad cell (missing title) is also named in the SAME refusal',
  );
  for (const id of ['batch-agg-1', 'batch-agg-2', 'batch-agg-3', 'batch-agg-1b', 'batch-agg-2b', 'batch-agg-3b']) {
    assert(readCell(root, id) === null, `${id} must not exist — nothing written on a failed batch`);
  }
});

await check('previewAddCells: a clean batch reports ok:true, every cell verdict ok, and writes nothing', async () => {
  const preview = previewAddCells(root, [makeCell('preview-clean-1'), makeCell('preview-clean-2')]);
  assert(preview.ok === true, 'batch-level ok true');
  assert(preview.cells.length === 2, 'two cell verdicts returned');
  assert(preview.cells.every((c) => c.ok === true && c.problems.length === 0), 'every cell verdict clean');
  assert(
    readCell(root, 'preview-clean-1') === null && readCell(root, 'preview-clean-2') === null,
    'a dry-run preview never writes, even on a clean batch',
  );
});

await check('previewAddCells: a dirty batch names EVERY failing cell (not just the first), the clean cell still verdicts ok, and writes nothing', async () => {
  const preview = previewAddCells(root, [
    makeCell('preview-ok'),
    makeCell('preview-bad-lane', { lane: 'huge' }),
    makeCell('preview-bad-title', { title: '' }),
  ]);
  assert(preview.ok === false, 'batch-level ok false');
  const byId = Object.fromEntries(preview.cells.map((c) => [c.id, c]));
  assert(byId['preview-ok'].ok === true, 'the one valid cell in the batch still verdicts ok');
  assert(
    byId['preview-bad-lane'].ok === false &&
      byId['preview-bad-lane'].problems.some((p) => p.toLowerCase().includes('lane')),
    'the lane cell is named with its own problem',
  );
  assert(
    byId['preview-bad-title'].ok === false && byId['preview-bad-title'].problems.length > 0,
    'the title cell is ALSO named — never swallowed by the first bad cell',
  );
  for (const id of ['preview-ok', 'preview-bad-lane', 'preview-bad-title']) {
    assert(readCell(root, id) === null, `${id} must not exist — a dry-run preview never writes`);
  }
});

// ─── cells: dependency-cycle refusal at every dep-mutating write (D2, ────────
// parallel-scheduler-2) — addCell, addCells, updateCell-when-deps-change all
// refuse fail-fast, all-or-nothing, before any writeCell. File overlap is
// NEVER checked here (D2: overlap stays legal, only cycles are illegal) —
// isolated temp roots so this section's ids never interact with the shared
// `root` used above/below.

function cycMakeCell(id, extra = {}) {
  return makeCell(id, { feature: 'cyc-demo', ...extra });
}

await check('addCells refuses an in-batch cycle (a<->b), nothing written, message names both ids', async () => {
  const cRoot = fs.mkdtempSync(path.join(os.tmpdir(), 'bee-cycle-batch-'));
  try {
    assertThrows(
      () =>
        addCells(cRoot, [
          cycMakeCell('cyc-a', { deps: ['cyc-b'] }),
          cycMakeCell('cyc-b', { deps: ['cyc-a'] }),
        ]),
      'cycle',
      'in-batch two-cycle refused',
    );
    assertThrows(
      () =>
        addCells(cRoot, [
          cycMakeCell('cyc-a2', { deps: ['cyc-b2'] }),
          cycMakeCell('cyc-b2', { deps: ['cyc-a2'] }),
        ]),
      'cyc-a2',
      'refusal message names the cycle ids',
    );
    assert(readCell(cRoot, 'cyc-a') === null, 'cyc-a must not exist — batch refused before any write');
    assert(readCell(cRoot, 'cyc-b') === null, 'cyc-b must not exist — batch refused before any write');
  } finally {
    fs.rmSync(cRoot, { recursive: true, force: true });
  }
});

await check('previewAddCells folds a batch-wide cycle into the cells it touches (ce-2), writes nothing', async () => {
  const cRoot = fs.mkdtempSync(path.join(os.tmpdir(), 'bee-cycle-preview-'));
  try {
    const preview = previewAddCells(cRoot, [
      cycMakeCell('prev-cyc-a', { deps: ['prev-cyc-b'] }),
      cycMakeCell('prev-cyc-b', { deps: ['prev-cyc-a'] }),
    ]);
    assert(preview.ok === false, 'a cycle refuses the whole batch');
    const byId = Object.fromEntries(preview.cells.map((c) => [c.id, c]));
    assert(
      byId['prev-cyc-a'].ok === false && byId['prev-cyc-a'].problems.some((p) => p.toLowerCase().includes('cycle')),
      'cycle named on prev-cyc-a',
    );
    assert(
      byId['prev-cyc-b'].ok === false && byId['prev-cyc-b'].problems.some((p) => p.toLowerCase().includes('cycle')),
      'cycle named on prev-cyc-b too',
    );
    assert(readCell(cRoot, 'prev-cyc-a') === null, 'a dry-run preview never writes, even on a cycle');
  } finally {
    fs.rmSync(cRoot, { recursive: true, force: true });
  }
});

await check('addCell refuses a cycle formed against an existing on-disk cell (batch-vs-disk)', async () => {
  const cRoot = fs.mkdtempSync(path.join(os.tmpdir(), 'bee-cycle-disk-'));
  try {
    addCell(cRoot, cycMakeCell('cyc-disk-a', { deps: ['cyc-disk-b'] }));
    // cyc-disk-b does not exist yet — cyc-disk-a's dep is merely unsatisfiable
    // so far (no cycle: an unknown id can never close one). Adding cyc-disk-b
    // with a dep back on cyc-disk-a closes it against the ON-DISK cell.
    assertThrows(
      () => addCell(cRoot, cycMakeCell('cyc-disk-b', { deps: ['cyc-disk-a'] })),
      'cycle',
      'new cell forming a cycle with an on-disk cell is refused',
    );
    assert(readCell(cRoot, 'cyc-disk-b') === null, 'cyc-disk-b must not exist — refused before write');
  } finally {
    fs.rmSync(cRoot, { recursive: true, force: true });
  }
});

await check('addCell refuses a self-dependency (a lists itself in deps)', async () => {
  const cRoot = fs.mkdtempSync(path.join(os.tmpdir(), 'bee-cycle-self-'));
  try {
    assertThrows(
      () => addCell(cRoot, cycMakeCell('cyc-self', { deps: ['cyc-self'] })),
      'cycle',
      'self-dep refused',
    );
    assert(readCell(cRoot, 'cyc-self') === null, 'cyc-self must not exist');
  } finally {
    fs.rmSync(cRoot, { recursive: true, force: true });
  }
});

await check('updateCell refuses a patch that reintroduces a cycle via deps; the cell is untouched', async () => {
  const cRoot = fs.mkdtempSync(path.join(os.tmpdir(), 'bee-cycle-update-'));
  try {
    addCell(cRoot, cycMakeCell('cyc-upd-a', { deps: [] }));
    addCell(cRoot, cycMakeCell('cyc-upd-b', { deps: ['cyc-upd-a'] }));
    const before = readCell(cRoot, 'cyc-upd-a');
    await assertRejects(
      () => updateCell(cRoot, 'cyc-upd-a', { deps: ['cyc-upd-b'] }),
      'cycle',
      'update that closes a<->b via deps is refused',
    );
    const after = readCell(cRoot, 'cyc-upd-a');
    assert(JSON.stringify(after) === JSON.stringify(before), 'cyc-upd-a must be byte-unchanged after the refused update');
    // A field edit that does NOT touch deps is untouched by the cycle check —
    // proves the check is deps-gated, not a blanket re-validation.
    const ok = await updateCell(cRoot, 'cyc-upd-a', { title: 'renamed, no deps change' });
    assert(ok.title === 'renamed, no deps change', 'non-deps patch still applies normally');
  } finally {
    fs.rmSync(cRoot, { recursive: true, force: true });
  }
});

await check('addCell over a store with pre-existing unsatisfiable deps (missing/blocked/dropped) still succeeds — never mistaken for a cycle', async () => {
  const cRoot = fs.mkdtempSync(path.join(os.tmpdir(), 'bee-cycle-unsat-'));
  try {
    addCell(cRoot, cycMakeCell('cyc-unsat-blocked', { deps: [] }));
    await updateCell(cRoot, 'cyc-unsat-blocked', { title: 'still open' }); // no-op sanity
    // Simulate a blocked cell directly (blockCell requires the cell to exist first).
    await blockCell(cRoot, 'cyc-unsat-blocked', 'unrelated reason');
    addCell(cRoot, cycMakeCell('cyc-unsat-dropped', { deps: [] }));
    await dropCell(cRoot, 'cyc-unsat-dropped', 'unrelated reason');
    // A brand-new cell depending on a missing id, plus the blocked/dropped
    // cells above, plus a dep on a capped cell (satisfied) — none of this is
    // structurally a cycle; it must add cleanly.
    addCell(cRoot, cycMakeCell('cyc-unsat-new', { deps: ['cyc-unsat-blocked', 'cyc-unsat-dropped', 'cyc-unsat-missing'] }));
    assert(readCell(cRoot, 'cyc-unsat-new') !== null, 'cyc-unsat-new must be added — unsatisfiable deps are not cycles');
  } finally {
    fs.rmSync(cRoot, { recursive: true, force: true });
  }
});

await check('a pre-existing on-disk cycle (legacy store) never blocks unrelated writes; a write participating in it is still refused (parallel-scheduler-5)', async () => {
  const cRoot = fs.mkdtempSync(path.join(os.tmpdir(), 'bee-cycle-legacy-'));
  try {
    // Simulate a legacy (pre-guard, ≤0.1.42) store: hand-write a cyclic pair
    // directly on disk, bypassing addCell — the shape an upgraded host can
    // legitimately carry. D2 scopes the WRITE refusal to cycles the write
    // introduces or participates in; pre-existing cycles are `cells schedule`
    // diagnostics, never a store-wide write freeze.
    const dir = path.join(cRoot, '.bee', 'cells');
    fs.mkdirSync(dir, { recursive: true });
    for (const [id, dep] of [['cyc-legacy-a', 'cyc-legacy-b'], ['cyc-legacy-b', 'cyc-legacy-a']]) {
      fs.writeFileSync(path.join(dir, `${id}.json`), JSON.stringify(cycMakeCell(id, { deps: [dep] })));
    }
    // Unrelated acyclic add in another feature succeeds despite the legacy cycle.
    addCell(cRoot, makeCell('cyc-legacy-new', { feature: 'other-feature' }));
    assert(readCell(cRoot, 'cyc-legacy-new') !== null, 'unrelated add must succeed despite a legacy cycle elsewhere');
    // Unrelated deps patch succeeds too.
    const upd = await updateCell(cRoot, 'cyc-legacy-new', { deps: ['cyc-legacy-missing'] });
    assert(upd.deps.length === 1, 'unrelated deps patch must apply despite a legacy cycle elsewhere');
    // A patch on a cycle MEMBER that keeps the cycle closed is still refused…
    await assertRejects(
      () => updateCell(cRoot, 'cyc-legacy-a', { deps: ['cyc-legacy-b', 'cyc-legacy-missing'] }),
      'cycle',
      'a deps patch that keeps the cell inside the cycle is refused',
    );
    // …while a patch that BREAKS the cycle applies cleanly (the fix path).
    const healed = await updateCell(cRoot, 'cyc-legacy-a', { deps: [] });
    assert(healed.deps.length === 0, 'a deps patch that breaks the legacy cycle must be allowed');
  } finally {
    fs.rmSync(cRoot, { recursive: true, force: true });
  }
});

await check('addCells: file overlap between two batch cells is NOT refused (D2 — only cycles are illegal)', async () => {
  const cRoot = fs.mkdtempSync(path.join(os.tmpdir(), 'bee-cycle-overlap-'));
  try {
    const added = addCells(cRoot, [
      cycMakeCell('cyc-ov-a', { files: ['shared.mjs'] }),
      cycMakeCell('cyc-ov-b', { files: ['shared.mjs'] }),
    ]);
    assert(added.length === 2, 'both overlapping cells are added — overlap is legal per D2');
  } finally {
    fs.rmSync(cRoot, { recursive: true, force: true });
  }
});

await check('bee.mjs cells add CLI: a JSON array on --stdin creates the whole slice in one call', async () => {
  const cliPath = fileURLToPath(new URL('../bee.mjs', import.meta.url));
  const batch = [makeCell('batch-cli-1'), makeCell('batch-cli-2')];
  const ok = await runModuleWorker(cliPath, {
    args: ['cells', 'add', '--stdin'],
    cwd: root,
    input: JSON.stringify(batch),
  });
  assert(ok.status === 0, `batch add CLI exits 0, got ${ok.status}: ${ok.stderr}`);
  assert(ok.stdout.includes('Added batch-cli-1') && ok.stdout.includes('Added batch-cli-2'), 'every added id reported');
  assert(readCell(root, 'batch-cli-1') !== null && readCell(root, 'batch-cli-2') !== null, 'both cells exist');
  const single = await runModuleWorker(cliPath, {
    args: ['cells', 'add', '--stdin'],
    cwd: root,
    input: JSON.stringify(makeCell('batch-cli-single')),
  });
  assert(single.status === 0, `single-object add still exits 0, got ${single.status}: ${single.stderr}`);
  assert(readCell(root, 'batch-cli-single') !== null, 'single-object path unchanged');
});

// ─── cells: update verb (cells-update-verb) ─────────────────────────────────

await check('updateCell lands patched fields on an open cell; unpatched fields, status, trace byte-stable', async () => {
  addCell(root, makeCell('upd-1', { action: 'Old action per D1.' }));
  const before = readCell(root, 'upd-1');
  const updated = await updateCell(root, 'upd-1', { action: 'New action per D2.', files: ['a.txt'] });
  assert(updated.action === 'New action per D2.', 'action updated');
  assert(updated.files.length === 1 && updated.files[0] === 'a.txt', 'files updated');
  assert(updated.title === before.title, 'unpatched field unchanged');
  assert(updated.status === before.status, 'status unchanged');
  assert(JSON.stringify(updated.trace) === JSON.stringify(before.trace), 'trace unchanged');
});

await check('updateCell works on a blocked cell (rescue path), refuses an empty patch', async () => {
  addCell(root, makeCell('upd-2', { status: 'blocked' }));
  const updated = await updateCell(root, 'upd-2', { verify: 'node -e "process.exit(0)" # v2' });
  assert(updated.verify.includes('v2'), 'verify updated on blocked cell');
  await assertRejects(() => updateCell(root, 'upd-2', {}), 'empty', 'empty patch refused');
});

await check('updateCell refuses claimed, capped, and dropped cells with the file byte-unchanged', async () => {
  for (const status of ['claimed', 'capped', 'dropped']) {
    const id = `upd-door-${status}`;
    addCell(root, makeCell(id, { status }));
    const file = path.join(root, '.bee', 'cells', `${id}.json`);
    const before = fs.readFileSync(file, 'utf8');
    await assertRejects(() => updateCell(root, id, { title: 'nope' }), status, `${status} cell refused`);
    assert(fs.readFileSync(file, 'utf8') === before, `${id} file byte-unchanged after refusal`);
  }
});

await check('updateCell refuses every frozen key and unknown keys — whole patch, file untouched', async () => {
  addCell(root, makeCell('upd-3'));
  const file = path.join(root, '.bee', 'cells', 'upd-3.json');
  const before = fs.readFileSync(file, 'utf8');
  for (const key of ['id', 'feature', 'status', 'trace', 'tier']) {
    await assertRejects(
      () => updateCell(root, 'upd-3', { title: 'ok', [key]: 'x' }),
      'frozen',
      `frozen key ${key} refuses the whole patch`,
    );
  }
  await assertRejects(() => updateCell(root, 'upd-3', { totally_new: 1 }), 'unknown field', 'unknown key refused');
  await assertRejects(() => updateCell(root, 'upd-3', { title: '' }), 'non-empty string', 'invalid value refused');
  assert(fs.readFileSync(file, 'utf8') === before, 'upd-3 file untouched after all refusals');
});

await check('updateCell fails closed on a present-but-corrupt cell file and on a missing cell', async () => {
  const file = path.join(root, '.bee', 'cells', 'upd-corrupt.json');
  fs.writeFileSync(file, '{ not json');
  await assertRejects(() => updateCell(root, 'upd-corrupt', { title: 'x' }), 'not valid JSON', 'corrupt cell refused');
  assert(fs.readFileSync(file, 'utf8') === '{ not json', 'corrupt file untouched');
  fs.rmSync(file);
  await assertRejects(() => updateCell(root, 'upd-nope', { title: 'x' }), 'not found', 'missing cell refused');
});

await check('updateCell re-checks the standard/high-risk truths invariant on the merged result', async () => {
  addCell(root, makeCell('upd-4', { lane: 'standard', must_haves: { truths: ['t1'] } }));
  await assertRejects(
    () => updateCell(root, 'upd-4', { must_haves: { truths: [] } }),
    'truths',
    'emptied truths refused',
  );
  const ok = await updateCell(root, 'upd-4', { must_haves: { truths: ['t1', 't2'] } });
  assert(ok.must_haves.truths.length === 2, 'valid must_haves patch lands');
  await assertRejects(
    () => updateCell(root, 'upd-1', { lane: 'standard' }),
    'truths',
    'lane upgrade without truths refused',
  );
});

await check('bee.mjs cells update CLI: --file works one-line; unknown flag and missing --id refuse', async () => {
  const cliPath = fileURLToPath(new URL('../bee.mjs', import.meta.url));
  addCell(root, makeCell('upd-cli-1'));
  const patchFile = path.join(root, 'upd-cli-patch.json');
  fs.writeFileSync(patchFile, JSON.stringify({ title: 'CLI updated title' }));
  const ok = await runModuleWorker(cliPath, {
    args: ['cells', 'update', '--id', 'upd-cli-1', '--file', patchFile],
    cwd: root,
  });
  assert(ok.status === 0, `update CLI exits 0, got ${ok.status}: ${ok.stderr}`);
  assert(ok.stdout.includes('Updated upd-cli-1'), 'one-line confirmation printed');
  assert(readCell(root, 'upd-cli-1').title === 'CLI updated title', 'patch landed via CLI');
  const badFlag = await runModuleWorker(cliPath, {
    args: ['cells', 'update', '--id', 'upd-cli-1', '--file', patchFile, '--dry-run', 'x'],
    cwd: root,
  });
  assert(badFlag.status !== 0, 'unknown flag refuses');
  const noId = await runModuleWorker(cliPath, {
    args: ['cells', 'update', '--file', patchFile],
    cwd: root,
  });
  assert(noId.status !== 0, 'missing --id refuses');
});

// ─── cells: gate-locked claiming + deps ─────────────────────────────────────

await check('claimCell refuses while gate execution is false', async () => {
  await assertRejects(() => claimCell(root, 'demo-1', 'worker-a'), 'execution', 'gate lock');
});

await check('readyCells excludes cells with uncapped deps', async () => {
  const ready = readyCells(root, 'demo');
  const ids = ready.map((cell) => cell.id);
  assert(ids.includes('demo-1'), 'demo-1 should be ready');
  assert(!ids.includes('demo-2'), 'demo-2 depends on uncapped demo-1');
});

// ─── cells-archive-1: regression net pinning CURRENT listCells/readCell/ ───
// readyCells/depsAllCapped behavior over an isolated fixture, GREEN BEFORE any
// archive-fallback changes land (crit-pattern 20260714: net first). depsAllCapped
// itself is unexported — exercised indirectly through readyCells, its only caller
// besides claimCell. Every case below must stay byte-identical once the archive
// lookup lands (no archive dir exists in these fixtures).

await check('listCells (pre-archive baseline): sorted by id, feature/status filters, tolerant of an unparseable sibling file', async () => {
  const aRoot = fs.mkdtempSync(path.join(os.tmpdir(), 'bee-archive-net-list-'));
  try {
    addCell(aRoot, makeCell('arc-b', { feature: 'arc-feat' }));
    addCell(aRoot, makeCell('arc-a', { feature: 'arc-feat' }));
    addCell(aRoot, makeCell('arc-c', { feature: 'other-feat' }));
    // A non-JSON sibling and an unparseable .json sibling must both be skipped
    // tolerantly, never thrown.
    fs.writeFileSync(path.join(aRoot, '.bee', 'cells', 'garbage.json'), '{ not json');
    fs.writeFileSync(path.join(aRoot, '.bee', 'cells', 'notes.txt'), 'not a cell');
    const all = listCells(aRoot);
    assert(
      all.map((c) => c.id).join(',') === 'arc-a,arc-b,arc-c',
      `sorted by id, unparseable siblings skipped, got ${all.map((c) => c.id).join(',')}`,
    );
    const byFeature = listCells(aRoot, { feature: 'arc-feat' });
    assert(byFeature.map((c) => c.id).join(',') === 'arc-a,arc-b', 'feature filter narrows and stays sorted');
    const byStatus = listCells(aRoot, { status: 'open' });
    assert(byStatus.length === 3, 'status filter matches the all-open fixture');
    assert(listCells(aRoot, { status: 'capped' }).length === 0, 'status filter excludes non-matching cells');
  } finally {
    fs.rmSync(aRoot, { recursive: true, force: true });
  }
});

await check('readCell (pre-archive baseline): returns the cell object for an existing id, null for a missing id', async () => {
  const aRoot = fs.mkdtempSync(path.join(os.tmpdir(), 'bee-archive-net-readcell-'));
  try {
    addCell(aRoot, makeCell('arc-rd-1'));
    const got = readCell(aRoot, 'arc-rd-1');
    assert(got && got.id === 'arc-rd-1', 'existing cell returned');
    assert(readCell(aRoot, 'arc-rd-missing') === null, 'missing cell returns null, not a throw');
  } finally {
    fs.rmSync(aRoot, { recursive: true, force: true });
  }
});

await check('readyCells/depsAllCapped (pre-archive baseline): excludes an open cell whose dep is not capped or is missing; includes it once the dep is capped', async () => {
  const aRoot = fs.mkdtempSync(path.join(os.tmpdir(), 'bee-archive-net-ready-'));
  try {
    addCell(aRoot, makeCell('arc-dep-base', { feature: 'arc-ready' }));
    addCell(aRoot, makeCell('arc-dep-child', { feature: 'arc-ready', deps: ['arc-dep-base'] }));
    addCell(
      aRoot,
      makeCell('arc-dep-missing-child', { feature: 'arc-ready', deps: ['arc-dep-nope'] }),
    );
    let ready = readyCells(aRoot, 'arc-ready').map((c) => c.id);
    assert(ready.includes('arc-dep-base'), 'base cell (no deps) is ready');
    assert(!ready.includes('arc-dep-child'), 'child excluded — its dep is not capped');
    assert(!ready.includes('arc-dep-missing-child'), 'child with a missing dep excluded too');

    const state = readState(aRoot);
    state.approved_gates.execution = true;
    writeState(aRoot, state);
    await claimCell(aRoot, 'arc-dep-base', 'worker-arc');
    await capCell(aRoot, 'arc-dep-base', { files_changed: ['x.txt'], outcome: 'done' });

    ready = readyCells(aRoot, 'arc-ready').map((c) => c.id);
    assert(ready.includes('arc-dep-child'), 'child now ready — its dep is capped');
    assert(!ready.includes('arc-dep-missing-child'), 'still excluded — its dep is still missing');
  } finally {
    fs.rmSync(aRoot, { recursive: true, force: true });
  }
});

// ─── cells-archive-1: archive-aware lookup (cellsArchiveDir, readCell/────────
// resolveCellFile fallback, listCells includeArchived) — the actual new
// behavior, layered on top of the pinned baseline above.

await check('cellsArchiveDir composes .bee/cells/archive/<feature>/, and readCell/resolveCellFile fall back there when the active file is absent', async () => {
  const aRoot = fs.mkdtempSync(path.join(os.tmpdir(), 'bee-archive-net-fallback-'));
  try {
    assert(
      cellsArchiveDir(aRoot, 'shipped-feat') === path.join(aRoot, '.bee', 'cells', 'archive', 'shipped-feat'),
      'cellsArchiveDir composes the expected path',
    );
    // A cell that exists ONLY under the archive tree (simulating a later
    // archive-move cell), never written to the active .bee/cells/ dir.
    const archived = makeCell('arc-fb-1', { feature: 'shipped-feat', status: 'capped' });
    const archiveDir = cellsArchiveDir(aRoot, 'shipped-feat');
    // recursive:true also creates the .bee/cells/ parent — no active file for
    // arc-fb-1 exists anywhere under it, only this archived one.
    fs.mkdirSync(archiveDir, { recursive: true });
    fs.writeFileSync(path.join(archiveDir, 'arc-fb-1.json'), JSON.stringify(archived));

    const found = readCell(aRoot, 'arc-fb-1');
    assert(found && found.id === 'arc-fb-1' && found.status === 'capped', 'readCell falls back to the archive tree once .bee/cells/ exists');
    assert(readCell(aRoot, 'arc-fb-missing') === null, 'a truly nonexistent id (active nor archived) still returns null');

    const resolved = resolveCellFile(aRoot, 'arc-fb-1');
    assert(resolved === path.join(archiveDir, 'arc-fb-1.json'), 'resolveCellFile names the REAL archive path for an archived-only cell');
    assert(resolveCellFile(aRoot, 'arc-fb-missing') === null, 'resolveCellFile returns null for a cell in neither tree');

    // cellFile's own meaning (the active path) is unchanged — adding the
    // active-side cell now must make readCell/resolveCellFile prefer it.
    addCell(aRoot, makeCell('arc-fb-2', { feature: 'shipped-feat' }));
    assert(resolveCellFile(aRoot, 'arc-fb-2') === path.join(aRoot, '.bee', 'cells', 'arc-fb-2.json'), 'resolveCellFile prefers the active file when present');
  } finally {
    fs.rmSync(aRoot, { recursive: true, force: true });
  }
});

await check('readCell fallback is byte-identical to today when no archive dir exists at all (no throw, missing stays null)', async () => {
  const aRoot = fs.mkdtempSync(path.join(os.tmpdir(), 'bee-archive-net-none-'));
  try {
    addCell(aRoot, makeCell('arc-noarc-1'));
    assert(readCell(aRoot, 'arc-noarc-1') !== null, 'active cell still resolves with no archive dir present');
    assert(readCell(aRoot, 'arc-noarc-missing') === null, 'missing cell returns null, not a throw, with no archive dir present');
    assert(resolveCellFile(aRoot, 'arc-noarc-missing') === null, 'resolveCellFile also returns null, never throws, with no archive dir present');
  } finally {
    fs.rmSync(aRoot, { recursive: true, force: true });
  }
});

await check('listCells default scans ONLY the active .bee/cells/ dir (skips archive/), includeArchived:true folds in archived cells too, same sorted shape', async () => {
  const aRoot = fs.mkdtempSync(path.join(os.tmpdir(), 'bee-archive-net-listarchived-'));
  try {
    addCell(aRoot, makeCell('arc-lst-b', { feature: 'arc-lst-feat' }));
    addCell(aRoot, makeCell('arc-lst-d', { feature: 'arc-lst-feat' }));
    const archiveDir = cellsArchiveDir(aRoot, 'arc-lst-feat');
    fs.mkdirSync(archiveDir, { recursive: true });
    fs.writeFileSync(
      path.join(archiveDir, 'arc-lst-a.json'),
      JSON.stringify(makeCell('arc-lst-a', { feature: 'arc-lst-feat', status: 'capped' })),
    );
    fs.writeFileSync(
      path.join(archiveDir, 'arc-lst-c.json'),
      JSON.stringify(makeCell('arc-lst-c', { feature: 'arc-lst-feat', status: 'capped' })),
    );

    const activeOnly = listCells(aRoot, { feature: 'arc-lst-feat' });
    assert(
      activeOnly.map((c) => c.id).join(',') === 'arc-lst-b,arc-lst-d',
      `default listCells must skip archive/ entirely, got ${activeOnly.map((c) => c.id).join(',')}`,
    );

    const withArchived = listCells(aRoot, { feature: 'arc-lst-feat', includeArchived: true });
    assert(
      withArchived.map((c) => c.id).join(',') === 'arc-lst-a,arc-lst-b,arc-lst-c,arc-lst-d',
      `includeArchived:true folds in archived cells, same sorted-by-id shape, got ${withArchived.map((c) => c.id).join(',')}`,
    );

    const archivedCappedOnly = listCells(aRoot, { feature: 'arc-lst-feat', includeArchived: true, status: 'capped' });
    assert(
      archivedCappedOnly.map((c) => c.id).join(',') === 'arc-lst-a,arc-lst-c',
      'status filter applies to archived cells too',
    );
  } finally {
    fs.rmSync(aRoot, { recursive: true, force: true });
  }
});

// ─── hardening-1: P0 data-loss hardening for archiveFeature/unarchiveFeature
// (slug/containment validation, capped|dropped allowlist, atomic rollback,
// and the mutate-an-archived-cell fork guard). Each case below was RED
// against the pre-hardening code (see cell hardening-1 report) — the point of
// this net is that it stays green as the fix lands, not merely that it
// exists.

await check('archiveFeature refuses a path-traversal feature slug and moves nothing (P0 containment)', async () => {
  const aRoot = fs.mkdtempSync(path.join(os.tmpdir(), 'bee-archive-hard-traversal-'));
  try {
    const evilFeature = '../escape';
    const cellsDirPath = path.join(aRoot, '.bee', 'cells');
    fs.mkdirSync(cellsDirPath, { recursive: true });
    fs.writeFileSync(
      path.join(cellsDirPath, 'trav-1.json'),
      JSON.stringify(makeCell('trav-1', { feature: evilFeature, status: 'capped' })),
    );

    await assertRejects(
      () => archiveFeature(aRoot, evilFeature),
      'invalid feature',
      'a feature slug containing ".." / "/" must be refused before any move',
    );
    assert(fs.existsSync(path.join(cellsDirPath, 'trav-1.json')), 'cell file untouched after refusal');
    assert(
      !fs.existsSync(path.join(cellsDirPath, 'escape')),
      'no directory created outside the archive tree for the escaped path',
    );
  } finally {
    fs.rmSync(aRoot, { recursive: true, force: true });
  }
});

await check('archiveFeature refuses a feature containing a blocked cell — only ALL-capped/dropped features archive', async () => {
  const aRoot = fs.mkdtempSync(path.join(os.tmpdir(), 'bee-archive-hard-blocked-'));
  try {
    const cellsDirPath = path.join(aRoot, '.bee', 'cells');
    fs.mkdirSync(cellsDirPath, { recursive: true });
    fs.writeFileSync(
      path.join(cellsDirPath, 'blk-1.json'),
      JSON.stringify(makeCell('blk-1', { feature: 'blk-feat', status: 'capped' })),
    );
    fs.writeFileSync(
      path.join(cellsDirPath, 'blk-2.json'),
      JSON.stringify(makeCell('blk-2', { feature: 'blk-feat', status: 'blocked' })),
    );

    await assertRejects(
      () => archiveFeature(aRoot, 'blk-feat'),
      'blk-2 (blocked)',
      'a blocked sibling must refuse the whole archive, named in the error',
    );
    assert(fs.existsSync(path.join(cellsDirPath, 'blk-1.json')), 'capped sibling untouched — all-or-nothing refusal');
    assert(fs.existsSync(path.join(cellsDirPath, 'blk-2.json')), 'the blocked cell itself untouched');
  } finally {
    fs.rmSync(aRoot, { recursive: true, force: true });
  }
});

await check('mutating an archived cell id (updateCell/claimCell/capCell) refuses "archived" and never forks a duplicate active file', async () => {
  const aRoot = fs.mkdtempSync(path.join(os.tmpdir(), 'bee-archive-hard-mutate-'));
  try {
    const cellsDirPath = path.join(aRoot, '.bee', 'cells');
    fs.mkdirSync(cellsDirPath, { recursive: true });
    fs.writeFileSync(
      path.join(cellsDirPath, 'mut-1.json'),
      JSON.stringify(makeCell('mut-1', { feature: 'mut-feat', status: 'capped' })),
    );
    fs.writeFileSync(
      path.join(cellsDirPath, 'mut-2.json'),
      JSON.stringify(makeCell('mut-2', { feature: 'mut-feat', status: 'capped' })),
    );

    const state = readState(aRoot);
    state.approved_gates.execution = true;
    writeState(aRoot, state);

    const archived = await archiveFeature(aRoot, 'mut-feat');
    assert(
      archived.moved.includes('mut-1') && archived.moved.includes('mut-2'),
      `both cells should be archived, got ${JSON.stringify(archived)}`,
    );
    const activePath = path.join(cellsDirPath, 'mut-1.json');
    assert(!fs.existsSync(activePath), 'sanity: the active copy is gone after archiving');
    assert(readCell(aRoot, 'mut-1') !== null, 'sanity: readCell still resolves the id via the archive fallback');

    await assertRejects(() => updateCell(aRoot, 'mut-1', { title: 'hack' }), 'archived', 'updateCell on an archived id refuses');
    assert(!fs.existsSync(activePath), 'updateCell must not fork a duplicate active file');

    await assertRejects(() => claimCell(aRoot, 'mut-1', 'worker-x'), 'archived', 'claimCell on an archived id refuses');
    assert(!fs.existsSync(activePath), 'claimCell must not fork a duplicate active file');

    await assertRejects(
      () => capCell(aRoot, 'mut-1', { files_changed: ['x'], outcome: 'done' }),
      'archived',
      'capCell on an archived id refuses',
    );
    assert(!fs.existsSync(activePath), 'capCell must not fork a duplicate active file');
  } finally {
    fs.rmSync(aRoot, { recursive: true, force: true });
  }
});

// hardening-1-7-10 (D4): reopenCell/setTier/resetCellBudget/recordJudgeVerdict
// were the FOUR mutators that still forked a duplicate active file for an
// archived-only id, exactly the bug the test above already pins for
// updateCell/claimCell/capCell — this extends the same net to the remaining
// four.
await check('the 4 newly guarded mutators (reopenCell/setTier/resetCellBudget/recordJudgeVerdict) refuse an archived cell, never forking a duplicate active file', async () => {
  const aRoot = fs.mkdtempSync(path.join(os.tmpdir(), 'bee-archive-hard-mutate4-'));
  try {
    const cellsDirPath = path.join(aRoot, '.bee', 'cells');
    fs.mkdirSync(cellsDirPath, { recursive: true });
    fs.writeFileSync(
      path.join(cellsDirPath, 'mut4-1.json'),
      JSON.stringify(makeCell('mut4-1', { feature: 'mut4-feat', status: 'capped' })),
    );
    fs.writeFileSync(
      path.join(cellsDirPath, 'mut4-2.json'),
      JSON.stringify(makeCell('mut4-2', { feature: 'mut4-feat', status: 'capped' })),
    );

    // assertNotArchived refuses before any of these mutators ever look at
    // the cell's status, so the archived-only file's on-disk status is
    // irrelevant to the refusal below — only its LOCATION (archive tree,
    // not active) matters.
    const archived = await archiveFeature(aRoot, 'mut4-feat');
    assert(
      archived.moved.includes('mut4-1') && archived.moved.includes('mut4-2'),
      `both cells should be archived, got ${JSON.stringify(archived)}`,
    );
    const activePath = path.join(cellsDirPath, 'mut4-1.json');
    assert(!fs.existsSync(activePath), 'sanity: the active copy is gone after archiving');
    assert(readCell(aRoot, 'mut4-1') !== null, 'sanity: readCell still resolves the id via the archive fallback');

    await assertRejects(() => reopenCell(aRoot, 'mut4-1', 'rework needed'), 'archived', 'reopenCell on an archived id refuses');
    assert(!fs.existsSync(activePath), 'reopenCell must not fork a duplicate active file');

    await assertRejects(() => setTier(aRoot, 'mut4-1', 'generation'), 'archived', 'setTier on an archived id refuses');
    assert(!fs.existsSync(activePath), 'setTier must not fork a duplicate active file');

    await assertRejects(
      () => resetCellBudget(aRoot, 'mut4-1', 'why a retry is warranted', { operator: 'op-x' }),
      'archived',
      'resetCellBudget on an archived id refuses',
    );
    assert(!fs.existsSync(activePath), 'resetCellBudget must not fork a duplicate active file');

    await assertRejects(
      () =>
        recordJudgeVerdict(aRoot, 'mut4-1', {
          schema: JUDGE_VERDICT_SCHEMA,
          verdict: 'PASS',
          checks: [{ id: 'c1', status: 'PASS', evidence: 'looks good' }],
          fixability: 'automatic',
          confidence: 'high',
        }),
      'archived',
      'recordJudgeVerdict on an archived id refuses',
    );
    assert(!fs.existsSync(activePath), 'recordJudgeVerdict must not fork a duplicate active file');
  } finally {
    fs.rmSync(aRoot, { recursive: true, force: true });
  }
});

await check('writeCell throws typed CELL_ARCHIVED (never resurrects) when called directly on an id that resolves ONLY in the archive tree', async () => {
  const aRoot = fs.mkdtempSync(path.join(os.tmpdir(), 'bee-archive-hard-writecell-'));
  try {
    const cellsDirPath = path.join(aRoot, '.bee', 'cells');
    fs.mkdirSync(cellsDirPath, { recursive: true });
    fs.writeFileSync(
      path.join(cellsDirPath, 'wc-1.json'),
      JSON.stringify(makeCell('wc-1', { feature: 'wc-feat', status: 'capped' })),
    );
    const archived = await archiveFeature(aRoot, 'wc-feat');
    assert(archived.moved.includes('wc-1'), `wc-1 should be archived, got ${JSON.stringify(archived)}`);
    const activePath = path.join(cellsDirPath, 'wc-1.json');
    assert(!fs.existsSync(activePath), 'sanity: the active copy is gone after archiving');

    // Simulates the TOCTOU a mutator's own read-modify-write window leaves
    // open: it read the cell BEFORE the archive move (or otherwise built a
    // cell object with this id by hand) and now calls writeCell directly —
    // writeCell's own final check, held under the 'cells-archive' lock, must
    // refuse rather than silently creating a brand-new active file.
    assertThrowsCode(
      () => writeCell(aRoot, makeCell('wc-1', { feature: 'wc-feat', status: 'capped', title: 'resurrected' })),
      'CELL_ARCHIVED',
      'writeCell must refuse to resurrect an archived-only id',
    );
    assert(!fs.existsSync(activePath), 'writeCell must never fork a duplicate active file for an archived id');
  } finally {
    fs.rmSync(aRoot, { recursive: true, force: true });
  }
});

await check('writeCell throws typed CELLS_ARCHIVE_BUSY when the "cells-archive" lock is already held (a live archive/unarchive transaction)', async () => {
  const aRoot = fs.mkdtempSync(path.join(os.tmpdir(), 'bee-archive-hard-busy-'));
  try {
    fs.mkdirSync(path.join(aRoot, '.bee', 'cells'), { recursive: true });
    const held = acquireStoreLockOnceSync(aRoot, 'cells-archive');
    assert(held.acquired, 'test setup: must acquire the cells-archive lock to simulate a live archive/unarchive transaction');
    try {
      assertThrowsCode(
        () => writeCell(aRoot, makeCell('busy-1', { feature: 'busy-feat' })),
        'CELLS_ARCHIVE_BUSY',
        'writeCell must refuse (never wait) while the cells-archive lock is held',
      );
      assert(
        !fs.existsSync(path.join(aRoot, '.bee', 'cells', 'busy-1.json')),
        'no active file must appear when writeCell refuses on contention',
      );
    } finally {
      held.release();
    }
    // Once the lock is free again, the exact same write succeeds normally.
    const cell = writeCell(aRoot, makeCell('busy-1', { feature: 'busy-feat' }));
    assert(cell.id === 'busy-1', 'writeCell succeeds once the cells-archive lock is released');
  } finally {
    fs.rmSync(aRoot, { recursive: true, force: true });
  }
});

await check('unarchiveFeature refuses to overwrite an existing active file, typed, before any rename', async () => {
  const aRoot = fs.mkdtempSync(path.join(os.tmpdir(), 'bee-archive-hard-unarchive-collision-'));
  try {
    const cellsDirPath = path.join(aRoot, '.bee', 'cells');
    fs.mkdirSync(cellsDirPath, { recursive: true });
    fs.writeFileSync(
      path.join(cellsDirPath, 'ua-1.json'),
      JSON.stringify(makeCell('ua-1', { feature: 'ua-feat', status: 'capped' })),
    );
    fs.writeFileSync(
      path.join(cellsDirPath, 'ua-2.json'),
      JSON.stringify(makeCell('ua-2', { feature: 'ua-feat', status: 'capped' })),
    );
    const archived = await archiveFeature(aRoot, 'ua-feat');
    assert(
      archived.moved.includes('ua-1') && archived.moved.includes('ua-2'),
      `both cells should be archived, got ${JSON.stringify(archived)}`,
    );
    // A DIFFERENT active cell now happens to occupy ua-1's would-be
    // destination — e.g. the id was reused after the original archive, or a
    // stray file was left behind some other way. unarchiveFeature must
    // refuse the WHOLE batch rather than overwrite it.
    fs.writeFileSync(
      path.join(cellsDirPath, 'ua-1.json'),
      JSON.stringify(makeCell('ua-1', { feature: 'ua-feat', status: 'open', title: 'reused id, different cell' })),
    );

    await assertRejects(
      () => unarchiveFeature(aRoot, 'ua-feat'),
      'refused',
      'unarchiveFeature must refuse a collision against an existing active file',
    );

    const collided = JSON.parse(fs.readFileSync(path.join(cellsDirPath, 'ua-1.json'), 'utf8'));
    assert(collided.title === 'reused id, different cell', 'the existing active file must be untouched, never overwritten');
    assert(
      fs.existsSync(path.join(cellsArchiveDir(aRoot, 'ua-feat'), 'ua-2.json')),
      'ua-2 must stay archived — the whole batch refuses, nothing partially unarchived',
    );
    assert('ua-feat' in archivedSummary(aRoot), 'summary entry must be untouched by a refused unarchive');
  } finally {
    fs.rmSync(aRoot, { recursive: true, force: true });
  }
});

// hardening-1-7-10 (D4): this test used to inject a MID-LOOP rename failure
// (a pre-created directory sitting at roll-2's destination made renameSync
// throw EISDIR once the loop reached it) and assert the in-process catch
// block rolled roll-1 back. That exact setup — a destination that already
// exists before archiveFeature is ever called — is now caught by the NEW
// preflight collision check BEFORE any rename runs at all, so nothing ever
// gets renamed in the first place; the assertions below (nothing moved, no
// journal, summary untouched) reflect that, and the typed refusal replaces
// the old bare EISDIR propagation. The in-process rollback path this test
// used to cover is still exercised structurally by the journal-recovery test
// below, which simulates the harder case a preflight check can never catch —
// a process killed mid-loop with no chance to run its own rollback at all.
await check('archiveFeature preflights EVERY destination collision before any rename — typed refusal, nothing moved, no journal left behind', async () => {
  const aRoot = fs.mkdtempSync(path.join(os.tmpdir(), 'bee-archive-hard-preflight-'));
  try {
    const cellsDirPath = path.join(aRoot, '.bee', 'cells');
    fs.mkdirSync(cellsDirPath, { recursive: true });
    for (const id of ['roll-1', 'roll-2', 'roll-3']) {
      fs.writeFileSync(
        path.join(cellsDirPath, `${id}.json`),
        JSON.stringify(makeCell(id, { feature: 'roll-feat', status: 'capped' })),
      );
    }
    // Pre-creating ANYTHING at roll-2's destination — a directory works just
    // as well as a stray file — must refuse the WHOLE batch before touching
    // roll-1 or roll-3 at all, never partway through the loop.
    const archiveDir = cellsArchiveDir(aRoot, 'roll-feat');
    fs.mkdirSync(path.join(archiveDir, 'roll-2.json'), { recursive: true });

    await assertRejects(
      () => archiveFeature(aRoot, 'roll-feat'),
      'refused',
      'a pre-existing destination must refuse the whole archive batch, typed, before any rename',
    );

    assert(fs.existsSync(path.join(cellsDirPath, 'roll-1.json')), 'roll-1 must never be touched — preflight refuses before the first rename');
    assert(!fs.existsSync(path.join(archiveDir, 'roll-1.json')), 'roll-1 must not appear in the archive tree');
    assert(fs.existsSync(path.join(cellsDirPath, 'roll-2.json')), 'roll-2 stays active — its slot was the collision');
    assert(fs.existsSync(path.join(cellsDirPath, 'roll-3.json')), 'roll-3 stays active — never reached');
    assert(
      !('roll-feat' in archivedSummary(aRoot)),
      'summary.json must be untouched — a preflight refusal never gets as far as writing it',
    );
    assert(
      !fs.existsSync(path.join(archiveDir, '.journal.json')),
      'no journal is ever written for a refusal that happens before the first rename',
    );
  } finally {
    fs.rmSync(aRoot, { recursive: true, force: true });
  }
});

await check('archiveFeature journal rollback recovers a simulated half-archive (crash mid-loop, no in-process rollback ever ran)', async () => {
  const aRoot = fs.mkdtempSync(path.join(os.tmpdir(), 'bee-archive-hard-journal-'));
  try {
    const cellsDirPath = path.join(aRoot, '.bee', 'cells');
    fs.mkdirSync(cellsDirPath, { recursive: true });
    const archiveDir = cellsArchiveDir(aRoot, 'jrn-feat');
    fs.mkdirSync(archiveDir, { recursive: true });
    const ids = ['jrn-1', 'jrn-2', 'jrn-3'];
    const cellsById = {};
    for (const id of ids) {
      cellsById[id] = makeCell(id, { feature: 'jrn-feat', status: 'capped' });
    }
    // Simulate a process killed HALFWAY through a real archiveFeature run:
    // jrn-1 already landed in the archive tree, jrn-2/jrn-3 are still active,
    // and a journal describing the FULL planned batch is left behind exactly
    // as writeArchiveJournal would leave it before the crash — no in-process
    // catch/rollback ever ran (that is the scenario a preflight check can
    // never protect against: the process is simply gone).
    fs.writeFileSync(path.join(archiveDir, 'jrn-1.json'), JSON.stringify(cellsById['jrn-1']));
    fs.writeFileSync(path.join(cellsDirPath, 'jrn-2.json'), JSON.stringify(cellsById['jrn-2']));
    fs.writeFileSync(path.join(cellsDirPath, 'jrn-3.json'), JSON.stringify(cellsById['jrn-3']));
    const planned = ids.map((id) => ({
      id,
      from: path.join(cellsDirPath, `${id}.json`),
      to: path.join(archiveDir, `${id}.json`),
    }));
    fs.writeFileSync(
      path.join(archiveDir, '.journal.json'),
      JSON.stringify({ op: 'archive', feature: 'jrn-feat', planned, started_at: new Date().toISOString() }),
    );

    // The NEXT archiveFeature call for this feature must first notice the
    // leftover journal, restore jrn-1 back to active (the only move that
    // actually completed), delete the stale journal, and THEN proceed with
    // a clean, fresh archive of all three cells.
    const result = await archiveFeature(aRoot, 'jrn-feat');
    assert(
      result.moved.slice().sort().join(',') === 'jrn-1,jrn-2,jrn-3',
      `all three cells must be archived after recovery, got ${JSON.stringify(result.moved)}`,
    );
    for (const id of ids) {
      assert(fs.existsSync(path.join(archiveDir, `${id}.json`)), `${id} must end up archived after recovery + fresh archive`);
      assert(!fs.existsSync(path.join(cellsDirPath, `${id}.json`)), `${id} must not remain active`);
    }
    assert(!fs.existsSync(path.join(archiveDir, '.journal.json')), 'the journal must be gone once the fresh archive completes');
    assert(archivedSummary(aRoot)['jrn-feat'].capped === 3, 'summary reflects the fresh archive, not the half-migrated leftover');
  } finally {
    fs.rmSync(aRoot, { recursive: true, force: true });
  }
});

await check('unarchiveFeature journal rollback also recovers a simulated half-unarchive, symmetric to the archive direction', async () => {
  const aRoot = fs.mkdtempSync(path.join(os.tmpdir(), 'bee-archive-hard-journal-un-'));
  try {
    const cellsDirPath = path.join(aRoot, '.bee', 'cells');
    fs.mkdirSync(cellsDirPath, { recursive: true });
    const archiveDir = cellsArchiveDir(aRoot, 'jru-feat');
    fs.mkdirSync(archiveDir, { recursive: true });
    // Simulate a crash HALFWAY through unarchiving: jru-1 already landed
    // active, jru-2 is still archived, with a journal describing both moves
    // — no in-process rollback ever ran (the process is simply gone).
    fs.writeFileSync(
      path.join(cellsDirPath, 'jru-1.json'),
      JSON.stringify(makeCell('jru-1', { feature: 'jru-feat', status: 'capped' })),
    );
    fs.writeFileSync(
      path.join(archiveDir, 'jru-2.json'),
      JSON.stringify(makeCell('jru-2', { feature: 'jru-feat', status: 'dropped' })),
    );
    const planned = [
      { id: 'jru-1', from: path.join(archiveDir, 'jru-1.json'), to: path.join(cellsDirPath, 'jru-1.json') },
      { id: 'jru-2', from: path.join(archiveDir, 'jru-2.json'), to: path.join(cellsDirPath, 'jru-2.json') },
    ];
    fs.writeFileSync(
      path.join(archiveDir, '.journal.json'),
      JSON.stringify({ op: 'unarchive', feature: 'jru-feat', planned, started_at: new Date().toISOString() }),
    );
    // Seed the summary as archiveFeature would have left it, so the
    // subsequent unarchiveFeature call has a coherent entry to clear.
    const summaryFile = path.join(aRoot, '.bee', 'cells', 'archive', 'summary.json');
    fs.writeFileSync(summaryFile, JSON.stringify({ 'jru-feat': { capped: 1, dropped: 1, archived_at: new Date().toISOString() } }));

    // The NEXT unarchiveFeature call must first notice the leftover journal,
    // restore jru-1 back to the archive tree (the only move that actually
    // completed), delete the stale journal, and THEN proceed with a clean,
    // fresh unarchive of both cells.
    const moved = await unarchiveFeature(aRoot, 'jru-feat');
    assert(
      moved.slice().sort().join(',') === 'jru-1,jru-2',
      `both cells must end up active after recovery, got ${JSON.stringify(moved)}`,
    );
    for (const id of ['jru-1', 'jru-2']) {
      assert(fs.existsSync(path.join(cellsDirPath, `${id}.json`)), `${id} must end up active after recovery + fresh unarchive`);
    }
    assert(!fs.existsSync(path.join(archiveDir, '.journal.json')), 'the journal must be gone once the fresh unarchive completes');
    assert(!('jru-feat' in archivedSummary(aRoot)), 'summary entry must be cleared by the fresh unarchive');
  } finally {
    fs.rmSync(aRoot, { recursive: true, force: true });
  }
});

await check('claimCell refuses a cell with uncapped deps even after gate approval', async () => {
  const state = readState(root);
  state.phase = 'swarming';
  state.approved_gates.execution = true;
  writeState(root, state);
  await assertRejects(() => claimCell(root, 'demo-2', 'worker-a'), 'uncapped deps', 'dep lock');
});

await check('claimCell claims an open, dep-free cell', async () => {
  const cell = await claimCell(root, 'demo-1', 'worker-a');
  assert(cell.status === 'claimed', 'status should be claimed');
  assert(cell.trace.worker === 'worker-a', 'worker recorded');
});

// ─── D1: claimCellCrossSession is the path `cells claim --id` now runs ────
// through (bee.mjs handleCellsClaim) — the claim file is acquired BEFORE the
// cell JSON flips, so a losing concurrent claimant gets a typed CLAIMED
// refusal instead of silently double-claiming. D3: a null/absent sessionId is
// a legal sessionless claim (single-session use is unaffected).

await check('claimCellCrossSession backs "cells claim --id": winner claims both the cell JSON and the claims-store file; a second claimant on the SAME cell gets typed CLAIMED naming the winner + expiry, and the cell is untouched', async () => {
  addCell(root, makeCell('claimx-1'));
  const first = await claimCellCrossSession(root, { sessionId: 'sess-x1', worker: 'worker-x1', cellId: 'claimx-1' });
  assert(first.ok === true, `first claimant should win, got ${JSON.stringify(first)}`);
  assert(first.cell.status === 'claimed' && first.cell.trace.worker === 'worker-x1', 'cell JSON reflects the winner');
  assert(first.claim.session === 'sess-x1', 'claims-store file belongs to the winner');
  assert(readCell(root, 'claimx-1').status === 'claimed', 'on-disk cell is claimed');

  const second = await claimCellCrossSession(root, { sessionId: 'sess-x2', worker: 'worker-x2', cellId: 'claimx-1' });
  assert(second.ok === false && second.code === 'CLAIMED', `second claimant must lose with typed CLAIMED, got ${JSON.stringify(second)}`);
  assert(second.reason.includes('sess-x1'), 'refusal names the actual owner');
  assert(/expir/i.test(second.reason), 'refusal names the expiry');
  assert(readCell(root, 'claimx-1').trace.worker === 'worker-x1', 'the losing attempt never touched the cell JSON');
});

await check('claimCellCrossSession with sessionId null/undefined is a legal sessionless claim — the claim file omits "session" entirely; single-session flow (no env id) still claims successfully', async () => {
  addCell(root, makeCell('claimx-2'));
  const result = await claimCellCrossSession(root, { sessionId: null, worker: 'worker-sessionless', cellId: 'claimx-2' });
  assert(result.ok === true, `a null sessionId must still succeed, got ${JSON.stringify(result)}`);
  assert(!('session' in result.claim), 'the sessionless claim record omits "session" entirely');
  assert(result.cell.status === 'claimed', 'the cell is claimed regardless of session');
});

// ─── cells: test-stamped capping (test-simple, decision 412e9b3a) ──────────
// capCell no longer reads any per-cell verify/evidence machinery — the one
// test door (the declared commands.test run) lives in bee.mjs's shared
// cap/finish handler, which refuses BEFORE capCell on a red run and passes
// the green/undeclared stamp through capCell's `tests` param.

await check('capCell caps a claimed cell directly (no per-cell verify machinery) and unlocks dependents', async () => {
  const cell = await capCell(root, 'demo-1', { files_changed: ['src/x.js'], outcome: 'done' });
  assert(cell.status === 'capped', 'demo-1 capped');
  const ready = readyCells(root, 'demo').map((c) => c.id);
  assert(ready.includes('demo-2'), 'demo-2 becomes ready once its dep is capped');
});

await check('capCell stamps a green tests run verbatim on the trace (tests param, test-simple)', async () => {
  addCell(root, makeCell('tests-stamp-1'));
  await claimCell(root, 'tests-stamp-1', 'worker-t');
  const capped = await capCell(root, 'tests-stamp-1', {
    files_changed: ['a.js'],
    outcome: 'done',
    tests: { tests: 'green', results: '.bee/logs/test-results.json', ran_at: '2026-07-31T00:00:00.000Z' },
  });
  assert(capped.trace.tests === 'green', `trace.tests should be "green", got ${JSON.stringify(capped.trace.tests)}`);
  assert(capped.trace.results === '.bee/logs/test-results.json', `trace.results should point at the record, got ${JSON.stringify(capped.trace.results)}`);
  assert(capped.trace.ran_at === '2026-07-31T00:00:00.000Z', `trace.ran_at carried verbatim, got ${JSON.stringify(capped.trace.ran_at)}`);
});

await check('capCell stamps {tests: "undeclared"} for a no-commands.test repo, and stamps nothing when the tests param is absent', async () => {
  addCell(root, makeCell('tests-stamp-2'));
  await claimCell(root, 'tests-stamp-2', 'worker-t');
  const undeclared = await capCell(root, 'tests-stamp-2', { files_changed: ['a.js'], outcome: 'done', tests: { tests: 'undeclared' } });
  assert(undeclared.trace.tests === 'undeclared', `expected the undeclared stamp, got ${JSON.stringify(undeclared.trace.tests)}`);
  addCell(root, makeCell('tests-stamp-3'));
  await claimCell(root, 'tests-stamp-3', 'worker-t');
  const bare = await capCell(root, 'tests-stamp-3', { files_changed: ['a.js'], outcome: 'done' });
  assert(!('tests' in bare.trace), `a programmatic cap that ran nothing stamps no tests field, got ${JSON.stringify(bare.trace.tests)}`);
});

await check('capCell on a high-risk cell requires files_changed and outcome', async () => {
  addCell(
    root,
    makeCell('hr-1', {
      lane: 'high-risk',
      must_haves: { truths: ['Auth still works'], artifacts: [], key_links: [], prohibitions: [] },
    }),
  );
  await claimCell(root, 'hr-1', 'worker-b');
  await assertRejects(() => capCell(root, 'hr-1', {}), 'high-risk', 'high-risk trace tier');
  await capCell(root, 'hr-1', { files_changed: ['src/auth.js'], outcome: 'auth guard added' });
  assert(readCell(root, 'hr-1').status === 'capped', 'hr-1 capped with full trace');
});

await check('capCell refuses a small cell with empty files_changed (the door test-simple deliberately KEPT: it asks what the worker touched)', async () => {
  addCell(root, makeCell('ev-2'));
  await claimCell(root, 'ev-2', 'worker-c');
  await assertRejects(
    () => capCell(root, 'ev-2', { outcome: 'done' }),
    'files_changed',
    'empty files_changed must be refused for small+',
  );
  await capCell(root, 'ev-2', { files_changed: ['src/y.js'], outcome: 'done' });
  assert(readCell(root, 'ev-2').status === 'capped', 'ev-2 caps once files are recorded');
});

await check('tiny lane caps without files_changed (lanes scale strictness)', async () => {
  addCell(root, makeCell('tiny-1', { lane: 'tiny' }));
  await claimCell(root, 'tiny-1', 'worker-c');
  await capCell(root, 'tiny-1', { outcome: 'typo fixed' });
  assert(readCell(root, 'tiny-1').status === 'capped', 'tiny cell capped without output/files');
});

await check('capCell honors the cell-declared behavior_change (grooming fix — it classifies the cap for scribing debt)', async () => {
  addCell(root, makeCell('bc-decl', { behavior_change: true }));
  await claimCell(root, 'bc-decl', 'worker-c');
  const capped = await capCell(root, 'bc-decl', { files_changed: ['a.js'], outcome: 'done' });
  assert(capped.trace.behavior_change === true, 'trace.behavior_change carried from the cell declaration');
});

// ─── E6 (derived-check-hardening): behavior_change resolves from either the
// top-level field OR trace.behavior_change — live evidence was vd-9/vd-10/
// vd-11, each authored with ONLY trace.behavior_change: true and each capped
// silently missing scribing-debt detection.

await check('capCell resolves behavior_change from trace.behavior_change when the top-level field is unset (E6)', async () => {
  addCell(root, makeCell('bc-trace-only', { trace: { behavior_change: true } }));
  assert(readCell(root, 'bc-trace-only').behavior_change === undefined, 'fixture carries no top-level behavior_change');
  await claimCell(root, 'bc-trace-only', 'worker-e6');
  const capped = await capCell(root, 'bc-trace-only', { files_changed: ['a.js'], outcome: 'done' });
  assert(capped.trace.behavior_change === true, 'trace-only declaration resolved to true at cap (E6 fix)');
});

await check('capCell keeps an explicit top-level behavior_change:false even when trace.behavior_change is true (E6)', async () => {
  addCell(root, makeCell('bc-top-wins', { behavior_change: false, trace: { behavior_change: true } }));
  await claimCell(root, 'bc-top-wins', 'worker-e6');
  const capped = await capCell(root, 'bc-top-wins', { files_changed: ['a.js'], outcome: 'done' });
  assert(capped.status === 'capped', 'bc-top-wins caps normally');
  assert(capped.trace.behavior_change === false, 'explicit top-level false wins over a conflicting trace value');
});

await check('isKnownPhase accepts the enum + terminal alias and rejects drift', async () => {
  assert(isKnownPhase('swarming') === true, 'enum phase accepted');
  assert(isKnownPhase('compounding-complete') === true, 'terminal alias accepted');
  assert(isKnownPhase('merged') === false, 'invented phase rejected');
});

await check('blockCell records the reason', async () => {
  addCell(root, makeCell('blk-1'));
  await blockCell(root, 'blk-1', 'reservation conflict');
  assert(readCell(root, 'blk-1').status === 'blocked', 'blk-1 blocked');
});

// ─── D1: revision ledger (trace.attempts) + failure-signature normalizer ──

await check('normalizeFailureSignature is deterministic for the same logical failure under timestamp/path/hex noise, and differs for a different failure', async () => {
  const a = 'FAIL 2026-07-19T10:22:03.451Z /home/alice/repo/src/foo.js assertion abc123def456 failed';
  const b = 'FAIL 2026-07-20T02:11:59Z /Users/bob/work/src/foo.js assertion 9f8e7d6c5b4a failed';
  const sigA = normalizeFailureSignature(a);
  const sigB = normalizeFailureSignature(b);
  assert(sigA === sigB, `same logical failure under timestamp/path/hex noise should normalize identically, got ${sigA} vs ${sigB}`);
  assert(/^[0-9a-f]{12}$/.test(sigA), `signature should be 12 lowercase hex chars, got "${sigA}"`);
  const different = normalizeFailureSignature('FAIL totally unrelated assertion blew up');
  assert(different !== sigA, 'a genuinely different failure must normalize to a different signature');
  const empty = normalizeFailureSignature(null);
  assert(/^[0-9a-f]{12}$/.test(empty), `null output still normalizes to a stable 12-hex signature, got "${empty}"`);
  assert(empty === normalizeFailureSignature(''), 'null and empty-string output normalize identically');
});

await check('normalizeFailureSignature prefers the first FAIL/Error/refus/denied line over surrounding noise', async () => {
  const output = 'running suite...\n3/45 passed\nFAIL assertion mismatch on line 12\nmore trailing noise';
  const sig = normalizeFailureSignature(output);
  assert(sig === normalizeFailureSignature('other run\nFAIL assertion mismatch on line 12\ndone'), 'the picked diagnostic line ignores unrelated surrounding noise');
});

await check('recordTestsRedAttempt appends a tests-red ledger entry — note carries the excerpt first line, failure_signature derives from it, claim_session/claimed_at from the live claim (test-simple)', async () => {
  addCell(root, makeCell('ledger-1'));
  const claimed = await claimCellCrossSession(root, { sessionId: 'sess-ledger-1', worker: 'worker-ledger', cellId: 'ledger-1' });
  assert(claimed.ok === true, `precondition: claim should succeed, got ${JSON.stringify(claimed)}`);
  const liveClaim = readClaim(root, 'ledger-1');

  await recordTestsRedAttempt(root, 'ledger-1', { excerptLine: 'FAIL first red run' });
  const afterFirst = readCell(root, 'ledger-1');
  assert(Array.isArray(afterFirst.trace.attempts) && afterFirst.trace.attempts.length === 1, `expected 1 attempt, got ${JSON.stringify(afterFirst.trace.attempts)}`);
  const entry1 = afterFirst.trace.attempts[0];
  assert(entry1.n === 1, `first entry n should be 1, got ${entry1.n}`);
  assert(entry1.verdict === 'tests-red', `first entry verdict should be tests-red, got ${entry1.verdict}`);
  assert(entry1.note === 'FAIL first red run', `note carries the excerpt first line verbatim, got ${entry1.note}`);
  assert(entry1.failure_signature === normalizeFailureSignature('FAIL first red run'), 'signature derives from the excerpt line via the shared normalizer');
  assert(entry1.claim_session === 'sess-ledger-1', `claim_session should come from the live claim, got ${entry1.claim_session}`);
  assert(entry1.claimed_at === liveClaim.claimed_at, `claimed_at should be copied from the live claim file, got ${entry1.claimed_at}`);
  assert(entry1.worker === 'worker-ledger', `worker should carry the claiming worker, got ${entry1.worker}`);
  assert('at' in entry1 && typeof entry1.at === 'string', 'entry carries its own timestamp');

  await recordTestsRedAttempt(root, 'ledger-1', { excerptLine: 'FAIL second red run' });
  const afterSecond = readCell(root, 'ledger-1');
  assert(afterSecond.trace.attempts.length === 2, `expected 2 attempts, got ${afterSecond.trace.attempts.length}`);
  assert(afterSecond.trace.attempts[0].failure_signature === entry1.failure_signature, 'the first entry is never rewritten by a later append');
  assert(afterSecond.trace.attempts[1].n === 2, `second entry n should be 2, got ${afterSecond.trace.attempts[1].n}`);
});

await check('recordTestsRedAttempt tolerates a null excerpt (no output at all) and refuses an unknown cell id', async () => {
  addCell(root, makeCell('ledger-red-noexcerpt'));
  await claimCell(root, 'ledger-red-noexcerpt', 'worker-red');
  await recordTestsRedAttempt(root, 'ledger-red-noexcerpt', {});
  const entry = readCell(root, 'ledger-red-noexcerpt').trace.attempts[0];
  assert(entry.verdict === 'tests-red' && entry.note === null && entry.failure_signature === null, `a null excerpt records a bare tests-red entry, got ${JSON.stringify(entry)}`);
  await assertRejects(() => recordTestsRedAttempt(root, 'no-such-red-cell', { excerptLine: 'x' }), 'not found', 'unknown cell id refuses');
});

await check('blockCell appends a "blocked" ledger entry whose note is the block reason and whose failure_signature derives from it', async () => {
  addCell(root, makeCell('ledger-block-1'));
  await claimCell(root, 'ledger-block-1', 'worker-block');
  await blockCell(root, 'ledger-block-1', 'reservation conflict on src/x.js');
  const entry = readCell(root, 'ledger-block-1').trace.attempts[0];
  assert(entry.verdict === 'blocked', `expected blocked verdict, got ${entry.verdict}`);
  assert(entry.note === 'reservation conflict on src/x.js', `note should carry the block reason verbatim, got ${entry.note}`);
  assert(entry.failure_signature === normalizeFailureSignature('reservation conflict on src/x.js'), 'blockCell signature derives from the reason via the same normalizer');
});

await check('a sessionless claim records claim_session null but still carries the live claim file\'s claimed_at (D1+Δ1 undercount-safe, F2)', async () => {
  addCell(root, makeCell('ledger-sessionless-1'));
  const claimed = await claimCellCrossSession(root, { sessionId: null, worker: 'worker-sl', cellId: 'ledger-sessionless-1' });
  assert(claimed.ok === true, `precondition: sessionless claim should succeed, got ${JSON.stringify(claimed)}`);
  await recordTestsRedAttempt(root, 'ledger-sessionless-1', { excerptLine: 'FAIL x' });
  const entry = readCell(root, 'ledger-sessionless-1').trace.attempts[0];
  assert(entry.claim_session === null, `sessionless claim must record claim_session null, got ${entry.claim_session}`);
  assert(typeof entry.claimed_at === 'string' && entry.claimed_at.length > 0, 'claimed_at is still copied from the live (sessionless) claim file');
});

await check('trace.attempts entries survive capCell — appended to, never dropped by the trace spread', async () => {
  addCell(root, makeCell('ledger-cap-1'));
  await claimCell(root, 'ledger-cap-1', 'worker-cap');
  await recordTestsRedAttempt(root, 'ledger-cap-1', { excerptLine: 'FAIL once' });
  await recordTestsRedAttempt(root, 'ledger-cap-1', { excerptLine: 'FAIL twice, differently' });
  const capped = await capCell(root, 'ledger-cap-1', { files_changed: ['a.js'], outcome: 'done' });
  assert(Array.isArray(capped.trace.attempts) && capped.trace.attempts.length === 2, `capCell must preserve every prior ledger entry, got ${JSON.stringify(capped.trace.attempts)}`);
  assert(capped.trace.attempts[0].verdict === 'tests-red' && capped.trace.attempts[1].verdict === 'tests-red', 'entry order and verdicts survive cap byte-unchanged');
});

await check('updateCell refuses a {trace:{...}} patch on an open cell — the ledger cannot be edited around (D1+F1: trace is already frozen wholesale)', async () => {
  addCell(root, makeCell('ledger-update-1'));
  await claimCell(root, 'ledger-update-1', 'worker-upd');
  await blockCell(root, 'ledger-update-1', 'stuck', { sessionId: undefined });
  const file = path.join(root, '.bee', 'cells', 'ledger-update-1.json');
  const before = fs.readFileSync(file, 'utf8');
  await assertRejects(
    () => updateCell(root, 'ledger-update-1', { title: 'ok', trace: { attempts: [] } }),
    'frozen',
    'a patch attempting to touch trace.attempts must be refused wholesale, cell untouched',
  );
  assert(fs.readFileSync(file, 'utf8') === before, 'ledger-update-1 file byte-unchanged after the refused trace patch');
});

// ─── D2 (self-correcting-loop): cell-lifetime budgets at the claim door ────
// max_claims/max_failed_attempts/max_same_signature enforced INSIDE the
// O_EXCL critical section of claimCellCrossSession, typed CELL_BUDGET_
// EXHAUSTED/REPEATED_FAILURE refusals, an audited reset-budget door, and
// claim-next SELECTION skipping bricked candidates (Δ3/F3) so the pool
// never bricks.

await check('claimCellCrossSession: a fresh cell with no attempts ledger claims exactly as today — D2 defaults never bite on a first claim (D6 compatibility floor)', async () => {
  addCell(root, makeCell('budget-fresh-1'));
  const result = await claimCellCrossSession(root, { sessionId: 'sess-budget-fresh', worker: 'w', cellId: 'budget-fresh-1' });
  assert(result.ok === true, `first claim on a fresh cell must succeed under default budgets, got ${JSON.stringify(result)}`);
});

await check('claimCellCrossSession: 3 claims exhaust the default max_claims budget — a 4th claim is refused typed CELL_BUDGET_EXHAUSTED naming the budget, and the just-acquired claim file is unwound (D2+Δ2)', async () => {
  addCell(root, makeCell('budget-claims-1'));
  for (let i = 0; i < 3; i += 1) {
    const claimed = await claimCellCrossSession(root, { sessionId: `sess-budget-claims-${i}`, worker: 'w', cellId: 'budget-claims-1' });
    assert(claimed.ok === true, `claim #${i + 1} should succeed under the default budget of 3, got ${JSON.stringify(claimed)}`);
    await recordTestsRedAttempt(root, 'budget-claims-1', { excerptLine: `FAIL distinct red number ${i}` });
    await unclaimCell(root, 'budget-claims-1', { sessionId: `sess-budget-claims-${i}` });
  }
  const fourth = await claimCellCrossSession(root, { sessionId: 'sess-budget-claims-3', worker: 'w', cellId: 'budget-claims-1' });
  assert(fourth.ok === false, `the 4th claim must be refused, got ${JSON.stringify(fourth)}`);
  assert(fourth.code === 'CELL_BUDGET_EXHAUSTED', `expected CELL_BUDGET_EXHAUSTED, got ${fourth.code}`);
  assert(fourth.budget && fourth.budget.name === 'max_claims', `refusal must name the exhausted budget, got ${JSON.stringify(fourth.budget)}`);
  assert(typeof fourth.fix === 'string' && fourth.fix.includes('reset-budget'), `refusal must name the reset door, got ${fourth.fix}`);
  assert(readClaim(root, 'budget-claims-1') === null, 'a refused claim must not leave an orphaned claim file behind (Δ2 unwind precedent cells.mjs:951)');
  assert(readCell(root, 'budget-claims-1').status === 'open', 'the cell itself stays untouched on a refused claim — only the transient claim-file acquisition was unwound');
});

await check('claimCellCrossSession: two failed attempts sharing an identical failure_signature refuse the NEXT claim typed REPEATED_FAILURE, independent of the max_claims/max_failed_attempts budgets (D2 same-signature)', async () => {
  addCell(root, makeCell('budget-sig-1'));
  for (let i = 0; i < 2; i += 1) {
    await claimCellCrossSession(root, { sessionId: `sess-budget-sig-${i}`, worker: 'w', cellId: 'budget-sig-1' });
    await recordTestsRedAttempt(root, 'budget-sig-1', { excerptLine: 'FAIL identical assertion' });
    await unclaimCell(root, 'budget-sig-1', { sessionId: `sess-budget-sig-${i}` });
  }
  const third = await claimCellCrossSession(root, { sessionId: 'sess-budget-sig-2', worker: 'w', cellId: 'budget-sig-1' });
  assert(third.ok === false, `a claim after 2 identical-signature fails must refuse, got ${JSON.stringify(third)}`);
  assert(third.code === 'REPEATED_FAILURE', `expected REPEATED_FAILURE, got ${third.code}`);
  assert(third.signature === normalizeFailureSignature('FAIL identical assertion'), `refusal must name the repeated signature, got ${third.signature}`);
  assert(typeof third.fix === 'string' && third.fix.includes('reset-budget'), `refusal must name the reset door, got ${third.fix}`);
});

await check('claimCellCrossSession: an explicit per-cell budgets override is honored over the defaults (D2)', async () => {
  addCell(root, makeCell('budget-custom-1', { budgets: { max_claims: 1, max_failed_attempts: 4, max_same_signature: 2 } }));
  const first = await claimCellCrossSession(root, { sessionId: 'sess-budget-custom-0', worker: 'w', cellId: 'budget-custom-1' });
  assert(first.ok === true, `first claim under a custom max_claims:1 should succeed, got ${JSON.stringify(first)}`);
  await recordTestsRedAttempt(root, 'budget-custom-1', { excerptLine: 'FAIL custom budget red' });
  await unclaimCell(root, 'budget-custom-1', { sessionId: 'sess-budget-custom-0' });
  const second = await claimCellCrossSession(root, { sessionId: 'sess-budget-custom-1', worker: 'w', cellId: 'budget-custom-1' });
  assert(second.ok === false && second.code === 'CELL_BUDGET_EXHAUSTED', `a 2nd claim under a custom max_claims:1 must refuse, got ${JSON.stringify(second)}`);
  assert(second.budget.limit === 1, `refusal must reflect the cell's own override, not the default, got ${JSON.stringify(second.budget)}`);
});

await check('resetCellBudget: audited reset appends a budget_resets marker, logs a decision, never touches attempts, and reopens the claim door (D2)', async () => {
  addCell(root, makeCell('budget-reset-1'));
  for (let i = 0; i < 3; i += 1) {
    await claimCellCrossSession(root, { sessionId: `sess-budget-reset-${i}`, worker: 'w', cellId: 'budget-reset-1' });
    await recordTestsRedAttempt(root, 'budget-reset-1', { excerptLine: `FAIL reset-fixture red number ${i}` });
    await unclaimCell(root, 'budget-reset-1', { sessionId: `sess-budget-reset-${i}` });
  }
  const blocked = await claimCellCrossSession(root, { sessionId: 'sess-budget-reset-3', worker: 'w', cellId: 'budget-reset-1' });
  assert(blocked.ok === false && blocked.code === 'CELL_BUDGET_EXHAUSTED', `precondition: the door should be exhausted, got ${JSON.stringify(blocked)}`);
  const attemptsBefore = readCell(root, 'budget-reset-1').trace.attempts.length;

  const reset = await resetCellBudget(root, 'budget-reset-1', 'manager approved a genuine retry', { operator: 'manager-1' });
  assert(Array.isArray(reset.trace.budget_resets) && reset.trace.budget_resets.length === 1, `reset must append exactly one budget_resets entry, got ${JSON.stringify(reset.trace.budget_resets)}`);
  assert(reset.trace.budget_resets[0].reason === 'manager approved a genuine retry', 'the reset reason is recorded verbatim');
  assert(reset.trace.budget_resets[0].by_actor === 'manager-1', `reset must record the acting operator as by_actor, got ${JSON.stringify(reset.trace.budget_resets[0])}`);
  assert(reset.trace.attempts.length === attemptsBefore, 'reset never rewrites or drops any attempts ledger entry');

  const decisions = activeDecisions(root, { recent: 1 });
  assert(decisions.length > 0 && decisions[0].decision.includes('budget-reset-1'), `resetCellBudget must log a decision naming the cell, got ${JSON.stringify(decisions)}`);

  const reopened = await claimCellCrossSession(root, { sessionId: 'sess-budget-reset-4', worker: 'w', cellId: 'budget-reset-1' });
  assert(reopened.ok === true, `after reset the door must reopen for a fresh claim, got ${JSON.stringify(reopened)}`);
});

await check('resetCellBudget requires a non-empty reason, and refuses an unknown cell id', async () => {
  addCell(root, makeCell('budget-reset-noreason-1'));
  await assertRejects(() => resetCellBudget(root, 'budget-reset-noreason-1', ''), 'reason', 'resetCellBudget must refuse an empty reason');
  await assertRejects(() => resetCellBudget(root, 'budget-reset-noreason-1', '   '), 'reason', 'resetCellBudget must refuse a whitespace-only reason');
  await assertRejects(() => resetCellBudget(root, 'no-such-cell-budget', 'a reason'), 'not found', 'resetCellBudget must refuse an unknown cell id');
});

// ─── D-GHF-C (GH #27.3+4): budget hard clamps + guarded, audit-first reset ──

await check('resolveCellBudgets (D-GHF-C): clamps a declared value above the hard max down to it, and falls back to DEFAULT_BUDGETS for a non-integer or below-floor declared value — no path ever raises a budget above the hard max', () => {
  assert(resolveCellBudgets({}).max_claims === 3, `no declared budgets -> DEFAULT max_claims 3, got ${resolveCellBudgets({}).max_claims}`);

  const clampedHigh = resolveCellBudgets({
    budgets: { max_claims: 999999, max_failed_attempts: 999999, max_same_signature: 999999 },
  });
  assert(clampedHigh.max_claims === 9, `max_claims must clamp to hard max 9, got ${clampedHigh.max_claims}`);
  assert(clampedHigh.max_failed_attempts === 12, `max_failed_attempts must clamp to hard max 12, got ${clampedHigh.max_failed_attempts}`);
  assert(clampedHigh.max_same_signature === 6, `max_same_signature must clamp to hard max 6, got ${clampedHigh.max_same_signature}`);

  // boundary: exactly at the hard max is honored verbatim; one above clamps down.
  assert(resolveCellBudgets({ budgets: { max_claims: 9 } }).max_claims === 9, 'a declared value exactly at the hard max is honored verbatim');
  assert(resolveCellBudgets({ budgets: { max_claims: 10 } }).max_claims === 9, 'one above the hard max clamps down to it');

  // non-integer / below-floor declared values fall back to DEFAULT, not the hard max.
  assert(resolveCellBudgets({ budgets: { max_claims: 2.5 } }).max_claims === 3, 'a non-integer declared value falls back to DEFAULT (3), never clamps');
  assert(resolveCellBudgets({ budgets: { max_claims: 0 } }).max_claims === 3, 'a zero declared value falls back to DEFAULT');
  assert(resolveCellBudgets({ budgets: { max_claims: -5 } }).max_claims === 3, 'a negative declared value falls back to DEFAULT');
  assert(resolveCellBudgets({ budgets: { max_claims: 'nine' } }).max_claims === 3, 'a non-numeric declared value falls back to DEFAULT');
});

await check('addCell (validateNewCell, D-GHF-C): refuses a malformed budgets object at authoring time — non-plain-object, unknown key, non-integer, and over-hard-max values are all rejected before write; a valid at-hard-max object is accepted verbatim', () => {
  assertThrows(() => addCell(root, makeCell('budget-authoring-array-1', { budgets: [1, 2, 3] })), 'budgets', 'an array budgets value must be refused');
  assertThrows(() => addCell(root, makeCell('budget-authoring-unknown-1', { budgets: { max_claims: 3, bogus_key: 1 } })), 'budgets', 'an unknown budgets key must be refused');
  assertThrows(() => addCell(root, makeCell('budget-authoring-noninteger-1', { budgets: { max_claims: 2.5 } })), 'budgets', 'a non-integer budgets value must be refused');
  assertThrows(() => addCell(root, makeCell('budget-authoring-toohigh-1', { budgets: { max_claims: 10 } })), 'budgets', 'a budgets value above the hard max must be refused');
  assertThrows(() => addCell(root, makeCell('budget-authoring-zero-1', { budgets: { max_claims: 0 } })), 'budgets', 'a zero budgets value must be refused');
  assert(readCell(root, 'budget-authoring-array-1') === null, 'a refused authoring call must not leave a partial cell file behind');

  const ok = addCell(root, makeCell('budget-authoring-ok-1', { budgets: { max_claims: 9, max_failed_attempts: 12, max_same_signature: 6 } }));
  assert(ok.budgets.max_claims === 9 && ok.budgets.max_failed_attempts === 12 && ok.budgets.max_same_signature === 6, `a valid at-hard-max budgets object must be accepted verbatim, got ${JSON.stringify(ok.budgets)}`);
});

await check('resetCellBudget (D-GHF-C): refused, typed RESET_NOT_NEEDED, on a cell that is not actually budget-blocked — checkCellBudgets(cell).ok stays the door, not a bare reason string', async () => {
  addCell(root, makeCell('budget-reset-healthy-1'));
  assert(checkCellBudgets(readCell(root, 'budget-reset-healthy-1')).ok === true, 'precondition: a fresh cell is not budget-blocked');
  let caught = null;
  try {
    await resetCellBudget(root, 'budget-reset-healthy-1', 'trying to reset anyway', { operator: 'manager-1' });
  } catch (error) {
    caught = error;
  }
  assert(caught !== null, 'resetCellBudget must refuse a reset on a healthy (non-budget-blocked) cell');
  assert(caught.code === 'RESET_NOT_NEEDED', `refusal must be typed RESET_NOT_NEEDED, got ${JSON.stringify(caught.code)}`);
  const after = readCell(root, 'budget-reset-healthy-1');
  assert(!Array.isArray(after.trace.budget_resets) || after.trace.budget_resets.length === 0, 'a refused reset must not append a budget_resets entry');
});

await check('resetCellBudget (D-GHF-C): refused without an actor — neither --operator nor BEE_AGENT_NAME supplied, message names both options', async () => {
  addCell(root, makeCell('budget-reset-noactor-1'));
  const savedEnv = process.env.BEE_AGENT_NAME;
  delete process.env.BEE_AGENT_NAME;
  try {
    await assertRejects(
      () => resetCellBudget(root, 'budget-reset-noactor-1', 'a reason'),
      'operator',
      'resetCellBudget must refuse without an actor, naming --operator',
    );
    await assertRejects(
      () => resetCellBudget(root, 'budget-reset-noactor-1', 'a reason'),
      'BEE_AGENT_NAME',
      'resetCellBudget must refuse without an actor, naming BEE_AGENT_NAME',
    );
  } finally {
    if (savedEnv === undefined) delete process.env.BEE_AGENT_NAME;
    else process.env.BEE_AGENT_NAME = savedEnv;
  }
  const after = readCell(root, 'budget-reset-noactor-1');
  assert(!Array.isArray(after.trace.budget_resets) || after.trace.budget_resets.length === 0, 'a refused actor-less reset must not append a budget_resets entry');
});

await check('resetCellBudget (D-GHF-C): the BEE_AGENT_NAME env fallback supplies the actor when --operator is omitted', async () => {
  addCell(root, makeCell('budget-reset-envactor-1'));
  for (let i = 0; i < 3; i += 1) {
    await claimCellCrossSession(root, { sessionId: `sess-envactor-${i}`, worker: 'w', cellId: 'budget-reset-envactor-1' });
    await recordTestsRedAttempt(root, 'budget-reset-envactor-1', { excerptLine: `FAIL env-actor red number ${i}` });
    await unclaimCell(root, 'budget-reset-envactor-1', { sessionId: `sess-envactor-${i}` });
  }
  const blocked = await claimCellCrossSession(root, { sessionId: 'sess-envactor-3', worker: 'w', cellId: 'budget-reset-envactor-1' });
  assert(blocked.ok === false, 'precondition: the door should be exhausted');
  const savedEnv = process.env.BEE_AGENT_NAME;
  process.env.BEE_AGENT_NAME = 'env-actor-1';
  try {
    const reset = await resetCellBudget(root, 'budget-reset-envactor-1', 'env fallback actor test');
    assert(reset.trace.budget_resets[0].by_actor === 'env-actor-1', `expected the BEE_AGENT_NAME env value as the actor, got ${JSON.stringify(reset.trace.budget_resets[0])}`);
  } finally {
    if (savedEnv === undefined) delete process.env.BEE_AGENT_NAME;
    else process.env.BEE_AGENT_NAME = savedEnv;
  }
});

await check('resetCellBudget (D-GHF-C): writes the audit decision BEFORE the cell write — a forced writeCell failure still leaves the decision recorded, and the cell file itself is untouched', async () => {
  // Root-skip (hardening-1-7-10 D1): this check simulates a write failure by
  // chmod-ing the cells dir to 0o555 (no write bit). On Linux, root (euid 0 —
  // some CI/container runners execute as root by default) bypasses directory
  // permission checks entirely, so the forced write would actually SUCCEED
  // instead of throwing, and `assert(threw, ...)` below would fail — not
  // because the audit-order behavior regressed, but because the simulation
  // itself cannot fire under root. Skip loudly rather than let a root runner
  // report a false red (or silently weaken the assertion for everyone else).
  if (process.geteuid?.() === 0) {
    console.log(
      'SKIP  resetCellBudget audit-order chmod(0o555) write-failure simulation: running as root (euid 0) — chmod cannot block root writes, so this simulation cannot fire. Skipped loudly, not weakened.',
    );
    return;
  }
  if (process.platform === 'win32') {
    console.log(
      'SKIP  resetCellBudget audit-order chmod(0o555) write-failure simulation: Windows — chmod is a no-op on directories, so the forced write failure cannot fire. Skipped loudly, not weakened.',
    );
    return;
  }
  const dir = makeStateRepo('bee-budget-audit-order-');
  try {
    writeJsonAtomic(path.join(dir, '.bee', 'state.json'), {
      schema_version: '1.0',
      phase: 'swarming',
      feature: 'demo-feat',
      mode: 'standard',
      approved_gates: { context: true, shape: true, execution: true, review: false },
      workers: [],
    });
    addCell(dir, makeCell('budget-audit-order-1'));
    for (let i = 0; i < 3; i += 1) {
      await claimCellCrossSession(dir, { sessionId: `sess-audit-order-${i}`, worker: 'w', cellId: 'budget-audit-order-1' });
      await recordTestsRedAttempt(dir, 'budget-audit-order-1', { excerptLine: `FAIL audit-order red number ${i}` });
      await unclaimCell(dir, 'budget-audit-order-1', { sessionId: `sess-audit-order-${i}` });
    }
    const blocked = await claimCellCrossSession(dir, { sessionId: 'sess-audit-order-3', worker: 'w', cellId: 'budget-audit-order-1' });
    assert(blocked.ok === false, 'precondition: the door should be exhausted');
    const before = fs.readFileSync(path.join(dir, '.bee', 'cells', 'budget-audit-order-1.json'), 'utf8');

    const cellsDir = path.join(dir, '.bee', 'cells');
    fs.chmodSync(cellsDir, 0o555);
    let threw = false;
    try {
      await resetCellBudget(dir, 'budget-audit-order-1', 'forced write failure test', { operator: 'test-op' });
    } catch {
      threw = true;
    } finally {
      fs.chmodSync(cellsDir, 0o755);
    }
    assert(threw, 'a forced writeCell failure must propagate as a rejection, not be silently swallowed');

    const after = fs.readFileSync(path.join(dir, '.bee', 'cells', 'budget-audit-order-1.json'), 'utf8');
    assert(after === before, 'the cell file itself must be byte-unchanged — the write never actually landed');

    const decisions = activeDecisions(dir, { recent: 1 });
    assert(
      decisions.length > 0 && decisions[0].decision.includes('budget-audit-order-1'),
      `the audit decision must exist even though the cell write failed, got ${JSON.stringify(decisions)}`,
    );
  } finally {
    fs.rmSync(dir, { recursive: true, force: true });
  }
});

await check(
  'gate_bypass="total" does NOT bypass CELL_BUDGET_EXHAUSTED or REPEATED_FAILURE — the budget check never reads bypass config at all; these are structural loop-safety stops, not approval gates (D2 explicit test row)',
  async () => {
    const dir = makeStateRepo('bee-budget-bypass-');
    try {
      writeJsonAtomic(path.join(dir, '.bee', 'state.json'), {
        schema_version: '1.0',
        phase: 'swarming',
        feature: 'demo-feat',
        mode: 'standard',
        approved_gates: { context: true, shape: true, execution: true, review: false },
        workers: [],
      });
      writeJsonAtomic(path.join(dir, '.bee', 'config.json'), { gate_bypass: 'total' });
      assert(bypassLevel(dir) === 'total', 'precondition: bypass level resolves to total');

      const oldAttempts = [0, 1, 2].map((i) => ({
        n: i + 1,
        at: new Date(Date.now() - (10 - i) * 1000).toISOString(),
        claim_session: `sess-old-${i}`,
        claimed_at: new Date(Date.now() - (10 - i) * 1000).toISOString(),
        worker: 'w',
        verdict: 'pass',
        failure_signature: null,
        note: null,
      }));
      makeCellFile(dir, 'bypass-1', { feature: 'demo-feat', status: 'open', deps: [], trace: { attempts: oldAttempts } });

      const result = await claimCellCrossSession(dir, { sessionId: 'sess-bypass-total', worker: 'w', cellId: 'bypass-1' });
      assert(result.ok === false && result.code === 'CELL_BUDGET_EXHAUSTED', `gate_bypass=total must NOT bypass the budget refusal, got ${JSON.stringify(result)}`);
    } finally {
      fs.rmSync(dir, { recursive: true, force: true });
    }
  },
);

await check(
  'claimNextCell: SELECTION skips a budget-exhausted candidate — the pool still finds another ready cell instead of surfacing the refusal (D2 Δ3/F3)',
  async () => {
    const dir = makeStateRepo('bee-claimnext-budget-skip-');
    try {
      writeJsonAtomic(path.join(dir, '.bee', 'state.json'), {
        schema_version: '1.0',
        phase: 'swarming',
        feature: 'demo-feat',
        mode: 'standard',
        approved_gates: { context: true, shape: true, execution: true, review: false },
        workers: [],
      });
      const oldAttempts = [0, 1, 2].map((i) => ({
        n: i + 1,
        at: new Date(Date.now() - (10 - i) * 1000).toISOString(),
        claim_session: `sess-old-${i}`,
        claimed_at: new Date(Date.now() - (10 - i) * 1000).toISOString(),
        worker: 'w',
        verdict: 'pass',
        failure_signature: null,
        note: null,
      }));
      makeCellFile(dir, 'exhausted-1', { feature: 'demo-feat', status: 'open', deps: [], trace: { attempts: oldAttempts } });
      makeCellFile(dir, 'healthy-1', { feature: 'demo-feat', status: 'open', deps: [] });

      const result = await claimNextCell(dir, { sessionId: 'sess-selector', worker: 'w' });
      assert(result.ok === true, `expected a healthy cell to be selected, got ${JSON.stringify(result)}`);
      assert(result.cell.id === 'healthy-1', `the budget-exhausted candidate must be skipped by selection, got ${result.cell.id}`);
    } finally {
      fs.rmSync(dir, { recursive: true, force: true });
    }
  },
);

await check(
  'claimNextCell: when the ONLY ready candidate is budget-exhausted, selection returns typed NO_APPROVED_WORK rather than surfacing CELL_BUDGET_EXHAUSTED — only a direct `cells claim --id` surfaces that refusal (D2 Δ3/F3)',
  async () => {
    const dir = makeStateRepo('bee-claimnext-budget-only-');
    try {
      writeJsonAtomic(path.join(dir, '.bee', 'state.json'), {
        schema_version: '1.0',
        phase: 'swarming',
        feature: 'demo-feat',
        mode: 'standard',
        approved_gates: { context: true, shape: true, execution: true, review: false },
        workers: [],
      });
      const oldAttempts = [0, 1, 2].map((i) => ({
        n: i + 1,
        at: new Date(Date.now() - (10 - i) * 1000).toISOString(),
        claim_session: `sess-old-${i}`,
        claimed_at: new Date(Date.now() - (10 - i) * 1000).toISOString(),
        worker: 'w',
        verdict: 'pass',
        failure_signature: null,
        note: null,
      }));
      makeCellFile(dir, 'exhausted-only-1', { feature: 'demo-feat', status: 'open', deps: [], trace: { attempts: oldAttempts } });

      const result = await claimNextCell(dir, { sessionId: 'sess-selector-2', worker: 'w' });
      assert(result.ok === false && result.code === 'NO_APPROVED_WORK', `expected NO_APPROVED_WORK when the only candidate is bricked, got ${JSON.stringify(result)}`);

      const direct = await claimCellCrossSession(dir, { sessionId: 'sess-selector-2', worker: 'w', cellId: 'exhausted-only-1' });
      assert(direct.ok === false && direct.code === 'CELL_BUDGET_EXHAUSTED', `direct claim --id must still surface the typed refusal, got ${JSON.stringify(direct)}`);
    } finally {
      fs.rmSync(dir, { recursive: true, force: true });
    }
  },
);

await check('deriveChangeClass resolves explicit change_class, the sole behavior_change=>behavior derivation, and null otherwise — no other auto-derivation (D3)', async () => {
  assert(deriveChangeClass({ change_class: 'api' }) === 'api', 'explicit change_class wins');
  assert(deriveChangeClass({ change_class: 'api', behavior_change: true }) === 'api', 'explicit change_class wins even over behavior_change:true');
  assert(deriveChangeClass({ behavior_change: true }) === 'behavior', 'absent change_class + behavior_change:true derives behavior');
  assert(deriveChangeClass({ behavior_change: false }) === null, 'absent change_class + behavior_change:false is unclassified');
  assert(deriveChangeClass({}) === null, 'absent change_class + absent behavior_change is unclassified');
  assert(deriveChangeClass(null) === null, 'null cell tolerated, never throws');
  assert(deriveChangeClass(undefined) === null, 'undefined cell tolerated, never throws');
  // test-economy D1: 'refactor' extends the enum to 7 members.
  // slice-tail-test-batching P2 (spec #80/#85): 'test' extends it to 8 — the
  // slice's one consolidated test-authoring cell.
  assert(CHANGE_CLASSES.includes('behavior') && CHANGE_CLASSES.includes('refactor') && CHANGE_CLASSES.includes('test') && CHANGE_CLASSES.length === 8, `expected the 8-member enum (D1 adds 'refactor', slice-tail-test-batching P2 adds 'test'), got ${JSON.stringify(CHANGE_CLASSES)}`);
});

await check('addCell validates optional change_class against the enum, naming CHANGE_CLASSES on refusal (D3)', async () => {
  assertThrows(
    () => addCell(root, makeCell('jsm-bad-class', { change_class: 'not-a-class' })),
    'change_class',
    'an invalid change_class must be refused',
  );
  const added = addCell(root, makeCell('jsm-good-class', { change_class: 'api' }));
  assert(added.change_class === 'api', 'a valid change_class is persisted');
});

await check('updateCell validates change_class the same way, and accepts null to un-set it back to derivation (D3)', async () => {
  await assertRejects(
    () => updateCell(root, 'jsm-good-class', { change_class: 'nonsense' }),
    'change_class',
    'update must refuse an invalid change_class',
  );
  const cleared = await updateCell(root, 'jsm-good-class', { change_class: null });
  assert(cleared.change_class === null, 'null un-sets change_class back to derivation');
  const revalidated = await updateCell(root, 'jsm-good-class', { change_class: 'security' });
  assert(revalidated.change_class === 'security', 'a subsequent valid change_class still applies');
});

// ─── derived-check-hardening E1: cap-time impact-registry cross-check ──────
// A dedicated fixture repo per test (own scripts/impact-registry.json), never
// the shared `root` above — the registry lookup is keyed by repo-relative
// path, and adding a registry file to the shared root could shadow unrelated
// cells' `files` elsewhere in this suite.
function makeImpactRegistryRepo(prefix, registryFileContents) {
  const dir = fs.mkdtempSync(path.join(os.tmpdir(), prefix));
  fs.mkdirSync(path.join(dir, '.bee', 'cells'), { recursive: true });
  writeJsonAtomic(path.join(dir, '.bee', 'onboarding.json'), {
    schema_version: '1.0',
    bee_version: '0.1.0',
  });
  // Gate 3 pre-approved (makeTaxonomyRepo precedent above) — claimCell
  // refuses on an unapproved execution gate, orthogonal to what E1 exercises.
  writeJsonAtomic(path.join(dir, '.bee', 'state.json'), {
    schema_version: '1.0',
    phase: 'swarming',
    feature: 'demo-feat',
    mode: 'standard',
    approved_gates: { context: true, shape: true, execution: true, review: false },
    workers: [],
  });
  if (registryFileContents !== undefined) {
    fs.mkdirSync(path.join(dir, 'scripts'), { recursive: true });
    fs.writeFileSync(path.join(dir, 'scripts', 'impact-registry.json'), registryFileContents);
  }
  return dir;
}

const IMPACT_REGISTRY_FIXTURE = JSON.stringify({
  version: 2,
  files: {
    'src/registry-fixture/thing.mjs': {
      direct: ['packages/bee/tests/test_thing_direct.mjs'],
      all: ['packages/bee/tests/test_thing_direct.mjs', 'packages/bee/tests/test_thing_transitive.mjs'],
    },
  },
});

await check(
  'capCell (E1): a verify omitting a direct-edge registry suite prints a named, non-blocking warning on stderr and still caps',
  async () => {
    const iRoot = makeImpactRegistryRepo('bee-impact-warn-', IMPACT_REGISTRY_FIXTURE);
    try {
      addCell(
        iRoot,
        makeCell('e1-warn-1', {
          lane: 'small',
          files: ['src/registry-fixture/thing.mjs'],
          verify: 'node packages/bee/tests/test_unrelated.mjs',
        }),
      );
      await claimCell(iRoot, 'e1-warn-1', 'worker-e1');
      const originalWrite = process.stderr.write;
      let stderrOut = '';
      process.stderr.write = (chunk) => {
        stderrOut += chunk;
        return true;
      };
      let capped;
      try {
        capped = await capCell(iRoot, 'e1-warn-1', {
          files_changed: ['src/registry-fixture/thing.mjs'],
          outcome: 'done',
        });
      } finally {
        process.stderr.write = originalWrite;
      }
      assert(capped.status === 'capped', 'a missing direct-edge suite must warn, never refuse the cap');
      assert(
        Array.isArray(capped.trace.warnings) &&
          capped.trace.warnings.some((w) => w.includes('packages/bee/tests/test_thing_direct.mjs')),
        `expected trace.warnings to name the missing direct-edge suite, got ${JSON.stringify(capped.trace.warnings)}`,
      );
      assert(
        stderrOut.includes('packages/bee/tests/test_thing_direct.mjs'),
        `expected the missing suite to be named on stderr, got ${JSON.stringify(stderrOut)}`,
      );
      assert(
        !stderrOut.includes('test_thing_transitive.mjs'),
        'level:1 must scope to DIRECT edges only — the transitive-only suite must never be named',
      );
    } finally {
      fs.rmSync(iRoot, { recursive: true, force: true });
    }
  },
);

await check('capCell (E1): a verify that already mentions the direct-edge suite gets no impact-registry warning', async () => {
  const iRoot = makeImpactRegistryRepo('bee-impact-clean-', IMPACT_REGISTRY_FIXTURE);
  try {
    addCell(
      iRoot,
      makeCell('e1-clean-1', {
        lane: 'small',
        files: ['src/registry-fixture/thing.mjs'],
        verify: 'node packages/bee/tests/test_thing_direct.mjs',
      }),
    );
    await claimCell(iRoot, 'e1-clean-1', 'worker-e1');
    const capped = await capCell(iRoot, 'e1-clean-1', {
      files_changed: ['src/registry-fixture/thing.mjs'],
      outcome: 'done',
    });
    assert(capped.status === 'capped', 'a matching verify must still cap');
    assert(
      !capped.trace.warnings || !capped.trace.warnings.some((w) => w.includes('impact-registry') || w.includes('E1')),
      `expected no impact-registry warning when the direct-edge suite is already covered, got ${JSON.stringify(capped.trace.warnings)}`,
    );
  } finally {
    fs.rmSync(iRoot, { recursive: true, force: true });
  }
});

await check('capCell (E1): a missing impact registry is a silent skip, never a throw', async () => {
  const iRoot = makeImpactRegistryRepo('bee-impact-missing-'); // no registry file at all
  try {
    addCell(
      iRoot,
      makeCell('e1-missing-1', {
        lane: 'small',
        files: ['src/registry-fixture/thing.mjs'],
        verify: 'node -e "process.exit(0)"',
      }),
    );
    await claimCell(iRoot, 'e1-missing-1', 'worker-e1');
    const capped = await capCell(iRoot, 'e1-missing-1', { files_changed: ['a.js'], outcome: 'done' });
    assert(capped.status === 'capped', 'a missing registry must never block a cap');
  } finally {
    fs.rmSync(iRoot, { recursive: true, force: true });
  }
});

await check('capCell (E1): a malformed impact registry (invalid JSON) is a silent skip, never a throw', async () => {
  const iRoot = makeImpactRegistryRepo('bee-impact-malformed-', '{ not valid json');
  try {
    addCell(
      iRoot,
      makeCell('e1-malformed-1', {
        lane: 'small',
        files: ['src/registry-fixture/thing.mjs'],
        verify: 'node -e "process.exit(0)"',
      }),
    );
    await claimCell(iRoot, 'e1-malformed-1', 'worker-e1');
    const capped = await capCell(iRoot, 'e1-malformed-1', { files_changed: ['a.js'], outcome: 'done' });
    assert(capped.status === 'capped', 'a malformed registry must never block a cap');
  } finally {
    fs.rmSync(iRoot, { recursive: true, force: true });
  }
});

// ─── D1 Δ2-amendment: EVERY claim-clearing transition releases the claim
// file — cap, unclaim, block, drop, reopen — not only the claim-next unwind.
// Without this a same-session round trip through one of these verbs would
// self-refuse CLAIMED for the claim's full TTL (block/reopen's round trip is
// covered end-to-end by scripts/test_claim_race.mjs scenario (c); this covers
// cap/unclaim/drop directly at the lib level).

await check('capCell releases the claim file on cap (D1 Δ2)', async () => {
  addCell(root, makeCell('rel-cap-1'));
  await claimCellCrossSession(root, { sessionId: 'sess-rel-cap', worker: 'w', cellId: 'rel-cap-1' });
  assert(readClaim(root, 'rel-cap-1') !== null, 'precondition: claim file exists after claim');
  // D4 (msh-4): the owning session must now authenticate its own mutations —
  // capCell runs AS 'sess-rel-cap', the session that claimed it.
  await capCell(root, 'rel-cap-1', { files_changed: ['a.js'], outcome: 'done', sessionId: 'sess-rel-cap' });
  assert(readClaim(root, 'rel-cap-1') === null, 'cap must release the claim file');
});

await check('unclaimCell releases the claim file, and the cell is re-claimable by the SAME session with no self-refusal (D1 Δ2)', async () => {
  addCell(root, makeCell('rel-unclaim-1'));
  await claimCellCrossSession(root, { sessionId: 'sess-rel-unclaim', worker: 'w', cellId: 'rel-unclaim-1' });
  assert(readClaim(root, 'rel-unclaim-1') !== null, 'precondition: claim file exists after claim');
  // D4 (msh-4): unclaim as the owning session — single-session use never refuses.
  await unclaimCell(root, 'rel-unclaim-1', { sessionId: 'sess-rel-unclaim' });
  assert(readClaim(root, 'rel-unclaim-1') === null, 'unclaim must release the claim file');
  const reclaimed = await claimCellCrossSession(root, { sessionId: 'sess-rel-unclaim', worker: 'w', cellId: 'rel-unclaim-1' });
  assert(reclaimed.ok === true, `same-session re-claim after unclaim must not self-refuse, got ${JSON.stringify(reclaimed)}`);
});

await check('dropCell releases the claim file (D1 Δ2)', async () => {
  addCell(root, makeCell('rel-drop-1'));
  await claimCellCrossSession(root, { sessionId: 'sess-rel-drop', worker: 'w', cellId: 'rel-drop-1' });
  assert(readClaim(root, 'rel-drop-1') !== null, 'precondition: claim file exists after claim');
  await dropCell(root, 'rel-drop-1', 'no longer needed');
  assert(readClaim(root, 'rel-drop-1') === null, 'drop must release the claim file');
});

await check('capCell/unclaimCell/blockCell/dropCell/reopenCell never fail when there is no claim file to release (cells claimed before msh-2, or never claimed)', async () => {
  addCell(root, makeCell('rel-none-1'));
  assert(readClaim(root, 'rel-none-1') === null, 'precondition: no claim file (never claimed via claimCellCrossSession)');
  await claimCell(root, 'rel-none-1', 'worker-plain'); // the bare, file-less claim path
  const capped = await capCell(root, 'rel-none-1', { files_changed: ['a.js'], outcome: 'done' });
  assert(capped.status === 'capped', 'cap succeeds cleanly with no claim file to release');
});

// ─── D4 (msh-4): claim-ownership check on cell mutators + audited force
// door. capCell/blockCell/unclaimCell/reopenCell read the
// live claim file: a LIVE claim carrying a session that differs from the
// caller's resolved session refuses (typed, names owner + expiry); an
// expired claim, an absent claim, a sessionless claim, or a matching session
// proceeds unchanged (dropCell stays untouched — CONTEXT D4 names only
// verify/cap/block/unclaim/reopen).

await check('a live claim mismatch refuses capCell/blockCell/unclaimCell, naming owner and expiry', async () => {
  addCell(root, makeCell('own-mismatch-1'));
  await claimCellCrossSession(root, { sessionId: 'sess-owner', worker: 'w', cellId: 'own-mismatch-1' });
  await assertRejects(
    () => capCell(root, 'own-mismatch-1', { files_changed: ['a.js'], outcome: 'done', sessionId: 'sess-intruder' }),
    'sess-owner',
    'capCell refuses a mismatched session',
  );
  await assertRejects(
    () => capCell(root, 'own-mismatch-1', { files_changed: ['a.js'], outcome: 'done', sessionId: 'sess-intruder' }),
    'expires',
    'the refusal names the expiry too',
  );
  await assertRejects(
    () => blockCell(root, 'own-mismatch-1', 'stuck', { sessionId: 'sess-intruder' }),
    'sess-owner',
    'blockCell refuses a mismatched session',
  );
  await assertRejects(
    () => unclaimCell(root, 'own-mismatch-1', { sessionId: 'sess-intruder' }),
    'sess-owner',
    'unclaimCell refuses a mismatched session',
  );
  const cell = readCell(root, 'own-mismatch-1');
  assert(cell.status === 'claimed', 'every refusal above left the cell untouched — still claimed');
});

await check('reopenCell also refuses a live-claim mismatch, naming owner and expiry', async () => {
  addCell(root, makeCell('own-mismatch-reopen', { status: 'blocked' }));
  claimCellFile(root, 'sess-owner', 'own-mismatch-reopen', 3600); // a lingering live claim on an already-blocked cell
  await assertRejects(
    () => reopenCell(root, 'own-mismatch-reopen', 'retry', { sessionId: 'sess-intruder' }),
    'sess-owner',
    'reopenCell refuses a mismatched session',
  );
  assert(readCell(root, 'own-mismatch-reopen').status === 'blocked', 'reopenCell refusal leaves status untouched');
});

await check('an expired claim proceeds unchanged for cap (rescue stays possible)', async () => {
  addCell(root, makeCell('own-expired-1'));
  await claimCell(root, 'own-expired-1', 'worker-x'); // bare claim (status only)
  writeJsonAtomic(claimPath(root, 'own-expired-1'), {
    cell: 'own-expired-1',
    session: 'sess-gone',
    claimed_at: new Date(Date.now() - 7200 * 1000).toISOString(),
    ttl_seconds: 60,
  });
  const capped = await capCell(root, 'own-expired-1', { files_changed: ['a.js'], outcome: 'done', sessionId: 'sess-rescuer' });
  assert(capped.status === 'capped', 'cap proceeds through an expired claim without refusal');
});

await check('a sessionless claim proceeds unchanged — single-session use never hits a refusal', async () => {
  addCell(root, makeCell('own-sessionless-1'));
  await claimCellCrossSession(root, { sessionId: null, worker: 'w', cellId: 'own-sessionless-1' });
  assert(readClaim(root, 'own-sessionless-1').session === undefined, 'precondition: sessionless claim omits the session key');
  const capped = await capCell(root, 'own-sessionless-1', {
    files_changed: ['a.js'],
    outcome: 'done',
    sessionId: 'sess-anyone',
  });
  assert(capped.status === 'capped', 'cap proceeds through a sessionless claim regardless of caller session');
});

await check(
  '--force-ownership bypasses a live-claim mismatch and appends an audited trace.ownership_overrides row (never trace.deviations)',
  async () => {
    addCell(root, makeCell('own-force-1'));
    await claimCellCrossSession(root, { sessionId: 'sess-owner', worker: 'w', cellId: 'own-force-1' });
    await blockCell(root, 'own-force-1', 'forced block over a foreign claim', { sessionId: 'sess-forcer', forceOwnership: true });
    const afterBlock = readCell(root, 'own-force-1');
    assert(
      Array.isArray(afterBlock.trace.ownership_overrides) && afterBlock.trace.ownership_overrides.length === 1,
      `blockCell force appends one override row, got ${JSON.stringify(afterBlock.trace.ownership_overrides)}`,
    );
    const row = afterBlock.trace.ownership_overrides[0];
    assert(
      row.verb === 'blockCell' && row.forced_by === 'sess-forcer' && row.owner_bypassed === 'sess-owner',
      `override row names forcer and bypassed owner, got ${JSON.stringify(row)}`,
    );

    await reopenCell(root, 'own-force-1', 'retry after forced block', { sessionId: 'sess-forcer' });
    await claimCell(root, 'own-force-1', 'w2');
    const capped = await capCell(root, 'own-force-1', {
      files_changed: ['a.js'],
      outcome: 'forced cap',
      deviations: ['unrelated planning deviation'], // capCell REPLACES deviations wholesale — proves ownership_overrides survives that wipe
      forceOwnership: true,
    });
    assert(
      capped.trace.ownership_overrides.length === 2,
      `cap force appends a SECOND row (append-only), got ${JSON.stringify(capped.trace.ownership_overrides)}`,
    );
    assert(capped.trace.ownership_overrides[1].verb === 'capCell', 'the second row is capCell\'s own audit line');
    assert(
      capped.trace.deviations.length === 1 && capped.trace.deviations[0] === 'unrelated planning deviation',
      'trace.deviations still holds only the cap-time deviations — the ownership audit key is untouched by capCell wholesale-replacing deviations',
    );
  },
);

await check(
  "a forced unclaim past another session's live claim still clears the claim file, so the forced-open cell is claimable by a new session (D4 Δ5)",
  async () => {
    addCell(root, makeCell('own-force-unclaim-1'));
    await claimCellCrossSession(root, { sessionId: 'sess-owner', worker: 'w', cellId: 'own-force-unclaim-1' });
    await unclaimCell(root, 'own-force-unclaim-1', { sessionId: 'sess-rescuer', forceOwnership: true });
    assert(
      readClaim(root, 'own-force-unclaim-1') === null,
      'forced unclaim releases the claim file — the cell must not stay self-refusing',
    );
    const reclaimed = await claimCellCrossSession(root, { sessionId: 'sess-rescuer', worker: 'w2', cellId: 'own-force-unclaim-1' });
    assert(reclaimed.ok === true, `the forced-open cell is claimable by the rescuing session, got ${JSON.stringify(reclaimed)}`);
    const afterReclaim = readCell(root, 'own-force-unclaim-1');
    assert(
      afterReclaim.trace.ownership_overrides.length === 1 && afterReclaim.trace.ownership_overrides[0].verb === 'unclaimCell',
      `the unclaim force audit row survives the reclaim, got ${JSON.stringify(afterReclaim.trace.ownership_overrides)}`,
    );
  },
);

// ─── jrt-1: internal logDecision( callers must survive a taxonomy-enforced
// repo, tagged for what the event actually is ──────────────────────────────
// Census: `docs/decisions/taxonomy.json` (dp-6, D7b) makes classifyDecisionTags
// refuse (typed DecisionsUntaggedRefusedError) any decision event with zero
// tags. A census of internal logDecision( callers found four sites passing no
// tags at all — three live in cells.mjs (capCell's --override-judge audit,
// resetCellBudget, and recordJudgeVerdict's NEEDS_REVISION reopen) and one in
// claims.mjs (sweepExpiredClaims' stale-claim reset). This repo (beegog) HAS a
// taxonomy, so all four were live failures here — an earlier feature's
// NEEDS_REVISION verdict had to be hand-written into decisions.jsonl because
// recordJudgeVerdict's own audit call threw and unwound the whole write.
// Each row below reproduces ONE call site inside a dedicated taxonomy-enforced
// fixture and proves the operation now both succeeds AND lands a tagged
// event — never a generic catch-all, always the tag(s) describing what the
// event IS, drawn only from names already in the real taxonomy.

await check('capCell --override-judge logs a tagged decision under a taxonomy, instead of the whole cap throwing (jrt-1)', async () => {
  const dir = makeTaxonomyRepo('bee-cells-taxonomy-ovr-');
  try {
    addCell(
      dir,
      makeCell('ovr-1', {
        lane: 'tiny',
        trace: {
          semantic_judge: [
            {
              schema: JUDGE_VERDICT_SCHEMA,
              verdict: 'NEEDS_REVISION',
              checks: [{ id: 'c1', status: 'FAIL', evidence: 'not yet' }],
              fixability: 'manual',
              confidence: 'low',
              recorded_at: new Date().toISOString(),
            },
          ],
        },
      }),
    );
    await assertRejects(
      () => capCell(dir, 'ovr-1', { outcome: 'done' }),
      'NEEDS_REVISION',
      'precondition: cap without --override-judge still refuses on a NEEDS_REVISION verdict',
    );
    const capped = await capCell(dir, 'ovr-1', {
      outcome: 'shipping with a known, tracked gap',
      overrideJudge: 'accepted risk, tracked in backlog',
    });
    assert(capped.status === 'capped', 'override-judge cap succeeds even under a taxonomy');
    const decisions = activeDecisions(dir, { recent: 5 });
    const logged = decisions.find((d) => d.decision.includes('ovr-1'));
    assert(
      logged,
      `capCell's override-judge audit must log a decision even under a taxonomy — before the fix this threw DECISIONS_UNTAGGED_REFUSED and the whole cap write never happened; decisions seen: ${JSON.stringify(decisions)}`,
    );
    assert(
      Array.isArray(logged.tags) && logged.tags.includes('cells') && logged.tags.includes('judge'),
      `override-judge decision should be tagged cells+judge (what the event IS), got ${JSON.stringify(logged.tags)}`,
    );
  } finally {
    fs.rmSync(dir, { recursive: true, force: true });
  }
});

await check('resetCellBudget logs a tagged decision under a taxonomy, instead of the whole reset throwing (jrt-1)', async () => {
  const dir = makeTaxonomyRepo('bee-cells-taxonomy-budget-');
  try {
    addCell(dir, makeCell('bud-1', { lane: 'tiny' }));
    for (let i = 0; i < 3; i += 1) {
      await claimCellCrossSession(dir, { sessionId: `sess-tax-bud-${i}`, worker: 'w', cellId: 'bud-1' });
      await recordTestsRedAttempt(dir, 'bud-1', { excerptLine: `FAIL taxonomy-fixture red number ${i}` });
      await unclaimCell(dir, 'bud-1', { sessionId: `sess-tax-bud-${i}` });
    }
    const blocked = await claimCellCrossSession(dir, { sessionId: 'sess-tax-bud-3', worker: 'w', cellId: 'bud-1' });
    assert(blocked.ok === false && blocked.code === 'CELL_BUDGET_EXHAUSTED', `precondition: the claim door should be exhausted, got ${JSON.stringify(blocked)}`);

    const reset = await resetCellBudget(dir, 'bud-1', 'manager approved a genuine retry under a taxonomy', { operator: 'manager-tax' });
    assert(Array.isArray(reset.trace.budget_resets) && reset.trace.budget_resets.length === 1, `reset must still append exactly one budget_resets entry, got ${JSON.stringify(reset.trace.budget_resets)}`);

    const decisions = activeDecisions(dir, { recent: 5 });
    const logged = decisions.find((d) => d.decision.includes('bud-1'));
    assert(
      logged,
      `resetCellBudget must log a decision even under a taxonomy — before the fix this threw DECISIONS_UNTAGGED_REFUSED and the whole reset write never happened; decisions seen: ${JSON.stringify(decisions)}`,
    );
    assert(
      Array.isArray(logged.tags) && logged.tags.includes('cells'),
      `reset-budget decision should be tagged cells, got ${JSON.stringify(logged.tags)}`,
    );
  } finally {
    fs.rmSync(dir, { recursive: true, force: true });
  }
});

await check('recordJudgeVerdict reopening a capped cell on NEEDS_REVISION logs a tagged decision under a taxonomy (jrt-1)', async () => {
  const dir = makeTaxonomyRepo('bee-cells-taxonomy-judge-');
  try {
    addCell(dir, makeCell('jr-1', { lane: 'tiny' }));
    await capCell(dir, 'jr-1', { outcome: 'first pass' });
    assert(readCell(dir, 'jr-1').status === 'capped', 'precondition: jr-1 is capped');

    const reverdicted = await recordJudgeVerdict(dir, 'jr-1', {
      schema: JUDGE_VERDICT_SCHEMA,
      verdict: 'NEEDS_REVISION',
      checks: [{ id: 'c1', status: 'FAIL', evidence: 'goal check found a gap' }],
      failure_signature: 'goal-check-gap',
      fixability: 'automatic',
      confidence: 'high',
    });
    assert(reverdicted.status === 'open', `a NEEDS_REVISION verdict must reopen the capped cell to open even under a taxonomy, got status ${reverdicted.status}`);

    const decisions = activeDecisions(dir, { recent: 5 });
    const logged = decisions.find((d) => d.decision.includes('jr-1'));
    assert(
      logged,
      `recordJudgeVerdict's reopen audit must log a decision even under a taxonomy — before the fix this threw DECISIONS_UNTAGGED_REFUSED and the whole verdict write never happened (not even the semantic_judge entry); decisions seen: ${JSON.stringify(decisions)}`,
    );
    assert(
      Array.isArray(logged.tags) && logged.tags.includes('cells') && logged.tags.includes('judge'),
      `judge-record reopen decision should be tagged cells+judge, got ${JSON.stringify(logged.tags)}`,
    );
  } finally {
    fs.rmSync(dir, { recursive: true, force: true });
  }
});

// ─── jrt-1: census — no future internal logDecision( call may ship untagged ─
// This is the actual regression coverage the defect class calls for: "a
// cross-cutting write-time rule shipped without sweeping its existing
// internal callers". Derived by SCANNING the source (never a hand-maintained
// list of line numbers) — any new logDecision( call added anywhere under
// .bee/bin/lib/** or packages/bee/lib/** that omits a `tags:`
// key fails this check, before it ever reaches a taxonomy-enforced repo.

// Skips string/template literals (including the «...» decision text this
// codebase writes, which can itself contain parens via ${...} interpolation)
// so paren-counting is never confused by call-site prose.
function skipStringLiteral(text, i, quote) {
  let j = i + 1;
  while (j < text.length) {
    if (text[j] === '\\') {
      j += 2;
      continue;
    }
    if (text[j] === quote) return j;
    j += 1;
  }
  return j;
}

// Returns the raw argument-list text between a call's `(` and its balanced
// `)`, given the index of the opening paren itself.
function extractBalancedArgs(text, openParenIndex) {
  let depth = 0;
  const start = openParenIndex + 1;
  for (let i = openParenIndex; i < text.length; i += 1) {
    const ch = text[i];
    if (ch === '(') {
      depth += 1;
    } else if (ch === ')') {
      depth -= 1;
      if (depth === 0) return text.slice(start, i);
    } else if (ch === '"' || ch === "'" || ch === '`') {
      i = skipStringLiteral(text, i, ch);
    } else if (ch === '/' && text[i + 1] === '/') {
      const nl = text.indexOf('\n', i);
      i = nl === -1 ? text.length : nl;
    } else if (ch === '/' && text[i + 1] === '*') {
      const end = text.indexOf('*/', i + 2);
      i = end === -1 ? text.length : end + 1;
    }
  }
  throw new Error('extractBalancedArgs: unbalanced parens scanning a logDecision( call — fixture or scanner bug.');
}

// Finds every logDecision( CALL site in `text` (never its `function
// logDecision(` declaration in decisions.mjs itself), returning
// {index, args} for each — `args` is the raw call-argument text, scanned for
// a top-level `tags:` key by the caller.
function findLogDecisionCalls(text) {
  const calls = [];
  const re = /logDecision\s*\(/g;
  let m;
  while ((m = re.exec(text))) {
    const before = text.slice(Math.max(0, m.index - 20), m.index);
    if (/\bfunction\s+$/.test(before)) continue; // the declaration itself, not a call
    const openParenIndex = m.index + m[0].length - 1;
    calls.push({ index: m.index, args: extractBalancedArgs(text, openParenIndex) });
  }
  return calls;
}

function lineOf(text, index) {
  return text.slice(0, index).split('\n').length;
}

// jrt-2: `excludeDirNames` is structural, not a hand-maintained path list — it
// names directories to prune from the walk (by basename, at any depth), the
// walk itself still discovers every .mjs file by scanning. Used to prune
// `tests/` (see the census below for why) without hand-listing the files
// inside it.
function collectMjsFiles(dir, excludeDirNames = []) {
  let out = [];
  for (const entry of fs.readdirSync(dir, { withFileTypes: true })) {
    const full = path.join(dir, entry.name);
    if (entry.isDirectory()) {
      if (excludeDirNames.includes(entry.name)) continue;
      out = out.concat(collectMjsFiles(full, excludeDirNames));
    } else if (entry.isFile() && entry.name.endsWith('.mjs')) out.push(full);
  }
  return out;
}

await check('census scanner: findLogDecisionCalls flags a tagless call, passes a tagged call, and ignores the declaration itself (jrt-1 — proves the census assertion below actually bites)', () => {
  const injectedTagless = `
    function reopen(root, id) {
      logDecision(root, {
        decision: \`«reopen: cell "\${id}" reopened — a paren \${(1 + 2)} inside the template»\`,
        rationale: 'no tags passed here',
        scope: 'repo',
        source: 'user',
      });
    }
  `;
  const taglessCalls = findLogDecisionCalls(injectedTagless);
  assert(taglessCalls.length === 1, `scanner must find exactly one call site in the injected snippet, got ${taglessCalls.length}`);
  assert(!/\btags\s*:/.test(taglessCalls[0].args), 'sanity: the injected call genuinely carries no tags: key');

  const taggedCall = `logDecision(root, { decision: 'x', rationale: 'y', tags: ['cells'] });`;
  const taggedCalls = findLogDecisionCalls(taggedCall);
  assert(taggedCalls.length === 1 && /\btags\s*:/.test(taggedCalls[0].args), 'a call that DOES pass tags must be recognized as tagged');

  const declarationOnly = `export function logDecision(\n  root,\n  { decision, rationale, tags = undefined },\n) {\n  return null;\n}\n`;
  assert(findLogDecisionCalls(declarationOnly).length === 0, 'the function logDecision( declaration itself must never be counted as a call site');
});

// jrt-2: jrt-1's census above scoped itself to `.bee/bin/lib/**` and
// `packages/bee/lib/**` from ASSUMPTION, not measurement — one
// directory too narrow. A repo-wide grep for `logDecision(` finds real
// callers directly in `bee.mjs` too (the CLI entrypoints, one level up from
// `lib/`). Widened to every non-test .mjs under `.bee/bin/**` and
// `packages/bee/**` — still DERIVED by scanning source, never a
// hand-maintained path list (see collectMjsFiles's excludeDirNames param
// above). `packages/bee/tests/` is pruned structurally: a test fixture's
// tagless logDecision( call (e.g. the injected snippet in the scanner-proof
// check above, or a future fixture like it) is its subject matter, not a
// production call site the taxonomy gate must ever see — flagging it would
// be a false positive, not a real offender.
await check('census: every logDecision( call inside .bee/bin/** and packages/bee/** (excluding packages/bee/tests/) carries at least one tag (jrt-2 — widened from jrt-1s lib/**-only scope, which measurement showed was too narrow)', () => {
  const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '../../..');
  const dirs = [path.join(repoRoot, '.bee', 'bin'), path.join(repoRoot, 'packages', 'bee')];
  const offenders = [];
  for (const dir of dirs) {
    for (const file of collectMjsFiles(dir, ['tests'])) {
      const text = fs.readFileSync(file, 'utf8');
      for (const call of findLogDecisionCalls(text)) {
        if (!/\btags\s*:/.test(call.args)) {
          offenders.push(`${path.relative(repoRoot, file)}:${lineOf(text, call.index)}`);
        }
      }
    }
  }
  assert(
    offenders.length === 0,
    `every internal logDecision( call must carry a tags: array (docs/decisions/taxonomy.json makes an untagged event a hard refusal) — offenders: ${offenders.join(', ') || '(none)'}`,
  );
});

// ─── msn-18b: linked-worktree claim topology (must_have, red-first per the
// cell contract) — a claim taken from a linked worktree lands in MAIN's
// control store, while the CELL FILE itself stays workspace-local (PLANE
// RULE), and the claim resolves back to the SAME cell from the claiming
// workspace (same-plane claim-to-cell resolution). Same manual worktree-
// fixture pattern test_state.mjs uses for resolveRoots/resolveContext.

function makeLinkedWorktreeCellFixture(mainPrefix, workPrefix, id) {
  const main = fs.mkdtempSync(path.join(os.tmpdir(), mainPrefix));
  const work = fs.mkdtempSync(path.join(os.tmpdir(), workPrefix));
  fs.mkdirSync(path.join(main, '.git'));
  const gitdir = path.join(main, '.git', 'worktrees', id);
  fs.mkdirSync(gitdir, { recursive: true });
  fs.writeFileSync(path.join(work, '.git'), `gitdir: ${gitdir}\n`);
  fs.writeFileSync(path.join(gitdir, 'gitdir'), path.join(work, '.git') + '\n');

  for (const dir of [main, work]) {
    fs.mkdirSync(path.join(dir, '.bee', 'cells'), { recursive: true });
    writeJsonAtomic(path.join(dir, '.bee', 'onboarding.json'), { schema_version: '1.0', bee_version: '0.1.0' });
    fs.mkdirSync(path.join(dir, 'docs', 'decisions'), { recursive: true });
    writeJsonAtomic(path.join(dir, 'docs', 'decisions', 'taxonomy.json'), {
      schema_version: 1,
      tags: [{ name: 'cells', description: 'Work cells: authoring, claims, caps' }],
      candidates: [],
    });
  }
  // Gate 3 pre-approved — the gate check reads the CLAIMING workspace's own
  // state.json (claimCell's gateSource = laneRecord || readState(root)),
  // which stays workspace-local regardless of msn-18b — a linked worktree
  // needs its OWN gate approval, exactly like today.
  writeJsonAtomic(path.join(work, '.bee', 'state.json'), {
    schema_version: '1.0',
    phase: 'swarming',
    feature: 'topo-feat',
    mode: 'standard',
    approved_gates: { context: true, shape: true, execution: true, review: false },
    workers: [],
  });
  return { main, work };
}

await check('TOPOLOGY (red-first): claimCellCrossSession from a linked worktree lands the claims-store record in MAIN, while the cell FILE stays in the claiming workspace', async () => {
  const { main, work } = makeLinkedWorktreeCellFixture('bee-topo-cells-main-', 'bee-topo-cells-work-', 'topo-cells-fixture');
  addCell(work, makeCell('topo-claim-1', { feature: 'topo-feat' }));

  const claimed = await claimCellCrossSession(work, { sessionId: 'topo-sess-1', worker: 'topo-worker-1', cellId: 'topo-claim-1' });
  assert(claimed.ok === true, `precondition: claim from the worktree must succeed, got ${JSON.stringify(claimed)}`);

  // The claims-store record is visible from MAIN — the whole point of the
  // control-plane split this cell implements.
  const claimAtMain = readClaim(main, 'topo-claim-1');
  assert(claimAtMain && claimAtMain.session === 'topo-sess-1', `expected the claim to be readable from main's store, got ${JSON.stringify(claimAtMain)}`);

  // The worktree's OWN local claims dir never received a copy — pre-msn-18b
  // baseline was the exact opposite (claim landed ONLY in the worktree's own
  // .bee/claims/, invisible from main); this is the red-first flip.
  const workClaimPath = claimPath(work, 'topo-claim-1');
  assert(!fs.existsSync(workClaimPath), `the worktree's own claims dir must NOT hold a copy — expected it physically at main instead, got a file at ${workClaimPath}`);
  assert(fs.existsSync(claimPath(main, 'topo-claim-1')), 'the claim must physically exist under MAIN\'s .bee/claims/');

  // Same-plane claim-to-cell resolution: the cell FILE itself stays in the
  // CLAIMING workspace (PLANE RULE) — never migrated to main.
  const cellAtWork = readCell(work, 'topo-claim-1');
  assert(cellAtWork && cellAtWork.status === 'claimed', 'the cell file must be readable (and reflect claimed) from the claiming WORKSPACE');
  assert(!fs.existsSync(path.join(main, '.bee', 'cells', 'topo-claim-1.json')), 'the cell file must NOT have been written to main — cells/backlog/decisions stay workspace-local');

  // A second claimant, querying from MAIN directly, sees the SAME live
  // claim and is correctly refused — proving the shared store is really
  // shared, not just readable one-way.
  const secondFromMain = await claimCellCrossSession(main, { sessionId: 'topo-sess-2', worker: 'topo-worker-2', cellId: 'topo-claim-1' });
  assert(secondFromMain.ok === false && secondFromMain.code === 'CLAIMED', `a second claim attempt from MAIN on the same cell id must see the worktree's live claim and refuse, got ${JSON.stringify(secondFromMain)}`);
});

await check('TOPOLOGY: claimNextCell\'s cross-session hold check (findSessionConflicts) sees a reservation made from a DIFFERENT worktree of the same main', async () => {
  const { main, work } = makeLinkedWorktreeCellFixture('bee-topo-holdcheck-main-', 'bee-topo-holdcheck-work-', 'topo-holdcheck-fixture');
  addCell(work, makeCell('topo-hold-1', { feature: 'topo-feat', files: ['src/topo/held.ts'] }));

  const otherSessionReservation = await reserve(main, {
    agent: 'other-worker',
    cell: 'other-cell',
    path: 'src/topo/held.ts',
    session: 'other-live-session',
  });
  assert(otherSessionReservation.ok === true, 'precondition: another session\'s reservation at main must succeed');

  // claimNextCell, run from the WORKTREE, must see that main-side hold and
  // skip the cell rather than claiming over it.
  const result = await claimNextCell(work, { sessionId: 'topo-sess-holdcheck', worker: 'topo-worker-holdcheck' });
  assert(result.ok === false && result.code === 'NO_APPROVED_WORK', `expected claim-next to see the cross-worktree hold and refuse NO_APPROVED_WORK, got ${JSON.stringify(result)}`);
});

await check('solo/main repos byte-identical: claimCellCrossSession behaves exactly as before this cell (no linked worktree involved)', async () => {
  addCell(root, makeCell('topo-solo-1'));
  const claimed = await claimCellCrossSession(root, { sessionId: 'topo-solo-sess', worker: 'topo-solo-worker', cellId: 'topo-solo-1' });
  assert(claimed.ok === true, `byte-identical solo behavior expected, got ${JSON.stringify(claimed)}`);
  assert(readClaim(root, 'topo-solo-1').session === 'topo-solo-sess', 'claim visible at the same root, unchanged');
  assert(readCell(root, 'topo-solo-1').status === 'claimed', 'cell file visible at the same root, unchanged');
});

// test-simple (decision 412e9b3a): the wc-1/wc-4 proof-marker and
// proof-door tables are deleted with trace.proof and the evidence doors.

printSummaryAndExit();
