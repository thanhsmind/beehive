#!/usr/bin/env node
// test_guards.mjs — guard-lib contract tests (checkWrite/checkRead + lane
// enforcement/presentation readers + cross-session hold hard block), split
// out of test_lib.mjs (cs-2b) to shrink the monolith. Same PASS/FAIL/exit-1
// contract as every other suite here — see scripts/lib/test-fixture.mjs.

import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import {
  makeTempRepo,
  makeCell,
  check,
  assert,
  assertThrows,
  assertRejects,
  printSummaryAndExit,
} from '../../../../scripts/lib/test-fixture.mjs';
import { defaultState, readState, writeState } from '../lib/state.mjs';
import { readCell, claimCell } from '../lib/cells.mjs';
import { reserve, reservationsPath } from '../lib/reservations.mjs';
import { createSession } from '../lib/claims.mjs';
// fsh-3 (lane store): namespace imports so a not-yet-implemented export fails
// its own row ("… is not a function") instead of crashing the whole module
// graph at import time — the RED-first evidence stays per-row.
import * as laneStore from '../lib/state.mjs';
import * as laneBinding from '../lib/claims.mjs';
import { checkWrite, checkRead, extractBashTargets, checkAskUserQuestion } from '../lib/guards.mjs';
import { buildPromptReminder, buildSessionPreamble } from '../lib/inject.mjs';
import { readJson, writeJsonAtomic } from '../lib/fsutil.mjs';

// Hermeticity (hardening-1-7-10 D1 + okf-integration-close-f4 f4-4, defense
// in depth): this suite must never inherit the harness's own identity.
// run_verify.mjs already scrubs all three vars for every child suite it
// spawns; deleting BEE_AGENT_NAME here at BOOTSTRAP means a bare
// `node skills/.../test_guards.mjs`, run directly under the very
// `BEE_AGENT_NAME=<name>` prefix AGENTS.md critical rule 5 mandates for
// write-heavy commands, is equally hermetic instead of leaking that name
// into checkWrite's cross-session hold checks and turning "the acting
// session's own hold must never block its own write" red. The later
// save/delete/restore pairs in individual cases below are a DIFFERENT
// mechanism — each sets the var deliberately to exercise the swarming
// branch and puts it back — and this bootstrap delete is what gives them a
// clean starting value to restore to.
delete process.env.BEE_AGENT_NAME;

const root = makeTempRepo();

// Self-containment fix (cs-2b split): makeStateRepo/makeCellFile are defined
// in test_lib.mjs's "bee.mjs state CLI"/"bee.mjs state start-feature" sections
// (now test_cli_state.mjs, a different file); laneFile/writeLaneFixture are
// defined in the "lanes" section (now test_state.mjs, also a different file).
// All four were only reachable here via function-declaration hoisting across
// the whole monolith. The enforcement/presentation/cross-session-hold rows
// below need them. Verbatim copies, same shape, same behavior, zero check
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

function laneFile(dir, feature) {
  return path.join(dir, '.bee', 'lanes', `${feature}.json`);
}

function writeLaneFixture(dir, feature, extra = {}) {
  laneStore.writeLane(dir, {
    schema_version: '1.0',
    feature,
    mode: null,
    phase: 'idle',
    approved_gates: { context: false, shape: false, execution: false, review: false },
    summary: '',
    next_action: '',
    created_at: new Date().toISOString(),
    ...extra,
  });
}

// ─── guards ─────────────────────────────────────────────────────────────────

await check('checkWrite blocks source writes while idle (intake gate); config can disable it', async () => {
  const state = defaultState(); // phase: idle
  const denied = checkWrite(root, state, 'src/app.ts');
  assert(denied.allow === false && denied.kind === 'intake', 'intake deny expected while idle');
  assert(denied.reason.includes('bee-hive'), 'intake reason should point at bee-hive routing');
  const docsOk = checkWrite(root, state, 'docs/notes.md');
  assert(docsOk.allow === true, 'docs/ writes stay allowed while idle');
  const configPath = path.join(root, '.bee', 'config.json');
  const before = readJson(configPath, {});
  writeJsonAtomic(configPath, { ...before, guards: { idle_gate: false } });
  const off = checkWrite(root, state, 'src/app.ts');
  assert(off.allow === true, 'idle gate must be disableable via guards.idle_gate=false');
  writeJsonAtomic(configPath, before || {});
});

await check('checkAskUserQuestion turns opaque "Invalid tool parameters" into a clear, specific deny; fail-open on odd shapes', async () => {
  // Valid question is allowed.
  const ok = { questions: [{ question: 'Which approach?', header: 'Approach', multiSelect: false, options: [{ label: 'A', description: 'do A' }, { label: 'B', description: 'do B' }] }] };
  assert(checkAskUserQuestion(ok).allow === true, 'a valid AskUserQuestion must be allowed');
  // header > 12 chars is now a FIXABLE violation (ask-guard-autofix D1/D2):
  // auto-rewritten to the first 11 chars right-trimmed + '…', call proceeds
  // allowed, with the rewrite reported in `notes` and the original input
  // never mutated.
  const longHeaderInput = { questions: [{ question: 'q', header: 'Worktree switch', options: [{ label: 'A', description: 'x' }, { label: 'B', description: 'y' }] }] };
  const longHeader = checkAskUserQuestion(longHeaderInput);
  assert(longHeader.allow === true, `an over-long header must be auto-fixed and allowed, not denied, got ${JSON.stringify(longHeader)}`);
  assert(
    longHeader.fixed?.questions?.[0]?.header === 'Worktree sw…',
    `the fixed header must be 'Worktree sw…', got ${JSON.stringify(longHeader.fixed)}`,
  );
  assert(
    Array.isArray(longHeader.notes) &&
      longHeader.notes.length === 1 &&
      /Worktree switch/.test(longHeader.notes[0]) &&
      /Worktree sw…/.test(longHeader.notes[0]),
    `notes must name old -> new, got ${JSON.stringify(longHeader.notes)}`,
  );
  assert(
    longHeaderInput.questions[0].header === 'Worktree switch',
    'the original toolInput must never be mutated — fixed is a deep clone',
  );
  // >4 options denied; <2 options denied.
  assert(checkAskUserQuestion({ questions: [{ question: 'q', header: 'h', options: [1, 2, 3, 4, 5].map((n) => ({ label: `L${n}`, description: 'd' })) }] }).allow === false, '5 options must deny');
  assert(checkAskUserQuestion({ questions: [{ question: 'q', header: 'h', options: [{ label: 'only', description: 'd' }] }] }).allow === false, '1 option must deny');
  // >4 questions denied.
  assert(checkAskUserQuestion({ questions: [1, 2, 3, 4, 5].map(() => ({ question: 'q', header: 'h', options: [{ label: 'A', description: 'd' }, { label: 'B', description: 'd' }] })) }).allow === false, '5 questions must deny');
  // missing label / description denied.
  assert(checkAskUserQuestion({ questions: [{ question: 'q', header: 'h', options: [{ description: 'no label' }, { label: 'B', description: 'd' }] }] }).allow === false, 'missing label must deny');
  assert(checkAskUserQuestion({ questions: [{ question: 'q', header: 'h', options: [{ label: 'A' }, { label: 'B', description: 'd' }] }] }).allow === false, 'missing description must deny');
  // Fail-open: unrecognized / absent shapes are never blocked.
  assert(checkAskUserQuestion({}).allow === true, 'no questions key -> allow (fail-open)');
  assert(checkAskUserQuestion(null).allow === true, 'null input -> allow (fail-open)');
  assert(checkAskUserQuestion({ questions: 'weird' }).allow === true, 'non-array questions -> allow (fail-open)');
});

