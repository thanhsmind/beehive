#!/usr/bin/env node
// test_compaction_module.mjs — the suite for lib/compaction.mjs (feature
// compaction-hardening, cell cz-3; decisions D3/D4/D5/D9/D10/D12/D13/D17).
//
// It targets the SOURCE module (skills/bee-hive/templates/lib/compaction.mjs),
// never the .bee/bin/lib mirror — D17: the template tree is the edit target and
// scripts/tests/test_lib_mirror.mjs is what proves the mirror matches it.
//
// The five obligations this file exists to hold, each one a rule that a plain
// "it works" reading of the module would silently break:
//
//   1. THE COUNTING RULE IS OFF-BY-ONE-PRONE (D5). Only `precompact` records
//      are counted and only a `precompact` counts itself. A `resume` carries
//      the plain prior count — never +1 again. Counting a resume inclusively
//      makes ONE compaction read as TWO and fires D9's advisory a whole cycle
//      early, so the test drives TWO COMPLETE cycles and asserts the threshold
//      fires on the SECOND precompact, not the first and not the first resume.
//   2. A FAILED LOG APPEND IS INVISIBLE (D4). Asserted twice: the return value
//      in-process, and the EXIT CODE of a child process whose only work is the
//      append — an assertion about a return value alone cannot see a throw that
//      escapes to the top level.
//   3. A LANE-BOUND SESSION LOGS ITS OWN LANE, never the default state.json —
//      resolvePipeline is called WITH the session id (inject.mjs:301-305).
//   4. A BROKEN LANE BINDING SURFACES ITS TYPED CODE (D12), all three of
//      LANE_INVALID / LANE_MISSING / LANE_CORRUPT, never swallowed into a
//      silent fallback.
//   5. THE SWEEP MUTATES NOTHING (D13). Proven by hashing the whole `.bee/`
//      tree before and after two consecutive runs — the HASH is the proof, a
//      stable stdout is not.

import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import crypto from 'node:crypto';
import { spawnSync } from 'node:child_process';
import { fileURLToPath, pathToFileURL } from 'node:url';

import { check, assert, printSummaryAndExit } from '../lib/test-fixture.mjs';
import {
  COMPACT_EVENTS,
  SURVIVAL_WARNING_THRESHOLD,
  ANCHOR_NUDGE_KEY,
  compactionLogPath,
  readCompactionRecords,
  readCompactionCounts,
  appendCompactionRecord,
  survivalWarning,
  anchorMissing,
  compactCheck,
} from '../../skills/bee-hive/templates/lib/compaction.mjs';
// multisession-native-16: `.bee/reservations.json` is a rebuildable
// PROJECTION over lease-store.mjs now (reservations.mjs's own module
// header) — compaction.mjs's `reservations` check reads live leases through
// reservations.mjs's listReservations, so a fixture reservation must be a
// real lease, seeded via lease-store.mjs's synchronous acquireLeases
// (same source tree this file already targets, D17).
import { acquireLeases } from '../../skills/bee-hive/templates/lib/lease-store.mjs';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const REPO_ROOT = path.join(__dirname, '..', '..');
const MODULE_URL = pathToFileURL(
  path.join(REPO_ROOT, 'skills', 'bee-hive', 'templates', 'lib', 'compaction.mjs'),
).href;

// ─── fixtures ───────────────────────────────────────────────────────────────

const tempRoots = [];

function writeJson(file, obj) {
  fs.mkdirSync(path.dirname(file), { recursive: true });
  fs.writeFileSync(file, `${JSON.stringify(obj, null, 2)}\n`, 'utf8');
}

function makeRepo({ phase = 'swarming', feature = 'demo', execution = true, mode = 'standard' } = {}) {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), 'bee-compaction-'));
  tempRoots.push(root);
  fs.mkdirSync(path.join(root, '.bee', 'logs'), { recursive: true });
  writeJson(path.join(root, '.bee', 'state.json'), {
    schema_version: '1.0',
    phase,
    mode,
    feature,
    approved_gates: { context: true, shape: true, execution, review: false },
    workers: [],
    summary: '',
    next_action: '',
  });
  return root;
}

