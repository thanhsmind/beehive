#!/usr/bin/env node
// test_intent.mjs — lib/intent.mjs contract tests (intent-anchor ia-1,
// CONTEXT.md D1-D6): the store's round-trip and immutability, D2's
// no-feature anchoring, D5's fail-open silence, both renderers, and D6 — the
// ≥2-compaction simulation that is the whole justification for the feature.
//
// Same PASS/FAIL/exit-1 contract as every other suite here — see
// scripts/lib/test-fixture.mjs.

import fs from 'node:fs';
import path from 'node:path';
import {
  makeTempRepo,
  check,
  assert,
  assertThrows,
  printSummaryAndExit,
} from '../../../scripts/lib/test-fixture.mjs';
import { writeJsonAtomic } from '../lib/fsutil.mjs';
import { defaultState, writeState } from '../lib/state.mjs';
import {
  writeIntent,
  readIntent,
  advanceIntent,
  clearIntent,
  locateIntentKey,
  intentKeyCandidates,
  intentPath,
  activeFeature,
  precompactBlock,
  resumeBlock,
  sanitizeIntentKey,
  DEFAULT_INTENT_KEY,
  INTENT_PRECOMPACT_HEADER,
  INTENT_PRECOMPACT_FOOTER,
  INTENT_RESUME_HEADER,
} from '../lib/intent.mjs';

const root = makeTempRepo();

// The one string every assertion below is really about: the user's own words.
// Deliberately long and specific, with punctuation and an embedded newline, so
// "verbatim" means something stronger than "a substring survived".
const VERBATIM_REQUEST =
  'Make the /orders endpoint idempotent under retries — the same Idempotency-Key must never\n' +
  'create a second order, and please do NOT change the existing response shape.';
const ACCEPTANCE =
  'Replaying an identical POST /orders with the same Idempotency-Key returns the first order and creates no second row; the response body is byte-identical to today.';

// ─── D2: an anchor is writable with NO active feature ──────────────────────

await check('D2 — an anchor is writable with no active feature (idle state keys on the default)', async () => {
  writeState(root, { ...defaultState(), phase: 'idle', feature: null });
  assert(activeFeature(root) === null, 'an idle repo has no active feature');
  const anchor = writeIntent(root, { request: VERBATIM_REQUEST, acceptance: ACCEPTANCE });
  assert(anchor.key === DEFAULT_INTENT_KEY, `expected the default key, got "${anchor.key}"`);
  assert(fs.existsSync(intentPath(root, DEFAULT_INTENT_KEY)), 'the anchor file must exist on disk');
  const read = readIntent(root);
  assert(read !== null, 'the anchor must read back with no feature in play');
  assert(read.request === VERBATIM_REQUEST, 'request must round-trip byte-for-byte');
});

await check('D2 — a session id anchors featureless work when one is available', async () => {
  const sessionRoot = makeTempRepo();
  writeState(sessionRoot, { ...defaultState(), phase: 'idle', feature: null });
  const anchor = writeIntent(
    sessionRoot,
    { request: VERBATIM_REQUEST, acceptance: ACCEPTANCE },
    { sessionId: 'sess-abc-123' },
  );
  assert(anchor.key === 'sess-abc-123', `expected the session key, got "${anchor.key}"`);
  // A reader with the session id finds it; a reader without one falls through
  // the candidate list and finds nothing — so a hook and the CLI agree.
  assert(readIntent(sessionRoot, { sessionId: 'sess-abc-123' }) !== null, 'the session-keyed anchor must be readable');
  assert(readIntent(sessionRoot) === null, 'another session must not pick up this session-keyed anchor');
  fs.rmSync(sessionRoot, { recursive: true, force: true });
});

await check('a stale feature on a TERMINAL phase never counts as active (the phase decides, not the field)', async () => {
  const staleRoot = makeTempRepo();
  writeState(staleRoot, { ...defaultState(), phase: 'compounding-complete', feature: 'last-thing' });
  assert(activeFeature(staleRoot) === null, 'compounding-complete leaves no active feature');
  writeState(staleRoot, { ...defaultState(), phase: 'swarming', feature: 'live-thing' });
  assert(activeFeature(staleRoot) === 'live-thing', 'a live phase resolves its feature');
  fs.rmSync(staleRoot, { recursive: true, force: true });
});

// ─── D1: key resolution, round trip, immutability ──────────────────────────

