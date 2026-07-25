#!/usr/bin/env node
// test_compact_capsule.mjs — the suite for the compact capsule and for the
// three renderers it shares with the full session preamble (feature
// compaction-hardening, cell cz-5; decisions D6/D7/D8/D15/D19/D26/D27).
//
// It targets the SOURCE modules (packages/bee/lib/), never the
// .bee/bin/lib mirror — D17: the template tree is the edit target and
// scripts/tests/test_lib_mirror.mjs is what proves the mirror matches it.
//
// The five obligations this file exists to hold. Each one is a rule a plain
// "the capsule renders something sensible" reading would silently break, and
// each one is UNPROVABLE from any other suite in this feature:
//
//   1. NO CAPSULE BYTE MAY BE ANCHOR-CORRELATED (D19 + the additivity row at
//      hooks/test_hook_contracts.mjs:2740-2780). compactCheck reports "an
//      anchor exists" as one of its seven checks, so a STATE MISMATCH line
//      rendered off the raw sweep varies with anchor presence BY
//      CONSTRUCTION. The proof is a paired render — the same repo, rendered
//      before and after an anchor is written, asserted BYTE-EQUAL — run for a
//      clean sweep AND for a FAILING one, because only the failing pair
//      exercises the line that carries the anchor check's own text.
//      This cannot be deferred to the hook-contract suite: during cz-5 the
//      hook is not wired to the capsule at all, so that row compares two full
//      preambles and never touches the builder. The red would first appear in
//      cz-6, whose declared files exclude compaction.mjs.
//   2. handoffOutcome IS MANDATORY (D26/D27). Every existing row exercising
//      compact + a planned-next handoff (hooks/test_hook_contracts.mjs:2427-2455,
//      :2864-2872) regexes only the WAIT heading and never the reason line, so
//      a capsule that drops the parameter passes every other suite in this
//      feature and the full verify while a compacted session silently loses
//      the explanation of WHY adoption was refused. Asserted here both ways:
//      the line appears with the outcome, and it is absent without it.
//   3. THE ITEM ORDER IS THE CONTRACT (D6 items 2-12). Asserted POSITIONALLY,
//      so reordering fails the suite rather than passing on a set-membership
//      technicality.
//   4. THE FULL PREAMBLE IS BYTE-FROZEN (D8) against a COMMITTED golden at
//      scripts/tests/fixtures/preamble-golden.txt. Deliberately NOT a
//      `git show HEAD:` reconstruction: that form passes at cz-5's cap and
//      then decays into a tautology for cz-6/cz-7/cz-8, which all re-run this
//      suite after HEAD already holds the changed file. Only the version
//      literal is normalized, so a release bump never reds it.
//      This is also the test matrix's NEGATIVE CONTROL row: a deliberately
//      broken renderer extraction shows up here as a diff, rather than being
//      inferred from other rows staying green.
//   5. D15's ADDITIVITY IS SCOPED HONESTLY. A bare repo renders a capsule with
//      NO survival warning and NO mismatch line — the shape D15 pins, and the
//      one a naive "always report the sweep" capsule breaks.
//
// Regenerating the golden: `node scripts/tests/test_compact_capsule.mjs --update-golden`
// rewrites scripts/tests/fixtures/preamble-golden.txt from the fixtures below and
// exits. Never run it to "fix" a red row without reading the diff first — the
// whole value of a committed golden is that it makes an unintended byte change
// loud.
//
// PROVENANCE OF THE COMMITTED GOLDEN: its bytes were generated ONCE, at cz-5's
// authoring time, from the PRE-EXTRACTION buildSessionPreamble (cz-4's HEAD,
// commit c60c4bb) — that is what makes the very first green run of this row a
// real proof that pulling three renderers out of inject.mjs changed nothing,
// rather than a restatement of whatever the new code happens to emit. From
// here on it is an ordinary committed fixture: nothing in this suite consults
// git, so cz-6/cz-7/cz-8 re-run it against exactly these bytes.

import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import crypto from 'node:crypto';
import { fileURLToPath } from 'node:url';

import { check, assert, printSummaryAndExit } from '../lib/test-fixture.mjs';
import { runModuleWorker } from '../lib/run-module-worker.mjs';
import { BEE_VERSION } from '../../packages/bee/lib/state.mjs';
import {
  buildSessionPreamble,
  onboardingLine,
  bypassBannerLines,
  handoffBlockLines,
  firstOpenGate,
} from '../../packages/bee/lib/inject.mjs';
import {
  buildCompactCapsule,
  appendCompactionRecord,
  nonAdoptingHandoffOutcome,
  readCompactionRecords,
} from '../../packages/bee/lib/compaction.mjs';
import { readIntent, resumeBlock } from '../../packages/bee/lib/intent.mjs';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const REPO_ROOT = path.join(__dirname, '..', '..');
const GOLDEN_FILE = path.join(REPO_ROOT, 'scripts', 'tests', 'fixtures', 'preamble-golden.txt');
const UPDATE_GOLDEN = process.argv.includes('--update-golden');
const REAL_LIB_DIR = path.join(REPO_ROOT, '.bee', 'bin', 'lib');
const SESSION_INIT_HOOK = path.join(REPO_ROOT, 'packages', 'bee', 'hooks', 'bee-session-init.mjs');