await check('checkAskUserQuestion: multi-question call only rewrites the long header, other questions byte-identical', async () => {
  const input = {
    questions: [
      { question: 'q1', header: 'Deploy staging', options: [{ label: 'A', description: 'x' }, { label: 'B', description: 'y' }] },
      { question: 'q2', header: 'OK', options: [{ label: 'C', description: 'z' }, { label: 'D', description: 'w' }] },
    ],
  };
  const verdict = checkAskUserQuestion(input);
  assert(verdict.allow === true, `a call with one long header among several must be allowed, got ${JSON.stringify(verdict)}`);
  assert(
    verdict.fixed.questions[0].header === 'Deploy stag…',
    `the long header must be truncated, got ${JSON.stringify(verdict.fixed)}`,
  );
  assert(
    JSON.stringify(verdict.fixed.questions[1]) === JSON.stringify(input.questions[1]),
    'the untouched question must stay byte-identical to the original',
  );
  assert(verdict.notes.length === 1, `only the fixed header should produce a note, got ${JSON.stringify(verdict.notes)}`);
});

await check('checkAskUserQuestion: a fixable header alongside an unfixable violation still denies with the unfixable reason (deny wins)', async () => {
  const input = {
    questions: [
      { question: 'q1', header: 'Deploy staging', options: [{ label: 'A', description: 'x' }, { label: 'B', description: 'y' }] },
      { question: 'q2', header: 'OK', options: [{ label: 'only-one', description: 'z' }] },
    ],
  };
  const verdict = checkAskUserQuestion(input);
  assert(verdict.allow === false && verdict.kind === 'ask-schema', `a fixable+unfixable mix must deny, got ${JSON.stringify(verdict)}`);
  assert(/1 option/.test(verdict.reason), `the deny reason must name the unfixable violation, got ${verdict.reason}`);
  assert(verdict.fixed === undefined, 'a deny verdict must never carry a fixed field');
});

await check('checkAskUserQuestion: an exactly-12-char header is left untouched (not a violation)', async () => {
  const twelve = 'ExactlyTwelv';
  assert(twelve.length === 12, 'fixture sanity: header must be exactly 12 chars');
  const input = { questions: [{ question: 'q', header: twelve, options: [{ label: 'A', description: 'x' }, { label: 'B', description: 'y' }] }] };
  const verdict = checkAskUserQuestion(input);
  assert(
    verdict.allow === true && verdict.fixed === undefined,
    `an exactly-12-char header must be allowed untouched with no fixed field, got ${JSON.stringify(verdict)}`,
  );
});

await check('checkWrite denies executable/code files under docs/history/ (the .md-only knowledge layer) in every phase (GitHub #17)', async () => {
  // Active work (execution approved) — the intake gate is NOT the reason here.
  const active = { ...defaultState(), phase: 'validating', approved_gates: { context: true, shape: true, execution: true, review: false } };
  const shDeny = checkWrite(root, active, 'docs/history/industry-count-company-registered/verify.sh');
  assert(shDeny.allow === false && shDeny.kind === 'docs-history-code', `a .sh under docs/history/ must be denied, got ${JSON.stringify(shDeny)}`);
  assert(/spikes|project|\.md/.test(shDeny.reason), 'the reason should point at .bee/spikes/ or the project scripts');
  // Other code extensions too.
  for (const p of ['docs/history/f/helper.mjs', 'docs/history/f/tool.py', 'docs/history/f/x.js']) {
    assert(checkWrite(root, active, p).allow === false, `${p} should be denied`);
  }
  // But .md knowledge under docs/history/ stays allowed, and code elsewhere is unaffected by THIS rule.
  assert(checkWrite(root, active, 'docs/history/f/report.md').allow === true, 'a .md under docs/history/ stays allowed');
  assert(checkWrite(root, active, 'docs/history/f/evidence.json').allow === true, 'a .json under docs/history/ stays allowed');
  assert(checkWrite(root, active, 'scripts/verify.sh').allow === true, 'a .sh outside docs/history/ is not this rule\'s concern');
});

await check('checkWrite blocks source writes at compounding-complete — a closed feature is not an open door (c2c46488)', async () => {
  // The killer case: the feature closed, so phase is the terminal alias and the
  // gates are STILL approved from that closed feature. Before the fix, the idle
  // branch missed the phase, the gated branch saw execution:true, and the write
  // fell through to allow — every post-feature edit skipped bee entirely.
  const state = {
    ...defaultState(),
    phase: 'compounding-complete',
    approved_gates: { context: true, shape: true, execution: true, review: true },
  };
  const denied = checkWrite(root, state, 'assets/css/tasks.css');
  assert(
    denied.allow === false && denied.kind === 'intake',
    'intake deny expected at compounding-complete even with every gate still approved',
  );
  assert(
    denied.reason.includes('compounding-complete'),
    'the deny reason must name the actual phase, not hardcode "idle"',
  );
  const docsOk = checkWrite(root, state, 'docs/specs/tasks.md');
  assert(docsOk.allow === true, 'docs/ (scribing, compounding) must stay writable at compounding-complete');
  const beeOk = checkWrite(root, state, '.bee/cells/demo-9.json');
  assert(beeOk.allow === true, '.bee/ bookkeeping must stay writable at compounding-complete');
  const configPath = path.join(root, '.bee', 'config.json');
  const before = readJson(configPath, {});
  writeJsonAtomic(configPath, { ...before, guards: { idle_gate: false } });
  const off = checkWrite(root, state, 'assets/css/tasks.css');
  assert(off.allow === true, 'guards.idle_gate=false must disable the gate for both terminal phases, not just idle');
  writeJsonAtomic(configPath, before || {});
});

await check('checkWrite blocks source writes in a gated phase without execution approval', async () => {
  const state = { ...defaultState(), phase: 'planning' };
  const denied = checkWrite(root, state, 'src/app.ts');
  assert(denied.allow === false && denied.kind === 'gate', 'gate deny expected');
  const allowed = checkWrite(root, state, 'docs/history/demo/plan.md');
  assert(allowed.allow === true, 'docs/history/ writes allowed in gated phases');
});

await check('checkWrite blocks unreserved conflicting writes during swarming', async () => {
  await reserve(root, { agent: 'worker-a', cell: 'demo-2', path: 'src/core/engine.ts' });
  const state = { ...defaultState(), phase: 'swarming', approved_gates: { ...defaultState().approved_gates, execution: true } };
  const denied = checkWrite(root, state, 'src/core/engine.ts', 'worker-b');
  assert(denied.allow === false && denied.kind === 'reservation', 'reservation deny expected');
  const own = checkWrite(root, state, 'src/core/engine.ts', 'worker-a');
  assert(own.allow === true, 'holder may write its reserved path');
});

// ─── multisession-native-13: intent/lease split (D4, advisor consult slice 3
// condition D) ────────────────────────────────────────────────────────────
// Condition D binding: intra-swarm agent-keyed reservations stay HARD by
// default (kind:'lease', the row shape from before this cell); ONLY an
// explicitly kind:'intent' row whose overlap is broad/glob-only (not the
// exact same path) downgrades to an advisory allow+warning. pathsOverlap
// itself is never touched by this cell — schedule.mjs/state.mjs/cells.mjs
// keep exactly the same broad-overlap semantics for wave planning.

await check("checkWrite: a broad 'intent' (src/api/*) does NOT hard-block a disjoint exact write — allow:true with a warning (D4, prohibition: no hard deny from an intent record)", async () => {
  await reserve(root, { agent: 'planner', cell: 'intent-1', path: 'src/api/*', kind: 'intent' });
  const state = { ...defaultState(), phase: 'swarming', approved_gates: { ...defaultState().approved_gates, execution: true } };
  const verdict = checkWrite(root, state, 'src/api/orders/x.ts', 'worker-c');
  assert(verdict.allow === true, `a broad intent must never hard-block a disjoint write, got ${JSON.stringify(verdict)}`);
  assert(
    typeof verdict.warning === 'string' && verdict.warning.includes('src/api/*') && verdict.warning.includes('intent'),
    `an advisory warning must name the covering intent, got ${JSON.stringify(verdict.warning)}`,
  );
});

await check("checkWrite: an exact-path 'lease' (default kind) still hard-blocks a conflicting write — Condition D regression, unchanged from before this cell", async () => {
  await reserve(root, { agent: 'worker-lease', cell: 'lease-1', path: 'src/lease/exact.ts' });
  const state = { ...defaultState(), phase: 'swarming', approved_gates: { ...defaultState().approved_gates, execution: true } };
  const denied = checkWrite(root, state, 'src/lease/exact.ts', 'worker-other');
  assert(
    denied.allow === false && denied.kind === 'reservation' && denied.warning === undefined,
    `an exact lease conflict must stay a hard deny with no warning field, got ${JSON.stringify(denied)}`,
  );
});