function addSession(root, id, extra = {}) {
  const now = new Date().toISOString();
  writeJson(path.join(root, '.bee', 'sessions', `${id}.json`), {
    id,
    started_at: now,
    last_heartbeat: now,
    ...extra,
  });
}

function addLane(root, feature, { phase = 'executing', mode = 'small', execution = true } = {}) {
  writeJson(path.join(root, '.bee', 'lanes', `${feature}.json`), {
    schema_version: '1.0',
    feature,
    mode,
    phase,
    approved_gates: { context: true, shape: true, execution, review: false },
    summary: '',
    next_action: '',
    created_at: new Date().toISOString(),
  });
}

function addCell(root, { id, feature = 'demo', status = 'open', deps = [], session = null, lane = 'small' }) {
  writeJson(path.join(root, '.bee', 'cells', `${id}.json`), {
    id,
    feature,
    title: `Cell ${id}`,
    lane,
    status,
    deps,
    action: 'Do the thing.',
    verify: 'node -e "process.exit(0)"',
    trace: { worker: 'tester', claim_session: session, claimed_at: new Date().toISOString() },
  });
}

function addAnchor(root, key, request = 'do the thing the user actually asked for') {
  writeJson(path.join(root, '.bee', 'intent', `${key}.json`), {
    schema_version: '1.0',
    key,
    written_at: new Date().toISOString(),
    request,
    acceptance: 'the thing is done',
    next_action: null,
    feature: null,
    lane: null,
    cell: null,
    do_not_reverse: [],
    stop_conditions: [],
  });
}

// Deliberate small duplicate of reservations.mjs's own reserve()<->lease
// field mapping (see that module's header) — seeds a real lease-store lease
// directly via the synchronous acquireLeases rather than reservations.mjs's
// async reserve(), so every check() row in this file can stay synchronous.
function addReservations(root, rows) {
  for (const row of rows) {
    acquireLeases(
      root,
      [
        {
          type: 'path',
          id: row.path,
          mode: 'write',
          workflow_id: row.cell,
          // Control-char-wrapped sentinel — never a real session id — mirrors
          // reservations.mjs's own SESSIONLESS_SESSION_ID so an omitted
          // `session` here reproduces a genuinely session-less/"legacy" row
          // through the shim.
          session_id: row.session || '\u0000bee-reservation-sessionless\u0000',
          workspace_id: `agent:${row.agent}`,
          epoch: 0,
          ttl: row.ttl,
          kind: 'lease',
        },
      ],
      { now: row.now },
    );
  }
}

function reservationRow({ agent = 'tester', cell, filePath, session, ageSeconds = 0, ttl = 3600 }) {
  return { agent, cell, path: filePath, session, ttl, now: Date.now() - ageSeconds * 1000 };
}

/** sha256 over every path + byte in a directory tree — the idempotence proof. */
function hashTree(dir) {
  const hash = crypto.createHash('sha256');
  const walk = (abs, rel) => {
    const entries = fs
      .readdirSync(abs, { withFileTypes: true })
      .sort((a, b) => a.name.localeCompare(b.name));
    for (const entry of entries) {
      const childAbs = path.join(abs, entry.name);
      const childRel = rel ? `${rel}/${entry.name}` : entry.name;
      if (entry.isDirectory()) {
        hash.update(`D:${childRel}\n`);
        walk(childAbs, childRel);
      } else {
        hash.update(`F:${childRel}:`);
        hash.update(fs.readFileSync(childAbs));
        hash.update('\n');
      }
    }
  };
  walk(dir, '');
  return hash.digest('hex');
}

function checkNamed(result, name) {
  const found = result.checks.find((entry) => entry.name === name);
  assert(found, `compactCheck must report a "${name}" check — got ${result.checks.map((c) => c.name).join(', ')}`);
  return found;
}