// Async rows (the through-the-hook section) are registered at module level but
// settle later; `check` returns their promise, so they are collected here and
// awaited before the summary — otherwise printSummaryAndExit would run while
// they are still in flight and report a green suite that never finished.
const pending = [];
const acheck = (name, fn) => {
  pending.push(check(name, fn));
};

// ─── fixtures ───────────────────────────────────────────────────────────────

const tempRoots = [];

function writeJson(file, obj) {
  fs.mkdirSync(path.dirname(file), { recursive: true });
  fs.writeFileSync(file, `${JSON.stringify(obj, null, 2)}\n`, 'utf8');
}

function makeRepo({
  phase = 'swarming',
  mode = 'standard',
  feature = 'demo',
  execution = true,
  shape = true,
  context = true,
  nextAction = '',
  onboarding = { schema_version: '1.0', bee_version: BEE_VERSION },
  config = null,
} = {}) {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), 'bee-capsule-'));
  tempRoots.push(root);
  fs.mkdirSync(path.join(root, '.bee', 'logs'), { recursive: true });
  writeJson(path.join(root, '.bee', 'state.json'), {
    schema_version: '1.0',
    phase,
    mode,
    feature,
    approved_gates: { context, shape, execution, review: false },
    workers: [],
    summary: '',
    next_action: nextAction,
  });
  if (onboarding) writeJson(path.join(root, '.bee', 'onboarding.json'), onboarding);
  if (config) writeJson(path.join(root, '.bee', 'config.json'), config);
  return root;
}

function addSession(root, id, extra = {}) {
  writeJson(path.join(root, '.bee', 'sessions', `${id}.json`), {
    id,
    started_at: '2026-01-01T00:00:00.000Z',
    last_heartbeat: '2026-01-01T00:00:00.000Z',
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
    created_at: '2026-01-01T00:00:00.000Z',
  });
}

function addCell(root, { id, feature = 'demo', status = 'open', deps = [], session = null, lane = 'small', verify = 'node -e "process.exit(0)"' }) {
  writeJson(path.join(root, '.bee', 'cells', `${id}.json`), {
    id,
    feature,
    title: `Cell ${id}`,
    lane,
    status,
    deps,
    action: 'Do the thing.',
    verify,
    trace: { worker: 'tester', claim_session: session, claimed_at: '2026-01-01T00:00:00.000Z' },
  });
}

function addAnchor(root, key = 'demo') {
  writeJson(path.join(root, '.bee', 'intent', `${key}.json`), {
    schema_version: '1.0',
    key,
    written_at: '2026-01-01T00:00:00.000Z',
    request: 'do the thing the user actually asked for',
    acceptance: 'the thing is done',
    next_action: null,
    feature: null,
    lane: null,
    cell: null,
    do_not_reverse: [],
    stop_conditions: [],
  });
}

function addHandoff(root, handoff) {
  writeJson(path.join(root, '.bee', 'HANDOFF.json'), handoff);
}