await check("checkWrite: an 'intent' row that collapses onto the EXACT write target still hard-blocks — a same-resource collision is never merely advisory, regardless of its kind label", async () => {
  await reserve(root, { agent: 'planner', cell: 'intent-2', path: 'src/exact-intent/target.ts', kind: 'intent' });
  const state = { ...defaultState(), phase: 'swarming', approved_gates: { ...defaultState().approved_gates, execution: true } };
  const denied = checkWrite(root, state, 'src/exact-intent/target.ts', 'worker-other');
  assert(
    denied.allow === false && denied.kind === 'reservation',
    `an intent that IS the exact write target must still hard-deny, got ${JSON.stringify(denied)}`,
  );
});

await check('checkWrite: a mix of one hard lease conflict and one advisory intent conflict on the SAME path still denies (hard wins) — never silently drops the lease conflict because an intent also overlapped', async () => {
  await reserve(root, { agent: 'planner', cell: 'intent-3', path: 'src/mixed/*', kind: 'intent' });
  await reserve(root, { agent: 'worker-lease', cell: 'lease-3', path: 'src/mixed/file.ts' });
  const state = { ...defaultState(), phase: 'swarming', approved_gates: { ...defaultState().approved_gates, execution: true } };
  const denied = checkWrite(root, state, 'src/mixed/file.ts', 'worker-other');
  assert(
    denied.allow === false && denied.kind === 'reservation' && denied.reason.includes('worker-lease'),
    `hard lease conflict must win over a co-occurring advisory intent, got ${JSON.stringify(denied)}`,
  );
});

await check('checkWrite: root .spikes/ is governed (not allowlisted) while .bee/spikes/ stays allowed (D2 8ed35504)', async () => {
  const state = defaultState(); // phase: idle
  const rootSpikesDenied = checkWrite(root, state, '.spikes/demo/notes.md');
  assert(
    rootSpikesDenied.allow === false && rootSpikesDenied.kind === 'intake',
    'root .spikes/ must be blocked at idle now that .spikes/ is removed from GATE_ALLOWED_PREFIXES (D2) — spikes live under .bee/spikes/ now',
  );
  const beeSpikesAllowed = checkWrite(root, state, '.bee/spikes/demo/notes.md');
  assert(beeSpikesAllowed.allow === true, '.bee/spikes/ stays allowed via the existing .bee/ prefix');
});

await check('checkRead denies secrets with a privacy marker, and generated dirs', async () => {
  const secret = checkRead('.env.production');
  assert(secret.allow === false && secret.kind === 'privacy', 'privacy deny expected');
  assert(secret.marker.startsWith('@@BEE_PRIVACY@@'), 'marker present');
  const scout = checkRead('packages/app/node_modules/foo/index.js');
  assert(scout.allow === false && scout.kind === 'scout', 'scout deny expected');
  assert(checkRead('src/index.ts').allow === true, 'normal source reads allowed');
});

await check('extractBashTargets flags sed -i and redirection targets', async () => {
  const sed = extractBashTargets('sed -i "s/a/b/" src/config.ts');
  assert(sed.paths.includes('src/config.ts'), `sed target detected, got ${JSON.stringify(sed.paths)}`);
  const redir = extractBashTargets('echo hi > out/log.txt');
  assert(redir.paths.includes('out/log.txt'), 'redirection target detected');
  const broad = extractBashTargets('rm -rf .');
  assert(broad.broadWrite === true, 'rm -rf . is a broad write');
  // fd-duplication is NOT a file write (guards.mjs bug fix, decision 0014)
  const dup = extractBashTargets('node bee_status.mjs --json 2>&1');
  assert(!dup.paths.includes('&1') && dup.paths.length === 0, `2>&1 is not a write target, got ${JSON.stringify(dup.paths)}`);
  const dup2 = extractBashTargets('cmd 1>&2');
  assert(!dup2.paths.some((p) => p.startsWith('&')), 'fd dup &2 not treated as a file');
  const realRedir = extractBashTargets('cmd 2>err.log');
  assert(realRedir.paths.includes('err.log'), 'a real stderr redirect to a file is still caught');
});

// ─── fsh-5: enforcement readers resolve through the session's lane (D2/D4) ──
// LIB CAPABILITY ONLY — hooks thread these in S3/S4. claimCell's execution
// gate comes from the CELL's own feature lane when one exists (the per-feature
// lane is keyed by cell.feature — the cell field named `lane` is the risk
// tier, a different thing); checkWrite optionally resolves phase/gates from a
// bound session via resolvePipeline. Zero lanes on disk = byte-identical to
// today, pinned by every pre-existing claimCell/checkWrite row above passing
// unmodified.

await check("lanes: claimCell resolves the execution gate from the cell's feature lane — an unapproved lane refuses even when the default gate is true, and an approved lane authorizes even when the default gate is false (D2 authority boundary)", async () => {
  const dir = makeStateRepo('bee-lane-claim-gate-');
  try {
    // default pipeline fully approved — it must NOT authorize a lane cell
    writeJsonAtomic(path.join(dir, '.bee', 'state.json'), {
      schema_version: '1.0',
      phase: 'swarming',
      feature: 'default-feat',
      approved_gates: { context: true, shape: true, execution: true, review: false },
      workers: [],
    });
    makeCellFile(dir, 'lg-1', { feature: 'lane-feat', status: 'open' });
    writeLaneFixture(dir, 'lane-feat', { phase: 'validating' }); // all four gates false
    await assertRejects(
      () => claimCell(dir, 'lg-1', 'worker-l'),
      'execution',
      "the lane's unapproved execution gate refuses the claim even though the DEFAULT execution gate is true",
    );
    assert(readCell(dir, 'lg-1').status === 'open', 'refusal leaves the cell open');
    // the lane's own approval authorizes — the default gate is irrelevant to a lane cell
    writeLaneFixture(dir, 'lane-feat', {
      phase: 'swarming',
      approved_gates: { context: true, shape: true, execution: true, review: false },
    });
    writeJsonAtomic(path.join(dir, '.bee', 'state.json'), {
      schema_version: '1.0',
      phase: 'idle',
      feature: null,
      approved_gates: { context: false, shape: false, execution: false, review: false },
      workers: [],
    });
    const claimed = await claimCell(dir, 'lg-1', 'worker-l');
    assert(
      claimed.status === 'claimed' && claimed.trace.worker === 'worker-l',
      "the lane's execution approval authorizes the claim even while the default gate is false",
    );
  } finally {
    fs.rmSync(dir, { recursive: true, force: true });
  }
});

await check("lanes: claimCell for a cell whose feature has NO lane record keeps today's default-gate behavior (D4 zero-lane parity); a corrupt lane record refuses loudly, never falls back to the default gate", async () => {
  const dir = makeStateRepo('bee-lane-claim-default-');
  try {
    writeJsonAtomic(path.join(dir, '.bee', 'state.json'), {
      schema_version: '1.0',
      phase: 'idle',
      feature: null,
      approved_gates: { context: false, shape: false, execution: false, review: false },
      workers: [],
    });
    makeCellFile(dir, 'dg-1', { feature: 'plain-feat', status: 'open' });
    await assertRejects(
      () => claimCell(dir, 'dg-1', 'worker-d'),
      'execution',
      'no lane record → the default gate governs, refusing while unapproved',
    );
    writeJsonAtomic(path.join(dir, '.bee', 'state.json'), {
      schema_version: '1.0',
      phase: 'swarming',
      feature: 'plain-feat',
      approved_gates: { context: true, shape: true, execution: true, review: false },
      workers: [],
    });
    const claimed = await claimCell(dir, 'dg-1', 'worker-d');
    assert(claimed.status === 'claimed', 'default-gate claim proceeds once approved — no lane on disk, no lane logic');
    // a present-but-corrupt lane record must refuse the claim loudly: guessing
    // back to the default gate would let it authorize a lane cell (D2 boundary)
    makeCellFile(dir, 'cg-1', { feature: 'lane-corrupt', status: 'open' });
    fs.mkdirSync(path.join(dir, '.bee', 'lanes'), { recursive: true });
    fs.writeFileSync(laneFile(dir, 'lane-corrupt'), '{ not json', 'utf8');
    await assertRejects(
      () => claimCell(dir, 'cg-1', 'worker-d'),
      'lane',
      'a corrupt lane record refuses the claim loudly instead of falling back to the default gate',
    );
    assert(readCell(dir, 'cg-1').status === 'open', 'refusal leaves the cell untouched');
  } finally {
    fs.rmSync(dir, { recursive: true, force: true });
  }
});