// ─── 1. the counting rule (D5/D9) ───────────────────────────────────────────

check('two complete compaction cycles: D9 fires on the SECOND precompact, never the first', () => {
  const root = makeRepo();
  const sid = 'sess-count';
  addSession(root, sid);
  addCell(root, { id: 'k-1', status: 'claimed', session: sid });

  const first = appendCompactionRecord(root, { event: 'precompact', sessionId: sid });
  assert(first.event === 'precompact', 'the first record is a precompact');
  assert(first.cell === 'k-1', `the record names the session's claimed cell — got ${first.cell}`);
  assert(first.compact_index === 1, `first precompact compact_index must be 1 — got ${first.compact_index}`);
  assert(
    first.cell_compact_count === 1,
    `first precompact cell_compact_count must be 1 — got ${first.cell_compact_count}`,
  );
  assert(
    survivalWarning(first.cell_compact_count) === null,
    'the FIRST compaction must not fire the D9 advisory',
  );

  const firstResume = appendCompactionRecord(root, { event: 'resume', sessionId: sid });
  assert(
    firstResume.compact_index === 1,
    `a resume carries the plain prior count — compact_index must stay 1, got ${firstResume.compact_index}`,
  );
  assert(
    firstResume.cell_compact_count === 1,
    `a resume never re-increments — cell_compact_count must stay 1, got ${firstResume.cell_compact_count}`,
  );
  assert(
    survivalWarning(firstResume.cell_compact_count) === null,
    'the resume of the FIRST compaction must not fire D9 — counting a resume inclusively would fire it a full cycle early',
  );

  const second = appendCompactionRecord(root, { event: 'precompact', sessionId: sid });
  assert(second.compact_index === 2, `second precompact compact_index must be 2 — got ${second.compact_index}`);
  assert(
    second.cell_compact_count === 2,
    `second precompact cell_compact_count must be 2 — got ${second.cell_compact_count}`,
  );
  const warning = survivalWarning(second.cell_compact_count);
  assert(typeof warning === 'string', 'the SECOND compaction must fire the D9 advisory');
  assert(warning.includes('2'), `the D9 advisory names the count — got "${warning}"`);
  assert(
    /survived/i.test(warning) && /capping/i.test(warning),
    `the D9 advisory states the survival count and the cap-and-hand-off suggestion — got "${warning}"`,
  );

  const secondResume = appendCompactionRecord(root, { event: 'resume', sessionId: sid });
  assert(
    secondResume.compact_index === 2 && secondResume.cell_compact_count === 2,
    `the second resume carries 2/2, never 3 — got ${secondResume.compact_index}/${secondResume.cell_compact_count}`,
  );

  const records = readCompactionRecords(root);
  assert(records.length === 4, `four records were appended — got ${records.length}`);
  assert(
    records.filter((record) => record.event === 'precompact').length === 2,
    'exactly two of the four records are precompact records',
  );
});

check('the record carries every D5 field, and only precompact records are counted', () => {
  const root = makeRepo({ feature: 'demo' });
  const sid = 'sess-shape';
  addSession(root, sid);
  addCell(root, { id: 'k-1', status: 'claimed', session: sid });
  addAnchor(root, 'demo');

  const record = appendCompactionRecord(root, { event: 'precompact', sessionId: sid });
  const expected = [
    'ts',
    'event',
    'session',
    'lane',
    'feature',
    'phase',
    'cell',
    'compact_index',
    'cell_compact_count',
    'anchor_present',
  ];
  for (const field of expected) {
    assert(Object.prototype.hasOwnProperty.call(record, field), `the record must carry "${field}" (D5)`);
  }
  assert(record.session === sid, 'the record names the session');
  assert(record.feature === 'demo' && record.phase === 'swarming', 'feature/phase come from the resolved pipeline');
  assert(record.lane === null, 'an unbound session logs lane null');
  assert(record.anchor_present === true, 'anchor_present is true when an anchor exists');
  assert(typeof record.ts === 'string' && !Number.isNaN(Date.parse(record.ts)), 'ts is an ISO timestamp');

  const counts = readCompactionCounts(root, { sessionId: sid, cell: 'k-1' });
  assert(
    counts.compact_index === 1 && counts.cell_compact_count === 1,
    `readCompactionCounts returns the PLAIN prior precompact counts — got ${JSON.stringify(counts)}`,
  );

  appendCompactionRecord(root, { event: 'resume', sessionId: sid });
  const afterResume = readCompactionCounts(root, { sessionId: sid, cell: 'k-1' });
  assert(
    afterResume.compact_index === 1 && afterResume.cell_compact_count === 1,
    `a resume record is never counted — got ${JSON.stringify(afterResume)}`,
  );
});