/** sha256 over every path + byte in a directory tree. */
function hashTree(dir) {
  const hash = crypto.createHash('sha256');
  const walk = (abs, rel) => {
    const entries = fs.readdirSync(abs, { withFileTypes: true }).sort((a, b) => a.name.localeCompare(b.name));
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

function lineIndex(text, predicate, label) {
  const index = text.split('\n').findIndex(predicate);
  assert(index >= 0, `the capsule must carry ${label} — got:\n${text}`);
  return index;
}

// ─── 1. the item order (D6 items 2-12), asserted positionally ───────────────

function fullyLoadedRepo() {
  const root = makeRepo({
    phase: 'swarming',
    nextAction: 'finish cz-5 and cap it',
    onboarding: null, // item 3: the onboarding-MISSING line
    config: {
      schema_version: '1.0',
      gate_bypass: 'total',
      commands: { setup: 'npm ci', verify: 'node scripts/run_verify.mjs' },
    },
    // The execution gate is REVOKED under a claimed cell: a second real
    // mismatch (so the block is proven to render more than one line) and the
    // only way a swarming session still owes a gate — item 8.
    execution: false,
  });
  const sid = 'sess-full';
  addSession(root, sid);
  // An UNCAPPED dependency: a real, anchor-independent mismatch (item 2).
  addCell(root, { id: 'k-0', status: 'open' });
  addCell(root, { id: 'k-1', status: 'claimed', session: sid, deps: ['k-0'], verify: 'node scripts/test_thing.mjs' });
  addHandoff(root, {
    kind: 'planned-next',
    phase: 'swarming',
    feature: 'demo',
    mode: 'standard',
    cells_in_flight: ['k-1'],
    next_action: 'start k-2',
    writer_session: 'sess-other',
  });
  // Two precompact records for (session, cell) — item 11 plus the D9 warning.
  appendCompactionRecord(root, { event: 'precompact', sessionId: sid });
  appendCompactionRecord(root, { event: 'precompact', sessionId: sid });
  return { root, sid };
}

check('the capsule renders D6 items 2-12 in order, asserted positionally', () => {
  const { root, sid } = fullyLoadedRepo();
  const capsule = buildCompactCapsule(root, {
    sessionId: sid,
    handoffOutcome: nonAdoptingHandoffOutcome(root),
  });

  const at = {
    mismatch: lineIndex(capsule, (l) => l.startsWith('⚠ STATE MISMATCH'), 'item 2 (the STATE MISMATCH line)'),
    onboarding: lineIndex(capsule, (l) => l.startsWith('- Onboarding: MISSING'), 'item 3 (the onboarding-MISSING line)'),
    handoff: lineIndex(capsule, (l) => l === '### HANDOFF present — present it and WAIT — never auto-resume', 'item 4 (the HANDOFF block)'),
    bypass: lineIndex(capsule, (l) => l.includes('GATE BYPASS'), 'item 5 (the gate-bypass banner)'),
    phase: lineIndex(capsule, (l) => /^- Phase: .+ \| Mode: .+ \| Feature: .+ \| Lane: /.test(l), 'item 6 (phase/mode/feature/lane)'),
    cell: lineIndex(capsule, (l) => l.startsWith('- Cell: k-1'), 'item 7 (the claimed cell)'),
    verify: lineIndex(capsule, (l) => l.startsWith('- Verify: '), "item 7's verify command"),
    deps: lineIndex(capsule, (l) => l.startsWith('- Deps: '), "item 7's dependency status"),
    gate: lineIndex(capsule, (l) => l.startsWith('- Gate pending: '), 'item 8 (the first open gate)'),
    next: lineIndex(capsule, (l) => l.startsWith('- Next action: '), 'item 9 (next_action)'),
    commands: lineIndex(capsule, (l) => l.startsWith('### Standard commands'), 'item 10 (the recorded commands)'),
    survival: lineIndex(capsule, (l) => l.startsWith('- Compactions survived: '), 'item 11 (the survival count)'),
    patterns: lineIndex(capsule, (l) => l.startsWith('- Critical patterns: '), 'item 12 (the critical-patterns pointer)'),
  };

  const order = [
    'mismatch', 'onboarding', 'handoff', 'bypass', 'phase', 'cell', 'verify', 'deps',
    'gate', 'next', 'commands', 'survival', 'patterns',
  ];
  for (let i = 1; i < order.length; i += 1) {
    assert(
      at[order[i - 1]] < at[order[i]],
      `D6 fixes the item order: "${order[i - 1]}" (line ${at[order[i - 1]]}) must precede "${order[i]}" (line ${at[order[i]]}) — got:\n${capsule}`,
    );
  }

  // item 7's dependency STATUS, not just the dep id.
  const depsLine = capsule.split('\n')[at.deps];
  assert(/k-0/.test(depsLine) && /open/.test(depsLine), `the dependency status is reported, not just the id — got "${depsLine}"`);
  // item 11's D9 warning fires at the second compaction of this unit.
  assert(/survived 2 compactions/.test(capsule), `the D9 advisory rides the survival count — got:\n${capsule}`);
  // item 12 is a POINTER, never the digest (D7).
  assert(!capsule.includes('### Critical patterns (digest)'), 'D7: the capsule carries a pointer, never the 10-line digest');
  // The startup-only sections stay out of the capsule (D6's whole point).
  for (const section of ['### Project map', '### Recent decisions', 'Knowledge context', 'Scribing debt']) {
    assert(!capsule.includes(section), `"${section}" is startup orientation and must never ride the capsule (D6)`);
  }
});

check('the capsule NEVER renders the intent anchor — the hook owns it (D19)', () => {
  const { root, sid } = fullyLoadedRepo();
  addAnchor(root, 'demo');
  const capsule = buildCompactCapsule(root, { sessionId: sid, handoffOutcome: null });
  assert(!capsule.includes('INTENT ANCHOR'), `D19: the hook prefixes the anchor; the capsule is the preamble replacement only — got:\n${capsule}`);
  assert(!capsule.includes('do the thing the user actually asked for'), 'the capsule must not re-render the anchor body');
});

check('the `- Phase:` label is preserved verbatim (the :2762 ordering check depends on it)', () => {
  const { root, sid } = fullyLoadedRepo();
  const capsule = buildCompactCapsule(root, { sessionId: sid, handoffOutcome: null });
  assert(capsule.includes('- Phase: '), 'the label is exactly "- Phase: " — renaming it makes indexOf return -1 and the hook-contract row pass or fail for the wrong reason');
});

// ─── 2. NO CAPSULE BYTE IS ANCHOR-CORRELATED (D19, cz-5 STEP 4b) ────────────
//
// The pair is the SAME repo rendered before and after the anchor is written —
// strictly stronger than two sibling fixtures, since it rules out any
// difference other than `.bee/intent/` (temp-dir names included).

function assertAnchorIndependent(root, sessionId, label) {
  const beeDir = path.join(root, '.bee');
  const before = buildCompactCapsule(root, { sessionId, handoffOutcome: null });
  const hashBefore = hashTree(beeDir);
  addAnchor(root, 'demo');
  const after = buildCompactCapsule(root, { sessionId, handoffOutcome: null });
  assert(
    before === after,
    `${label}: no capsule byte may vary with anchor presence (D19) — the with-anchor render differs from the no-anchor control.\n--- no anchor ---\n${before}\n--- with anchor ---\n${after}`,
  );
  assert(hashTree(beeDir) !== hashBefore, `${label}: the fixture must actually differ — the anchor write is the whole point of the pair`);
}

check('anchor-independence, CLEAN sweep: the paired renders are byte-equal', () => {
  const root = makeRepo();
  const sid = 'sess-clean';
  addSession(root, sid);
  addCell(root, { id: 'k-0', status: 'capped' });
  addCell(root, { id: 'k-1', status: 'claimed', session: sid, deps: ['k-0'] });
  assertAnchorIndependent(root, sid, 'clean sweep');
});

check('anchor-independence, FAILING sweep: the STATE MISMATCH line never names the anchor check', () => {
  const root = makeRepo();
  const sid = 'sess-dirty';
  addSession(root, sid);
  // A real, anchor-independent mismatch: the claimed cell's dep is uncapped.
  addCell(root, { id: 'k-0', status: 'open' });
  addCell(root, { id: 'k-1', status: 'claimed', session: sid, deps: ['k-0'] });
  const noAnchor = buildCompactCapsule(root, { sessionId: sid, handoffOutcome: null });
  assert(noAnchor.includes('⚠ STATE MISMATCH'), 'the failing pair must actually render the mismatch line — otherwise it proves nothing');
  assert(
    !/anchor/i.test(noAnchor.split('\n').filter((l) => l.startsWith('- ') || l.startsWith('⚠')).join('\n')),
    `the mismatch block must never name the anchor check — compactCheck reports "an anchor exists" and rendering it makes the capsule anchor-correlated by construction. Got:\n${noAnchor}`,
  );
  assertAnchorIndependent(root, sid, 'failing sweep');
});

// ─── 3. handoffOutcome is mandatory (D26/D27) ──────────────────────────────

check('a planned-next handoff renders `- Adoption not applied:` when the outcome is passed (D27)', () => {
  const root = makeRepo();
  const sid = 'sess-handoff';
  addSession(root, sid);
  addHandoff(root, {
    kind: 'planned-next',
    phase: 'swarming',
    feature: 'demo',
    mode: 'standard',
    cells_in_flight: ['k-1'],
    next_action: 'start k-2',
    writer_session: 'sess-other',
  });
  const outcome = nonAdoptingHandoffOutcome(root);
  assert(outcome && outcome.ok === false && outcome.code === 'WRONG_SOURCE', `a planned-next handoff on a compact start is a WRONG_SOURCE refusal — got ${JSON.stringify(outcome)}`);

  const withOutcome = buildCompactCapsule(root, { sessionId: sid, handoffOutcome: outcome });
  assert(
    /^- Adoption not applied: .+/m.test(withOutcome),
    `D27: the capsule must explain WHY adoption was refused — got:\n${withOutcome}`,
  );
  assert(withOutcome.includes('never auto-adopts'), 'the reason text is the hook\'s own, not a paraphrase');
  assert(withOutcome.includes('### HANDOFF present — present it and WAIT — never auto-resume'), 'D6 item 4: the wait-and-never-auto-resume heading is verbatim');

  // The parameter is LOAD-BEARING: drop it and the explanation disappears.
  // This is the exact silent loss no existing row could see.
  const withoutOutcome = buildCompactCapsule(root, { sessionId: sid });
  assert(
    !withoutOutcome.includes('- Adoption not applied:'),
    'without the outcome there is nothing to explain — if this line appears anyway the capsule is fabricating it',
  );
  assert(
    withoutOutcome !== withOutcome,
    'handoffOutcome must change the rendering; if it does not, the parameter is decorative and D27 is unmet',
  );

  // A pause handoff has no adoption story at all.
  addHandoff(root, { kind: 'pause', phase: 'swarming', feature: 'demo', mode: 'standard', next_action: 'resume k-1' });
  const pause = buildCompactCapsule(root, { sessionId: sid, handoffOutcome: nonAdoptingHandoffOutcome(root) });
  assert(nonAdoptingHandoffOutcome(root) === null, 'a pause handoff never produces an adoption outcome');
  assert(!pause.includes('- Adoption not applied:'), 'a pause handoff never carries an adoption-refused line');
  assert(pause.includes('- Saved next action: resume k-1'), 'the pause block still renders verbatim');
});

// ─── 4. D15 — the bare repo, and the lane-bound session ────────────────────

check('D15: a bare repo renders a capsule with no warning and no mismatch line', () => {
  const bare = makeRepo({ phase: 'idle', mode: null, feature: null, execution: false, shape: false, context: false });
  for (const sessionId of [null, 'sess-bare']) {
    if (sessionId) addSession(bare, sessionId);
    const capsule = buildCompactCapsule(bare, { sessionId, handoffOutcome: null });
    assert(!capsule.includes('⚠ STATE MISMATCH'), `no claimed cell, no anchor, no records — nothing mismatches (D15). sessionId=${sessionId}, got:\n${capsule}`);
    assert(!capsule.includes('Compactions survived'), `no prior compaction records means no survival line (D15). sessionId=${sessionId}`);
    assert(!capsule.includes('survived'), `no D9 warning on a bare repo (D15). sessionId=${sessionId}`);
    assert(capsule.includes('- Phase: idle'), `the capsule still orients: it names the phase. Got:\n${capsule}`);
    assert(capsule.includes('- Cell: none claimed'), 'with no claimed cell the capsule says so rather than omitting item 7 silently');
    assert(capsule.includes('- Critical patterns: '), 'D7: the pointer is never dropped, not even on a bare repo');
  }
});

check('a lane-bound session shows its OWN lane, never the default state.json', () => {
  const root = makeRepo({ phase: 'swarming', feature: 'default-feature', mode: 'standard' });
  addLane(root, 'lane-feat', { phase: 'executing', mode: 'small' });
  addSession(root, 'sess-lane', { lane: 'lane-feat' });
  const capsule = buildCompactCapsule(root, { sessionId: 'sess-lane', handoffOutcome: null });
  assert(
    capsule.includes('- Phase: executing | Mode: small | Feature: lane-feat | Lane: lane-feat'),
    `the bound lane's own record drives phase/mode/feature, and the lane is named (D6 item 6) — got:\n${capsule}`,
  );
  assert(!capsule.includes('default-feature'), 'a lane-bound session must never be shown the default pipeline');

  const unbound = buildCompactCapsule(root, { sessionId: null, handoffOutcome: null });
  assert(unbound.includes('| Lane: none'), `an unbound session reports no lane — got:\n${unbound}`);
});

check('the STATE MISMATCH block names each failed check and the rule that settles it (D13)', () => {
  const root = makeRepo({ execution: false });
  const sid = 'sess-mismatch';
  addSession(root, sid);
  addCell(root, { id: 'k-1', status: 'claimed', session: sid });
  const capsule = buildCompactCapsule(root, { sessionId: sid, handoffOutcome: null });
  assert(capsule.includes('⚠ STATE MISMATCH'), `a revoked execution gate under a claimed cell is a mismatch — got:\n${capsule}`);
  assert(
    capsule.includes('disk state overrides conversational recollection'),
    'D13: the line states the rule, so a just-compacted model knows which side wins',
  );
  assert(/execution_gate/.test(capsule), 'each failed check is named');
  assert(/GATE_REVOKED/.test(capsule), "the typed code is surfaced, never flattened into prose");
});

// ─── 5. the three extracted renderers (cz-5 STEP 1) ────────────────────────

check('the three renderers are exported and render the preamble bytes verbatim', () => {
  assert(
    onboardingLine(null) === '- Onboarding: MISSING — run bee-hive onboarding before anything else.',
    'the onboarding-MISSING line is byte-identical to inject.mjs:319',
  );
  assert(
    onboardingLine({ bee_version: '0.0.1' }) === `- Onboarding: installed at bee 0.0.1 but plugin is ${BEE_VERSION} — re-run onboarding to refresh vendored helpers.`,
    'the drift arm is byte-identical to inject.mjs:321-323',
  );
  assert(
    onboardingLine({ bee_version: BEE_VERSION }) === `- Onboarding: ok (bee ${BEE_VERSION})`,
    'the ok arm is byte-identical to inject.mjs:325',
  );

  assert(bypassBannerLines('off').length === 0, "an inactive bypass renders no line at all");
  assert(bypassBannerLines('normal').length === 1, 'normal is a one-line banner');
  assert(bypassBannerLines('total').length === 2, 'full/total add the "does NOT stop" paragraph');
  assert(bypassBannerLines('total')[0].startsWith('- ⚡'), 'the banner is a preamble list item');

  const handoff = { kind: 'planned-next', phase: 'p', feature: 'f', mode: 'm', cells_in_flight: ['c-1'], next_action: 'n' };
  const block = handoffBlockLines(handoff, { ok: false, reason: 'because' });
  assert(block[0] === '### HANDOFF present — present it and WAIT — never auto-resume', 'the heading is verbatim (D6 item 4)');
  assert(block[0] !== '', 'BLANK-LINE OWNERSHIP: the separator belongs to the CALLER, never to the block');
  assert(block[block.length - 1] === '- Adoption not applied: because', 'the D26 reason line is part of the block');
  assert(handoffBlockLines(null).length === 0, 'no handoff, no block');

  assert(firstOpenGate({ phase: 'swarming', approved_gates: { context: true, shape: false } }) === 'shape', 'the first open gate is the first unapproved one');
  assert(firstOpenGate({ phase: 'idle', approved_gates: {} }) === null, 'a terminal phase owes no gate');
  assert(
    firstOpenGate({ phase: 'swarming', approved_gates: { context: true, shape: true, execution: true } }) === null,
    'review is not a pending gate outside a review session',
  );
});

// ─── 6. the committed golden (D8) ──────────────────────────────────────────
//
// Every case below is a full buildSessionPreamble render. Between them they
// exercise all three onboarding arms, both bypass shapes, and all three
// handoff arms — including the planned-next-refused one D26 names as the case
// no existing row covers.

function normalizeVersion(text) {
  // The version literal is the ONE legitimately moving byte-run: it appears in
  // the `## bee v…` heading (inject.mjs:317) and in two of the three
  // onboarding arms. Normalizing it here is what keeps the next release bump
  // from reddening a golden that is otherwise still true.
  return text.split(BEE_VERSION).join('<BEE_VERSION>');
}

function goldenCases() {
  const cases = [];
  const withDecisions = (root) => {
    fs.mkdirSync(path.join(root, 'docs', 'history', 'learnings'), { recursive: true });
    fs.writeFileSync(
      path.join(root, 'docs', 'history', 'learnings', 'critical-patterns.md'),
      '<!-- a comment line, filtered -->\nPAT01: never build on red.\n\nPAT02: the trace is the record.\n',
      'utf8',
    );
    fs.writeFileSync(
      path.join(root, '.bee', 'decisions.jsonl'),
      `${JSON.stringify({ id: 'd-1', type: 'decide', date: '2026-01-02', decision: 'The golden is committed, never reconstructed', rationale: 'r', scope: 'repo' })}\n`,
      'utf8',
    );
  };

  // A — the ordinary shape: onboarding ok, commands recorded, no handoff.
  {
    const root = makeRepo({
      nextAction: 'cap cz-5',
      config: { schema_version: '1.0', commands: { setup: 'npm ci', test: 'npm test', verify: 'node scripts/run_verify.mjs' } },
    });
    withDecisions(root);
    cases.push({ name: 'onboarding-ok', text: buildSessionPreamble(root, { sessionId: null, handoffOutcome: null }) });
  }

  // B — onboarding MISSING + the two-line total bypass banner.
  {
    const root = makeRepo({ onboarding: null, config: { schema_version: '1.0', gate_bypass: 'total' } });
    withDecisions(root);
    cases.push({ name: 'onboarding-missing-bypass-total', text: buildSessionPreamble(root, { sessionId: null, handoffOutcome: null }) });
  }

  // C — the version-drift arm + the one-line normal banner.
  {
    const root = makeRepo({
      onboarding: { schema_version: '1.0', bee_version: '0.0.1' },
      config: { schema_version: '1.0', gate_bypass: 'normal' },
      phase: 'reviewing',
    });
    withDecisions(root);
    cases.push({ name: 'onboarding-drift-bypass-normal', text: buildSessionPreamble(root, { sessionId: null, handoffOutcome: null }) });
  }

  // D — a pause handoff, no adoption attempted (handoffOutcome null).
  {
    const root = makeRepo();
    withDecisions(root);
    addHandoff(root, { kind: 'pause', phase: 'swarming', feature: 'demo', mode: 'standard', cells_in_flight: ['k-1', 'k-2'], next_action: 'resume k-1' });
    cases.push({ name: 'handoff-pause', text: buildSessionPreamble(root, { sessionId: null, handoffOutcome: null }) });
  }

  // E — a planned-next handoff whose adoption was REFUSED (D26): the one case
  // no existing row covers, and the reason line is the whole point.
  {
    const root = makeRepo();
    withDecisions(root);
    addHandoff(root, { kind: 'planned-next', phase: 'swarming', feature: 'demo', mode: 'standard', cells_in_flight: ['k-1'], next_action: 'start k-2', writer_session: 'sess-other' });
    cases.push({
      name: 'handoff-planned-next-refused',
      text: buildSessionPreamble(root, {
        sessionId: 'sess-golden',
        handoffOutcome: { ok: false, code: 'WRONG_SOURCE', reason: 'a planned-next handoff never auto-adopts on source "compact" — only "clear"/"startup" qualify (D1).' },
      }),
    });
  }

  // F — a planned-next handoff ADOPTED: the start-now arm.
  {
    const root = makeRepo();
    withDecisions(root);
    addCell(root, { id: 'k-9', status: 'claimed', verify: 'node scripts/test_thing.mjs' });
    cases.push({
      name: 'handoff-adopted',
      text: buildSessionPreamble(root, { sessionId: 'sess-golden', handoffOutcome: { ok: true, next_cell: 'k-9' } }),
    });
  }

  // G — a lane-bound session with another lane live.
  {
    const root = makeRepo({ feature: 'default-feature' });
    withDecisions(root);
    addLane(root, 'lane-feat', { phase: 'executing', mode: 'small' });
    addLane(root, 'other-lane', { phase: 'planning', mode: 'small' });
    addSession(root, 'sess-golden-lane', { lane: 'lane-feat' });
    cases.push({ name: 'lane-bound', text: buildSessionPreamble(root, { sessionId: 'sess-golden-lane', handoffOutcome: null }) });
  }

  return cases.map((entry) => ({ name: entry.name, text: normalizeVersion(entry.text) }));
}

const GOLDEN_DELIMITER = (name) => `===== CASE ${name} =====`;

function serializeGolden(cases) {
  return cases.map((entry) => `${GOLDEN_DELIMITER(entry.name)}\n${entry.text}\n`).join('');
}

function parseGolden(raw) {
  const parsed = new Map();
  const lines = raw.split('\n');
  if (lines.length > 0 && lines[lines.length - 1] === '') lines.pop();
  let current = null;
  let buf = [];
  for (const line of lines) {
    const match = /^===== CASE (.+) =====$/.exec(line);
    if (match) {
      if (current !== null) parsed.set(current, buf.join('\n'));
      current = match[1];
      buf = [];
      continue;
    }
    buf.push(line);
  }
  if (current !== null) parsed.set(current, buf.join('\n'));
  return parsed;
}

if (UPDATE_GOLDEN) {
  fs.mkdirSync(path.dirname(GOLDEN_FILE), { recursive: true });
  fs.writeFileSync(GOLDEN_FILE, serializeGolden(goldenCases()), 'utf8');
  console.log(`wrote ${path.relative(REPO_ROOT, GOLDEN_FILE)}`);
  for (const root of tempRoots) fs.rmSync(root, { recursive: true, force: true });
  process.exit(0);
}

check('buildSessionPreamble matches the COMMITTED golden byte for byte (D8)', () => {
  assert(
    fs.existsSync(GOLDEN_FILE),
    `the golden must be a committed fixture at scripts/tests/fixtures/preamble-golden.txt — never a "git show HEAD:" reconstruction, which passes at cz-5's cap and becomes a tautology for every later cell. Regenerate with: node scripts/tests/test_compact_capsule.mjs --update-golden`,
  );
  const expected = parseGolden(fs.readFileSync(GOLDEN_FILE, 'utf8'));
  const actual = goldenCases();
  assert(actual.length === expected.size, `the golden holds ${expected.size} case(s), the suite renders ${actual.length} — regenerate it deliberately`);
  for (const entry of actual) {
    assert(expected.has(entry.name), `the golden has no case "${entry.name}"`);
    const want = expected.get(entry.name);
    assert(
      entry.text === want,
      `buildSessionPreamble drifted for case "${entry.name}" (D8: startup/clear/resume stay byte-identical).\n--- golden ---\n${want}\n--- actual ---\n${entry.text}`,
    );
  }
  // The golden must actually bite: it carries the lines the three extracted
  // renderers own, so a broken extraction cannot pass it silently.
  const missing = parseGolden(fs.readFileSync(GOLDEN_FILE, 'utf8'));
  assert(missing.get('onboarding-missing-bypass-total').includes('- Onboarding: MISSING —'), 'the golden covers the onboarding-MISSING renderer');
  assert(missing.get('onboarding-missing-bypass-total').includes('GATE BYPASS: TOTAL AUTOPILOT'), 'the golden covers the bypass-banner renderer');
  assert(missing.get('handoff-pause').includes('### HANDOFF present — present it and WAIT — never auto-resume'), 'the golden covers the HANDOFF renderer');
  assert(missing.get('handoff-planned-next-refused').includes('- Adoption not applied: '), 'the golden covers D26\'s adoption-refused line');
  assert(missing.get('handoff-adopted').includes('### PLANNED-NEXT ADOPTED'), 'the golden covers the start-now arm');
});

// ─── 7. THROUGH THE HOOK (cz-6, D5/D6/D8/D19/D27) ──────────────────────────
//
// Everything above renders the builders in-process. These rows execute the
// real hooks/bee-session-init.mjs against a vendored temp repo, which is the
// only place three obligations are observable at all:
//
//   * THE ANCHOR MUST APPEAR EXACTLY ONCE, ASSERTED BY COUNT (D19). The hook
//     keeps prefixing it and the capsule never renders it — but a capsule that
//     ALSO rendered it would still satisfy `startsWith("## INTENT ANCHOR")`
//     and would still satisfy the existing additivity row, which compares two
//     compact renders to each other. Only a count catches the double.
//   * handoffOutcome MUST REACH THE CAPSULE FROM THE CALL SITE (D27). cz-5
//     proved the renderer honours the parameter; nothing yet proved the hook
//     passes it. A hook that drops it is green everywhere else in this feature
//     and in the full verify, while a compacted session holding a planned-next
//     handoff silently loses the `- Adoption not applied:` line.
//   * `resume` STAYS THE FULL PREAMBLE (D8). The capsule branch is keyed on
//     one source value; the negative control is the sibling source that also
//     leads with the anchor, so a branch written on ANCHOR_LEAD_SOURCES rather
//     than on `compact` reds here and nowhere else.

/** Vendor the real .bee/bin/lib into a fixture — the hook loads lib from root. */
function vendorLib(root) {
  const libDir = path.join(root, '.bee', 'bin', 'lib');
  fs.mkdirSync(libDir, { recursive: true });
  for (const name of fs.readdirSync(REAL_LIB_DIR)) {
    if (name.endsWith('.mjs')) fs.copyFileSync(path.join(REAL_LIB_DIR, name), path.join(libDir, name));
  }
  return root;
}

async function runSessionStart(root, { source, sessionId }) {
  const result = await runModuleWorker(SESSION_INIT_HOOK, {
    input: JSON.stringify({
      hook_event_name: 'SessionStart',
      source,
      session_id: sessionId,
      cwd: root,
    }),
    cwd: root,
    timeout: 30_000,
  });
  assert(
    result.status === 0,
    `SessionStart is fail-open and must exit 0 — got status=${result.status}\nstderr: ${result.stderr}`,
  );
  return result.stdout || '';
}

function occurrences(haystack, needle) {
  return haystack.split(needle).length - 1;
}

/** Split `anchor + "\n\n" + body` back apart, asserting the shape as it goes. */
function splitAnchorLed(out, root, sessionId) {
  const anchor = resumeBlock(readIntent(root, { sessionId }));
  assert(anchor, 'the fixture must hold a readable anchor — otherwise this row proves nothing');
  assert(
    out.startsWith(`${anchor}\n\n`),
    `the emitted block leads with the anchor and one blank line (D19) — got:\n${out}`,
  );
  return out.slice(anchor.length + 2);
}

acheck('THROUGH THE HOOK: source=compact emits anchor-then-CAPSULE, anchor exactly ONCE by count (D6/D19)', async () => {
  const root = vendorLib(
    makeRepo({
      nextAction: 'finish cz-6 and cap it',
      config: { schema_version: '1.0', commands: { setup: 'npm ci', verify: 'node scripts/run_verify.mjs' } },
    }),
  );
  addAnchor(root, 'demo');
  const sid = 'sess-hook-compact';
  const out = await runSessionStart(root, { source: 'compact', sessionId: sid });

  assert(
    occurrences(out, '## INTENT ANCHOR') === 1,
    `the anchor must appear EXACTLY ONCE — the hook owns it and the capsule must not re-render it (D19). ` +
      `Counted ${occurrences(out, '## INTENT ANCHOR')} in:\n${out}`,
  );
  assert(
    occurrences(out, 'do the thing the user actually asked for') === 1,
    'the verbatim objective is printed once, not twice — a doubled anchor still passes startsWith()',
  );

  const body = splitAnchorLed(out, root, sid);
  assert(
    body === buildCompactCapsule(root, { sessionId: sid, handoffOutcome: null }),
    `D6: the body below the anchor is the capsule, byte for byte — got:\n${body}`,
  );
  assert(
    body.startsWith(`## bee v${BEE_VERSION} — compact capsule (source=compact)`),
    `the compact branch must emit the capsule, not the startup preamble — got:\n${body}`,
  );
  assert(!body.includes('- Gates: '), 'the full preamble\'s gates line is startup orientation the capsule drops (D6)');
  assert(body.includes('- Critical patterns: '), 'D7: the capsule carries the pointer');

  // D5: source=compact is the `resume` EVENT — one record, and only one.
  const records = readCompactionRecords(root);
  assert(records.length === 1, `exactly one compaction record is appended per compact start (D5) — got ${records.length}`);
  assert(records[0].event === 'resume', `the SessionStart(compact) record's event is "resume" (D5) — got "${records[0].event}"`);
  assert(records[0].session === sid, 'the record names the acting session');
  assert(records[0].compact_index === 0, 'D5: a resume carries the PLAIN prior precompact count, never +1');
});

acheck('THROUGH THE HOOK: compact + planned-next renders `- Adoption not applied:` — handoffOutcome reached the capsule (D27)', async () => {
  const root = vendorLib(makeRepo());
  addHandoff(root, {
    kind: 'planned-next',
    phase: 'swarming',
    feature: 'demo',
    mode: 'standard',
    cells_in_flight: ['k-1'],
    next_action: 'start k-2',
    writer_session: 'sess-other',
  });
  const sid = 'sess-hook-handoff';
  const out = await runSessionStart(root, { source: 'compact', sessionId: sid });

  // The line alone proves nothing: buildSessionPreamble renders it too (D26),
  // so an UN-WIRED hook would pass a bare "the line is there" assertion. The
  // discriminator is that the line rides the CAPSULE.
  assert(
    out.startsWith(`## bee v${BEE_VERSION} — compact capsule (source=compact)`),
    `the compact branch must emit the capsule — got:\n${out}`,
  );
  assert(
    out.includes('### HANDOFF present — present it and WAIT — never auto-resume'),
    `D6 item 4: the wait heading is verbatim — got:\n${out}`,
  );
  assert(
    /^- Adoption not applied: .+/m.test(out),
    `D27: the hook computes handoffOutcome and MUST pass it to the capsule — without it a compacted session ` +
      `is told to wait and never told why. Got:\n${out}`,
  );
  // Pin the PARAMETER, not just the resulting bytes: the emitted capsule is
  // the one built WITH the outcome, and is provably not the one built without.
  const withOutcome = buildCompactCapsule(root, { sessionId: sid, handoffOutcome: nonAdoptingHandoffOutcome(root) });
  const withoutOutcome = buildCompactCapsule(root, { sessionId: sid });
  assert(withOutcome !== withoutOutcome, 'the fixture must make the parameter observable — otherwise this row proves nothing');
  assert(out === withOutcome, `the hook must render the WITH-outcome capsule (D27) — got:\n${out}`);
  assert(out !== withoutOutcome, 'the hook dropped handoffOutcome at the capsule call site (D27)');
  assert(
    out.includes('never auto-adopts on source "compact"'),
    'the reason names the actual source, so it is the hook\'s own computed outcome and not a capsule-side guess',
  );
  assert(!out.includes('PLANNED-NEXT ADOPTED'), 'a compact start never adopts — ADOPT_SOURCES is untouched');
  assert(fs.existsSync(path.join(root, '.bee', 'HANDOFF.json')), 'the handoff stays on disk, unread and unadopted');
});

acheck('THROUGH THE HOOK: source=resume still emits anchor-then-FULL-preamble, byte-identical (D8)', async () => {
  const root = vendorLib(
    makeRepo({
      nextAction: 'keep going',
      config: { schema_version: '1.0', commands: { verify: 'node scripts/run_verify.mjs' } },
    }),
  );
  addAnchor(root, 'demo');
  const sid = 'sess-hook-resume';
  const out = await runSessionStart(root, { source: 'resume', sessionId: sid });

  assert(
    occurrences(out, '## INTENT ANCHOR') === 1,
    `the anchor still leads a resume exactly once — got ${occurrences(out, '## INTENT ANCHOR')} in:\n${out}`,
  );
  const body = splitAnchorLed(out, root, sid);
  assert(
    body === buildSessionPreamble(root, { sessionId: sid, handoffOutcome: null }),
    `D8: "resume" keeps today's full preamble, byte-identical — got:\n${body}`,
  );
  assert(body.includes('- Gates: '), 'the resume body is the full preamble, gates line included');
  assert(
    !body.includes('compact capsule (source=compact)'),
    'ANCHOR_LEAD_SOURCES is not the capsule predicate — only "compact" trims (D8)',
  );
  assert(
    readCompactionRecords(root).length === 0,
    'D5: only SessionStart(source=compact) writes a `resume` record; a plain resume writes nothing',
  );
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

await Promise.all(pending);

printSummaryAndExit();