await check("lanes: checkWrite with a bound sessionId resolves phase/gates from the session's lane; absent or unbound sessionId keeps today's record; a broken binding is a typed deny, never a silent default", async () => {
  const dir = makeStateRepo('bee-lane-checkwrite-');
  try {
    // default record at idle: a plain source write hits the intake gate today
    writeJsonAtomic(path.join(dir, '.bee', 'state.json'), {
      schema_version: '1.0',
      phase: 'idle',
      feature: null,
      approved_gates: { context: false, shape: false, execution: false, review: false },
      workers: [],
    });
    const state = readState(dir);
    const bare = checkWrite(dir, state, 'src/app.ts');
    assert(bare.allow === false && bare.kind === 'intake', "absent sessionId keeps today's exact behavior (intake deny at idle)");
    // bound session whose lane is mid-swarm with execution approved → allowed
    laneBinding.createSession(dir, { id: 'sess-w' });
    writeLaneFixture(dir, 'lane-w', {
      phase: 'swarming',
      approved_gates: { context: true, shape: true, execution: true, review: false },
    });
    laneBinding.bindSessionLane(dir, 'sess-w', 'lane-w');
    const boundOk = checkWrite(dir, state, 'src/app.ts', null, { sessionId: 'sess-w' });
    assert(
      boundOk.allow === true,
      `a bound session is governed by its lane (swarming, execution approved) — the idle default record no longer decides, got ${JSON.stringify(boundOk)}`,
    );
    // the lane in a gated phase without approval → gate deny through the lane
    writeLaneFixture(dir, 'lane-w', { phase: 'planning' });
    const boundDenied = checkWrite(dir, state, 'src/app.ts', null, { sessionId: 'sess-w' });
    assert(
      boundDenied.allow === false && boundDenied.kind === 'gate',
      `the bound lane's unapproved gate denies the write, got ${JSON.stringify(boundDenied)}`,
    );
    // an unbound session resolves to the default record — same deny as bare
    laneBinding.createSession(dir, { id: 'sess-u' });
    const unbound = checkWrite(dir, state, 'src/app.ts', null, { sessionId: 'sess-u' });
    assert(unbound.allow === false && unbound.kind === 'intake', 'an unbound session resolves to the default record');
    // a binding to a missing lane: typed deny naming the lane, never a silent default
    laneBinding.bindSessionLane(dir, 'sess-u', 'lane-ghost');
    const broken = checkWrite(dir, state, 'src/app.ts', null, { sessionId: 'sess-u' });
    assert(
      broken.allow === false && broken.kind === 'lane',
      `a broken binding is a typed lane deny, got ${JSON.stringify(broken)}`,
    );
    assert(
      typeof broken.reason === 'string' && broken.reason.includes('lane-ghost'),
      'the deny reason names the unresolvable lane',
    );
  } finally {
    fs.rmSync(dir, { recursive: true, force: true });
  }
});

// ─── fsh-7: cross-session hold hard block in the guard lib (D3, RED-first) ──
// PLACEMENT PIN (panel W1): D3 is unconditional on phase, so every deny test
// here deliberately runs the bound lane in phase 'swarming' with execution
// approved — the primary multi-terminal topology, not a tail-reaching phase
// a tail-placed check would happen to pass. checkWrite itself is otherwise
// untouched for the no-sessionId path (pinned above/elsewhere).

await check("checkWrite: a cross-session hold denies another session's write in swarming-with-execution-approved (phase-independence, C8) — names the holder session, agent, and expiry; the acting session's own hold and an expired hold never block; a legacy session-less reservation never blocks anybody", async () => {
  const dir = makeStateRepo('bee-hold-deny-');
  try {
    laneBinding.createSession(dir, { id: 'sess-hw' });
    laneBinding.createSession(dir, { id: 'sess-other' });
    writeLaneFixture(dir, 'lane-hw', {
      phase: 'swarming',
      approved_gates: { context: true, shape: true, execution: true, review: false },
    });
    laneBinding.bindSessionLane(dir, 'sess-hw', 'lane-hw');
    const state = readState(dir); // irrelevant here: the bound lane governs

    await reserve(dir, { agent: 'other-agent', cell: 'hw-1', path: 'src/hold/target.ts', session: 'sess-other' });
    const denied = checkWrite(dir, state, 'src/hold/target.ts', null, { sessionId: 'sess-hw' });
    assert(
      denied.allow === false && denied.kind === 'hold',
      `a cross-session hold must deny the write even in swarming+execution-approved, got ${JSON.stringify(denied)}`,
    );
    assert(
      denied.reason.includes('sess-other') && denied.reason.includes('other-agent'),
      `deny reason must name the holder session and agent, got: ${denied.reason}`,
    );
    assert(/expires|no expiry/.test(denied.reason), `deny reason must carry an expiry, got: ${denied.reason}`);

    // the acting session's own hold on a different path never blocks itself
    await reserve(dir, { agent: 'me-agent', cell: 'hw-1', path: 'src/hold/mine.ts', session: 'sess-hw' });
    const ownOk = checkWrite(dir, state, 'src/hold/mine.ts', null, { sessionId: 'sess-hw' });
    assert(ownOk.allow === true, `the acting session's own hold must never block its own write, got ${JSON.stringify(ownOk)}`);

    // an expired hold never blocks, even from a different session
    await reserve(dir, { agent: 'other-agent', cell: 'hw-1', path: 'src/hold/stale.ts', session: 'sess-other', ttl: 60 });
    const store = readJson(reservationsPath(dir), { reservations: [] });
    const row = store.reservations.find((r) => r.path === 'src/hold/stale.ts');
    row.reserved_at = new Date(Date.now() - 7200 * 1000).toISOString();
    writeJsonAtomic(reservationsPath(dir), store);
    const staleOk = checkWrite(dir, state, 'src/hold/stale.ts', null, { sessionId: 'sess-hw' });
    assert(staleOk.allow === true, `an expired hold must never block, got ${JSON.stringify(staleOk)}`);

    // a legacy session-less reservation (today's exact shape) never blocks a bound session either.
    // D3: clear env for this one reserve() call so "no --session passed" stays
    // genuinely session-less, matching a legacy row made before fsh-7/D3 existed.
    const savedLegacyEnv = process.env.CLAUDE_CODE_SESSION_ID;
    try {
      delete process.env.CLAUDE_CODE_SESSION_ID;
      await reserve(dir, { agent: 'legacy-agent', cell: 'hw-1', path: 'src/hold/legacy.ts' });
    } finally {
      if (savedLegacyEnv === undefined) delete process.env.CLAUDE_CODE_SESSION_ID;
      else process.env.CLAUDE_CODE_SESSION_ID = savedLegacyEnv;
    }
    const legacyOk = checkWrite(dir, state, 'src/hold/legacy.ts', null, { sessionId: 'sess-hw' });
    assert(legacyOk.allow === true, `a session-less reservation row must never block a bound session's write, got ${JSON.stringify(legacyOk)}`);
  } finally {
    fs.rmSync(dir, { recursive: true, force: true });
  }
});