check('counts are scoped to the session and to the (session, cell) pair', () => {
  const root = makeRepo();
  addSession(root, 'sess-a');
  addSession(root, 'sess-b');
  addCell(root, { id: 'k-1', status: 'claimed', session: 'sess-a' });
  addCell(root, { id: 'k-2', status: 'claimed', session: 'sess-b' });

  appendCompactionRecord(root, { event: 'precompact', sessionId: 'sess-a' });
  appendCompactionRecord(root, { event: 'precompact', sessionId: 'sess-a' });
  const other = appendCompactionRecord(root, { event: 'precompact', sessionId: 'sess-b' });

  assert(
    other.compact_index === 1 && other.cell_compact_count === 1,
    `another session's records never bleed into this one — got ${JSON.stringify(other)}`,
  );
  const a = readCompactionCounts(root, { sessionId: 'sess-a', cell: 'k-1' });
  assert(a.compact_index === 2 && a.cell_compact_count === 2, `sess-a keeps its own counts — got ${JSON.stringify(a)}`);
  const crossCell = readCompactionCounts(root, { sessionId: 'sess-a', cell: 'k-2' });
  assert(
    crossCell.compact_index === 2 && crossCell.cell_compact_count === 0,
    `the cell count is pair-scoped — got ${JSON.stringify(crossCell)}`,
  );
});

check('survivalWarning boundary: null below the threshold, a string at and above it', () => {
  assert(SURVIVAL_WARNING_THRESHOLD === 2, 'the D9 threshold is 2');
  assert(survivalWarning(0) === null, '0 compactions does not warn');
  assert(survivalWarning(1) === null, '1 compaction does not warn');
  assert(typeof survivalWarning(2) === 'string', '2 compactions warns');
  const three = survivalWarning(3);
  assert(typeof three === 'string' && three.includes('3'), '3 compactions warns and names the count');
  assert(survivalWarning(null) === null, 'a non-numeric count never warns');
  assert(survivalWarning('nope') === null, 'a non-numeric count never warns');
});

check('a session with no claimed cell logs cell null and a zero cell count', () => {
  const root = makeRepo();
  const sid = 'sess-nocell';
  addSession(root, sid);
  const record = appendCompactionRecord(root, { event: 'precompact', sessionId: sid });
  assert(record.cell === null, `no claimed cell means cell null — got ${record.cell}`);
  assert(record.cell_compact_count === 0, 'with no unit there is no per-unit survival count');
  assert(record.compact_index === 1, 'the session-level count still advances');
  assert(survivalWarning(record.cell_compact_count) === null, 'no unit, no advisory');
});

check('an unknown event is refused as an argument error, never logged', () => {
  const root = makeRepo();
  addSession(root, 'sess-bad');
  let threw = false;
  try {
    appendCompactionRecord(root, { event: 'compacted', sessionId: 'sess-bad' });
  } catch (error) {
    threw = true;
    assert(
      String(error.message).includes('precompact'),
      `the refusal names the two legal events — got "${error.message}"`,
    );
  }
  assert(threw, 'an unknown event throws rather than writing a record of an event that does not exist');
  assert(readCompactionRecords(root).length === 0, 'nothing was written');
  assert(
    Array.isArray(COMPACT_EVENTS) && COMPACT_EVENTS.length === 2,
    'exactly two events exist (D5): precompact and resume',
  );
});