await check('D1 — an active feature keys the anchor; the CLI and a hook resolve the same file', async () => {
  const featureRoot = makeTempRepo();
  writeState(featureRoot, { ...defaultState(), phase: 'swarming', feature: 'orders-idempotency' });
  const anchor = writeIntent(featureRoot, { request: VERBATIM_REQUEST, acceptance: ACCEPTANCE });
  assert(anchor.key === 'orders-idempotency', `expected the feature key, got "${anchor.key}"`);
  assert(anchor.feature === 'orders-idempotency', 'the feature field is filled from state when not passed');
  const candidates = intentKeyCandidates(featureRoot, { sessionId: 'sess-9' });
  assert(
    JSON.stringify(candidates) === JSON.stringify(['orders-idempotency', 'sess-9', DEFAULT_INTENT_KEY]),
    `unexpected candidate order: ${JSON.stringify(candidates)}`,
  );
  // A hook (session id in hand) still finds the feature-keyed anchor.
  assert(readIntent(featureRoot, { sessionId: 'sess-9' }).key === 'orders-idempotency', 'the hook path resolves the same anchor');
  fs.rmSync(featureRoot, { recursive: true, force: true });
});

await check('D1 — the full field shape round-trips, request byte-for-byte', async () => {
  const shapeRoot = makeTempRepo();
  const anchor = writeIntent(shapeRoot, {
    request: VERBATIM_REQUEST,
    acceptance: ACCEPTANCE,
    next_action: 'write the dedupe index migration',
    feature: 'orders-idempotency',
    lane: 'standard',
    cell: 'oi-2',
    do_not_reverse: 'the response shape stays byte-identical, Idempotency-Key stays the dedupe key',
    stop_conditions: 'a schema change would need a backfill, any auth surface is touched',
  });
  assert(anchor.request === VERBATIM_REQUEST, 'request stored verbatim');
  assert(anchor.acceptance === ACCEPTANCE, 'acceptance stored verbatim');
  assert(anchor.next_action === 'write the dedupe index migration', 'next_action stored');
  assert(anchor.lane === 'standard' && anchor.cell === 'oi-2', 'lane/cell stored');
  assert(anchor.do_not_reverse.length === 2, `expected 2 do_not_reverse entries, got ${JSON.stringify(anchor.do_not_reverse)}`);
  assert(anchor.stop_conditions.length === 2, `expected 2 stop_conditions, got ${JSON.stringify(anchor.stop_conditions)}`);
  const onDisk = JSON.parse(fs.readFileSync(intentPath(shapeRoot, anchor.key), 'utf8'));
  assert(onDisk.request === VERBATIM_REQUEST, 'the file on disk holds the verbatim request, newline included');
  fs.rmSync(shapeRoot, { recursive: true, force: true });
});

await check('D1 — advance() moves next_action ONLY; request and acceptance cannot be mutated', async () => {
  const advRoot = makeTempRepo();
  const before = writeIntent(advRoot, {
    request: VERBATIM_REQUEST,
    acceptance: ACCEPTANCE,
    next_action: 'step one',
  });
  const after = advanceIntent(advRoot, 'step two');
  assert(after.next_action === 'step two', 'next_action advanced');
  assert(after.request === before.request, 'request unchanged by advance');
  assert(after.acceptance === before.acceptance, 'acceptance unchanged by advance');
  assert(typeof after.advanced_at === 'string' && after.advanced_at, 'advance stamps advanced_at');
  // Structural proof, not just behavioral: advanceIntent takes exactly the
  // next action — there is no parameter through which a new request or
  // acceptance could ever reach the stored record.
  assert(advanceIntent.length <= 3, `advanceIntent must take (root, nextAction, options), got arity ${advanceIntent.length}`);
  const reread = readIntent(advRoot);
  assert(reread.request === VERBATIM_REQUEST, 'the re-read anchor still holds the original verbatim request');
  fs.rmSync(advRoot, { recursive: true, force: true });
});

await check('D1 — advance() on a repo with no anchor returns null rather than inventing one', async () => {
  const emptyRoot = makeTempRepo();
  assert(advanceIntent(emptyRoot, 'anything') === null, 'advance with no anchor must return null');
  fs.rmSync(emptyRoot, { recursive: true, force: true });
});