await check("checkWrite: with NO sessionId, a session-owned hold on the target path is never even consulted — byte-identical to today's exact reservation-guard behavior (own agent name still governs the swarming branch as before)", async () => {
  const dir = makeStateRepo('bee-hold-no-session-');
  try {
    const state = { ...defaultState(), phase: 'swarming', approved_gates: { ...defaultState().approved_gates, execution: true } };
    await reserve(dir, { agent: 'other-agent', cell: 'hw-2', path: 'src/hold/no-session.ts', session: 'sess-somebody' });
    const noSessionArg = checkWrite(dir, state, 'src/hold/no-session.ts');
    assert(
      noSessionArg.allow === true,
      `no sessionId means the hold check never runs — the write-guard behaves exactly as it did before fsh-7, got ${JSON.stringify(noSessionArg)}`,
    );
  } finally {
    fs.rmSync(dir, { recursive: true, force: true });
  }
});

await check('checkWrite: a present-but-corrupt reservation store RETURNS a typed {allow:false, kind:"holds-unreadable"} verdict for a session-aware write — never a throw (C7, panel B1); a missing store stays open exactly as today', async () => {
  const dir = makeStateRepo('bee-hold-corrupt-');
  try {
    laneBinding.createSession(dir, { id: 'sess-corrupt' });
    writeLaneFixture(dir, 'lane-corrupt-hw', {
      phase: 'swarming',
      approved_gates: { context: true, shape: true, execution: true, review: false },
    });
    laneBinding.bindSessionLane(dir, 'sess-corrupt', 'lane-corrupt-hw');
    const state = readState(dir);

    // missing store (nothing has reserved anything yet) stays open
    const openOk = checkWrite(dir, state, 'src/hold/whatever.ts', null, { sessionId: 'sess-corrupt' });
    assert(openOk.allow === true, `a missing reservation store must stay open, got ${JSON.stringify(openOk)}`);

    // a present-but-corrupt store must fail closed, never throw
    fs.mkdirSync(path.join(dir, '.bee'), { recursive: true });
    fs.writeFileSync(reservationsPath(dir), '{ not json', 'utf8');
    let corrupt;
    let threw = false;
    try {
      corrupt = checkWrite(dir, state, 'src/hold/whatever.ts', null, { sessionId: 'sess-corrupt' });
    } catch {
      threw = true;
    }
    assert(!threw, 'checkWrite must never throw on a corrupt reservation store — the hook is fail-open and would swallow a throw into an allow');
    assert(
      corrupt && corrupt.allow === false && corrupt.kind === 'holds-unreadable',
      `a corrupt store must be a typed {allow:false, kind:'holds-unreadable'} deny, got ${JSON.stringify(corrupt)}`,
    );

    // restoring a valid (even empty) store re-opens the write
    writeJsonAtomic(reservationsPath(dir), { reservations: [] });
    const restored = checkWrite(dir, state, 'src/hold/whatever.ts', null, { sessionId: 'sess-corrupt' });
    assert(restored.allow === true, `a valid, empty store must re-open the write, got ${JSON.stringify(restored)}`);
  } finally {
    fs.rmSync(dir, { recursive: true, force: true });
  }
});

// ─── fsh-6: presentation readers show the session's lane (D4) ───────────────
// buildSessionPreamble/buildPromptReminder gain an OPTIONAL sessionId param.
// Omitted (today's exact call shape) resolves to the default pipeline —
// byte-identical to every pinned no-sessionId row above. A bound sessionId
// shows THAT lane's phase/mode/feature/gates plus a one-line summary of any
// OTHER active (non-terminal) lanes. bee.mjs's buildStatus carries a new
// `lanes` block (per-lane phase/gates/bound sessions) alongside every
// pre-existing zero-lane field, unchanged. bee-chain-nudge/bee-session-close
// consult the acting session's pipeline for phase when payload.session_id
// names a bound session, default otherwise — covered in
// hooks/test_hook_contracts.mjs.

await check('buildSessionPreamble: omitting sessionId (or passing {}) renders byte-identical to today; an unbound session also resolves to the exact default preamble', async () => {
  const dir = makeStateRepo('bee-preamble-lane-bare-');
  try {
    writeState(dir, { ...defaultState(), phase: 'idle', mode: null, feature: null });
    const noArg = buildSessionPreamble(dir);
    const emptyOpts = buildSessionPreamble(dir, {});
    const nullSession = buildSessionPreamble(dir, { sessionId: null });
    assert(noArg === emptyOpts && emptyOpts === nullSession, 'omitted/{}/null sessionId all render the identical preamble');

    laneBinding.createSession(dir, { id: 'sess-bare' });
    const unbound = buildSessionPreamble(dir, { sessionId: 'sess-bare' });
    assert(unbound === noArg, 'an unbound session renders exactly the default preamble (D4 zero-lane parity)');
  } finally {
    fs.rmSync(dir, { recursive: true, force: true });
  }
});

await check("buildSessionPreamble: a bound sessionId shows that lane's own phase/mode/feature/gates and names other ACTIVE lanes in one line — never the bound lane itself, never a terminal one", async () => {
  const dir = makeStateRepo('bee-preamble-lane-bound-');
  try {
    laneBinding.createSession(dir, { id: 'sess-p' });
    writeLaneFixture(dir, 'lane-p', {
      phase: 'planning',
      mode: 'standard',
      approved_gates: { context: true, shape: false, execution: false, review: false },
    });
    laneBinding.bindSessionLane(dir, 'sess-p', 'lane-p');

    const soloBound = buildSessionPreamble(dir, { sessionId: 'sess-p' });
    assert(
      /Phase: planning \| Mode: standard \| Feature: lane-p/.test(soloBound),
      `preamble shows the bound lane's own phase/mode/feature, got:\n${soloBound}`,
    );
    assert(/context: approved/.test(soloBound) && /shape: pending/.test(soloBound), 'gates line reflects the bound lane, not the default record');
    assert(!/other active lane/.test(soloBound), 'no lanes-summary line when no OTHER lane exists');

    writeLaneFixture(dir, 'lane-other', { phase: 'swarming', mode: 'standard' });
    writeLaneFixture(dir, 'lane-closed', { phase: 'compounding-complete', mode: 'standard' });
    const withOthers = buildSessionPreamble(dir, { sessionId: 'sess-p' });
    assert(
      /1 other active lane\(s\): lane-other/.test(withOthers),
      `preamble names exactly the one OTHER active lane, got:\n${withOthers}`,
    );
    assert(!/lane-closed/.test(withOthers), 'a terminal (compounding-complete) lane is never counted as active');
    assert(!/lane-p,|, lane-p/.test(withOthers.match(/other active lane\(s\): (.*)$/m)?.[1] ?? ''), 'the bound lane never lists itself in the summary');
  } finally {
    fs.rmSync(dir, { recursive: true, force: true });
  }
});

await check("buildSessionPreamble: an unresolvable binding (missing lane) falls back to the default record instead of blocking the informational preamble", async () => {
  const dir = makeStateRepo('bee-preamble-lane-broken-');
  try {
    writeState(dir, { ...defaultState(), phase: 'idle' });
    laneBinding.createSession(dir, { id: 'sess-ghost' });
    laneBinding.bindSessionLane(dir, 'sess-ghost', 'lane-ghost');
    const bare = buildSessionPreamble(dir);
    const broken = buildSessionPreamble(dir, { sessionId: 'sess-ghost' });
    assert(broken === bare, 'a broken binding renders the same preamble as the default (never throws, never blocks)');
  } finally {
    fs.rmSync(dir, { recursive: true, force: true });
  }
});