// ─── 2. the append never changes the caller (D4) ────────────────────────────

check('a failed log append leaves the return value and the exit code unchanged (D4)', () => {
  const root = makeRepo();
  const sid = 'sess-io';
  addSession(root, sid);
  addCell(root, { id: 'k-1', status: 'claimed', session: sid });
  // Poison the log path with a DIRECTORY: appendFileSync throws EISDIR on
  // every platform, which is the write failure D4's local try/catch must
  // absorb without touching the caller.
  fs.mkdirSync(compactionLogPath(root), { recursive: true });

  const record = appendCompactionRecord(root, { event: 'precompact', sessionId: sid });
  assert(record !== null && record.event === 'precompact', 'the caller still gets its record back');
  assert(
    record.compact_index === 1 && record.cell_compact_count === 1,
    `the counts are still computed — got ${JSON.stringify(record)}`,
  );

  const script = path.join(root, 'caller.mjs');
  fs.writeFileSync(
    script,
    [
      `import { appendCompactionRecord } from ${JSON.stringify(MODULE_URL)};`,
      `appendCompactionRecord(${JSON.stringify(root)}, { event: 'precompact', sessionId: ${JSON.stringify(sid)} });`,
      '',
    ].join('\n'),
    'utf8',
  );
  const spawned = spawnSync(process.execPath, [script], { encoding: 'utf8' });
  assert(
    spawned.status === 0,
    `a caller whose only work is the failing append must still exit 0 — got ${spawned.status}: ${spawned.stderr}`,
  );
});

// ─── 3. a lane-bound session logs its own lane ─────────────────────────────

check('a lane-bound session records the LANE feature/phase, not the default state.json', () => {
  const root = makeRepo({ phase: 'swarming', feature: 'default-feature' });
  addLane(root, 'lane-feat', { phase: 'executing', mode: 'small' });
  addSession(root, 'sess-lane', { lane: 'lane-feat' });
  addSession(root, 'sess-plain');

  const bound = appendCompactionRecord(root, { event: 'precompact', sessionId: 'sess-lane' });
  assert(bound.lane === 'lane-feat', `the bound lane is named — got ${bound.lane}`);
  assert(bound.feature === 'lane-feat', `feature comes from the lane record — got ${bound.feature}`);
  assert(bound.phase === 'executing', `phase comes from the lane record — got ${bound.phase}`);

  const plain = appendCompactionRecord(root, { event: 'precompact', sessionId: 'sess-plain' });
  assert(plain.lane === null, 'an unbound session has no lane');
  assert(
    plain.feature === 'default-feature' && plain.phase === 'swarming',
    `an unbound session still reads the default pipeline — got ${plain.feature}/${plain.phase}`,
  );
});

check('an unresolvable lane binding never logs the default pipeline as if it were the lane', () => {
  const root = makeRepo({ phase: 'swarming', feature: 'default-feature' });
  addSession(root, 'sess-broken', { lane: 'ghost-lane' });
  const record = appendCompactionRecord(root, { event: 'precompact', sessionId: 'sess-broken' });
  assert(record.lane === 'ghost-lane', 'the bound lane name is still recorded');
  assert(
    record.feature === null && record.phase === null,
    `an unresolvable binding logs no feature/phase rather than the WRONG pipeline's — got ${record.feature}/${record.phase}`,
  );
});

// ─── 4. the sweep surfaces typed lane refusals (D12) ───────────────────────

check('compactCheck surfaces LANE_MISSING rather than falling back silently', () => {
  const root = makeRepo();
  addSession(root, 'sess-lm', { lane: 'ghost-lane' });
  const result = compactCheck(root, { sessionId: 'sess-lm' });
  const lane = checkNamed(result, 'lane_binding');
  assert(lane.ok === false, 'a missing lane is a failed check');
  assert(lane.code === 'LANE_MISSING', `the typed code is surfaced — got ${lane.code}`);
  assert(result.ok === false, 'the overall sweep fails');
  assert(
    result.mismatches.some((entry) => entry.code === 'LANE_MISSING'),
    'the typed code appears in the mismatch list the capsule renders',
  );
});