await check('D1 — re-setting the SAME request is idempotent; a DIFFERENT one refuses without --force', async () => {
  const immRoot = makeTempRepo();
  writeIntent(immRoot, { request: VERBATIM_REQUEST, acceptance: ACCEPTANCE });
  // idempotent
  const again = writeIntent(immRoot, { request: VERBATIM_REQUEST, acceptance: ACCEPTANCE });
  assert(again.request === VERBATIM_REQUEST, 're-setting the same objective is allowed');
  assertThrows(
    () => writeIntent(immRoot, { request: 'something else entirely', acceptance: ACCEPTANCE }),
    'immutable once set',
    'a different request must refuse',
  );
  assertThrows(
    () => writeIntent(immRoot, { request: VERBATIM_REQUEST, acceptance: 'different acceptance' }),
    'immutable once set',
    'different acceptance must refuse',
  );
  assert(readIntent(immRoot).request === VERBATIM_REQUEST, 'the refused writes changed nothing');
  const forced = writeIntent(
    immRoot,
    { request: 'a genuinely new objective', acceptance: 'the new objective is met' },
    { force: true },
  );
  assert(forced.request === 'a genuinely new objective', '--force replaces the objective deliberately');
  fs.rmSync(immRoot, { recursive: true, force: true });
});

await check('writeIntent refuses a missing/blank request or acceptance', async () => {
  const badRoot = makeTempRepo();
  assertThrows(() => writeIntent(badRoot, { acceptance: ACCEPTANCE }), 'request', 'missing request refuses');
  assertThrows(() => writeIntent(badRoot, { request: '   ', acceptance: ACCEPTANCE }), 'request', 'blank request refuses');
  assertThrows(() => writeIntent(badRoot, { request: VERBATIM_REQUEST }), 'acceptance', 'missing acceptance refuses');
  fs.rmSync(badRoot, { recursive: true, force: true });
});

await check('clear() removes the anchor and is idempotent', async () => {
  const clrRoot = makeTempRepo();
  writeIntent(clrRoot, { request: VERBATIM_REQUEST, acceptance: ACCEPTANCE });
  assert(locateIntentKey(clrRoot) === DEFAULT_INTENT_KEY, 'the anchor is located before clearing');
  const first = clearIntent(clrRoot);
  assert(first.cleared === true, 'the first clear removes it');
  assert(readIntent(clrRoot) === null, 'nothing reads back after a clear');
  const second = clearIntent(clrRoot);
  assert(second.cleared === false, 'a second clear is a no-op, never an error');
  fs.rmSync(clrRoot, { recursive: true, force: true });
});

// ─── D5: fail-open silence ─────────────────────────────────────────────────

await check('D5 — a missing anchor reads as null and both renderers return "" (never throw)', async () => {
  const silentRoot = makeTempRepo();
  assert(readIntent(silentRoot) === null, 'missing anchor => null');
  assert(precompactBlock(null) === '', 'precompactBlock(null) is empty');
  assert(resumeBlock(null) === '', 'resumeBlock(null) is empty');
  assert(precompactBlock(undefined) === '' && resumeBlock(undefined) === '', 'undefined is empty too');
  assert(precompactBlock({}) === '' && resumeBlock({}) === '', 'a shapeless object renders empty, never a half-anchor');
  fs.rmSync(silentRoot, { recursive: true, force: true });
});

await check('D5 — a CORRUPT anchor reads exactly like a missing one (null, no throw)', async () => {
  const corruptRoot = makeTempRepo();
  fs.mkdirSync(path.dirname(intentPath(corruptRoot, DEFAULT_INTENT_KEY)), { recursive: true });
  fs.writeFileSync(intentPath(corruptRoot, DEFAULT_INTENT_KEY), '{not json at all', 'utf8');
  assert(readIntent(corruptRoot) === null, 'unparseable => null');
  // Parseable but not an anchor: a record with no request is not half an
  // objective, it is no objective.
  writeJsonAtomic(intentPath(corruptRoot, DEFAULT_INTENT_KEY), { acceptance: 'only this' });
  assert(readIntent(corruptRoot) === null, 'a request-less record => null');
  writeJsonAtomic(intentPath(corruptRoot, DEFAULT_INTENT_KEY), ['an', 'array']);
  assert(readIntent(corruptRoot) === null, 'a non-object record => null');
  fs.rmSync(corruptRoot, { recursive: true, force: true });
});

await check('keys are filename-safe and never explode (traversal, spaces, emptiness)', async () => {
  assert(sanitizeIntentKey('../../etc/passwd') === 'etc-passwd', `got "${sanitizeIntentKey('../../etc/passwd')}"`);
  assert(sanitizeIntentKey('  ') === DEFAULT_INTENT_KEY, 'a blank key degrades to the default');
  assert(sanitizeIntentKey('feature/with spaces') === 'feature-with-spaces', 'separators normalize');
  assert(sanitizeIntentKey('x'.repeat(400)).length === 120, 'keys are capped');
});