await check('buildPromptReminder: omitting sessionId is unchanged; a bound sessionId reflects that lane\'s phase/next_action/gate, an unresolvable binding falls back to the default', async () => {
  const dir = makeStateRepo('bee-reminder-lane-');
  try {
    writeState(dir, { ...defaultState(), phase: 'idle', next_action: 'Invoke bee-hive.' });
    const bare = buildPromptReminder(dir);
    assert(bare.text.includes('phase=idle'), 'omitted sessionId keeps the default pipeline');

    laneBinding.createSession(dir, { id: 'sess-r' });
    writeLaneFixture(dir, 'lane-r', {
      phase: 'planning',
      mode: 'standard',
      next_action: 'Prepare the current slice.',
      approved_gates: { context: true, shape: false, execution: false, review: false },
    });
    laneBinding.bindSessionLane(dir, 'sess-r', 'lane-r');
    const bound = buildPromptReminder(dir, { sessionId: 'sess-r' });
    assert(bound.text.includes('phase=planning'), `bound reminder reflects the lane's phase, got: ${bound.text}`);
    assert(bound.text.includes('mode=standard'), `bound reminder reflects the lane's mode, got: ${bound.text}`);
    assert(/next: Prepare the current slice\./.test(bound.text), `bound reminder reflects the lane's next_action, got: ${bound.text}`);
    assert(/gate pending: shape/.test(bound.text), `bound reminder's first open gate comes from the lane, got: ${bound.text}`);
    assert(bound.hash !== bare.hash, 'a different resolved pipeline hashes differently');

    laneBinding.createSession(dir, { id: 'sess-r2' });
    laneBinding.bindSessionLane(dir, 'sess-r2', 'lane-missing');
    const broken = buildPromptReminder(dir, { sessionId: 'sess-r2' });
    assert(broken.text === bare.text, 'an unresolvable binding falls back to the default pipeline, never throws');
  } finally {
    fs.rmSync(dir, { recursive: true, force: true });
  }
});

// ─── xwh-4: frozen regression net over the checkWrite decision table ────────
// Critical pattern 20260716: freeze a load-bearing function's CURRENT behavior
// in a regression net and see it GREEN before touching it. Every branch of
// checkWrite gets rows pinning today's exact allow/deny + reason-shape
// behavior. The net is TOLERANT of new fields (pins the fields that exist,
// never asserts the absence of others) so a purely additive change stays
// compatible. Any pre-existing row here that changes after an edit is a
// defect in the edit, not a row to update.

await check('NET branch 1 — direct-edit deny: .bee/state.json and .bee/backlog.jsonl are denied first-hit in EVERY phase, before GATE_ALLOWED_PREFIXES can allow .bee/', async () => {
  const dir = makeStateRepo('bee-net-direct-edit-');
  try {
    const phases = [
      { ...defaultState(), phase: 'idle' },
      { ...defaultState(), phase: 'planning' },
      { ...defaultState(), phase: 'swarming', approved_gates: { context: true, shape: true, execution: true, review: false } },
      { ...defaultState(), phase: 'compounding-complete', approved_gates: { context: true, shape: true, execution: true, review: true } },
    ];
    for (const state of phases) {
      const stateDeny = checkWrite(dir, state, '.bee/state.json');
      assert(
        stateDeny.allow === false && stateDeny.kind === 'direct-edit',
        `.bee/state.json must be a direct-edit deny in phase ${state.phase}, got ${JSON.stringify(stateDeny)}`,
      );
      assert(
        stateDeny.reason.includes('CLI-owned') && stateDeny.reason.includes('bee.mjs state'),
        `direct-edit reason names CLI ownership and the state verb, got: ${stateDeny.reason}`,
      );
      const backlogDeny = checkWrite(dir, state, '.bee/backlog.jsonl');
      assert(
        backlogDeny.allow === false && backlogDeny.kind === 'direct-edit' && backlogDeny.reason.includes('bee.mjs backlog add'),
        `.bee/backlog.jsonl must be a direct-edit deny naming bee.mjs backlog add in phase ${state.phase}, got ${JSON.stringify(backlogDeny)}`,
      );
    }
    // path normalization: ./ prefix and backslashes still hit the deny
    assert(checkWrite(dir, defaultState(), './.bee/state.json').allow === false, './-prefixed state.json still denied');
    assert(checkWrite(dir, defaultState(), '.bee\\state.json').allow === false, 'backslash state.json still denied');
    // other .bee/ files are NOT this rule's concern (idle allows .bee/ prefix)
    const otherBee = checkWrite(dir, defaultState(), '.bee/cells/x-1.json');
    assert(otherBee.allow === true, `.bee/cells/ stays allowed at idle, got ${JSON.stringify(otherBee)}`);
  } finally {
    fs.rmSync(dir, { recursive: true, force: true });
  }
});

await check('NET branch 2 — docs/history code-ext deny: code extensions deny with kind docs-history-code; .md/.json/extension-less allowed; precedence below direct-edit, above lane/hold/phase', async () => {
  const dir = makeStateRepo('bee-net-history-code-');
  try {
    const active = { ...defaultState(), phase: 'swarming', approved_gates: { context: true, shape: true, execution: true, review: false } };
    for (const p of ['docs/history/f/verify.sh', 'docs/history/f/helper.mjs', 'docs/history/f/tool.py', 'docs/history/f/x.ts']) {
      const deny = checkWrite(dir, active, p);
      assert(deny.allow === false && deny.kind === 'docs-history-code', `${p} must deny with docs-history-code, got ${JSON.stringify(deny)}`);
      assert(deny.reason.includes('docs/history/') && /spikes|scripts/.test(deny.reason), `reason points at spikes/scripts, got: ${deny.reason}`);
    }
    assert(checkWrite(dir, active, 'docs/history/f/CONTEXT.md').allow === true, '.md under docs/history/ allowed');
    assert(checkWrite(dir, active, 'docs/history/f/evidence.json').allow === true, '.json under docs/history/ allowed');
    assert(checkWrite(dir, active, 'docs/history/f/Makefile').allow === true, 'extension-less file under docs/history/ allowed');
    assert(checkWrite(dir, active, 'src/tool.py').allow === true, 'code outside docs/history/ untouched by this rule');
  } finally {
    fs.rmSync(dir, { recursive: true, force: true });
  }
});

await check('NET branch 3 — lane resolution: broken binding is a typed lane deny naming the lane; a bound lane governs phase/gates; unbound session falls to the default record', async () => {
  const dir = makeStateRepo('bee-net-lane-');
  try {
    writeJsonAtomic(path.join(dir, '.bee', 'state.json'), {
      schema_version: '1.0',
      phase: 'idle',
      feature: null,
      approved_gates: { context: false, shape: false, execution: false, review: false },
      workers: [],
    });
    const state = readState(dir);
    laneBinding.createSession(dir, { id: 'net-broken' });
    laneBinding.bindSessionLane(dir, 'net-broken', 'net-lane-ghost');
    const broken = checkWrite(dir, state, 'src/app.ts', null, { sessionId: 'net-broken' });
    assert(broken.allow === false && broken.kind === 'lane', `broken binding is a typed lane deny, got ${JSON.stringify(broken)}`);
    assert(broken.reason.startsWith('bee lane guard:') && broken.reason.includes('net-lane-ghost'), `lane reason shape pinned, got: ${broken.reason}`);
    laneBinding.createSession(dir, { id: 'net-bound' });
    writeLaneFixture(dir, 'net-lane-live', {
      phase: 'swarming',
      approved_gates: { context: true, shape: true, execution: true, review: false },
    });
    laneBinding.bindSessionLane(dir, 'net-bound', 'net-lane-live');
    const bound = checkWrite(dir, state, 'src/app.ts', null, { sessionId: 'net-bound' });
    assert(bound.allow === true, `a bound approved lane allows over the idle default record, got ${JSON.stringify(bound)}`);
    laneBinding.createSession(dir, { id: 'net-unbound' });
    const unbound = checkWrite(dir, state, 'src/app.ts', null, { sessionId: 'net-unbound' });
    assert(unbound.allow === false && unbound.kind === 'intake', 'unbound session resolves to the default record (intake at idle)');
  } finally {
    fs.rmSync(dir, { recursive: true, force: true });
  }
});