check('compactCheck surfaces LANE_CORRUPT rather than swallowing the parse failure', () => {
  const root = makeRepo();
  fs.mkdirSync(path.join(root, '.bee', 'lanes'), { recursive: true });
  fs.writeFileSync(path.join(root, '.bee', 'lanes', 'busted.json'), '{ not json at all', 'utf8');
  addSession(root, 'sess-lc', { lane: 'busted' });
  const lane = checkNamed(compactCheck(root, { sessionId: 'sess-lc' }), 'lane_binding');
  assert(lane.ok === false && lane.code === 'LANE_CORRUPT', `expected LANE_CORRUPT — got ${lane.code}`);
});

check('compactCheck surfaces LANE_INVALID for a lane name that is not a lane name', () => {
  const root = makeRepo();
  addSession(root, 'sess-li', { lane: '../escape' });
  const lane = checkNamed(compactCheck(root, { sessionId: 'sess-li' }), 'lane_binding');
  assert(lane.ok === false && lane.code === 'LANE_INVALID', `expected LANE_INVALID — got ${lane.code}`);
});

// ─── 5. the sweep's other checks (D12/D13) ─────────────────────────────────

check('compactCheck reports ok on a healthy session', () => {
  const root = makeRepo();
  const sid = 'sess-ok';
  addSession(root, sid);
  addCell(root, { id: 'k-0', status: 'capped' });
  addCell(root, { id: 'k-1', status: 'claimed', session: sid, deps: ['k-0'] });
  addAnchor(root, 'demo');
  addReservations(root, [reservationRow({ cell: 'k-1', filePath: 'src/a.mjs', session: sid })]);

  const result = compactCheck(root, { sessionId: sid });
  assert(result.ok === true, `a healthy session sweeps clean — mismatches: ${JSON.stringify(result.mismatches)}`);
  assert(result.mismatches.length === 0, 'no mismatches');
  for (const name of ['session_record', 'lane_binding', 'claimed_cells', 'execution_gate', 'deps_capped', 'reservations', 'anchor']) {
    assert(checkNamed(result, name).ok === true, `check "${name}" is green on a healthy session`);
  }
});

check('compactCheck flags a session record that is absent or whose stored id disagrees', () => {
  const absent = makeRepo();
  const missing = checkNamed(compactCheck(absent, { sessionId: 'sess-nope' }), 'session_record');
  assert(missing.ok === false && missing.code === 'SESSION_MISSING', `expected SESSION_MISSING — got ${missing.code}`);

  const mismatched = makeRepo();
  writeJson(path.join(mismatched, '.bee', 'sessions', 'sess-x.json'), {
    id: 'sess-y',
    started_at: new Date().toISOString(),
    last_heartbeat: new Date().toISOString(),
  });
  const wrong = checkNamed(compactCheck(mismatched, { sessionId: 'sess-x' }), 'session_record');
  assert(
    wrong.ok === false && wrong.code === 'SESSION_ID_MISMATCH',
    `a record whose stored id disagrees is its own failure — got ${wrong.code}`,
  );
});

check('compactCheck flags a cell this session claimed that is no longer claimed', () => {
  const root = makeRepo();
  const sid = 'sess-lost';
  addSession(root, sid);
  addCell(root, { id: 'k-1', status: 'capped', session: sid });
  const claimed = checkNamed(compactCheck(root, { sessionId: sid }), 'claimed_cells');
  assert(claimed.ok === false, 'a cell that moved out of claimed is a mismatch');
  assert(/k-1/.test(claimed.detail), `the failing cell is named — got "${claimed.detail}"`);
});

