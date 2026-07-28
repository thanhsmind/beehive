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
} from '../../../scripts/lib/test-fixture.mjs';
import { defaultState, readState, writeState } from '../lib/state.mjs';
import { readCell, claimCell } from '../lib/cells.mjs';
import { reserve, reservationsPath } from '../lib/reservations.mjs';
// multisession-native-16: `.bee/reservations.json` is a rebuildable
// projection over lease-store.mjs now, not the live store — a test that
// needs to force a reservation into the past backdates the underlying LEASE
// directly (renewLease), same fix applied throughout this cell's test
// changes. The corrupt-store tests further down in this file are UNAFFECTED
// by this shim and stay exactly as they were: guards.mjs's own
// reservationStoreCorrupt still reads this same literal reservationsPath
// file directly (out of this cell's scope — see reservations.mjs's own
// module header) and is never consulted by reserve()/checkWrite's live
// conflict path, so writing torn JSON straight to that path still exercises
// guards.mjs's fail-closed check exactly as before.
import { renewLease } from '../lib/lease-store.mjs';
import { createSession } from '../lib/claims.mjs';
// multisession-native-21: workspace registry fixtures (deny class (c)) and
// the lock-file primitive (the "hook never waits on a store lock"
// regression proves checkWrite never even reaches this file).
import { registerWorkspace, claimWriteOwnership, readWorkspace } from '../lib/workspace-store.mjs';
import { lockFilePath } from '../lib/lock.mjs';
// fsh-3 (lane store): namespace imports so a not-yet-implemented export fails
// its own row ("… is not a function") instead of crashing the whole module
// graph at import time — the RED-first evidence stays per-row.
import * as laneStore from '../lib/state.mjs';
import * as laneBinding from '../lib/claims.mjs';
import { checkWrite, checkRead, extractBashTargets, checkAskUserQuestion, checkGitBashCommand } from '../lib/guards.mjs';
import { buildPromptReminder, buildSessionPreamble } from '../lib/inject.mjs';
import { readJson, writeJsonAtomic } from '../lib/fsutil.mjs';

// Hermeticity (hardening-1-7-10 D1 + okf-integration-close-f4 f4-4, defense
// in depth): this suite must never inherit the harness's own identity.
// run_verify.mjs already scrubs all three vars for every child suite it
// spawns; deleting BEE_AGENT_NAME here at BOOTSTRAP means a bare
// `node skills/.../test_guards.mjs`, run directly under the very
// `BEE_AGENT_NAME=<name>` prefix AGENTS.md critical rule 4 mandates for
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