await check('NET branch 4 — cross-session hold: deny shape (session+agent+cell+expiry named), own hold open, corrupt store holds-unreadable, missing store open', async () => {
  const dir = makeStateRepo('bee-net-hold-');
  try {
    laneBinding.createSession(dir, { id: 'net-sess-a' });
    laneBinding.createSession(dir, { id: 'net-sess-b' });
    writeLaneFixture(dir, 'net-lane-hold', {
      phase: 'swarming',
      approved_gates: { context: true, shape: true, execution: true, review: false },
    });
    laneBinding.bindSessionLane(dir, 'net-sess-a', 'net-lane-hold');
    const state = readState(dir);
    // missing store stays open
    const openOk = checkWrite(dir, state, 'src/h/free.ts', null, { sessionId: 'net-sess-a' });
    assert(openOk.allow === true, `missing reservation store stays open, got ${JSON.stringify(openOk)}`);
    await reserve(dir, { agent: 'net-agent-b', cell: 'net-1', path: 'src/h/target.ts', session: 'net-sess-b' });
    const deny = checkWrite(dir, state, 'src/h/target.ts', null, { sessionId: 'net-sess-a' });
    assert(deny.allow === false && deny.kind === 'hold', `cross-session hold deny expected, got ${JSON.stringify(deny)}`);
    assert(
      deny.reason.startsWith('bee cross-session hold:') &&
        deny.reason.includes('net-sess-b') &&
        deny.reason.includes('net-agent-b') &&
        deny.reason.includes('net-1') &&
        /expires|no expiry/.test(deny.reason),
      `hold deny reason shape pinned (session, agent, cell, expiry), got: ${deny.reason}`,
    );
    // the acting session's own hold never denies itself
    await reserve(dir, { agent: 'net-agent-a', cell: 'net-1', path: 'src/h/mine.ts', session: 'net-sess-a' });
    assert(checkWrite(dir, state, 'src/h/mine.ts', null, { sessionId: 'net-sess-a' }).allow === true, 'own hold never blocks');
    // corrupt store: typed deny, never a throw
    fs.writeFileSync(reservationsPath(dir), '{ torn', 'utf8');
    let verdict;
    let threw = false;
    try {
      verdict = checkWrite(dir, state, 'src/h/free.ts', null, { sessionId: 'net-sess-a' });
    } catch {
      threw = true;
    }
    assert(!threw && verdict && verdict.allow === false && verdict.kind === 'holds-unreadable', `corrupt store is a typed holds-unreadable deny, got ${JSON.stringify(verdict)}`);
    assert(verdict.reason.includes('reservation store'), `holds-unreadable reason names the reservation store, got: ${verdict.reason}`);
    // no sessionId: the hold machinery never runs at all
    const swarm = { ...defaultState(), phase: 'swarming', approved_gates: { context: true, shape: true, execution: true, review: false } };
    const saved = process.env.BEE_AGENT_NAME;
    try {
      delete process.env.BEE_AGENT_NAME;
      assert(checkWrite(dir, swarm, 'src/h/target.ts').allow === true, 'no sessionId: session-hold check never consulted');
    } finally {
      if (saved === undefined) delete process.env.BEE_AGENT_NAME;
      else process.env.BEE_AGENT_NAME = saved;
    }
  } finally {
    fs.rmSync(dir, { recursive: true, force: true });
  }
});

await check('NET branch 5 — terminal-phase intake: idle and compounding-complete deny with kind intake naming the phase; allowed prefixes writable; guards.idle_gate=false disables', async () => {
  const dir = makeStateRepo('bee-net-intake-');
  try {
    for (const phase of ['idle', 'compounding-complete']) {
      const state = {
        ...defaultState(),
        phase,
        approved_gates: phase === 'compounding-complete' ? { context: true, shape: true, execution: true, review: true } : defaultState().approved_gates,
      };
      const deny = checkWrite(dir, state, 'src/app.ts');
      assert(deny.allow === false && deny.kind === 'intake', `intake deny at ${phase}, got ${JSON.stringify(deny)}`);
      assert(deny.reason.startsWith('bee intake gate:') && deny.reason.includes(phase) && deny.reason.includes('bee-hive'), `intake reason names the phase and bee-hive routing, got: ${deny.reason}`);
      assert(checkWrite(dir, state, 'docs/notes.md').allow === true, `docs/ writable at ${phase}`);
      assert(checkWrite(dir, state, '.bee/cells/n-1.json').allow === true, `.bee/ writable at ${phase}`);
      assert(checkWrite(dir, state, 'plans/next.md').allow === true, `plans/ writable at ${phase}`);
      assert(checkWrite(dir, state, 'AGENTS.md').allow === true, `AGENTS.md writable at ${phase}`);
    }
    const configPath = path.join(dir, '.bee', 'config.json');
    writeJsonAtomic(configPath, { guards: { idle_gate: false } });
    assert(checkWrite(dir, defaultState(), 'src/app.ts').allow === true, 'guards.idle_gate=false disables the intake gate');
    fs.rmSync(configPath, { force: true });
  } finally {
    fs.rmSync(dir, { recursive: true, force: true });
  }
});

await check('NET branch 6 — gated phases: exploring/planning/validating deny outside allowed prefixes with kind gate; execution approval opens them', async () => {
  const dir = makeStateRepo('bee-net-gate-');
  try {
    for (const phase of ['exploring', 'planning', 'validating']) {
      const state = { ...defaultState(), phase };
      const deny = checkWrite(dir, state, 'src/app.ts');
      assert(deny.allow === false && deny.kind === 'gate', `gate deny at ${phase}, got ${JSON.stringify(deny)}`);
      assert(deny.reason.startsWith('bee gate:') && deny.reason.includes(phase) && deny.reason.includes('execution'), `gate reason names phase and gate, got: ${deny.reason}`);
      assert(checkWrite(dir, state, 'docs/history/f/plan.md').allow === true, `docs/history/ writable at ${phase}`);
      const approved = { ...state, approved_gates: { context: true, shape: true, execution: true, review: false } };
      assert(checkWrite(dir, approved, 'src/app.ts').allow === true, `execution approval opens source at ${phase}`);
    }
  } finally {
    fs.rmSync(dir, { recursive: true, force: true });
  }
});

await check('NET branch 7 — swarming reservation: foreign reservation denies with kind reservation naming the holder; own agent and unreserved paths allowed; no agent identity means no check; unknown phase falls through open', async () => {
  const dir = makeStateRepo('bee-net-swarm-');
  try {
    const state = { ...defaultState(), phase: 'swarming', approved_gates: { context: true, shape: true, execution: true, review: false } };
    await reserve(dir, { agent: 'net-holder', cell: 'net-s1', path: 'src/s/engine.ts' });
    const deny = checkWrite(dir, state, 'src/s/engine.ts', 'net-writer');
    assert(deny.allow === false && deny.kind === 'reservation', `reservation deny expected, got ${JSON.stringify(deny)}`);
    assert(deny.reason.startsWith('bee reservation conflict:') && deny.reason.includes('net-holder') && deny.reason.includes('net-s1') && deny.reason.includes('[BLOCKED]'), `reservation reason shape pinned, got: ${deny.reason}`);
    assert(checkWrite(dir, state, 'src/s/engine.ts', 'net-holder').allow === true, 'holder writes its own reserved path');
    assert(checkWrite(dir, state, 'src/s/other.ts', 'net-writer').allow === true, 'unreserved path allowed in swarming');
    const saved = process.env.BEE_AGENT_NAME;
    try {
      delete process.env.BEE_AGENT_NAME;
      assert(checkWrite(dir, state, 'src/s/engine.ts').allow === true, 'no agent identity: reservation check never runs');
      process.env.BEE_AGENT_NAME = 'net-writer';
      const envDeny = checkWrite(dir, state, 'src/s/engine.ts');
      assert(envDeny.allow === false && envDeny.kind === 'reservation', 'BEE_AGENT_NAME env supplies the agent identity');
    } finally {
      if (saved === undefined) delete process.env.BEE_AGENT_NAME;
      else process.env.BEE_AGENT_NAME = saved;
    }
    // non-terminal, non-gated, non-swarming phase falls through open
    const executing = { ...defaultState(), phase: 'executing', approved_gates: { context: true, shape: true, execution: true, review: false } };
    assert(checkWrite(dir, executing, 'src/app.ts', 'net-writer').allow === true, 'executing phase falls through to allow');
  } finally {
    fs.rmSync(dir, { recursive: true, force: true });
  }
});