check('compactCheck flags a revoked execution gate and an uncapped dependency', () => {
  const revoked = makeRepo({ execution: false });
  addSession(revoked, 'sess-gate');
  addCell(revoked, { id: 'k-1', status: 'claimed', session: 'sess-gate' });
  const gate = checkNamed(compactCheck(revoked, { sessionId: 'sess-gate' }), 'execution_gate');
  assert(gate.ok === false, 'a claimed cell with an unapproved execution gate is a mismatch');

  const deps = makeRepo();
  addSession(deps, 'sess-deps');
  addCell(deps, { id: 'k-0', status: 'open' });
  addCell(deps, { id: 'k-1', status: 'claimed', session: 'sess-deps', deps: ['k-0'] });
  const depsCheck = checkNamed(compactCheck(deps, { sessionId: 'sess-deps' }), 'deps_capped');
  assert(depsCheck.ok === false, 'an uncapped dependency is a mismatch');
  assert(/k-0/.test(depsCheck.detail), `the uncapped dep is named — got "${depsCheck.detail}"`);
});

check('compactCheck reports a session-less reservation row as unbound, never as a mismatch (D13)', () => {
  const root = makeRepo();
  const sid = 'sess-unbound';
  addSession(root, sid);
  addCell(root, { id: 'k-1', status: 'claimed', session: sid });
  addAnchor(root, 'demo');
  // reservations.mjs:92-98 — a row with no `session` field is a legacy /
  // intra-swarm row. It is reported, and it is NOT a mismatch.
  addReservations(root, [reservationRow({ cell: 'k-1', filePath: 'src/a.mjs' })]);

  const result = compactCheck(root, { sessionId: sid });
  const reservations = checkNamed(result, 'reservations');
  assert(reservations.ok === true, 'a session-less row never fails the sweep');
  assert(reservations.unbound === 1, `the row is counted as unbound — got ${reservations.unbound}`);
  assert(/unbound/i.test(reservations.detail), `the detail says unbound — got "${reservations.detail}"`);
  assert(result.ok === true, 'the sweep as a whole is still clean');
});

check('compactCheck flags this session\'s own reservation once it has expired', () => {
  const root = makeRepo();
  const sid = 'sess-expired';
  addSession(root, sid);
  addCell(root, { id: 'k-1', status: 'claimed', session: sid });
  addReservations(root, [
    reservationRow({ cell: 'k-1', filePath: 'src/a.mjs', session: sid, ttl: 60, ageSeconds: 600 }),
  ]);
  const reservations = checkNamed(compactCheck(root, { sessionId: sid }), 'reservations');
  assert(reservations.ok === false, 'an expired hold this session believes it still owns is a mismatch');
  assert(/src\/a\.mjs/.test(reservations.detail), `the path is named — got "${reservations.detail}"`);
});

check('compactCheck skips the session-scoped checks when no session id is supplied', () => {
  const root = makeRepo();
  const result = compactCheck(root, {});
  const session = checkNamed(result, 'session_record');
  assert(session.ok === true && session.skipped === true, 'no session id means nothing to check, not a failure');
  assert(checkNamed(result, 'lane_binding').ok === true, 'the default pipeline resolves');
});

check('compactCheck mutates nothing: the .bee/ tree hashes identically across two runs (D13)', () => {
  const root = makeRepo();
  const sid = 'sess-idem';
  addSession(root, sid);
  addLane(root, 'demo-lane');
  addCell(root, { id: 'k-0', status: 'capped' });
  addCell(root, { id: 'k-1', status: 'claimed', session: sid, deps: ['k-0'] });
  addAnchor(root, 'demo');
  addReservations(root, [
    reservationRow({ cell: 'k-1', filePath: 'src/a.mjs', session: sid }),
    reservationRow({ cell: 'k-1', filePath: 'src/b.mjs' }),
  ]);
  appendCompactionRecord(root, { event: 'precompact', sessionId: sid });

  const beeDir = path.join(root, '.bee');
  const before = hashTree(beeDir);
  compactCheck(root, { sessionId: sid });
  const afterFirst = hashTree(beeDir);
  compactCheck(root, { sessionId: sid });
  const afterSecond = hashTree(beeDir);

  assert(afterFirst === before, 'the first sweep left the .bee/ tree byte-identical');
  assert(afterSecond === before, 'the second sweep left the .bee/ tree byte-identical');
});