// ─── renderers ─────────────────────────────────────────────────────────────

await check('precompactBlock is labelled top and bottom and carries the request VERBATIM', async () => {
  const anchor = writeIntent(makeTempRepo(), {
    request: VERBATIM_REQUEST,
    acceptance: ACCEPTANCE,
    next_action: 'write the dedupe index migration',
    do_not_reverse: 'the response shape stays byte-identical',
    stop_conditions: 'any auth surface is touched',
  });
  const block = precompactBlock(anchor);
  assert(block.startsWith(INTENT_PRECOMPACT_HEADER), 'the block opens with its label');
  assert(block.trimEnd().endsWith(INTENT_PRECOMPACT_FOOTER), 'the block closes with its label');
  assert(block.includes(VERBATIM_REQUEST), 'the verbatim request is present, newline and all');
  assert(block.includes(`DONE MEANS: ${ACCEPTANCE}`), 'acceptance is present');
  assert(block.includes('NEXT ACTION: write the dedupe index migration'), 'next action is present');
  assert(block.includes('DO NOT REVERSE:'), 'do-not-reverse is present');
  assert(block.includes('STOP IF:'), 'stop conditions are present');
  // Small on purpose: a large anchor is bloat, not durability.
  assert(block.split('\n').length <= 14, `the block must stay small, got ${block.split('\n').length} lines`);
});

await check('resumeBlock leads with the objective and says the workflow serves it', async () => {
  const anchor = writeIntent(makeTempRepo(), {
    request: VERBATIM_REQUEST,
    acceptance: ACCEPTANCE,
    next_action: 'write the dedupe index migration',
  });
  const block = resumeBlock(anchor);
  assert(block.startsWith(INTENT_RESUME_HEADER), 'the resume block opens with the read-this-first header');
  assert(block.includes(VERBATIM_REQUEST), 'the verbatim request is present');
  assert(
    block.indexOf(VERBATIM_REQUEST) < block.indexOf('Everything below is workflow state'),
    'the objective comes before the pointer to workflow state',
  );
  assert(block.split('\n').length <= 12, `the resume block must stay small, got ${block.split('\n').length} lines`);
});

await check('neither renderer ever truncates or re-wraps the request', async () => {
  const long = `${'word '.repeat(200)}TAIL-SENTINEL`;
  const anchor = writeIntent(makeTempRepo(), { request: long, acceptance: ACCEPTANCE });
  assert(precompactBlock(anchor).includes(long), 'precompact keeps a long request whole');
  assert(resumeBlock(anchor).includes(long), 'resume keeps a long request whole');
  assert(precompactBlock(anchor).includes('TAIL-SENTINEL'), 'nothing is cut off the end');
});

// ─── D6: the ≥2-compaction simulation ──────────────────────────────────────
//
// This is the row that justifies the feature, so it is a SIMULATION of the
// mechanism rather than an inspection of it: two arms, identical except that
// one has an anchor, driven through the same modelled compaction boundaries,
// running the REAL shipped renderers (not a prototype).
//
// The summarizer models exactly the priority inversion CONTEXT.md diagnoses:
//   * content re-injected from DISK at each session start comes back at full
//     strength (bee's scaffolding always does; the anchor does too, but only
//     in the anchor arm);
//   * content that is LABELLED do-not-summarize survives the summary verbatim
//     (that is what the PreCompact block buys);
//   * everything else — the conversation, where the user's request otherwise
//     lives alone — is compressed lossily, and compounds across boundaries.

const CHATTER = [
  'agent: reading src/orders/handler.ts and the router table',
  'agent: the retry path currently re-inserts, so a duplicate row is possible',
  'agent: drafting a dedupe index keyed on the request header',
  'agent: running the suite; two fixtures need the new column',
];

function summarize(text, keepRatio) {
  // Lossy and COMPOUNDING: applied to its own output it keeps shrinking,
  // which is why presence — not share — is the axis that matters.
  const words = text.split(/\s+/).filter(Boolean);
  const keep = Math.max(1, Math.floor(words.length * keepRatio));
  return words.slice(0, keep).join(' ');
}