// ─── xwh-4: cross-worktree foreign-hold branch (NEW behavior, RED-first) ────
// The write guard consults the shared cross-worktree holds ledger (xwh-1,
// worktree-holds.mjs) through the same topology resolution claim-next uses
// (xwh-3): ordinary checkout => holder 'main', ledger at the checkout's own
// root; granted linked worktree => holder = git-verified id, ledger at
// mainRoot; everything else (ungranted, unresolvable) => no consultation at
// all, fail-open. Runs after the cross-session hold branch, before every
// phase branch — a foreign checkout's hold denies even in
// swarming-with-execution-approved.

function writeHoldsLedger(dir, holds) {
  const runtime = path.join(dir, '.bee', 'runtime');
  fs.mkdirSync(runtime, { recursive: true });
  writeJsonAtomic(path.join(runtime, 'cross-worktree-holds.json'), { holds });
}

await check('checkWrite (xwh-4): a foreign checkout\'s ledger hold denies the write with a typed kind, naming the holding checkout, its feature, and the expiry — phase-independent (swarming with execution approved)', async () => {
  const dir = makeStateRepo('bee-xwh-foreign-deny-');
  try {
    const state = { ...defaultState(), phase: 'swarming', approved_gates: { context: true, shape: true, execution: true, review: false } };
    writeHoldsLedger(dir, [
      {
        path: 'src/held/feature.ts',
        holder: 'wt-featx',
        feature: 'feat-x',
        session: null,
        cell: 'fx-1',
        ttl_seconds: 3600,
        mirrored_at: new Date().toISOString(),
        released_at: null,
      },
    ]);
    const deny = checkWrite(dir, state, 'src/held/feature.ts', 'net-writer');
    assert(
      deny.allow === false && deny.kind === 'worktree-hold',
      `a foreign ledger hold must deny with kind worktree-hold, got ${JSON.stringify(deny)}`,
    );
    assert(
      deny.reason.includes('wt-featx') && deny.reason.includes('feat-x') && /expires|no expiry/.test(deny.reason),
      `the deny reason must name the holding checkout, its feature, and the expiry, got: ${deny.reason}`,
    );
  } finally {
    fs.rmSync(dir, { recursive: true, force: true });
  }
});

await check("checkWrite (xwh-4): the acting checkout's OWN ledger holds never deny (ordinary checkout acts as holder 'main'); a missing ledger stays open; expired and released foreign holds never block", async () => {
  const dir = makeStateRepo('bee-xwh-own-open-');
  try {
    const state = { ...defaultState(), phase: 'swarming', approved_gates: { context: true, shape: true, execution: true, review: false } };
    // missing ledger: byte-identical to today
    assert(checkWrite(dir, state, 'src/free.ts', 'net-writer').allow === true, 'missing ledger stays open');
    writeHoldsLedger(dir, [
      // the acting checkout's own hold (an ordinary checkout mirrors as 'main')
      { path: 'src/own.ts', holder: 'main', feature: 'feat-here', session: null, cell: null, ttl_seconds: 3600, mirrored_at: new Date().toISOString(), released_at: null },
      // an EXPIRED foreign hold
      { path: 'src/stale.ts', holder: 'wt-old', feature: 'feat-old', session: null, cell: null, ttl_seconds: 60, mirrored_at: new Date(Date.now() - 7200 * 1000).toISOString(), released_at: null },
      // a RELEASED foreign hold
      { path: 'src/done.ts', holder: 'wt-done', feature: 'feat-done', session: null, cell: null, ttl_seconds: 3600, mirrored_at: new Date().toISOString(), released_at: new Date().toISOString() },
    ]);
    assert(checkWrite(dir, state, 'src/own.ts', 'net-writer').allow === true, "the acting checkout's own hold never denies itself");
    assert(checkWrite(dir, state, 'src/stale.ts', 'net-writer').allow === true, 'an expired foreign hold never blocks');
    assert(checkWrite(dir, state, 'src/done.ts', 'net-writer').allow === true, 'a released foreign hold never blocks');
  } finally {
    fs.rmSync(dir, { recursive: true, force: true });
  }
});

await check('checkWrite (xwh-4): a present-but-corrupt holds ledger is a typed deny (holdsStoreCorrupt semantics: missing=open, unparseable=deny) — never a throw; restoring a valid ledger re-opens', async () => {
  const dir = makeStateRepo('bee-xwh-corrupt-');
  try {
    const state = { ...defaultState(), phase: 'swarming', approved_gates: { context: true, shape: true, execution: true, review: false } };
    const runtime = path.join(dir, '.bee', 'runtime');
    fs.mkdirSync(runtime, { recursive: true });
    fs.writeFileSync(path.join(runtime, 'cross-worktree-holds.json'), '{ torn ledger', 'utf8');
    let verdict;
    let threw = false;
    try {
      verdict = checkWrite(dir, state, 'src/whatever.ts', 'net-writer');
    } catch {
      threw = true;
    }
    assert(!threw, 'checkWrite must never throw on a corrupt holds ledger — the hook is fail-open and would swallow a throw into an allow');
    assert(
      verdict && verdict.allow === false && verdict.kind === 'worktree-holds-unreadable',
      `a corrupt holds ledger must be a typed {allow:false, kind:'worktree-holds-unreadable'} deny, got ${JSON.stringify(verdict)}`,
    );
    writeHoldsLedger(dir, []);
    assert(checkWrite(dir, state, 'src/whatever.ts', 'net-writer').allow === true, 'a valid (empty) ledger re-opens the write');
  } finally {
    fs.rmSync(dir, { recursive: true, force: true });
  }
});

await check('checkWrite (xwh-4): unresolvable topology fails OPEN — a checkout resolveRoots cannot place never consults the ledger, even a corrupt one', async () => {
  const dir = fs.mkdtempSync(path.join(os.tmpdir(), 'bee-xwh-unresolvable-'));
  try {
    fs.mkdirSync(path.join(dir, '.bee'), { recursive: true });
    writeJsonAtomic(path.join(dir, '.bee', 'onboarding.json'), { schema_version: '1.0', bee_version: '0.1.0' });
    // a .git FILE with empty content = an invalid linked-worktree marker:
    // resolveRoots throws WorktreeLinkInvalidError for it
    fs.writeFileSync(path.join(dir, '.git'), '', 'utf8');
    // even a corrupt ledger sitting right there must not deny — the topology
    // never resolved, so the consultation never runs (fail-open discipline)
    const runtime = path.join(dir, '.bee', 'runtime');
    fs.mkdirSync(runtime, { recursive: true });
    fs.writeFileSync(path.join(runtime, 'cross-worktree-holds.json'), '{ torn', 'utf8');
    const state = { ...defaultState(), phase: 'swarming', approved_gates: { context: true, shape: true, execution: true, review: false } };
    const verdict = checkWrite(dir, state, 'src/app.ts', 'net-writer');
    assert(verdict.allow === true, `an unresolvable topology must fail open, got ${JSON.stringify(verdict)}`);
  } finally {
    fs.rmSync(dir, { recursive: true, force: true });
  }
});

await check('checkWrite (xwh-4): DIRECT_EDIT_DENY covers .bee/runtime/cross-worktree-holds.json and .bee/runtime/worktree-grants.json — hand edits refused in every phase, CLI named in the fix', async () => {
  const dir = makeStateRepo('bee-xwh-direct-edit-');
  try {
    const phases = [
      defaultState(), // idle
      { ...defaultState(), phase: 'swarming', approved_gates: { context: true, shape: true, execution: true, review: false } },
    ];
    for (const state of phases) {
      for (const file of ['.bee/runtime/cross-worktree-holds.json', '.bee/runtime/worktree-grants.json']) {
        const deny = checkWrite(dir, state, file);
        assert(
          deny.allow === false && deny.kind === 'direct-edit',
          `${file} must be a direct-edit deny in phase ${state.phase}, got ${JSON.stringify(deny)}`,
        );
        assert(deny.reason.includes('CLI-owned'), `direct-edit reason keeps the CLI-owned voice, got: ${deny.reason}`);
      }
    }
    // other .bee/runtime/ files are not this rule's concern
    assert(checkWrite(dir, defaultState(), '.bee/runtime/something-else.json').allow === true, 'other .bee/runtime files unaffected');
  } finally {
    fs.rmSync(dir, { recursive: true, force: true });
  }
});

printSummaryAndExit();