// ─── 6. the anchor predicate (D10/D11) ─────────────────────────────────────

check('anchorMissing fires when work is active, a cell is claimed, and no anchor exists', () => {
  const root = makeRepo();
  const sid = 'sess-nudge';
  addSession(root, sid);
  addCell(root, { id: 'k-1', status: 'claimed', session: sid });

  const nudge = anchorMissing(root, { sessionId: sid });
  assert(nudge !== null, 'the nudge fires');
  assert(typeof nudge.command === 'string' && nudge.command.includes('intent set'), `the exact command is named — got ${nudge.command}`);
  assert(nudge.cell === 'k-1', `the claimed cell is carried for the dedup hash — got ${nudge.cell}`);
  assert(nudge.hash === `${sid}:demo:k-1`, `the D11 hash is <sessionId>:<feature>:<cell> — got ${nudge.hash}`);
  assert(typeof nudge.message === 'string' && nudge.message.includes(nudge.command), 'the message carries the command');
  assert(ANCHOR_NUDGE_KEY === 'anchor-missing-nudge', 'the D11 dedup key is anchor-missing-nudge');
});

check('anchorMissing stays silent once an anchor exists', () => {
  const root = makeRepo();
  const sid = 'sess-anchored';
  addSession(root, sid);
  addCell(root, { id: 'k-1', status: 'claimed', session: sid });
  addAnchor(root, 'demo');
  assert(anchorMissing(root, { sessionId: sid }) === null, 'an existing anchor silences the nudge');
});

check('anchorMissing stays silent in a terminal phase, even with the execution gate approved', () => {
  for (const phase of ['idle', 'compounding-complete']) {
    const root = makeRepo({ phase, execution: true });
    addSession(root, 'sess-term');
    addCell(root, { id: 'k-1', status: 'claimed', session: 'sess-term' });
    assert(
      anchorMissing(root, { sessionId: 'sess-term' }) === null,
      `phase "${phase}" is terminal (state.mjs:1655) — no work is active, so no nudge`,
    );
  }
});

check('anchorMissing fires on an approved execution gate with no claimed cell, and not otherwise', () => {
  const approved = makeRepo({ execution: true });
  addSession(approved, 'sess-gate-yes');
  const fires = anchorMissing(approved, { sessionId: 'sess-gate-yes' });
  assert(fires !== null, 'an approved execution gate alone is enough (D10)');
  assert(fires.cell === null, 'with no claimed cell the nudge carries a null cell');

  const unapproved = makeRepo({ execution: false });
  addSession(unapproved, 'sess-gate-no');
  assert(
    anchorMissing(unapproved, { sessionId: 'sess-gate-no' }) === null,
    'no claimed cell and no approved execution gate means no active work, so no nudge',
  );
});

check('anchorMissing reads the lane-bound pipeline, not the default one', () => {
  const root = makeRepo({ phase: 'idle', feature: 'default-feature', execution: false });
  addLane(root, 'lane-feat', { phase: 'executing', execution: true });
  addSession(root, 'sess-lane-nudge', { lane: 'lane-feat' });
  const nudge = anchorMissing(root, { sessionId: 'sess-lane-nudge' });
  assert(nudge !== null, 'the lane is mid-flight with execution approved — the nudge fires');
  assert(nudge.feature === 'lane-feat', `the nudge names the lane's feature — got ${nudge.feature}`);
});

// ─── cleanup ────────────────────────────────────────────────────────────────

process.on('exit', () => {
  for (const root of tempRoots) {
    try {
      fs.rmSync(root, { recursive: true, force: true });
    } catch {
      /* best-effort: a leftover temp dir is harmless */
    }
  }
});

printSummaryAndExit();