// One compaction boundary: labelled segments survive verbatim, everything
// else is summarized into a single lossy segment.
function compact(segments, keepRatio) {
  // Labelled blocks survive verbatim — but only once: a summarizer that kept
  // every copy would let the anchor grow without bound across boundaries, and
  // the anchor is small on purpose.
  const labelled = segments.filter((s) => s.text.includes(INTENT_PRECOMPACT_HEADER));
  const survivors = labelled.slice(-1);
  const compressed = segments
    .filter((s) => !s.text.includes(INTENT_PRECOMPACT_HEADER))
    .map((s) => s.text)
    .join(' ');
  return [
    { label: 'summary', text: `[summary of prior context] ${summarize(compressed, keepRatio)}` },
    ...survivors,
  ];
}

function simulate({ anchorRoot, compactions, keepRatio = 0.35 }) {
  // Session start: bee's scaffolding is re-injected from disk (it always
  // survives), and in the anchor arm the resume block LEADS it (D4).
  const scaffolding = {
    label: 'bee-preamble (re-injected from disk every start)',
    text: '## bee v1.x\n- Phase: swarming | Mode: standard | Feature: orders-idempotency\n- Gates: context: approved | shape: approved | execution: approved',
  };
  const reinject = () => {
    const anchor = anchorRoot ? readIntent(anchorRoot) : null;
    const lead = anchor ? resumeBlock(anchor) : '';
    return lead
      ? [{ label: 'INTENT ANCHOR (leads, D4)', text: lead }, scaffolding]
      : [scaffolding];
  };

  // Turn 1: the user's request arrives — in the conversation, and nowhere else
  // unless an anchor holds it.
  let segments = [...reinject(), { label: 'user turn', text: VERBATIM_REQUEST }];

  const boundaries = [];
  for (let i = 1; i <= compactions; i += 1) {
    for (const line of CHATTER) segments.push({ label: 'agent turn', text: line });
    // PreCompact (D3): the hook pushes the labelled anchor block.
    const anchor = anchorRoot ? readIntent(anchorRoot) : null;
    const preBlock = anchor ? precompactBlock(anchor) : '';
    if (preBlock) segments.push({ label: 'PreCompact advisory (labelled)', text: preBlock });
    segments = compact(segments, keepRatio);
    // ... and the session restarts with disk-backed content re-injected.
    segments = [...reinject(), ...segments];
    boundaries.push({
      boundary: i,
      present: segments.some((s) => s.text.includes(VERBATIM_REQUEST)),
      context: segments.map((s) => `--- ${s.label} ---\n${s.text}`).join('\n'),
    });
  }
  return boundaries;
}

await check('D6 — over ≥2 compaction boundaries the VERBATIM request survives WITH the anchor', async () => {
  const simRoot = makeTempRepo();
  writeState(simRoot, { ...defaultState(), phase: 'swarming', feature: 'orders-idempotency' });
  writeIntent(simRoot, {
    request: VERBATIM_REQUEST,
    acceptance: ACCEPTANCE,
    next_action: 'write the dedupe index migration',
  });
  const boundaries = simulate({ anchorRoot: simRoot, compactions: 3 });
  assert(boundaries.length === 3, 'three boundaries were simulated');
  for (const b of boundaries) {
    assert(b.present, `the verbatim request must be present after compaction ${b.boundary}`);
  }
  // Fidelity, not just presence: the block that carries it is the labelled
  // one, and it leads the reconstituted context.
  const last = boundaries[boundaries.length - 1].context;
  assert(last.startsWith('--- INTENT ANCHOR (leads, D4) ---'), 'the anchor leads the reconstituted context');
  assert(
    last.indexOf(VERBATIM_REQUEST) < last.indexOf('## bee v1.x'),
    'the objective is stated BEFORE the workflow state — the inversion is corrected',
  );
  console.log('\n--- D6 simulation, WITH anchor (context after compaction 2) ---');
  console.log(boundaries[1].context);
  console.log('--- end ---\n');
  fs.rmSync(simRoot, { recursive: true, force: true });
});

await check('D6 — the control WITHOUT an anchor loses the verbatim request by compaction 2', async () => {
  const boundaries = simulate({ anchorRoot: null, compactions: 3 });
  assert(!boundaries[1].present, 'the control must NOT still hold the verbatim request after 2 compactions');
  assert(!boundaries[2].present, 'and it is still gone after 3');
  // The control must also not be trivially rigged: bee's scaffolding is fully
  // present in it. That is the whole point — the workflow survives, the goal
  // does not.
  assert(boundaries[2].context.includes('Phase: swarming'), "bee's own scaffolding survives at full strength in the control");
  console.log('\n--- D6 simulation, NO anchor / control (context after compaction 2) ---');
  console.log(boundaries[1].context);
  console.log('--- end ---\n');
});

fs.rmSync(root, { recursive: true, force: true });
printSummaryAndExit();