await check('checkWrite terminal-phase idle gate reads the resolved controlRoot\'s config, not root\'s own (GH #83)', async () => {
  // Companion-mounted path: root and controlRoot name DIFFERENT
  // .bee/config.json files. sessionId MUST stay null so phase comes from the
  // passed state and the call actually reaches the terminal-phase branch
  // (a sessionId routes through resolvePipeline and can short-circuit
  // earlier).
  const rootA = makeStateRepo('bee-idle-gate-root-');
  const rootB = makeStateRepo('bee-idle-gate-control-');
  const state = { ...defaultState(), phase: 'compounding-complete' };

  // Direction 1: controlRoot (rootB) disables idle_gate; root's (rootA) own
  // config leaves it default-enabled. The decision must follow controlRoot.
  writeJsonAtomic(path.join(rootB, '.bee', 'config.json'), { guards: { idle_gate: false } });
  const allowed = checkWrite(rootA, state, 'src/x.js', null, { controlRoot: rootB });
  assert(
    allowed.allow === true,
    "idle gate must follow controlRoot's config (disabled) even though root's own config leaves it enabled"
  );

  // Direction 2: controlRoot (rootB) is back to default-enabled; root's
  // (rootA) own config disables it. The decision must still follow
  // controlRoot — deny.
  fs.rmSync(path.join(rootB, '.bee', 'config.json'), { force: true });
  writeJsonAtomic(path.join(rootA, '.bee', 'config.json'), { guards: { idle_gate: false } });
  const denied = checkWrite(rootA, state, 'src/x.js', null, { controlRoot: rootB });
  assert(
    denied.allow === false && denied.kind === 'intake',
    "idle gate must follow controlRoot's config (enabled) even though root's own config disables it"
  );
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

await check('extractBashTargets: blanket staging flags count as broad writes (bsg-1)', async () => {
  const addA = extractBashTargets('git add -A');
  assert(addA.broadWrite === true, 'git add -A is a broad write');
  const addAll = extractBashTargets('git add --all');
  assert(addAll.broadWrite === true, 'git add --all is a broad write');
  const addU = extractBashTargets('git add -u');
  assert(addU.broadWrite === true, 'git add -u is a broad write');
  const commitA = extractBashTargets('git commit -a');
  assert(commitA.broadWrite === true, 'git commit -a is a broad write');
  const commitAm = extractBashTargets('git commit -am "msg"');
  assert(commitAm.broadWrite === true, 'git commit -am is a broad write');

  const commitM = extractBashTargets('git commit -m "msg"');
  assert(commitM.broadWrite === false && commitM.paths.length === 0, `plain git commit -m stays a no-op, got ${JSON.stringify(commitM)}`);
  const commitAmend = extractBashTargets('git commit --amend');
  assert(commitAmend.broadWrite === false && commitAmend.paths.length === 0, `--amend must not match --all substring, got ${JSON.stringify(commitAmend)}`);
  const addFile = extractBashTargets('git add src/file.js');
  assert(
    addFile.broadWrite === false && addFile.paths.length === 1 && addFile.paths[0] === 'src/file.js',
    `git add of an explicit path stays exact, got ${JSON.stringify(addFile)}`,
  );
  const logAll = extractBashTargets('git log --all');
  assert(logAll.broadWrite === false && logAll.paths.length === 0, `git log --all is untouched, got ${JSON.stringify(logAll)}`);
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

// ─── msn-18a topology (advisor-digest-slice4 binding condition 2) ──────────
// checkWrite's `root` is the physical checkout being written to (the hook's
// ctx.root) — for a write happening inside a LINKED WORKTREE, that root is
// the worktree's own checkout, never main's, even though the session/lane
// binding this write is governed by lives in MAIN's store (a worktree never
// gets its own session/lane store — msn-18a's control plane is shared).
// Before msn-18a's controlRoot fix, resolveWriteRecord passed this worktree
// root straight into resolvePipeline, which could not find the session (or,
// once found, the lane) in the worktree's own (nonexistent) `.bee/sessions`/
// `.bee/lanes` — a hard deny. Post-fix, resolveWriteRecord resolves through
// resolveContext(root).controlRoot first, landing back in MAIN's store.
await check('checkWrite: a write from a LINKED WORKTREE with a session bound to a lane in MAIN\'s store passes the lane guard (msn-18a, condition 2) — pre-fix this hard-denied because the worktree has no session/lane store of its own', async () => {
  const main = fs.mkdtempSync(path.join(os.tmpdir(), 'bee-guard-wt-main-'));
  const work = fs.mkdtempSync(path.join(os.tmpdir(), 'bee-guard-wt-work-'));
  try {
    // Real linked-worktree gitdir shape (same manual fixture pattern
    // test_state.mjs's resolveContext tests use — main .git/worktrees/<id> +
    // reverse gitdir pointer).
    const id = 'guard-wt-fixture';
    const gitdir = path.join(main, '.git', 'worktrees', id);
    fs.mkdirSync(gitdir, { recursive: true });
    fs.writeFileSync(path.join(work, '.git'), `gitdir: ${gitdir}\n`);
    fs.writeFileSync(path.join(gitdir, 'gitdir'), path.join(work, '.git') + '\n');

    // The session and its lane live ONLY in MAIN's store — a worktree never
    // gets its own (msn-18a: control plane is shared, not per-checkout).
    laneBinding.createSession(main, { id: 'sess-wt' });
    writeLaneFixture(main, 'feature-wt', {
      phase: 'swarming',
      approved_gates: { context: true, shape: true, execution: true, review: false },
    });
    laneBinding.bindSessionLane(main, 'sess-wt', 'feature-wt');

    // work's own .bee tree has NO session, NO lane — proving resolution
    // reaches all the way back to main's store rather than finding a
    // worktree-local copy that happens to also exist.
    assert(!fs.existsSync(path.join(work, '.bee', 'sessions')), 'the worktree must have no session store of its own for this proof');
    assert(!fs.existsSync(path.join(work, '.bee', 'lanes')), 'the worktree must have no lane store of its own for this proof');

    const verdict = checkWrite(work, defaultState(), 'src/app.ts', null, { sessionId: 'sess-wt' });
    assert(
      verdict.allow === true,
      `a worktree write governed by a lane-bound session must pass the lane guard against MAIN's control store, got ${JSON.stringify(verdict)}`,
    );
  } finally {
    fs.rmSync(main, { recursive: true, force: true });
    fs.rmSync(work, { recursive: true, force: true });
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
    await renewLease(dir, { type: 'path', id: 'src/hold/stale.ts' }, { ttl: 60, now: Date.now() - 7200 * 1000 });
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

await check('NET branch 6 — gated phases: exploring/planning deny outside allowed prefixes with kind gate; execution approval opens them', async () => {
  const dir = makeStateRepo('bee-net-gate-');
  try {
    for (const phase of ['exploring', 'planning']) {
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

// ─── xwh-4: cross-worktree foreign-hold branch ──────────────────────────────
// The write guard consults the shared cross-worktree holds ledger (xwh-1,
// worktree-holds.mjs) through the same topology resolution claim-next uses
// (xwh-3): ordinary checkout => holder 'main', ledger at the checkout's own
// root; granted linked worktree => holder = git-verified id, ledger at
// mainRoot; everything else (ungranted, unresolvable) => no consultation at
// all, fail-open. Runs after the cross-session hold branch, before every
// phase branch.
//
// multisession-native-14 (D4, issue #56 3.5, NEW behavior, RED-first): a
// foreign checkout's hold on a NORMAL (non-exclusive) path now downgrades to
// an advisory allow+warning instead of a hard deny — only paths matching the
// exclusive-resource list (migrations, lockfiles, release/manifest
// artifacts, generated client dirs; built-in defaults + config-extended via
// guards.exclusive_paths) keep the original hard block. Before this cell,
// EVERY foreign hold denied unconditionally — this first test below is the
// red-first regression: run against the pre-change guards.mjs it fails as a
// hard deny; against the post-change guards.mjs it passes as advisory
// allow+warning.

function writeHoldsLedger(dir, holds) {
  const runtime = path.join(dir, '.bee', 'runtime');
  fs.mkdirSync(runtime, { recursive: true });
  writeJsonAtomic(path.join(runtime, 'cross-worktree-holds.json'), { holds });
}

await check('checkWrite (xwh-4/msn-14): a foreign checkout\'s hold on a NORMAL path is advisory — allow:true with a warning naming the holding checkout, its feature, the expiry, and the merge-time consequence — phase-independent (swarming with execution approved)', async () => {
  const dir = makeStateRepo('bee-xwh-foreign-advisory-');
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
    const verdict = checkWrite(dir, state, 'src/held/feature.ts', 'net-writer');
    assert(
      verdict.allow === true,
      `a foreign hold on a normal path must ALLOW (advisory, not a hard block), got ${JSON.stringify(verdict)}`,
    );
    assert(
      typeof verdict.warning === 'string' && verdict.warning.length > 0,
      `a foreign-held normal path must never be a SILENT allow — a warning is required, got ${JSON.stringify(verdict)}`,
    );
    assert(
      verdict.warning.includes('wt-featx') && verdict.warning.includes('feat-x') && /expires|no expiry/.test(verdict.warning),
      `the warning must name the holding checkout, its feature, and the expiry, got: ${verdict.warning}`,
    );
    assert(
      /merge/i.test(verdict.warning),
      `the warning must name the merge-time consequence ("bee worktree merge" will surface real conflicts), got: ${verdict.warning}`,
    );
  } finally {
    fs.rmSync(dir, { recursive: true, force: true });
  }
});

await check('checkWrite (xwh-4/msn-14): a foreign hold on an EXCLUSIVE-marked resource (built-in defaults: migrations, lockfiles, release-manifest.json, .bee/onboarding.json, generated/) still hard-denies cross-worktree, same reason shape as before this cell', async () => {
  const dir = makeStateRepo('bee-xwh-exclusive-deny-');
  try {
    const state = { ...defaultState(), phase: 'swarming', approved_gates: { context: true, shape: true, execution: true, review: false } };
    const exclusivePaths = [
      'db/migrations/0001_init.sql',
      'package-lock.json',
      'packages/api/yarn.lock',
      'docs/history/codex-harness-hardening/release-manifest.json',
      '.bee/onboarding.json',
      'src/generated/client.ts',
    ];
    writeHoldsLedger(
      dir,
      exclusivePaths.map((p, i) => ({
        path: p,
        holder: 'wt-excl',
        feature: 'feat-excl',
        session: null,
        cell: `excl-${i}`,
        ttl_seconds: 3600,
        mirrored_at: new Date().toISOString(),
        released_at: null,
      })),
    );
    for (const p of exclusivePaths) {
      const deny = checkWrite(dir, state, p, 'net-writer');
      assert(
        deny.allow === false && deny.kind === 'worktree-hold',
        `exclusive path "${p}" must still hard-deny cross-worktree, got ${JSON.stringify(deny)}`,
      );
      assert(
        deny.reason.includes('wt-excl') && deny.reason.includes('feat-excl') && /expires|no expiry/.test(deny.reason),
        `the deny reason must name the holding checkout, its feature, and the expiry, got: ${deny.reason}`,
      );
    }
  } finally {
    fs.rmSync(dir, { recursive: true, force: true });
  }
});

await check('checkWrite (xwh-4/msn-14): guards.exclusive_paths in .bee/config.json EXTENDS the built-in defaults (does not replace them) — a config-declared glob hard-denies cross-worktree, and built-in defaults still hard-deny with no config at all', async () => {
  const dir = makeStateRepo('bee-xwh-exclusive-config-');
  try {
    const state = { ...defaultState(), phase: 'swarming', approved_gates: { context: true, shape: true, execution: true, review: false } };
    writeHoldsLedger(dir, [
      { path: 'secrets/vault.bin', holder: 'wt-cfg', feature: 'feat-cfg', session: null, cell: 'cfg-1', ttl_seconds: 3600, mirrored_at: new Date().toISOString(), released_at: null },
      { path: 'package-lock.json', holder: 'wt-cfg', feature: 'feat-cfg', session: null, cell: 'cfg-2', ttl_seconds: 3600, mirrored_at: new Date().toISOString(), released_at: null },
    ]);
    // no config yet: the custom path is advisory (not in the built-in list), the built-in lockfile still hard-denies
    assert(checkWrite(dir, state, 'secrets/vault.bin', 'net-writer').allow === true, 'a non-listed path is advisory before any config extension');
    assert(checkWrite(dir, state, 'package-lock.json', 'net-writer').allow === false, 'a built-in default keeps hard-denying with zero config');
    const configPath = path.join(dir, '.bee', 'config.json');
    writeJsonAtomic(configPath, { guards: { exclusive_paths: ['**/vault.bin'] } });
    const deny = checkWrite(dir, state, 'secrets/vault.bin', 'net-writer');
    assert(deny.allow === false && deny.kind === 'worktree-hold', `a config-extended exclusive glob must hard-deny, got ${JSON.stringify(deny)}`);
    // the built-in default is still active alongside the extension (extends, never replaces)
    assert(checkWrite(dir, state, 'package-lock.json', 'net-writer').allow === false, 'built-in defaults stay active alongside a config extension');
    fs.rmSync(configPath, { force: true });
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

// ─── msn-21: single resolveContext resolution, workspace-scoped hard
// leases, and the new workspace-ownership deny class ──────────────────────

await check("checkWrite (msn-21, deny class (b)): a foreign session's exact lease taken in a DIFFERENT workspace never hard-blocks (repo-relative path collision across physical checkouts is not a real conflict) — the SAME-workspace case still hard-blocks byte-identically", async () => {
  const dir = makeStateRepo('bee-msn21-ws-lease-');
  try {
    laneBinding.createSession(dir, { id: 'sess-main-actor' });
    laneBinding.createSession(dir, { id: 'sess-main-holder' });
    laneBinding.createSession(dir, { id: 'sess-other-ws-holder', workspace_id: 'wt-other' });
    writeLaneFixture(dir, 'lane-msn21-ws', {
      phase: 'swarming',
      approved_gates: { context: true, shape: true, execution: true, review: false },
    });
    laneBinding.bindSessionLane(dir, 'sess-main-actor', 'lane-msn21-ws');
    const state = readState(dir);

    // A lease held by a session whose OWN workspace_id is a different
    // workspace ('wt-other') never hard-blocks a 'main'-workspace actor,
    // even though the path string collides exactly.
    await reserve(dir, { agent: 'other-ws-agent', cell: 'ws-1', path: 'src/cross-ws.ts', session: 'sess-other-ws-holder' });
    const crossWs = checkWrite(dir, state, 'src/cross-ws.ts', null, { sessionId: 'sess-main-actor' });
    assert(crossWs.allow === true, `a lease held in a DIFFERENT workspace must never hard-block, got ${JSON.stringify(crossWs)}`);

    // A lease held by a session in the SAME ('main') workspace still hard-blocks — unchanged.
    await reserve(dir, { agent: 'same-ws-agent', cell: 'ws-1', path: 'src/same-ws.ts', session: 'sess-main-holder' });
    const sameWs = checkWrite(dir, state, 'src/same-ws.ts', null, { sessionId: 'sess-main-actor' });
    assert(
      sameWs.allow === false && sameWs.kind === 'hold',
      `a lease held in the SAME workspace must still hard-block byte-identically, got ${JSON.stringify(sameWs)}`,
    );
  } finally {
    fs.rmSync(dir, { recursive: true, force: true });
  }
});

function ownershipRepo(prefix) {
  const dir = makeStateRepo(prefix);
  // The DEFAULT (non-lane) pipeline governs — an idle default record with
  // execution NOT approved would hit the intake/gate branch before ever
  // reaching the ownership check, so every fixture below runs 'swarming'...
  // no: swarming is the OTHER exempted branch. Use 'validating' with
  // execution approved so writes reach past the gate branches and the
  // ownership check is the thing actually deciding, same placement
  // discipline test_guards.mjs's own D3 panel pin uses for cross-session holds.
  writeJsonAtomic(path.join(dir, '.bee', 'state.json'), {
    schema_version: '1.0',
    phase: 'validating',
    feature: 'msn21-owned',
    mode: 'standard',
    approved_gates: { context: true, shape: true, execution: true, review: false },
    workers: [],
  });
  return dir;
}

await check('checkWrite (msn-21, deny class (c)): a non-owner session refuses a write in a workspace another LIVE session already write-owns — named in the reason, allowed once the owner is stale, allowed for the owner itself, allowed when unregistered', async () => {
  const dir = ownershipRepo('bee-msn21-owner-deny-');
  try {
    const state = readState(dir);
    laneBinding.createSession(dir, { id: 'sess-owner' });
    laneBinding.createSession(dir, { id: 'sess-intruder' });

    // Before anybody has claimed ownership, the workspace is unregistered —
    // never blocks (a solo caller always becomes owner byte-identical
    // prohibition — checkWrite itself never claims, it only refuses to
    // block an unclaimed workspace).
    const beforeClaim = checkWrite(dir, state, 'src/owned.ts', null, { sessionId: 'sess-intruder' });
    assert(beforeClaim.allow === true, `an unregistered workspace must never block, got ${JSON.stringify(beforeClaim)}`);

    await registerWorkspace(dir, { id: 'main', type: 'main', root: dir });
    await claimWriteOwnership(dir, 'main', 'sess-owner');

    // The owner itself is always allowed.
    const ownerWrite = checkWrite(dir, state, 'src/owned.ts', null, { sessionId: 'sess-owner' });
    assert(ownerWrite.allow === true, `the workspace's own owner must never be blocked, got ${JSON.stringify(ownerWrite)}`);

    // A different, LIVE session refuses, naming the owner.
    const intruderWrite = checkWrite(dir, state, 'src/owned.ts', null, { sessionId: 'sess-intruder' });
    assert(
      intruderWrite.allow === false && intruderWrite.kind === 'workspace-ownership',
      `a non-owner session must refuse with kind 'workspace-ownership', got ${JSON.stringify(intruderWrite)}`,
    );
    assert(intruderWrite.reason.includes('sess-owner'), `the refusal must name the live owner, got: ${intruderWrite.reason}`);
    assert(/--isolate/.test(intruderWrite.reason), `the refusal must name the --isolate escape hatch, got: ${intruderWrite.reason}`);

    // A sessionless write (no sessionId) never consults ownership at all —
    // byte-identical to every pre-msn-21 sessionless call.
    const sessionless = checkWrite(dir, state, 'src/owned.ts');
    assert(sessionless.allow === true, `a sessionless write must never consult workspace ownership, got ${JSON.stringify(sessionless)}`);

    // A DEAD owner (heartbeat past the staleness window) never blocks.
    const longAgo = Date.now() - 3600 * 1000 * 10;
    laneBinding.createSession(dir, { id: 'sess-dead-owner', now: longAgo });
    // Force the transfer regardless of the CURRENT owner's real liveness —
    // this call is only fixture setup for the "dead owner" scenario;
    // checkWrite's own production isOwnerLive equivalent (unfaked) is what
    // the assertion below actually exercises.
    await claimWriteOwnership(dir, 'main', 'sess-dead-owner', { now: longAgo, isOwnerLive: () => false });
    const staleOwnerWrite = checkWrite(dir, state, 'src/owned.ts', null, { sessionId: 'sess-intruder' });
    assert(staleOwnerWrite.allow === true, `a stale/dead owner must never block, got ${JSON.stringify(staleOwnerWrite)}`);
  } finally {
    fs.rmSync(dir, { recursive: true, force: true });
  }
});

await check("checkWrite (msn-21, deny class (c) scoping): the ownership deny ONLY engages where applyWritePolicy's isolated mode governs — a lane-bound session, a swarming-phase write, and a config write_policy of 'observe'/'shared-disjoint' all pass through untouched even with a different LIVE owner", async () => {
  const dir = ownershipRepo('bee-msn21-owner-scope-');
  try {
    const state = readState(dir);
    laneBinding.createSession(dir, { id: 'sess-owner-2' });
    laneBinding.createSession(dir, { id: 'sess-intruder-2' });
    await registerWorkspace(dir, { id: 'main', type: 'main', root: dir });
    await claimWriteOwnership(dir, 'main', 'sess-owner-2');

    // A lane-bound session ('source' resolves to 'lane', not 'default') is untouched.
    writeLaneFixture(dir, 'lane-msn21-owner', {
      phase: 'swarming',
      approved_gates: { context: true, shape: true, execution: true, review: false },
    });
    laneBinding.bindSessionLane(dir, 'sess-intruder-2', 'lane-msn21-owner');
    const laneGoverned = checkWrite(dir, state, 'src/lane-owned.ts', null, { sessionId: 'sess-intruder-2' });
    assert(
      laneGoverned.allow === true,
      `a lane-bound session must pass through the ownership check untouched (lanes keep their existing branches), got ${JSON.stringify(laneGoverned)}`,
    );

    // A DEFAULT-pipeline session in phase 'swarming' with an agent identity
    // is governed by the existing reservation branch, not ownership. An
    // UNBOUND session resolves through resolvePipeline to the DEFAULT
    // record read fresh from disk (never the in-memory `state` argument
    // passed to checkWrite once sessionId is set) — so the fixture must
    // overwrite .bee/state.json itself to actually govern this call.
    laneBinding.createSession(dir, { id: 'sess-intruder-3' });
    writeState(dir, { ...defaultState(), phase: 'swarming', approved_gates: { context: true, shape: true, execution: true, review: false } });
    const swarmingWrite = checkWrite(dir, readState(dir), 'src/swarm-owned.ts', 'some-agent', { sessionId: 'sess-intruder-3' });
    assert(
      swarmingWrite.allow === true,
      `a swarming-phase write must pass through the ownership check untouched (the reservation branch governs it), got ${JSON.stringify(swarmingWrite)}`,
    );
    // restore the 'validating' default record for the remaining sub-assertions
    writeState(dir, { ...defaultState(), phase: 'validating', feature: 'msn21-owned', mode: 'standard', approved_gates: { context: true, shape: true, execution: true, review: false } });

    // config.guards.write_policy: 'observe' skips ownership entirely.
    const configPath = path.join(dir, '.bee', 'config.json');
    const beforeConfig = readJson(configPath, {});
    writeJsonAtomic(configPath, { ...beforeConfig, guards: { ...(beforeConfig.guards || {}), write_policy: 'observe' } });
    laneBinding.createSession(dir, { id: 'sess-intruder-4' });
    const observeWrite = checkWrite(dir, state, 'src/observe-owned.ts', null, { sessionId: 'sess-intruder-4' });
    assert(observeWrite.allow === true, `write_policy 'observe' must skip the ownership check entirely, got ${JSON.stringify(observeWrite)}`);
    writeJsonAtomic(configPath, beforeConfig || {});
  } finally {
    fs.rmSync(dir, { recursive: true, force: true });
  }
});

await check('checkWrite (msn-21): a legacy no-records repo (no .bee/runtime/workspaces/ store, no session workspace_id anywhere) is byte-identical — the ownership check never denies, the cross-session hold check hard-blocks exactly as before', async () => {
  const dir = makeStateRepo('bee-msn21-legacy-');
  try {
    assert(!fs.existsSync(path.join(dir, '.bee', 'runtime', 'workspaces')), 'fixture sanity: no workspace store exists at all');
    laneBinding.createSession(dir, { id: 'sess-legacy-a' });
    laneBinding.createSession(dir, { id: 'sess-legacy-b' });
    writeLaneFixture(dir, 'lane-legacy', {
      phase: 'swarming',
      approved_gates: { context: true, shape: true, execution: true, review: false },
    });
    laneBinding.bindSessionLane(dir, 'sess-legacy-a', 'lane-legacy');
    const state = readState(dir);

    // No workspace store at all -> the ownership check never denies, no
    // matter which session writes.
    const open = checkWrite(dir, state, 'src/legacy.ts', null, { sessionId: 'sess-legacy-a' });
    assert(open.allow === true, `a legacy repo with no workspace store must stay open, got ${JSON.stringify(open)}`);

    // The pre-msn-21 cross-session hard-lease behavior is unchanged: both
    // sessions default to workspace_id 'main' (OMITTED on their records),
    // so the SAME-workspace scoping is a no-op here — hard block preserved.
    await reserve(dir, { agent: 'legacy-agent', cell: 'lg-1', path: 'src/legacy-hold.ts', session: 'sess-legacy-b' });
    const held = checkWrite(dir, state, 'src/legacy-hold.ts', null, { sessionId: 'sess-legacy-a' });
    assert(
      held.allow === false && held.kind === 'hold',
      `a legacy repo's cross-session hard lease must hard-block exactly as before msn-21, got ${JSON.stringify(held)}`,
    );
  } finally {
    fs.rmSync(dir, { recursive: true, force: true });
  }
});

await check('checkWrite (msn-21): a present-but-corrupt workspace record fails CLOSED with a typed workspace-unreadable deny — never a throw; a missing workspace record stays open', async () => {
  const dir = ownershipRepo('bee-msn21-owner-corrupt-');
  try {
    const state = readState(dir);
    laneBinding.createSession(dir, { id: 'sess-corrupt-actor' });
    fs.mkdirSync(path.join(dir, '.bee', 'runtime', 'workspaces'), { recursive: true });
    fs.writeFileSync(path.join(dir, '.bee', 'runtime', 'workspaces', 'main.json'), '{ not json', 'utf8');
    let verdict;
    let threw = false;
    try {
      verdict = checkWrite(dir, state, 'src/owned-corrupt.ts', null, { sessionId: 'sess-corrupt-actor' });
    } catch {
      threw = true;
    }
    assert(!threw, 'checkWrite must never throw on a corrupt workspace record');
    assert(
      verdict && verdict.allow === false && verdict.kind === 'workspace-unreadable',
      `a corrupt workspace record must be a typed {allow:false, kind:'workspace-unreadable'} deny, got ${JSON.stringify(verdict)}`,
    );
    fs.rmSync(path.join(dir, '.bee', 'runtime', 'workspaces', 'main.json'));
    const restored = checkWrite(dir, state, 'src/owned-corrupt.ts', null, { sessionId: 'sess-corrupt-actor' });
    assert(restored.allow === true, `a missing (never-registered) workspace record must stay open, got ${JSON.stringify(restored)}`);
  } finally {
    fs.rmSync(dir, { recursive: true, force: true });
  }
});

await check('checkWrite (msn-21, worktree-topology condition 7): from a LINKED WORKTREE, the workspace-ownership check resolves against MAIN\'s control store, never a nonexistent worktree-local one', async () => {
  const main = fs.mkdtempSync(path.join(os.tmpdir(), 'bee-msn21-wt-main-'));
  const work = fs.mkdtempSync(path.join(os.tmpdir(), 'bee-msn21-wt-work-'));
  try {
    const id = 'msn21-wt-fixture';
    const gitdir = path.join(main, '.git', 'worktrees', id);
    fs.mkdirSync(gitdir, { recursive: true });
    fs.writeFileSync(path.join(work, '.git'), `gitdir: ${gitdir}\n`);
    fs.writeFileSync(path.join(gitdir, 'gitdir'), path.join(work, '.git') + '\n');
    fs.mkdirSync(path.join(main, '.bee'), { recursive: true });
    writeJsonAtomic(path.join(main, '.bee', 'onboarding.json'), { schema_version: '1.0', bee_version: '0.1.0' });
    // An unbound session resolves to the DEFAULT record read fresh from
    // MAIN's own state.json (never the worktree's, which has none) — give
    // it an allowed phase so the second (owner) assertion below is decided
    // by the ownership check, not an unrelated intake-gate deny.
    writeJsonAtomic(path.join(main, '.bee', 'state.json'), {
      schema_version: '1.0',
      phase: 'validating',
      feature: 'msn21-wt-owned',
      mode: 'standard',
      approved_gates: { context: true, shape: true, execution: true, review: false },
      workers: [],
    });

    // Ownership registered/claimed ONLY in main's store — the worktree has
    // no .bee/runtime/workspaces/ of its own (ungranted: not registered in
    // worktree-grants.json, so ctx.workspaceId falls back to 'main' — this
    // write is governed by the SAME 'main' workspace record main itself
    // writes to, proving the ownership check reaches main's store rather
    // than silently finding "nothing registered" in a worktree-local one).
    laneBinding.createSession(main, { id: 'sess-wt-owner' });
    laneBinding.createSession(main, { id: 'sess-wt-intruder' });
    await registerWorkspace(main, { id: 'main', type: 'main', root: main });
    await claimWriteOwnership(main, 'main', 'sess-wt-owner');
    assert(!fs.existsSync(path.join(work, '.bee', 'runtime', 'workspaces')), 'the worktree must have no workspace store of its own for this proof');

    const state = defaultState();
    const denied = checkWrite(work, state, 'src/wt-owned.ts', null, { sessionId: 'sess-wt-intruder' });
    assert(
      denied.allow === false && denied.kind === 'workspace-ownership',
      `a write from the linked worktree must resolve ownership against MAIN's store, got ${JSON.stringify(denied)}`,
    );
    assert(denied.reason.includes('sess-wt-owner'), `the refusal must name the owner found in MAIN's store, got: ${denied.reason}`);

    const allowed = checkWrite(work, state, 'src/wt-owned.ts', null, { sessionId: 'sess-wt-owner' });
    assert(allowed.allow === true, `the owner itself, writing from the worktree, must be allowed, got ${JSON.stringify(allowed)}`);
  } finally {
    fs.rmSync(main, { recursive: true, force: true });
    fs.rmSync(work, { recursive: true, force: true });
  }
});

await check('checkWrite (msn-21): the hook posture regression — checkWrite never acquires ANY store lock (readWorkspace/readSession are plain reads), so a write resolves promptly even while another process genuinely holds the workspace lock file', async () => {
  const dir = ownershipRepo('bee-msn21-no-wait-');
  try {
    const state = readState(dir);
    laneBinding.createSession(dir, { id: 'sess-lockcheck' });
    await registerWorkspace(dir, { id: 'main', type: 'main', root: dir });
    await claimWriteOwnership(dir, 'main', 'sess-lockcheck');

    // Simulate a genuinely live holder of the SAME lock claimWriteOwnership/
    // attachWorkspace would take (withStoreLock's own 'workspace:<id>' name,
    // workspace-store.mjs's withWorkspaceLock) — an O_EXCL lockfile is NOT
    // even how withStoreLock/lock.mjs marks a hold (it uses a directory or
    // sentinel file the retry loop polls for), but the point of this
    // regression is structural: checkWrite must complete near-instantly no
    // matter what sits at that path, because it never calls
    // withStoreLock/acquireStoreLockOnceSync at all — only readWorkspace,
    // a plain fs.readFileSync.
    const locksDirPath = path.dirname(lockFilePath(dir, 'workspace:main'));
    fs.mkdirSync(locksDirPath, { recursive: true });
    fs.writeFileSync(lockFilePath(dir, 'workspace:main'), JSON.stringify({ pid: process.pid, ts: new Date().toISOString() }));

    const started = Date.now();
    const verdict = checkWrite(dir, state, 'src/no-wait.ts', null, { sessionId: 'sess-lockcheck' });
    const elapsedMs = Date.now() - started;
    assert(verdict.allow === true, `the owner's own write must still be allowed, got ${JSON.stringify(verdict)}`);
    assert(elapsedMs < 500, `checkWrite must never wait on a store lock — took ${elapsedMs}ms with a lock file present`);
  } finally {
    fs.rmSync(dir, { recursive: true, force: true });
  }
});

// ─── concurrent-worker whole-tree git guard (gc-2) ──────────────────────────
// The incident this covers: in one parallel wave, two `git add` index sweeps
// folded workers' files into a sibling's commit, and one whole-tree revert
// DELETED a live worker's in-progress edit while that worker held a valid file
// reservation. Every row below runs at phase `swarming` on purpose — the whole
// point is that the pre-gc-2 guard returned null for every non-terminal phase.

function makeGitConcurrencyRepo(prefix) {
  const dir = makeStateRepo(prefix);
  // resolveContext/controlRootFor both walk for a .git node; makeStateRepo
  // (unlike makeTempRepo) does not create one.
  fs.mkdirSync(path.join(dir, '.git'), { recursive: true });
  return dir;
}

// One live session holding N reservations under N distinct agent nicknames —
// the intra-session swarm shape. `activeWorkers` alone reports this as ONE
// worker (subagents share their parent session's id and heartbeat), which is
// exactly why the guard counts reservation agents too.
async function seedWorkers(dir, agents, { session = 'sess-swarm' } = {}) {
  laneBinding.createSession(dir, { id: session });
  for (const agent of agents) {
    await reserve(dir, { agent, cell: `cell-${agent}`, path: `src/${agent}.js`, ttl: 3600, session });
  }
}

const swarmingState = () => ({ ...defaultState(), phase: 'swarming' });

await check('checkGitBashCommand (gc-2): every whole-tree git verb is refused while >1 worker is live', async () => {
  const dir = makeGitConcurrencyRepo('bee-gc2-multi-');
  try {
    await seedWorkers(dir, ['exec-a', 'exec-b']);
    const state = swarmingState();
    const commands = [
      ['git reset --hard', 'reset'],
      ['git reset HEAD~1', 'reset'],
      ['git stash', 'stash'],
      ['git stash pop', 'stash'],
      ['git checkout .', 'checkout'],
      ['git clean -fd', 'clean'],
      ['git restore --staged src/a.js', 'restore'],
      ['git revert HEAD', 'revert'],
      ['git rebase main', 'rebase'],
      ['git merge feature-x', 'merge'],
      ['git cherry-pick abc123', 'cherry-pick'],
      ['git apply patch.diff', 'apply'],
      ['git add src/a.js', 'add'],
      ['git add -A', 'add'],
      ['git commit -m "sweep"', 'commit'],
      ['git commit -am "sweep"', 'commit -a'],
      ['git commit -m "broad" -- .', 'commit'],
    ];
    for (const [command, verb] of commands) {
      const verdict = checkGitBashCommand(dir, state, command, { cwd: dir });
      assert(
        verdict && verdict.allow === false && verdict.kind === 'git-concurrent-tree',
        `"${command}" must be refused under 2 live workers, got ${JSON.stringify(verdict)}`,
      );
      assert(
        verdict.reason.includes(`\`git ${verb}\``),
        `"${command}" refusal must name the verb \`git ${verb}\`, got: ${verdict.reason}`,
      );
      assert(verdict.reason.includes('2 workers are live'), `"${command}" refusal must name the worker count, got: ${verdict.reason}`);
      // The refusal is only useful if it hands back the sanctioned route.
      assert(verdict.reason.includes('git status'), `"${command}" refusal must name read-only inspection, got: ${verdict.reason}`);
      assert(verdict.reason.includes('GIT_INDEX_FILE'), `"${command}" refusal must name the temp-index route, got: ${verdict.reason}`);
      assert(verdict.reason.includes('git add -N'), `"${command}" refusal must name the intent-to-add fallback, got: ${verdict.reason}`);
      assert(
        verdict.reason.includes('git commit -- <your paths>'),
        `"${command}" refusal must name the path-scoped commit, got: ${verdict.reason}`,
      );
    }
  } finally {
    fs.rmSync(dir, { recursive: true, force: true });
  }
});

await check('checkGitBashCommand (gc-2): a single-worker session is completely unaffected', async () => {
  const dir = makeGitConcurrencyRepo('bee-gc2-solo-');
  try {
    await seedWorkers(dir, ['exec-solo']);
    const state = swarmingState();
    for (const command of ['git reset --hard', 'git stash', 'git checkout .', 'git clean -fd', 'git add -A', 'git commit -m "x"']) {
      assert(
        checkGitBashCommand(dir, state, command, { cwd: dir }) === null,
        `"${command}" must stay allowed for a solo worker, got ${JSON.stringify(checkGitBashCommand(dir, state, command, { cwd: dir }))}`,
      );
    }
  } finally {
    fs.rmSync(dir, { recursive: true, force: true });
  }
});

await check("checkGitBashCommand (gc-2): the orchestrator's own release/merge work is never blocked (no reservations, one live session)", async () => {
  const dir = makeGitConcurrencyRepo('bee-gc2-orchestrator-');
  try {
    laneBinding.createSession(dir, { id: 'sess-orchestrator' });
    const state = swarmingState();
    for (const command of ['git merge feature-x', 'git rebase main', 'git checkout main', 'git commit -m "release"']) {
      assert(
        checkGitBashCommand(dir, state, command, { cwd: dir }) === null,
        `"${command}" must stay allowed with a single live session, got ${JSON.stringify(checkGitBashCommand(dir, state, command, { cwd: dir }))}`,
      );
    }
  } finally {
    fs.rmSync(dir, { recursive: true, force: true });
  }
});

await check('checkGitBashCommand (gc-2): two live SESSIONS in the same checkout count as two workers (no reservations needed)', async () => {
  const dir = makeGitConcurrencyRepo('bee-gc2-sessions-');
  try {
    // Guards the row above from passing vacuously: with nothing reserved, the
    // count comes purely from the derived live-session view.
    laneBinding.createSession(dir, { id: 'sess-one' });
    laneBinding.createSession(dir, { id: 'sess-two' });
    const verdict = checkGitBashCommand(dir, swarmingState(), 'git reset --hard', { cwd: dir });
    assert(
      verdict && verdict.allow === false && verdict.reason.includes('2 workers are live'),
      `two live sessions must count as two workers, got ${JSON.stringify(verdict)}`,
    );
  } finally {
    fs.rmSync(dir, { recursive: true, force: true });
  }
});

await check('checkGitBashCommand (gc-2): a worker in ANOTHER workspace does not count — cross-worktree merge stays open', async () => {
  const dir = makeGitConcurrencyRepo('bee-gc2-workspace-');
  try {
    // Two live sessions, but the second is stamped to a different physical
    // checkout: its whole-tree verbs cannot reach this tree, so counting it
    // would over-block this session for nothing.
    laneBinding.createSession(dir, { id: 'sess-here' });
    laneBinding.createSession(dir, { id: 'sess-elsewhere', workspace_id: 'other-worktree' });
    const state = swarmingState();
    assert(
      checkGitBashCommand(dir, state, 'git merge feature-x', { cwd: dir }) === null,
      'a live session in another workspace must not make this checkout look multi-worker',
    );
  } finally {
    fs.rmSync(dir, { recursive: true, force: true });
  }
});

await check('checkGitBashCommand (gc-2): path-scoped commits, reads, and the temp-index route stay allowed under >1 worker', async () => {
  const dir = makeGitConcurrencyRepo('bee-gc2-allowed-');
  try {
    await seedWorkers(dir, ['exec-a', 'exec-b']);
    const state = swarmingState();
    const allowed = [
      // read-only inspection
      'git status',
      'git diff',
      'git diff --cached',
      'git log --oneline -5',
      'git show HEAD',
      // exactly what the rules ask a worker to use
      'git commit -m "gc-2: my work" -- src/exec-a.js',
      'git commit -m "gc-2" -- src/exec-a.js src/exec-a2.js',
      'git add -N src/brand-new.js',
      'git add --intent-to-add src/brand-new.js',
      'git stash list',
      'git stash show',
      'git apply --check patch.diff',
      // the temp-index route the refusal itself prescribes
      'GIT_INDEX_FILE=/tmp/idx git read-tree HEAD',
      'GIT_INDEX_FILE=/tmp/idx git update-index --add src/exec-a.js',
      'GIT_INDEX_FILE=/tmp/idx git write-tree',
      'git commit-tree abc123 -p HEAD -m "gc-2"',
      'git update-ref HEAD def456',
    ];
    for (const command of allowed) {
      const verdict = checkGitBashCommand(dir, state, command, { cwd: dir });
      assert(
        verdict === null,
        `"${command}" must stay allowed under 2 live workers, got ${JSON.stringify(verdict)}`,
      );
    }
  } finally {
    fs.rmSync(dir, { recursive: true, force: true });
  }
});

await check('checkGitBashCommand (gc-2): an UNRESOLVABLE worker count refuses (unreadable means obligation, not clearance)', async () => {
  const dir = makeGitConcurrencyRepo('bee-gc2-unresolvable-');
  try {
    // Not a single worker anywhere — the refusal here comes purely from the
    // torn store, proving the conservative branch and not a real count.
    fs.writeFileSync(reservationsPath(dir), '{ "reservations": [ NOT JSON');
    const state = swarmingState();
    const verdict = checkGitBashCommand(dir, state, 'git reset --hard', { cwd: dir });
    assert(
      verdict && verdict.allow === false && verdict.kind === 'git-concurrent-tree',
      `an unresolvable worker count must refuse, got ${JSON.stringify(verdict)}`,
    );
    assert(
      verdict.reason.includes('could not be resolved') && verdict.reason.includes('treated as more than one worker'),
      `the refusal must say the count was unresolvable and conservatively treated as multi-worker, got: ${verdict.reason}`,
    );
    // A read stays a read even then — an over-denying guard must never lock a
    // session out of diagnosing its own mess (critical pattern 20260716).
    assert(
      checkGitBashCommand(dir, state, 'git status', { cwd: dir }) === null,
      'git status must stay allowed even when the worker count is unresolvable',
    );
  } finally {
    fs.rmSync(dir, { recursive: true, force: true });
  }
});

await check('checkGitBashCommand (gc-2): the terminal-phase intake gate is untouched by the new branch', async () => {
  const dir = makeGitConcurrencyRepo('bee-gc2-intake-');
  try {
    await seedWorkers(dir, ['exec-solo']);
    const idle = { ...defaultState(), phase: 'idle' };
    const push = checkGitBashCommand(dir, idle, 'git push origin main', { cwd: dir });
    assert(
      push && push.allow === false && push.kind === 'git-push',
      `git push must still hit the intake gate at a terminal phase, got ${JSON.stringify(push)}`,
    );
    const readOnly = checkGitBashCommand(dir, idle, 'git status', { cwd: dir });
    assert(
      readOnly && readOnly.allow === true && readOnly.kind === 'git-read-only',
      `read-only git must still be exempted at a terminal phase, got ${JSON.stringify(readOnly)}`,
    );
    // ...and the concurrency rule outranks it: a whole-tree verb under >1
    // worker is refused as a CONCURRENCY denial, not an intake one.
    await reserve(dir, { agent: 'exec-second', cell: 'cell-2', path: 'src/second.js', ttl: 3600, session: 'sess-swarm' });
    const concurrent = checkGitBashCommand(dir, idle, 'git reset --hard', { cwd: dir });
    assert(
      concurrent && concurrent.allow === false && concurrent.kind === 'git-concurrent-tree',
      `the concurrency rule must outrank the intake gate, got ${JSON.stringify(concurrent)}`,
    );
  } finally {
    fs.rmSync(dir, { recursive: true, force: true });
  }
});

printSummaryAndExit();
